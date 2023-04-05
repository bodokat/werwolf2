#![allow(clippy::borrowed_box)]

use async_trait::async_trait;
use dyn_clone::DynClone;
use once_cell::sync::Lazy;

use std::{any::Any, fmt::Display};

use crate::game::GameData;

pub static ALL_ROLES: Lazy<Vec<&dyn Role>> = Lazy::new(|| {
    let r: [Box<dyn Role>; 8] = [
        Box::new(Dieb),
        Box::new(Doppel),
        Box::new(Dorfbewohner),
        Box::new(Freimaurer),
        Box::new(Werwolf),
        Box::new(Seherin),
        Box::new(Unruhestifterin),
        Box::new(Schlaflose),
    ];
    IntoIterator::into_iter(r)
        .map(|b| (Box::leak(b) as &_))
        .collect()
});

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

pub trait Role: DynClone + Display + Send + Sync + Any {
    fn build(&self) -> Box<dyn RoleBehavior>;

    fn team(&self) -> Team;

    fn group(&self) -> Group;

    fn name(&self) -> String;
}

// impl<T> Role for T
// where
//     T: 'static + RoleBehavior + Clone,
// {
//     fn build(&self) -> Box<dyn RoleBehavior> {
//         let t: T = self.clone();
//         Box::new(t)
//     }

//     fn team(&self) -> Team {
//         self.team()
//     }

//     fn group(&self) -> Group {
//         self.group()
//     }
// }

#[async_trait]
pub trait RoleBehavior: Send + Sync {
    async fn before_ask<'a>(&mut self, _data: &GameData<'a>, _index: usize) {}

    fn before_action<'a>(&mut self, _data: &mut GameData<'a>, _index: usize) {}

    async fn ask<'a>(&mut self, _data: &GameData<'a>, _index: usize) {}

    fn action<'a>(&mut self, _data: &mut GameData<'a>, _index: usize) {}

    async fn after<'a>(&mut self, _data: &GameData<'a>, _index: usize) {}
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
