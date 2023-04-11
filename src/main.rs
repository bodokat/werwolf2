#![warn(clippy::pedantic)]
#![allow(clippy::default_trait_access)]

mod game;
mod lobby;
mod roles;
mod utils;

use std::{net::SocketAddr, path::Path};

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
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let port = match std::env::var("PORT").map(|s| s.parse()) {
        Ok(Ok(port)) => port,
        _ => 3000,
    };
    let web_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("web/dist");
    let index_path = web_path.join("index.html");

    if !index_path.is_file() {
        tracing::error!("index.html not found");
    }

    let app = Router::new()
        .nest("/api", server::router())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        .layer(CorsLayer::new().allow_origin(Any))
        .fallback_service(ServeDir::new(web_path).not_found_service(ServeFile::new(index_path)));

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("listening on http://{addr}");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
