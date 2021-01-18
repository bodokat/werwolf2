use async_trait::async_trait;
use futures::{stream::FuturesUnordered, StreamExt};
use itertools::Itertools;
use serenity::{client::Context, framework::standard::CommandResult, model::prelude::User};
use std::{collections::HashMap, fmt::Display};
use tokio_stream::wrappers::ReceiverStream;

use crate::controller::ReactionAction;

#[derive(PartialEq, Eq)]
pub enum Team {
    Dorf,
    Wolf,
}

#[derive(PartialEq, Eq)]
pub enum Group {
    Mensch,
    Wolf,
}

pub enum Action<'a> {
    Swap(&'a User, &'a User),
    Copy { from: &'a User, to: &'a User },
    SayRole(&'a User),
}

impl<'a> Action<'a> {
    pub fn perform(&self, roles: &mut HashMap<&'a User, &Box<dyn Role>>, ctx: &Context) {
        match self {
            Action::Swap(u1, u2) => {
                let a = roles.get_mut(u1).unwrap() as *mut &Box<dyn Role>;
                let b = roles.get_mut(u2).unwrap() as *mut &Box<dyn Role>;
                // SAFETY: the only reason why we can't call std::mem::swap is that we would have to borrow [roles] mutably twice
                unsafe {
                    std::ptr::swap(a, b);
                }
            }
            Action::Copy { from, to } => {
                roles.insert(to, roles.get(from).unwrap());
            }
            Action::SayRole(u) => {
                let role_name = roles.get(u).unwrap().to_string();
                let user_id = u.id;
                let ctx = ctx.clone();
                tokio::spawn(async move {
                    user_id
                        .create_dm_channel(&ctx)
                        .await
                        .unwrap()
                        .say(&ctx, format!("Deine Rolle ist {}", role_name))
                        .await
                });
            }
        }
    }
}

#[async_trait]
pub trait Role: Display + Send + Sync {
    async fn action<'a>(
        &self,
        _player: &'a User,
        _player_roles: &HashMap<&'a User, &Box<dyn Role>>,
        _extra_roles: &[Box<dyn Role>],
        _ctx: &Context,
        _receiver: &mut ReceiverStream<ReactionAction>,
    ) -> CommandResult<Vec<Action<'a>>> {
        Ok(vec![])
    }

    fn team(&self) -> Team;

    fn group(&self) -> Group;
}

mod dorfbewohner;
pub use dorfbewohner::Dorfbewohner;

mod werwolf;
pub use werwolf::Werwolf;

mod seherin;
pub use seherin::Seherin;

mod dieb;
pub use dieb::Dieb;

mod unruhestifterin;
pub use unruhestifterin::Unruhestifterin;

mod schlaflose;
pub use schlaflose::Schlaflose;

mod doppel;
pub use doppel::Doppel;
