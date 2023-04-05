use std::convert::TryFrom;

use axum::extract::ws::Message;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct Player {
    pub name: String,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct LobbySettings {
    pub available_roles: Vec<String>,
    pub roles: Vec<usize>,
    pub admin: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ToClient {
    Welcome {
        settings: LobbySettings,
        players: Vec<String>,
    },
    NewSettings(LobbySettings),
    Joined(Player),
    Left(Player),
    Started,

    Text(String),
    Question {
        id: usize,
        text: String,
        options: Vec<String>,
    },
    Ended,
}

impl From<&ToClient> for Message {
    fn from(value: &ToClient) -> Self {
        Self::Text(serde_json::to_string(value).unwrap())
    }
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ToServer {
    Response { id: usize, choice: usize },

    Start,
    Kick(Player),
    ChangeRoles(Vec<usize>),
}

impl TryFrom<&str> for ToServer {
    type Error = serde_json::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        serde_json::from_str(value)
    }
}
