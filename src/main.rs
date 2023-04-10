#![warn(clippy::all)]

mod game;
mod lobby;
mod roles;
mod utils;

use std::net::SocketAddr;

use axum::Router;
use tower_http::{
    cors::{Any, CorsLayer},
    services::{ServeDir, ServeFile},
    trace::{self, TraceLayer},
};
use tracing::Level;

pub mod message;
mod server;

#[tokio::main]
async fn main() {
    // tracing_subscriber::registry()
    //     .with(tracing_subscriber::EnvFilter::new(
    //         std::env::var("RUST_LOG").unwrap_or_else(|_| "werwolf=trace".into()),
    //     ))
    //     .with(tracing_subscriber::fmt::layer())
    //     .init();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let app = Router::new()
        .nest("/api", server::router())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        .layer(CorsLayer::new().allow_origin(Any))
        .fallback_service(
            ServeDir::new("./web/dist").not_found_service(ServeFile::new("./web/dist/index.html")),
        );

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on http://{addr}");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
