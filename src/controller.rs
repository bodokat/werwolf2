use std::collections::HashMap;

use serenity::{
    client::{Context, EventHandler},
    model::{
        channel::{Message, Reaction},
        id::{GuildId, UserId},
    },
};

use tokio::sync::RwLock;

use async_trait::async_trait;

use crate::lobby::{Lobby, LobbyMessage};
use crate::PREFIX;

pub enum ReactionAction {
    Added(Reaction),
    Removed(Reaction),
}

impl ReactionAction {
    pub fn inner(&self) -> &Reaction {
        match self {
            ReactionAction::Added(inner) => inner,
            ReactionAction::Removed(inner) => inner,
        }
    }
}

#[derive(Default)]
pub struct Controller {
    lobbies: RwLock<HashMap<GuildId, Lobby>>,
    messengers: RwLock<HashMap<UserId, Lobby>>,
}

impl Controller {
    pub fn new() -> Self {
        Default::default()
    }
}

#[async_trait]
impl EventHandler for Controller {
    async fn reaction_add(&self, _ctx: Context, add_reaction: Reaction) {
        if let Some(id) = add_reaction.user_id {
            if let Some(lobby) = self.messengers.read().await.get(&id) {
                let _ = lobby
                    .0
                    .try_send(LobbyMessage::Reaction(ReactionAction::Added(add_reaction)));
            }
        }
    }
    async fn reaction_remove(&self, _ctx: Context, removed_reaction: Reaction) {
        if let Some(id) = removed_reaction.user_id {
            if let Some(lobby) = self.messengers.read().await.get(&id) {
                let _ = lobby
                    .0
                    .try_send(LobbyMessage::Reaction(ReactionAction::Removed(
                        removed_reaction,
                    )));
            }
        }
    }
    async fn message(&self, ctx: Context, message: Message) {
        if !message.content.starts_with(PREFIX) {
            return;
        }
        let mut args = message.content.trim_start_matches(PREFIX).split(' ');
        let command = match args.next() {
            Some(s) => s.to_ascii_lowercase(),
            None => {
                return;
            }
        };

        let channel_id = message.channel_id;

        match command.as_str() {
            "join" => {
                let mut guard = self.lobbies.write().await;
                let lobby = guard.entry(message.guild_id.unwrap()).or_insert_with(|| {
                    println!("Creating new lobby");
                    Lobby::new(ctx.clone())
                });
                let author_id = message.author.id;
                let res = lobby.0.send(LobbyMessage::Join(message.author)).await;
                if let Err(tokio::sync::mpsc::error::SendError(LobbyMessage::Join(author))) = res {
                    // Lobby is closed, create a new one
                    println!("Creating new lobby (was closed)");
                    *lobby = Lobby::new(ctx.clone());
                    let _ = lobby.0.send(LobbyMessage::Join(author)).await;
                }

                self.messengers
                    .write()
                    .await
                    .insert(author_id, lobby.clone());

                channel_id
                    .say(ctx, "Du bist der Lobby beigetreten")
                    .await
                    .unwrap();
            }
            "leave" => {
                let removed = self.messengers.write().await.remove(&message.author.id);
                if let Some(lobby) = removed {
                    // if we get an Error, the Lobby is already closed (should never happen), so we can ignore it
                    let _ = lobby.0.send(LobbyMessage::Leave(message.author)).await;
                }
                channel_id
                    .say(ctx, "Du hast die Lobby verlassen")
                    .await
                    .unwrap();
            }
            "start" => {
                if let Some(lobby) = self.lobbies.read().await.get(&message.guild_id.unwrap()) {
                    let _ = lobby.0.send(LobbyMessage::Start).await;
                }
            }

            _ => (),
        }
    }
}
