mod server;

#[tokio::main]
async fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert!(!(args.len() == 1), "expected 1 argument: Port number");
    eprintln!("Server starting at localhost:{}", args[1]);
    server::start(args[1].parse::<u16>().unwrap()).await;
}
