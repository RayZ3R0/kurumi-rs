//! Command modules that implement various bot commands.

pub mod general;

use crate::framework::command_handler::CommandHandler;

/// Register all commands with the command handler.
pub fn register_commands(handler: &mut CommandHandler) {
    // Register general commands
    general::register_commands(handler);

    // You can add more command categories here as they are implemented
    // admin::register_commands(handler);
    // fun::register_commands(handler);
}
