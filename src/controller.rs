use std::collections::HashMap;

use itertools::Itertools;
use serenity::{
    client::{Context, EventHandler},
    model::{
        channel::{ChannelType, Message, Reaction},
        id::{GuildId, UserId},
    },
};

use tokio::sync::RwLock;

use async_trait::async_trait;

use crate::lobby::{Lobby, LobbyMessage};
use crate::PREFIX;

#[derive(Debug)]
pub enum ReactionAction {
    Added(Reaction),
    Removed(Reaction),
}

impl ReactionAction {
    pub fn inner(&self) -> &Reaction {
        match self {
            ReactionAction::Added(inner) | ReactionAction::Removed(inner) => inner,
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
        Controller::default()
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
            "join" | "j" => {
                let guild_id = match message.guild_id {
                    Some(x) => x,
                    None => {
                        let _ = message
                            .channel_id
                            .say(ctx, "Du kannst nicht von Direknachrichten joinen")
                            .await;
                        return;
                    }
                };
                let mut guard = self.lobbies.write().await;
                let lobby = guard.entry(guild_id).or_insert_with(|| {
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
            }
            "add" => {
                let guild_id = match message.guild_id {
                    Some(x) => x,
                    None => return,
                };

                let channels = match ctx.http.get_channels(guild_id.0).await {
                    Ok(x) => x,
                    Err(e) => {
                        println!("error getting channel: {}", e);
                        return;
                    }
                };

                let mut to_add = None;
                let ctx = &ctx;
                for channel in channels.iter().filter(|c| c.kind == ChannelType::Voice) {
                    let members = match channel.members(ctx).await {
                        Ok(x) => x,
                        Err(e) => {
                            println!("error getting members: {}", e);
                            return;
                        }
                    };
                    if members.iter().any(|m| m.user.id == message.author.id) {
                        to_add = Some(members);
                        break;
                    }
                }

                let to_add = match to_add {
                    Some(x) => x,
                    None => {
                        let _ = message
                            .reply(
                                ctx,
                                "Du musst in einem Sprachkanal sein, um !add zu benutzen",
                            )
                            .await;
                        return;
                    }
                };

                let _ = message
                    .channel_id
                    .say(
                        ctx,
                        format!(
                            "Füge die folgenden Spieler hinzu: {}",
                            to_add.iter().map(|m| m.user.name.clone()).join(", ")
                        ),
                    )
                    .await;

                let mut guard = self.lobbies.write().await;
                let lobby = guard.entry(guild_id).or_insert_with(|| {
                    println!("Creating new lobby");
                    Lobby::new(ctx.clone())
                });
                let mut messengers = self.messengers.write().await;
                for member in to_add.into_iter() {
                    let user_id = member.user.id;
                    let res = lobby.0.send(LobbyMessage::Join(member.user)).await;
                    if let Err(tokio::sync::mpsc::error::SendError(LobbyMessage::Join(user))) = res
                    {
                        // Lobby is closed, create a new one
                        println!("Creating new lobby (was closed)");
                        *lobby = Lobby::new(ctx.clone());
                        let _ = lobby.0.send(LobbyMessage::Join(user)).await;
                    }

                    messengers.insert(user_id, lobby.clone());
                }
            }
            "leave" | "l" => {
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
            "start" | "s" => {
                if let Some(lobby) = self.lobbies.read().await.get(&message.guild_id.unwrap()) {
                    let _ = lobby.0.send(LobbyMessage::Start).await;
                }
            }

            _ => (),
        }
    }
}
