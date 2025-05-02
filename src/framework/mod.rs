//! Core bot framework components for handling commands and events.

pub mod command_handler;
pub mod context;
pub mod event_handler;

pub use command_handler::CommandHandler;
pub use event_handler::EventDispatcher;

use std::sync::Arc;

/// Framework configuration for the bot.
pub struct Framework {
    /// Handles command registration and execution.
    pub command_handler: CommandHandler,
    /// Dispatches events to registered listeners.
    pub event_dispatcher: EventDispatcher,
}

impl Framework {
    /// Creates a new Framework instance.
    pub fn new() -> Self {
        Self {
            command_handler: CommandHandler::new(),
            event_dispatcher: EventDispatcher::new(),
        }
    }

    /// Registers all commands and event handlers.
    pub async fn register_all(&mut self) {
        // Register commands from the commands module
        crate::commands::register_commands(&mut self.command_handler);

        // Register event handlers from the events module
        let command_handler = std::mem::replace(&mut self.command_handler, CommandHandler::new());
        crate::events::register_events(&mut self.event_dispatcher, command_handler);
    }

    /// Get an Arc reference to this Framework for sharing across event handlers.
    pub fn arc(self) -> Arc<Self> {
        Arc::new(self)
    }
}
