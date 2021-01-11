use std::collections::HashMap;

use serenity::{
    client::{Context, EventHandler},
    model::{channel::Reaction, id::UserId},
};

use async_trait::async_trait;

use crate::lobby::{Lobby, Message};

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
struct Controller {
    context: Context,
    lobbies: HashMap<UserId, Lobby>,
}

#[async_trait]
impl EventHandler for Controller {
    async fn reaction_add(&self, _ctx: Context, add_reaction: Reaction) {
        if let Some(id) = add_reaction.user_id {
            if let Some(lobby) = self.lobbies.get(&id) {
                let res = lobby
                    .0
                    .send(Message::Reaction(ReactionAction::Added(add_reaction)))
                    .await;
                if let Err(_) = res {
                    lobby.re
                }
            }
        }
    }
    async fn reaction_remove(&self, _ctx: Context, removed_reaction: Reaction) {
        if let Some(id) = removed_reaction.user_id {
            if let Some(lobby) = self.lobbies.get(&id) {
                lobby
                    .0
                    .send(Message::Reaction(ReactionAction::Removed(removed_reaction)))
                    .await;
            }
        }
    }
}
