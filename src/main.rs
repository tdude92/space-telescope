use std::net::TcpListener;

use space_telescope::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8000").expect("Failed to bind TcpListener.");
    run(listener)?.await
}
