use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::{channel::Message, id::ChannelId, user::User},
    prelude::*,
};
use tokio::spawn;

use crate::game::start_game;

pub struct LobbyMap;

impl TypeMapKey for LobbyMap {
    type Value = Arc<RwLock<HashMap<ChannelId, Option<Lobby>>>>;
}

impl LobbyMap {
    pub fn new() -> <Self as TypeMapKey>::Value {
        Arc::new(RwLock::new(HashMap::new()))
    }
}

pub struct Lobby {
    pub players: HashSet<User>,
}
impl Lobby {
    pub fn new() -> Self {
        Self {
            players: HashSet::new(),
        }
    }
}

#[command]
pub async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let lobby_map = {
        ctx.data
            .read()
            .await
            .get::<LobbyMap>()
            .expect("expected LobbyData in TypeMap")
            .clone()
    };

    let joined = {
        let mut lobby_guard = lobby_map.write().await;
        let lobby = lobby_guard.entry(msg.channel_id).or_insert_with(|| {
            tokio::spawn(msg.channel_id.say(ctx.http.clone(), "Starte neue Lobby"));
            Some(Lobby::new())
        });
        match lobby {
            Some(data) => Some(data.players.insert(msg.author.clone())),
            None => None,
        }
    };

    match joined {
        Some(true) => {
            msg.reply(ctx, "ist der Lobby beigetreten").await?;
        }
        Some(false) => {
            msg.reply(ctx, "du bist schon in der Lobby").await?;
        }
        None => {
            msg.reply(ctx, "dieses Spiel ist schon gestartet").await?;
        }
    }
    Ok(())
}

#[command]
pub async fn players(ctx: &Context, msg: &Message) -> CommandResult {
    let lobby_map = {
        ctx.data
            .read()
            .await
            .get::<LobbyMap>()
            .expect("expected LobbyData in TypeMap")
            .clone()
    };
    let lobby_map = lobby_map.read().await;
    let response = {
        match lobby_map.get(&msg.channel_id) {
            Some(m) => match m {
                Some(data) => Some(format!(
                    "Current players: {}",
                    data.players
                        .iter()
                        .map(|user| user.name.clone())
                        .collect::<Vec<String>>()
                        .join(", ")
                )),
                None => None,
            },
            None => Some("Hier ist im Moment keine Lobby. Starte eine mit !join".to_string()),
        }
    };
    if let Some(response) = response {
        msg.reply(ctx, response).await?;
    }

    Ok(())
}

#[command]
pub async fn start(ctx: &Context, msg: &Message) -> CommandResult {
    let lobby_map = {
        ctx.data
            .read()
            .await
            .get::<LobbyMap>()
            .expect("expected LobbyData in TypeMap")
            .clone()
    };
    let mut lobby_map = lobby_map.write().await;
    let data = lobby_map.get_mut(&msg.channel_id);
    match data {
        Some(data) => match data.take() {
            Some(data) => {
                let ctx2 = ctx.clone();
                let id = msg.channel_id;
                spawn(async move {
                    let _ = start_game(&ctx2, &data).await;
                    ctx2.data
                        .read()
                        .await
                        .get::<LobbyMap>()
                        .expect("expected LobbyData in TypeMap")
                        .write()
                        .await
                        .insert(id, Some(data));
                });
            }
            None => {
                msg.reply(ctx, "dieses Spiel ist schon gestartet").await?;
            }
        },
        None => {
            msg.channel_id
                .say(ctx, "Hier ist im Moment keine Lobby. Starte eine mit !join")
                .await?;
        }
    }
    return Ok(());
}
