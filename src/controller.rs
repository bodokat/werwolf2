use std::collections::HashMap;

use serenity::{
    client::{Context, EventHandler},
    model::{
        channel::{ChannelType, Message},
        id::{GuildId, UserId},
        interactions::{self, ButtonStyle, Interaction, InteractionResponseType},
        prelude::User,
    },
};

use tokio::sync::RwLock;

use async_trait::async_trait;

use crate::{
    lobby::{Lobby, LobbyMessage},
    PREFIX,
};

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
    async fn interaction_create(&self, ctx: Context, mut interaction: Interaction) {
        match &interaction.data {
            Some(interactions::InteractionData::ApplicationCommand(data)) => {
                match data.name.as_str() {
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
                    "button" => {
                        interaction
                            .create_interaction_response(&ctx, |c| {
                                c.kind(InteractionResponseType::ChannelMessageWithSource);
                                c.interaction_response_data(|m| {
                                    m.content("Choose one!");
                                    m.components(|c| {
                                        c.create_action_row(|a| {
                                            a.create_button(|b| {
                                                b.label("Hello");
                                                b.style(ButtonStyle::Primary);
                                                b.custom_id("hello")
                                            });
                                            a.create_button(|b| {
                                                b.label("Delete");
                                                b.style(ButtonStyle::Danger);
                                                b.custom_id("delete")
                                            })
                                        })
                                    })
                                })
                            })
                            .await
                            .expect("Error sending response");
                    }
                    _ => (),
                }
            }
            Some(interactions::InteractionData::MessageComponent(data)) => {
                if let Some(user) = interaction.user.as_ref() {
                    if let Some(lobby) = self.messengers.read().await.get(&user.id) {
                        let _ = lobby.0.try_send(LobbyMessage::Interaction(interaction));
                        return;
                    }
                }

                match data.custom_id.as_str() {
                    "hello" => {
                        interaction
                            .create_interaction_response(&ctx, |r| {
                                r.kind(InteractionResponseType::UpdateMessage);
                                r.interaction_response_data(|d| d.content("Hi :)"))
                            })
                            .await
                            .expect("Error editing message");
                    }
                    "delete" => {
                        match interaction.message.unwrap() {
                            interactions::InteractionMessage::Regular(m) => m,
                            interactions::InteractionMessage::Ephemeral(_) => return,
                        }
                        .delete(&ctx)
                        .await
                        .expect("Error delting message");
                    }
                    _ => (),
                }
            }
            None => (),
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
