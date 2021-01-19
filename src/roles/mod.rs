#![allow(clippy::borrowed_box)]

use async_trait::async_trait;
use dyn_clone::DynClone;
use futures::{stream::FuturesUnordered, StreamExt};
use itertools::Itertools;
use serenity::{client::Context, model::prelude::User};
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

pub trait Role: DynClone + Display + Send + Sync {
    fn build(&self) -> Box<dyn RoleData>;

    fn team(&self) -> Team;

    fn group(&self) -> Group;
}

impl<T> Role for T
where
    T: 'static + RoleData + Clone,
{
    fn build(&self) -> Box<dyn RoleData> {
        let t: T = self.clone();
        Box::new(t)
    }

    fn team(&self) -> Team {
        self.team()
    }

    fn group(&self) -> Group {
        self.group()
    }
}

#[async_trait]
pub trait RoleData: Display + Send + Sync {
    async fn ask(
        &mut self,
        _player: &User,
        _player_roles: &HashMap<&User, &Box<dyn Role>>,
        _extra_roles: &[Box<dyn Role>],
        _ctx: &Context,
        _receiver: &mut ReceiverStream<ReactionAction>,
    ) {
    }

    fn action(
        &self,
        _player: &User,
        _player_roles: &mut HashMap<&User, &Box<dyn Role>>,
        _extra_roles: &[Box<dyn Role>],
        _ctx: &Context,
    ) {
    }

    fn after(
        &self,
        _player: &User,
        _player_roles: &mut HashMap<&User, &Box<dyn Role>>,
        _extra_roles: &[Box<dyn Role>],
        _ctx: &Context,
    ) {
    }

    fn team(&self) -> Team;

    fn group(&self) -> Group;
}

dyn_clone::clone_trait_object!(Role);

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
