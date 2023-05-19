#![allow(non_upper_case_globals)]

use async_trait::async_trait;

use std::{any::Any, fmt::Display};

use crate::game::Data;

mod dieb;
mod doppel;
mod dorfbewohner;
mod freimaurer;
mod gunstling;
mod schlaflose;
mod seherin;
mod unruhestifterin;
mod werwolf;
pub use self::{
    dieb::Dieb, doppel::Doppel, dorfbewohner::Dorfbewohner, freimaurer::Freimaurer,
    gunstling::Gunstling, schlaflose::Schlaflose, seherin::Seherin,
    unruhestifterin::Unruhestifterin, werwolf::Werwolf,
};
pub static ALL_ROLES: &[&dyn Role] = &[
    Dieb,
    Doppel,
    Dorfbewohner,
    Freimaurer,
    Gunstling,
    Schlaflose,
    Seherin,
    Unruhestifterin,
    Werwolf,
];

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

pub trait Role: Send + Sync {
    fn build(&self) -> Box<dyn RoleBehavior>;

    fn team(&self) -> Team;

    fn group(&self) -> Group;

    fn name(&self) -> String;

    fn as_any(&self) -> &dyn Any;
}

#[async_trait]
pub trait RoleBehavior: Send + Sync {
    async fn before_ask<'a>(&mut self, _data: &Data<'a>, _index: usize) {}

    fn before_action(&mut self, _data: &mut Data<'_>, _index: usize) {}

    async fn ask<'a>(&mut self, _data: &Data<'a>, _index: usize) {}

    fn action(&mut self, _data: &mut Data<'_>, _index: usize) {}

    async fn after<'a>(&mut self, _data: &Data<'a>, _index: usize) {}
}
