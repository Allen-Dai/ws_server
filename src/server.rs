use futures_util::TryFutureExt;
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};
use warp::Filter;

type Users = Arc<RwLock<HashMap<usize, mpsc::UnboundedSender<Message>>>>;
static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

pub async fn start(port: u16) {
    pretty_env_logger::init();
    let users = Users::default();
    let users = warp::any().map(move || users.clone());
    let rust_ml = warp::path("ml")
        .and(warp::ws())
        .and(users)
        .map(|ws: warp::ws::Ws, users| ws.on_upgrade(move |socket| user_connected(socket, users)));

    warp::serve(rust_ml).run(([127, 0, 0, 1], port)).await;
}

async fn user_connected(ws: WebSocket, users: Users) {
    let my_id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);
    let (mut user_ws_tx, mut user_ws_rx) = ws.split();
    let (tx, rx) = mpsc::unbounded_channel();
    let mut rx = UnboundedReceiverStream::new(rx);

    tokio::task::spawn(async move {
        while let Some(msg) = rx.next().await {
            user_ws_tx
                .send(msg)
                .unwrap_or_else(|e| {
                    eprintln!("ws error: {}", e);
                })
                .await;
        }
    });

    users.write().await.insert(my_id, tx);
    eprintln!("User connected");

    while let Some(result) = user_ws_rx.next().await {
        let msg = match result {
            Ok(msg) => {
                //eprintln!("Server recv: {:?}", msg);
                msg
            }
            Err(e) => {
                eprintln!("ws error(uid={}): {}", my_id, e);
                break;
            }
        };
        user_message(my_id, msg, &users).await;
    }
    user_disconnected(my_id, &users).await;
}

async fn user_message(my_id: usize, msg: Message, users: &Users) {
    let msg = if let Ok(s) = msg.to_str() {
        s
    } else {
        return;
    };

    for (&uid, tx) in users.read().await.iter() {
        if my_id != uid {
            if let Err(_disconnected) = tx.send(Message::text(msg.clone())) {
                todo!();
            }
        }
    }
}

async fn user_disconnected(my_id: usize, users: &Users) {
    users.write().await.remove(&my_id);
}
