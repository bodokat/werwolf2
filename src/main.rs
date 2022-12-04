#![warn(clippy::all)]

use serenity::client::Client;

mod controller;
mod game;
mod lobby;
mod roles;
mod utils;

use controller::Controller;

use std::net::SocketAddr;

use axum_extra::routing::SpaRouter;
use futures::SinkExt;

use axum::Router;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod chat;
pub mod message;

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
        .merge(SpaRouter::new("/", "web/build"));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on http://{addr}");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
