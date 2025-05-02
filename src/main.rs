mod bot;
mod commands;
mod events;
mod framework;
mod models;
mod utils;

use std::env;

use dotenv::dotenv;
use tracing::{error, info};

use crate::bot::{load_config, load_token, Bot};

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenv().ok();

    // Initialize logging with environment variables (RUST_LOG=info,kurumi_rs=debug)
    tracing_subscriber::fmt::init();

    info!("Starting Kurumi Discord Bot...");

    // Load the Discord token
    let token = match load_token() {
        Ok(token) => token,
        Err(e) => {
            error!("Failed to load Discord token: {}", e);
            return;
        }
    };

    // Load bot configuration
    let config = match load_config() {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            return;
        }
    };

    // Create and start the bot
    let bot = Bot::new(token, config);

    // Start the bot
    if let Err(why) = bot.start().await {
        error!("Bot error: {:?}", why);
    }
}
