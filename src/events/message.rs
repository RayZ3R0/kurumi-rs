//! Handler for Message events.

use async_trait::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;
use tracing::{debug, instrument};

use crate::framework::command_handler::CommandHandler;
use crate::framework::event_handler::EventHandler;

/// Handles Message events sent by users.
pub struct MessageHandler {
    /// The command handler to process commands.
    command_handler: CommandHandler,
}

impl MessageHandler {
    /// Create a new MessageHandler with the given CommandHandler.
    pub fn new(command_handler: CommandHandler) -> Self {
        Self { command_handler }
    }
}

#[async_trait]
impl EventHandler for MessageHandler {
    fn event_type(&self) -> &'static str {
        "message"
    }

    #[instrument(skip(self, ctx, msg), fields(content = %msg.content, author = %msg.author.tag()))]
    async fn on_message(&self, ctx: Context, msg: &Message) {
        // Skip messages from bots
        if msg.author.bot {
            return;
        }

        debug!("Received message: {}", msg.content);

        // Process commands
        if let Err(e) = self.command_handler.handle_message(&ctx, msg).await {
            debug!("Error handling command: {:?}", e);
        }
    }
}
