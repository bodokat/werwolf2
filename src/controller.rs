use std::collections::HashMap;

use futures::channel::mpsc::Sender;
use serenity::{
    client::Context,
    model::{channel::Reaction, id::UserId},
};

use crate::lobby::Lobby;

pub enum ReactionAction {
    Added(Reaction),
    Removed(Reaction),
}
struct Controller<'a> {
    context: Context,
    lobbies: HashMap<UserId, &'a mut Lobby>,
}
