use std::net::TcpStream;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::WebSocket;
use tungstenite::{connect, Message};
use url::Url;

type ClientResult<T> = Result<T, Box<dyn std::error::Error>>;

pub struct Client {
    socket: WebSocket<MaybeTlsStream<TcpStream>>,
}

impl Client {
    pub fn new(url: &str) -> ClientResult<Self> {
        let (socket, _) = connect(Url::parse(url)?)?;
        Ok(Client { socket })
    }

    pub fn send(&mut self, msg: &str) -> ClientResult<()> {
        self.socket.write_message(Message::Text(msg.into()))?;
        Ok(())
    }
}
