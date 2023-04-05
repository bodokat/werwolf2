use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use futures::{lock::Mutex, SinkExt, StreamExt};
use rand::{
    distributions::{Alphanumeric, DistString},
    thread_rng,
};
use tokio::sync::{broadcast, RwLock};

pub struct ChatServer {
    pub rooms: RwLock<HashMap<String, Arc<ChatRoom>>>,
}

impl ChatServer {
    pub fn new() -> Self {
        Self {
            rooms: Default::default(),
        }
    }

    async fn create_room(&self) -> String {
        let room = Arc::new(ChatRoom::new());
        let mut code = Alphanumeric.sample_string(&mut thread_rng(), 8);
        let mut rooms = self.rooms.write().await;
        loop {
            match rooms.entry(code.clone()) {
                std::collections::hash_map::Entry::Occupied(_) => {}
                std::collections::hash_map::Entry::Vacant(e) => {
                    e.insert(room);
                    break;
                }
            }
            code = Alphanumeric.sample_string(&mut thread_rng(), 8)
        }
        code
    }
}

pub struct ChatRoom {
    users: Mutex<HashSet<String>>,
    tx: broadcast::Sender<String>,
}

impl ChatRoom {
    fn new() -> Self {
        Self {
            users: Default::default(),
            tx: broadcast::channel(100).0,
        }
    }

    async fn add_user(&self, stream: WebSocket) {
        let (mut sender, mut receiver) = stream.split();

        let mut username = None;
        while let Some(Ok(message)) = receiver.next().await {
            if let Message::Text(name) = message {
                let mut users = self.users.lock().await;
                if !users.contains(&name) {
                    users.insert(name.clone());
                    username = Some(name);
                    break;
                } else {
                    let _ = sender.send(Message::Text("Username already taken".into()));
                    return;
                }
            }
        }
        let username = match username {
            Some(name) => name,
            None => return,
        };

        let mut rx = self.tx.subscribe();

        let _ = self.tx.send(format!("{username} joined"));

        let mut send_task = tokio::spawn(async move {
            while let Ok(msg) = rx.recv().await {
                if sender.send(Message::Text(msg)).await.is_err() {
                    break;
                }
            }
        });

        let tx = self.tx.clone();
        let name = username.clone();

        let mut recv_task = tokio::spawn(async move {
            while let Some(Ok(msg)) = receiver.next().await {
                if let Message::Text(msg) = msg {
                    let _ = tx.send(format!("[{name}]: {msg}"));
                }
            }
        });

        tokio::select! {
            _ = (&mut send_task) => recv_task.abort(),
            _ = (&mut recv_task) => send_task.abort(),
        };

        let _ = self.tx.send(format!("{username} left"));
        self.users.lock().await.remove(&username);
    }
}

pub fn router() -> Router {
    let app_state = Arc::new(ChatServer::new());
    Router::new()
        .route("/room/:room", get(join_room))
        .route("/new", post(create_room))
        .with_state(app_state)
}

async fn create_room(
    State(server): State<Arc<ChatServer>>
) -> impl IntoResponse {
    server.create_room().await

}

async fn join_room(
    ws: WebSocketUpgrade,
    Path(room): Path<String>,
    State(state): State<Arc<ChatServer>>,
) -> Result<Response, StatusCode> {
    let room = state.rooms.read().await.get(&room).ok_or(StatusCode::NOT_FOUND)?.clone();
    
    Ok(ws.on_upgrade(move |socket| 
        // needed to fix "cannot return reference to local data `room`" error
        async move { room.add_user(socket).await }))
}
