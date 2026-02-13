use std::net::SocketAddr;
use std::time::Duration;
use axum::Router;
use axum::routing::{get, post};
use tokio::net::TcpListener;

mod server;
mod models;
mod parse;

async fn start_webserver() {
    println!("Starting webserver...");

    let app = Router::new()
        .route("/", get(server::get_calendar))
        .route("/", post(server::process_schedule));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3069));
    let listener = TcpListener::bind(addr).await.unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    println!("Webserver started on {}:{}", addr.ip(), addr.port());
}

#[tokio::main]
async fn main() {
    start_webserver().await;

    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
