#![warn(clippy::all)]

mod game;
mod lobby;
mod roles;
mod utils;

use std::net::SocketAddr;

use tower_http::services::ServeDir;

use axum::Router;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod chat;
pub mod message;
mod server;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "werwolf=trace".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new()
        .nest("/chat", chat::router())
        .nest("/api", server::router())
        .fallback_service(ServeDir::new("web/build"));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on http://{addr}");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
