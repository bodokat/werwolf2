#![warn(clippy::all)]

use serenity::{
    client::{Client, Context},
    framework::standard::{
        macros::{command, group, hook},
        CommandResult, StandardFramework,
    },
    model::channel::Message,
};

mod controller;
mod game;
mod lobby;
mod roles;
mod utils;

use controller::Controller;

pub const PREFIX: &str = "!";

#[group]
#[commands(ping)]
struct General;

#[hook]
async fn before(_ctx: &Context, msg: &Message, command_name: &str) -> bool {
    println!(
        "Running command '{}' invoked by '{}'",
        command_name,
        msg.author.tag()
    );
    true
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("failed to load .env");

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!"))
        .before(before)
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let token = std::env::var("TOKEN").expect("No Token in environment");
    let mut client = Client::builder(token)
        .framework(framework)
        .event_handler(Controller::new())
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}
