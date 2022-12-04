use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ToClient {
    Joined(Player),
    Left(Player),
    Message { from: Player, content: String },
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Player {
    name: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ToServer {
    Message { from: Player, content: String },
}
