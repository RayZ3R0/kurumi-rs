//! General utility commands for the bot.

mod ping;

use crate::framework::command_handler::CommandHandler;

/// Register all general commands with the command handler.
pub fn register_commands(handler: &mut CommandHandler) {
    // Register the ping command
    handler.register_command(ping::PingCommand);

    // Add more general commands here as they're implemented
    // handler.register_command(help::HelpCommand);
}
