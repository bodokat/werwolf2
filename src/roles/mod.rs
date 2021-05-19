#![allow(clippy::borrowed_box)]

use async_trait::async_trait;
use dyn_clone::DynClone;
use futures::{stream::FuturesUnordered, StreamExt};
use itertools::Itertools;
use std::{collections::HashMap, fmt::Display};
use tokio_stream::wrappers::ReceiverStream;

use crate::{controller::ReactionAction, game::GameData};

#[derive(PartialEq, Eq)]
pub enum Team {
    Dorf,
    Wolf,
}

#[derive(PartialEq, Eq)]
pub enum Group {
    Mensch,
    Freimaurer,
    Wolf,
}

pub trait Role: DynClone + Display + Send + Sync {
    fn build(&self) -> Box<dyn RoleBehavior>;

    fn team(&self) -> Team;

    fn group(&self) -> Group;
}

impl<T> Role for T
where
    T: 'static + RoleBehavior + Clone,
{
    fn build(&self) -> Box<dyn RoleBehavior> {
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
pub trait RoleBehavior: Display + Send + Sync {
    async fn before_ask<'a>(
        &mut self,
        _data: &GameData<'a>,
        _reactions: &mut ReceiverStream<ReactionAction>,
        _index: usize,
    ) {
    }

    fn before_action<'a>(&mut self, _data: &mut GameData<'a>, _index: usize) {}

    async fn ask<'a>(
        &mut self,
        _data: &GameData<'a>,
        _reactions: &mut ReceiverStream<ReactionAction>,
        _index: usize,
    ) {
    }

    fn action<'a>(&mut self, _data: &mut GameData<'a>, _index: usize) {}

    async fn after<'a>(
        &mut self,
        _data: &GameData<'a>,
        _reactions: &mut ReceiverStream<ReactionAction>,
        _index: usize,
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

mod freimaurer;
pub use freimaurer::Freimaurer;

mod gunstling;
pub use gunstling::Gunstling;
