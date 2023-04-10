use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Path, State, WebSocketUpgrade},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use rand::{
    distributions::{Alphanumeric, DistString},
    thread_rng,
};
use tokio::sync::{oneshot, RwLock};

use crate::lobby::Lobby;

pub struct GameServer {
    pub lobbies: Arc<RwLock<HashMap<String, Lobby>>>,
}

impl GameServer {
    pub fn new() -> Self {
        Self {
            lobbies: Default::default(),
        }
    }

    async fn create_lobby(&self) -> String {
        let mut code = Alphanumeric.sample_string(&mut thread_rng(), 8);
        let mut lobbies = self.lobbies.write().await;
        loop {
            match lobbies.entry(code.clone()) {
                std::collections::hash_map::Entry::Occupied(_) => {}
                std::collections::hash_map::Entry::Vacant(e) => {
                    let (remove_tx, remove_rx) = oneshot::channel();
                    e.insert(Lobby::new(remove_tx));
                    let lobbies = self.lobbies.clone();
                    let code2 = code.clone();
                    tokio::spawn(async move {
                        let _ = remove_rx.await;
                        println!("Removing lobby: {code2}");
                        lobbies.write().await.remove(&code2);
                    });
                    return code;
                }
            }
            code = Alphanumeric.sample_string(&mut thread_rng(), 8);
        }
    }
}

pub fn router() -> Router {
    let app_state = Arc::new(GameServer::new());
    Router::new()
        .route("/join/:lobby", get(join_lobby))
        .route("/new", post(create_lobby))
        .with_state(app_state)
}

async fn create_lobby(State(server): State<Arc<GameServer>>) -> impl IntoResponse {
    server.create_lobby().await
}

async fn join_lobby(
    ws: WebSocketUpgrade,
    Path(lobby): Path<String>,
    State(server): State<Arc<GameServer>>,
) -> Result<Response, StatusCode> {
    let lobby = server
        .lobbies
        .read()
        .await
        .get(&lobby)
        .ok_or(StatusCode::NOT_FOUND)?
        .clone();
    Ok(ws.on_upgrade(move |socket| async move {
        lobby
            .0
            .send(crate::lobby::LobbyEvent::New(socket))
            .await
            .unwrap();
    }))
}
