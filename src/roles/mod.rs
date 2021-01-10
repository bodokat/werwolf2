use futures::{future::ready, stream::FuturesUnordered, StreamExt};
use itertools::Itertools;
use serenity::{
    client::Context, collector::ReactionCollectorBuilder, framework::standard::CommandResult,
    model::prelude::User,
};
use std::collections::HashMap;

use crate::game::Swap;

#[derive(Hash, Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug)]
pub enum Role {
    Werwolf,
    Dorfbewohner,
    Seherin,
    Dieb,
    Unruhestifterin,
}

impl Role {
    pub async fn action<'a>(
        &self,
        user: &'a User,
        players: &HashMap<&'a User, Role>,
        extra_roles: &Vec<Role>,
        ctx: &Context,
    ) -> CommandResult<Option<Swap<'a>>> {
        match self {
            Role::Werwolf => werwolf::action(user, players, extra_roles, ctx).await,
            Role::Seherin => seherin::action(user, players, extra_roles, ctx).await,
            Role::Dieb => dieb::action(user, players, ctx).await,
            Role::Unruhestifterin => unruhestifterin::action(user, players, ctx).await,
            Role::Dorfbewohner => Ok(None),
        }
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Role::Werwolf => "Werwolf",
                Role::Dorfbewohner => "Dorfbewohner",
                Role::Seherin => "Seherin",
                Role::Dieb => "Dieb",
                Role::Unruhestifterin => "Unruhestifterin",
            }
        )
    }
}

mod werwolf;

mod seherin;

mod dieb;

mod unruhestifterin;
