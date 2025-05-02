//! Handler for the Ready event.

use async_trait::async_trait;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use tracing::{error, info};

use crate::framework::event_handler::EventHandler;
use crate::utils::helpers::BotConfigKey;

/// Handles the Ready event, which is sent when the bot connects to Discord.
pub struct ReadyHandler;

#[async_trait]
impl EventHandler for ReadyHandler {
    fn event_type(&self) -> &'static str {
        "ready"
    }

    async fn on_ready(&self, ctx: Context, ready: &Ready) {
        info!("Bot is ready! Logged in as {}", ready.user.name);

        // Load configuration
        let bot_name = &ready.user.name;
        let guild_count = ready.guilds.len();

        // Get bot configuration
        let config = {
            let data_read = ctx.data.read().await;
            match data_read.get::<BotConfigKey>() {
                Some(config) => format!("Command prefix: {}", config.prefix),
                None => "No configuration loaded".to_string(),
            }
        };

        info!("{} is connected to {} servers", bot_name, guild_count);
        info!("{}", config);
    }
}
