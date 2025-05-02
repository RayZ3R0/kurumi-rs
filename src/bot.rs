//! The main bot implementation.

use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use std::env;
use std::sync::Arc;
use tracing::{error, info};

use crate::events::{MessageHandler, ReadyHandler};
use crate::framework::command_handler::CommandHandler;
use crate::framework::event_handler::EventDispatcher;
use crate::models::BotConfig;
use crate::utils::helpers::BotConfigKey;

/// The main bot structure.
pub struct Bot {
    /// The Discord token used for authentication.
    token: String,
    /// The bot's configuration.
    config: BotConfig,
    /// The command handler for processing commands.
    command_handler: CommandHandler,
}

impl Bot {
    /// Create a new Bot instance.
    pub fn new(token: String, config: BotConfig) -> Self {
        // Create command handler with the configured prefix
        let command_handler = CommandHandler::new().with_prefix(config.prefix.clone());

        Self {
            token,
            config,
            command_handler,
        }
    }

    /// Register a command with the bot.
    pub fn register_command(
        mut self,
        command: impl crate::framework::command_handler::Command + 'static,
    ) -> Self {
        self.command_handler.register_command(command);
        self
    }

    /// Start the bot.
    pub async fn start(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Create the event handler
        let mut event_dispatcher = EventDispatcher::new();

        // Register event handlers
        event_dispatcher.register_handler(ReadyHandler);
        event_dispatcher.register_handler(MessageHandler::new(self.command_handler));

        // Set up the client with the token from environment
        let intents = GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::DIRECT_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT
            | GatewayIntents::GUILDS;

        let mut client = Client::builder(&self.token, intents)
            .event_handler_arc(Arc::new(BotEventHandler {
                dispatcher: event_dispatcher,
            }))
            .await?;

        // Add the configuration to the client data
        {
            let mut data = client.data.write().await;
            data.insert::<BotConfigKey>(self.config);
        }

        info!("Starting bot...");

        // Start listening for events
        client.start().await?;

        Ok(())
    }
}

/// Serenity event handler that dispatches events to our custom handlers.
struct BotEventHandler {
    /// The event dispatcher.
    dispatcher: EventDispatcher,
}

#[serenity::async_trait]
impl EventHandler for BotEventHandler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        self.dispatcher.dispatch_ready(ctx, &ready).await;
    }

    async fn message(&self, ctx: Context, msg: Message) {
        self.dispatcher.dispatch_message(ctx, &msg).await;
    }

    // Add more event handlers as needed
}

/// Load the bot token from the environment or a file.
pub fn load_token() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // Try to load from environment variable first
    match env::var("DISCORD_TOKEN") {
        Ok(token) => Ok(token),
        Err(_) => {
            // If that fails, try to load from a token file
            match std::fs::read_to_string(".token") {
                Ok(token) => Ok(token.trim().to_string()),
                Err(_) => Err("Discord token not found. Set the DISCORD_TOKEN environment variable or create a .token file.".into()),
            }
        }
    }
}

/// Load bot configuration from a file or create a default.
pub fn load_config() -> Result<BotConfig, Box<dyn std::error::Error + Send + Sync>> {
    let config_path = "config/config.toml";

    match BotConfig::load(config_path) {
        Ok(config) => {
            info!("Loaded configuration from {}", config_path);
            Ok(config)
        }
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            error!("Using default configuration");
            Ok(BotConfig::default())
        }
    }
}
