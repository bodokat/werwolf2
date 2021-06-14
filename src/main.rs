#![warn(clippy::all)]

use serenity::client::Client;

mod controller;
mod game;
mod lobby;
mod roles;
mod utils;

use controller::Controller;

pub const PREFIX: &str = "!";

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("failed to load .env");

    // Login with a bot token from the environment
    let token = std::env::var("TOKEN").expect("No Token in environment");
    let mut client = Client::builder(token)
        .event_handler(Controller::new())
        .application_id(775071223083696188)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
