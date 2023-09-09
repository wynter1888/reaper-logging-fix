#![warn(clippy::pedantic)]

use serenity::{framework::StandardFramework, prelude::GatewayIntents, Client};
use sqlx::postgres::PgPoolOptions;
use tracing::{error, info};

mod commands;
mod common;
mod database;
mod events;
mod models;

#[tokio::main]
async fn main() {
    println!("cargo:rerun-if-changed=migrations");
    tracing_subscriber::fmt::init();

    info!("Getting environment variables");
    let discord_token = std::env::var("DISCORD_TOKEN").unwrap();
    let main_db_username = std::env::var("DB_USER").unwrap_or("postgres".to_string());
    let main_db_password = std::env::var("DB_PASSWORD").unwrap();
    let main_db_host = std::env::var("DB_HOST").unwrap_or("localhost".to_string());
    let main_db_port = std::env::var("DB_PORT").unwrap_or("5432".to_string());
    let main_db_name = std::env::var("DB_NAME").unwrap_or("postgres".to_string());

    // Main database connection
    let connection_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        main_db_username, main_db_password, main_db_host, main_db_port, main_db_name
    );
    info!("Establishing connection to main database");
    let main_database = PgPoolOptions::new().connect(&connection_url).await.unwrap();
    info!("Running outstanding migrations");
    sqlx::migrate!().run(&main_database).await.unwrap();
    info!("Connected to main database");

    // Discord client connection
    let handler = models::handler::Handler { main_database };
    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&discord_token, intents)
        .event_handler(handler)
        .framework(StandardFramework::new())
        .await
        .unwrap();
    if let Err(err) = client.start().await {
        error!(
            "Attempted to start Reaper Discord client, but failed with error: {}",
            err
        );
    }
}
