use std::collections::HashMap;

use std::collections::hash_map::Entry;
use std::convert::TryFrom;
use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};

use tokio::sync::{mpsc, oneshot};
use tokio::sync::{Mutex, RwLock};

use crate::game::start_game;
use crate::message::{self, ToClient, ToServer};
use crate::roles::{self, Role};

#[derive(Clone)]
pub struct Lobby(pub mpsc::Sender<LobbyEvent>);

#[derive(Debug)]
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
    players: RwLock<HashMap<String, mpsc::UnboundedSender<PlayerMessage>>>,
    settings: RwLock<LobbySettings>,
    remove_lobby: Mutex<Option<oneshot::Sender<()>>>,
}

impl Lobby {
    pub fn new(remove_lobby: oneshot::Sender<()>) -> Self {
        let (tx, rx) = mpsc::channel(32);

        let inner = Arc::new(LobbyInner {
            players: Default::default(),
            remove_lobby: Mutex::new(Some(remove_lobby)),
            settings: RwLock::new(LobbySettings {
                available_roles: roles::ALL_ROLES.clone(),
                role_amounts: vec![0; roles::ALL_ROLES.len()],
                ..Default::default()
            }),
        });

        tokio::spawn(inner.lobby_loop(rx));

        Lobby(tx)
    }
}

impl LobbyInner {
    async fn lobby_loop(self: Arc<Self>, mut rx: mpsc::Receiver<LobbyEvent>) {
        while let Some(msg) = rx.recv().await {
            tracing::info!("New Lobby Message");
            match msg {
                LobbyEvent::New(mut socket) => {
                    let this = self.clone();
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
                            .await
                            .unwrap();
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
                                    v.insert(tx.clone());
                                    drop(players);

                                    tx.send(PlayerMessage::Other(ToClient::NameAccepted {
                                        name: name.clone(),
                                    }))
                                    .unwrap();
                                    let msg = message::ToClient::Joined {
                                        player: message::Player { name: name.clone() },
                                    };
                                    this.players.read().await.values().for_each(|s| {
                                        s.send(PlayerMessage::Other(msg.clone())).unwrap();
                                    });
                                    {
                                        let settings = this.settings.read().await;
                                        if settings.admin.is_none() {
                                            drop(settings);
                                            let mut lobby_settings = this.settings.write().await;
                                            lobby_settings.admin = Some(name.clone());
                                            let msg = ToClient::NewSettings(
                                                lobby_settings.clone().into(),
                                            );
                                            drop(lobby_settings);
                                            this.players.read().await.values().for_each(|s| {
                                                s.send(PlayerMessage::Other(msg.clone())).unwrap();
                                            });
                                        }
                                    }

                                    tokio::spawn(handle_player_messages(
                                        this,
                                        name.clone(),
                                        socket,
                                        rx,
                                    ));
                                    return;
                                }
                                Entry::Occupied(_) => {
                                    socket.send((&ToClient::NameRejected).into()).await.unwrap();
                                    continue;
                                }
                            }
                        }
                    });
                }
            }
        }
    }
    async fn remove(self: Arc<Self>, player: String) {
        let mut players = self.players.write().await;
        players.remove(&player);
        if players.len() == 0 {
            drop(players);
            self.remove_lobby.lock().await.take().map(|s| s.send(()));
        }
    }
}

#[derive(Debug)]
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
    name: String,
    mut socket: WebSocket,
    mut messages: mpsc::UnboundedReceiver<PlayerMessage>,
) {
    let mut queries: HashMap<usize, oneshot::Sender<usize>> = HashMap::new();
    let mut next_id: usize = 0;
    let handle_message = |msg: ToServer, queries: &mut HashMap<usize, oneshot::Sender<usize>>| {
        tracing::info!("new message: {:?}", msg);
        match msg {
            message::ToServer::Response { id, choice } => {
                queries.remove(&id).map(|s| s.send(choice));
            }
            message::ToServer::Start => {
                tokio::spawn({
                    let lobby = lobby.clone();
                    let name = name.clone();
                    async move {
                        let (players, settings) =
                            tokio::join!(lobby.players.read(), lobby.settings.read());
                        if !(settings.admin == Some(name)) {
                            return;
                        }
                        if settings.role_amounts.iter().sum::<usize>() < players.len() {
                            return;
                        }
                        players.values().for_each(|s| {
                            s.send(PlayerMessage::Other(message::ToClient::Started))
                                .unwrap();
                        });
                        let players_vec = players.iter().map(|(n, s)| (n, s.clone())).collect();
                        start_game(players_vec, &(*lobby.settings.read().await)).await;
                        players.values().for_each(|s| {
                            s.send(PlayerMessage::Other(ToClient::Ended)).unwrap();
                        })
                    }
                });
            }
            message::ToServer::ChangeRoles { new_roles } => {
                tokio::spawn({
                    let lobby = lobby.clone();
                    let name = name.clone();
                    async move {
                        if !(lobby.settings.read().await.admin == Some(name)) {
                            return;
                        }
                        {
                            lobby.settings.write().await.role_amounts = new_roles;
                        }
                        let settings = lobby.settings.read().await.clone();
                        lobby.players.read().await.values().for_each(|s| {
                            s.send(PlayerMessage::Other(message::ToClient::NewSettings(
                                settings.clone().into(),
                            )))
                            .unwrap();
                        })
                    }
                });
            } //TODO
            message::ToServer::Kick { player: _ } => {} //TODO
        }
    };

    loop {
        tokio::select! {
            msg = socket.recv() => {
                match msg {
                    Some(Ok(msg)) => {
                        if let Message::Text(msg) = msg{
                        match message::ToServer::try_from(msg.as_str()) {
                            Ok(msg) => {
                            handle_message(msg,&mut queries);}
                            Err(e) => {
                                tracing::warn!("Deserialization error: {e}, message: {msg}");
                            }
                        }}
                    }
                    _ => {(lobby.clone()).remove(name).await; return;}
                }
            }
            msg = messages.recv() => {
                if let Some(msg) = msg {
                    match msg {
                        PlayerMessage::Other(msg) => {
                            socket.send((&msg).into()).await.unwrap();
                        }
                        PlayerMessage::Question {
                            text,
                            options,
                            response,
                        } => {
                            let id = next_id;
                            next_id += 1;
                            queries.insert(id, response);
                            socket
                                .send((&message::ToClient::Question { id, text, options }).into())
                                .await.unwrap();
                        }
                    }
                }
            }
        }
    }
}
