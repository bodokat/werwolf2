use std::convert::TryFrom;

use axum::extract::ws::Message;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
#[tsync::tsync]
pub enum ToClient {
    Welcome {
        settings: LobbySettings,
        players: Vec<String>,
    },
    NewSettings {
        settings: LobbySettings,
    },
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
#[tsync::tsync]
pub enum ToServer {
    Start,
    Response { id: usize, choice: usize },

    Kick { player: Player },
    ChangeRoles { new_roles: Vec<usize> },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[tsync::tsync]
pub struct Player {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[tsync::tsync]
pub struct LobbySettings {
    pub available_roles: Vec<String>,
    pub roles: Vec<usize>,
    #[serde(skip_serializing_if = "Option::is_none")] // required for ts to work
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
