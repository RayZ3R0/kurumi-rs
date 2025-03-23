use super::CommandInfo;
use crate::commands::{CommandError, CommandResult};
use async_trait::async_trait;
use serenity::{client::Context, model::channel::Message, model::permissions::Permissions};

/// Trait for implementing message-based commands
#[async_trait]
pub trait MessageCommand: CommandInfo {
    /// Execute the command with given context and arguments
    async fn execute(&self, ctx: &Context, msg: &Message, args: Vec<&str>) -> CommandResult<()>;
}

/// Helper function to parse command arguments
pub fn parse_args(content: &str, prefix: &str) -> Option<(String, Vec<&str>)> {
    if !content.starts_with(prefix) {
        return None;
    }

    let content = content.trim_start_matches(prefix).trim();
    let mut parts = content.split_whitespace();

    if let Some(command) = parts.next() {
        let args: Vec<&str> = parts.collect();
        return Some((command.to_string(), args));
    }

    None
}

/// Helper function to check if a message is a valid command
pub fn is_command(msg: &Message, prefix: &str) -> bool {
    msg.content.starts_with(prefix)
}

/// Helper function to handle basic command validation
pub async fn validate_command(
    ctx: &Context,
    msg: &Message,
    required_permissions: Permissions,
    owner_only: bool,
    owners: &[u64],
) -> CommandResult<()> {
    // Check if command is owner only
    if owner_only && !owners.contains(&msg.author.id.0) {
        return Err(CommandError::MissingPermissions(Permissions::empty()));
    }

    // Check user permissions if in guild
    if let Some(guild_id) = msg.guild_id {
        if let Some(member) = guild_id.member(&ctx.http, msg.author.id).await? {
            let permissions = member.permissions?;
            if !permissions.contains(required_permissions) {
                return Err(CommandError::MissingPermissions(required_permissions));
            }
        }
    }

    Ok(())
}

// Command implementations will be added in separate modules
mod ping;

// Re-export commands
pub use ping::PingCommand;

/// Get all available message commands
pub fn get_commands() -> Vec<Box<dyn MessageCommand>> {
    vec![
        Box::new(PingCommand::default()),
        // Add more commands here as they are implemented
    ]
}

/// Find a command by name
pub fn get_command(name: &str) -> Option<Box<dyn MessageCommand>> {
    get_commands().into_iter().find(|cmd| cmd.name() == name)
}
