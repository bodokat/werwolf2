use std::convert::TryFrom;

use axum::extract::ws::Message;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

// note: we use "Enum::Variant()" syntax so serde_json generates a sane representation

#[derive(Serialize, Deserialize, Debug, TS, Clone)]
#[ts(export)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum ToClient {
    Welcome {
        settings: LobbySettings,
        players: Vec<String>,
    },
    NewSettings(LobbySettings),
    Joined {
        player: Player,
    },
    Left {
        player: Player,
    },
    Started,
    NameAccepted {
        name: String,
    },
    NameRejected,

    Text {
        text: String,
    },
    Question {
        id: usize,
        text: String,
        options: Vec<String>,
    },
    Ended,
}

#[derive(Serialize, Deserialize, Debug, TS)]
#[ts(export)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum ToServer {
    Start,
    Response { id: usize, choice: usize },

    Kick { player: Player },
    ChangeRoles { new_roles: Vec<usize> },
}

#[derive(Serialize, Deserialize, Debug, TS, Clone)]
#[ts(export)]
pub struct Player {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, TS, Clone)]
#[ts(export)]
pub struct LobbySettings {
    pub available_roles: Vec<String>,
    pub roles: Vec<usize>,
    pub admin: Option<String>,
}

impl From<&ToClient> for Message {
    fn from(value: &ToClient) -> Self {
        Self::Text(serde_json::to_string(value).unwrap())
    }
}

impl TryFrom<&str> for ToServer {
    type Error = serde_json::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        serde_json::from_str(value)
    }
}
