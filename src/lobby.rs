use std::collections::HashMap;

use std::collections::hash_map::Entry;
use std::convert::TryFrom;
use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};

use tokio::sync::RwLock;
use tokio::sync::{mpsc, oneshot};

use crate::game::start_game;
use crate::message::{self};
use crate::roles::{self, Role};

#[derive(Clone)]
pub struct Lobby(pub mpsc::Sender<LobbyEvent>);

pub enum LobbyEvent {
    New(WebSocket),
}

#[derive(Default, Clone)]
pub struct LobbySettings {
    pub available_roles: Vec<&'static dyn Role>,
    pub role_amounts: Vec<usize>,
    pub admin: Option<String>,
}

impl From<LobbySettings> for message::LobbySettings {
    fn from(value: LobbySettings) -> Self {
        Self {
            available_roles: value.available_roles.iter().map(|r| r.name()).collect(),
            roles: value.role_amounts,
            admin: value.admin,
        }
    }
}

struct LobbyInner {
    name: String,
    players: RwLock<HashMap<String, mpsc::UnboundedSender<PlayerMessage>>>,
    settings: RwLock<LobbySettings>,
    remove_lobby: oneshot::Sender<()>,
}

impl Lobby {
    pub fn new(name: String, remove_lobby: oneshot::Sender<()>) -> Self {
        let (tx, rx) = mpsc::channel(32);

        let inner = Arc::new(LobbyInner {
            name,
            players: Default::default(),
            remove_lobby,
            settings: RwLock::new(LobbySettings {
                available_roles: roles::ALL_ROLES.clone(),
                ..Default::default()
            }),
        });

        tokio::spawn(lobby_loop(inner, rx));

        Lobby(tx)
    }
}

async fn lobby_loop(lobby: Arc<LobbyInner>, mut rx: mpsc::Receiver<LobbyEvent>) {
    while let Some(msg) = rx.recv().await {
        match msg {
            LobbyEvent::New(mut socket) => {
                let this = lobby.clone();
                tokio::spawn(async move {
                    let players = {
                        let lock = this.players.write().await;
                        lock.keys().cloned().collect()
                    };
                    socket
                        .send(
                            (&message::ToClient::Welcome {
                                settings: (*this.settings.read().await).clone().into(),
                                players,
                            })
                                .into(),
                        )
                        .await;
                    loop {
                        let response = match socket.recv().await {
                            Some(Ok(msg)) => {
                                if let Message::Text(t) = msg {
                                    t
                                } else {
                                    continue;
                                }
                            }
                            _ => return,
                        };

                        let mut players = this.players.write().await;

                        let entry = players.entry(response);
                        match entry {
                            Entry::Vacant(v) => {
                                let name = v.key().clone();
                                let (tx, rx) = mpsc::unbounded_channel();
                                v.insert(tx);
                                drop(players);
                                let msg = message::ToClient::Joined(message::Player { name });
                                this.players.read().await.values().for_each(|s| {
                                    s.send(PlayerMessage::Other(msg.clone()));
                                });
                                tokio::spawn(handle_player_messages(this, socket, rx));
                                return;
                            }
                            Entry::Occupied(_) => {
                                socket.send(Message::Text("false".into())).await;
                                continue;
                            }
                        }
                    }
                });
            }
        }
    }
}

pub enum PlayerMessage {
    Other(message::ToClient),
    Question {
        text: String,
        options: Vec<String>,
        response: oneshot::Sender<usize>,
    },
}

async fn handle_player_messages(
    lobby: Arc<LobbyInner>,
    mut socket: WebSocket,
    mut messages: mpsc::UnboundedReceiver<PlayerMessage>,
) {
    let mut queries: HashMap<usize, oneshot::Sender<usize>> = HashMap::new();
    let mut next_id: usize = 0;
    loop {
        tokio::select! {
            msg = socket.recv() => {
                if let Some(Ok(Message::Text(msg))) = msg {
                    if let Ok(msg) = message::ToServer::try_from(msg.as_str()) {
                        match msg {
                            message::ToServer::Response { id, choice } => {
                                queries.remove(&id).map(|s| s.send(choice));
                            }
                            message::ToServer::Start => {
                                tokio::spawn({
                                    let lobby = lobby.clone();
                                    async move {
                                        lobby.players.read().await.values().for_each(|s| {
                                            s.send(PlayerMessage::Other(message::ToClient::Started));
                                        });
                                        let hash_map = lobby.players.read().await;
                                        let players =
                                            hash_map.iter().map(|(n, s)| (n, s.clone())).collect();
                                        start_game(players, &(*lobby.settings.read().await)).await
                                    }
                                });
                            }
                            message::ToServer::ChangeRoles(new_roles) => {
                                tokio::spawn({
                                    let lobby = lobby.clone();
                                    async move {
                                        {lobby.settings.write().await.role_amounts = new_roles;}
                                        let settings = lobby.settings.read().await.clone();
                                        lobby.players.read().await.values().for_each(|s| {
                                            s.send(PlayerMessage::Other(message::ToClient::NewSettings(settings.clone().into())));
                                        })
                                    }
                                });
                            } //TODO
                            message::ToServer::Kick(_) => {} //TODO
                        }
                    }
                }
            }
            msg = messages.recv() => {
                if let Some(msg) = msg {
                    match msg {
                        PlayerMessage::Other(msg) => {
                            socket.send((&msg).into()).await;
                        }
                        PlayerMessage::Question {
                            text,
                            options,
                            response,
                        } => {
                            let id = next_id;
                            next_id += 1;
                            queries.insert(next_id, response);
                            socket
                                .send((&message::ToClient::Question { id, text, options }).into())
                                .await;
                        }
                    }
                }
            }
        }
    }
}
