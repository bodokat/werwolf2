use std::collections::HashMap;

use serenity::{
    client::{Context, EventHandler},
    model::{
        channel::{ChannelType, Message, Reaction},
        id::{GuildId, UserId},
        interactions::{Interaction, InteractionResponseType},
        prelude::User,
    },
};

use tokio::sync::RwLock;

use async_trait::async_trait;

use crate::{
    lobby::{Lobby, LobbyMessage},
    PREFIX,
};

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

    async fn interaction_create(&self, ctx: Context, mut interaction: Interaction) {
        if let Some(x) = &interaction.data {
            match x.name.as_str() {
                "ping" => interaction
                    .create_interaction_response(ctx, |r| {
                        r.kind(InteractionResponseType::ChannelMessageWithSource);
                        r.interaction_response_data(|m| m.content("Pong"))
                    })
                    .await
                    .expect("Error sending message"),
                "join" => {
                    interaction
                        .create_interaction_response(&ctx, |r| {
                            r.kind(InteractionResponseType::DeferredChannelMessageWithSource)
                        })
                        .await
                        .expect("Error sending interaction response");
                    let guild_id = match interaction.guild_id {
                        Some(id) => id,
                        None => {
                            interaction
                                .edit_original_interaction_response(&ctx, |m| {
                                    m.content("Du kannst nur aus Server-Kanälen joinen")
                                })
                                .await
                                .expect("Error sending interaction response");
                            return;
                        }
                    };
                    let user = interaction.member.take().unwrap().user;
                    let user_name = user.name.clone();
                    self.join(guild_id, user, &ctx).await;
                    interaction
                        .edit_original_interaction_response(&ctx, |m| {
                            m.content(format!("Willkommen in der Lobby, {}", user_name,))
                        })
                        .await
                        .expect("Error sending interaction response");
                }
                "joinall" => {
                    interaction
                        .create_interaction_response(&ctx, |r| {
                            r.kind(InteractionResponseType::DeferredChannelMessageWithSource)
                        })
                        .await
                        .expect("Error sending interaction response");
                    let guild_id = match interaction.guild_id {
                        Some(id) => id,
                        None => {
                            interaction
                                .edit_original_interaction_response(&ctx, |m| {
                                    m.content("Du kannst nur aus Server-Kanälen joinen")
                                })
                                .await
                                .expect("Error sending interaction response");
                            return;
                        }
                    };
                    self.join_all(guild_id, interaction.member.unwrap().user, &ctx)
                        .await;
                }
                "leave" => {
                    let user = match interaction.member {
                        Some(x) => x.user,
                        None => interaction.user.unwrap(),
                    };
                    self.leave(user).await;
                }
                "start" => {
                    self.start(interaction.guild_id.unwrap()).await;
                }
                _ => (),
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
                self.join(guild_id, message.author, &ctx).await;
            }
            "joinall" => {
                let guild_id = match message.guild_id {
                    Some(x) => x,
                    None => return,
                };
                self.join_all(guild_id, message.author, &ctx).await;
            }
            "leave" | "l" => {
                self.leave(message.author).await;
                channel_id
                    .say(ctx, "Du hast die Lobby verlassen")
                    .await
                    .unwrap();
            }
            "start" | "s" => {
                self.start(message.guild_id.unwrap()).await;
            }

            _ => (),
        }
    }
}

impl Controller {
    async fn join(&self, guild: GuildId, user: User, ctx: &Context) {
        let mut guard = self.lobbies.write().await;
        let lobby = guard.entry(guild).or_insert_with(|| {
            println!("Creating new lobby");
            Lobby::new(ctx.clone())
        });
        let user_id = user.id;
        let res = lobby.0.send(LobbyMessage::Join(user)).await;
        if let Err(tokio::sync::mpsc::error::SendError(LobbyMessage::Join(user))) = res {
            // Lobby is closed, create a new one
            println!("Creating new lobby (was closed)");
            *lobby = Lobby::new(ctx.clone());
            let _ = lobby.0.send(LobbyMessage::Join(user)).await;
        }

        self.messengers.write().await.insert(user_id, lobby.clone());
    }
    async fn join_all(&self, guild: GuildId, user: User, ctx: &Context) {
        let channels = match ctx.http.get_channels(guild.0).await {
            Ok(x) => x,
            Err(e) => {
                println!("error getting channel: {}", e);
                return;
            }
        };

        let mut to_add = None;
        for channel in channels.iter().filter(|c| c.kind == ChannelType::Voice) {
            let members = match channel.members(ctx).await {
                Ok(x) => x,
                Err(e) => {
                    println!("error getting members: {}", e);
                    return;
                }
            };
            if members.iter().any(|m| m.user.id == user.id) {
                to_add = Some(members);
                break;
            }
        }

        let to_add = match to_add {
            Some(x) => x,
            None => {
                // TODO
                return;
            }
        };

        let mut guard = self.lobbies.write().await;
        let lobby = guard.entry(guild).or_insert_with(|| {
            println!("Creating new lobby");
            Lobby::new(ctx.clone())
        });
        let mut messengers = self.messengers.write().await;
        for member in to_add.into_iter() {
            let user_id = member.user.id;
            let res = lobby.0.send(LobbyMessage::Join(member.user)).await;
            if let Err(tokio::sync::mpsc::error::SendError(LobbyMessage::Join(user))) = res {
                // Lobby is closed, create a new one
                println!("Creating new lobby (was closed)");
                *lobby = Lobby::new(ctx.clone());
                let _ = lobby.0.send(LobbyMessage::Join(user)).await;
            }

            messengers.insert(user_id, lobby.clone());
        }
    }
    async fn leave(&self, user: User) {
        let removed = self.messengers.write().await.remove(&user.id);
        if let Some(lobby) = removed {
            // if we get an Error, the Lobby is already closed (should never happen), so we can ignore it
            let _ = lobby.0.send(LobbyMessage::Leave(user)).await;
        }
    }
    async fn start(&self, guild: GuildId) {
        if let Some(lobby) = self.lobbies.read().await.get(&guild) {
            let _ = lobby.0.send(LobbyMessage::Start).await;
        }
    }
}
