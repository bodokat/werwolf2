#![warn(clippy::pedantic)]
#![allow(clippy::default_trait_access)]

mod game;
mod lobby;
mod roles;
mod server;
mod utils;

use std::path::PathBuf;

use axum::Router;
use tower_http::{
    cors::{Any, CorsLayer},
    services::{ServeDir, ServeFile},
    trace::{self, TraceLayer},
};
use tracing::Level;

pub mod message;

#[allow(clippy::unused_async)]
#[shuttle_runtime::main]
async fn axum(
    #[shuttle_static_folder::StaticFolder] static_folder: PathBuf,
) -> shuttle_axum::ShuttleAxum {
    let app = Router::new()
        .nest("/api", server::router())
        .fallback_service(
            ServeDir::new(&static_folder)
                .not_found_service(ServeFile::new(static_folder.join("index.html"))),
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        .layer(CorsLayer::new().allow_origin(Any));

    Ok(app.into())
}
