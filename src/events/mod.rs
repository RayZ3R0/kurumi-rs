//! Event handlers for Discord events.

mod message;
mod ready;

pub use message::MessageHandler;
pub use ready::ReadyHandler;

use crate::framework::command_handler::CommandHandler;
use crate::framework::event_handler::EventDispatcher;

/// Register all event handlers with the event dispatcher.
pub fn register_events(dispatcher: &mut EventDispatcher, command_handler: CommandHandler) {
    // Register the ready event handler
    dispatcher.register_handler(ReadyHandler);

    // Register the message event handler
    dispatcher.register_handler(MessageHandler::new(command_handler));

    // Add more event handlers here as needed
}
