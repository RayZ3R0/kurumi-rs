mod bot;
mod commands;
mod events;
mod framework;
mod models;
mod utils;

use std::env;

use dotenv::dotenv;
use tracing::{debug, error, info, Level};
use tracing_subscriber::FmtSubscriber;

use crate::bot::{load_config, load_token, Bot};
use crate::commands::general::ping::PingCommand;

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    if let Ok(_) = dotenv() {
        debug!("Loaded .env file");
    } else {
        debug!("No .env file found, using environment variables");
    }

    // Initialize more detailed logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");

    info!("Starting Kurumi Discord Bot...");
    debug!("Initializing bot with debug logging enabled");

    // Load the Discord token
    let token = match load_token() {
        Ok(token) => {
            let masked_token = if token.len() > 10 {
                format!("{}...{}", &token[0..6], &token[token.len() - 4..])
            } else {
                "***".to_string()
            };
            info!("Successfully loaded Discord token: {}", masked_token);
            token
        }
        Err(e) => {
            error!("Failed to load Discord token: {}", e);
            return;
        }
    };

    // Load bot configuration
    let config = match load_config() {
        Ok(config) => {
            info!("Successfully loaded configuration");
            debug!(
                "Config: prefix={}, owner count={}",
                config.prefix,
                config.owners.len()
            );
            config
        }
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            return;
        }
    };

    // Create and register commands with the bot
    info!("Registering commands...");
    let bot = Bot::new(token, config).register_command(PingCommand);

    // Start the bot
    info!("Attempting to connect to Discord...");
    if let Err(why) = bot.start().await {
        error!("Bot error: {:?}", why);
    }
}
