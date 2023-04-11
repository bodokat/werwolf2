#![allow(clippy::match_on_vec_items)]

use async_trait::async_trait;
use dyn_clone::DynClone;
use once_cell::sync::Lazy;

use std::{any::Any, fmt::Display};

use crate::game::Data;

pub static ALL_ROLES: Lazy<Vec<&dyn Role>> = Lazy::new(|| {
    let r: [Box<dyn Role>; 8] = [
        Box::new(Dorfbewohner),
        Box::new(Werwolf),
        Box::new(Seherin),
        Box::new(Unruhestifterin),
        Box::new(Freimaurer),
        Box::new(Dieb),
        Box::new(Schlaflose),
        Box::new(Doppel),
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

#[async_trait]
pub trait RoleBehavior: Send + Sync {
    async fn before_ask<'a>(&mut self, _data: &Data<'a>, _index: usize) {}

    fn before_action(&mut self, _data: &mut Data<'_>, _index: usize) {}

    async fn ask<'a>(&mut self, _data: &Data<'a>, _index: usize) {}

    fn action(&mut self, _data: &mut Data<'_>, _index: usize) {}

    async fn after<'a>(&mut self, _data: &Data<'a>, _index: usize) {}
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
