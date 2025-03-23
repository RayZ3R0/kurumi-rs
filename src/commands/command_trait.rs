use async_trait::async_trait;
use serenity::{
    client::Context,
    model::{channel::Message, permissions::Permissions},
    Error as SerenityError,
};

/// Errors that can occur during command execution
#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    /// Command was not found in the registry
    #[error("Command not found")]
    NotFound,

    /// User lacks required permissions to execute the command
    #[error("Missing required permissions: {0}")]
    MissingPermissions(Permissions),

    /// Command arguments were invalid or improperly formatted
    #[error("Invalid arguments: {0}")]
    InvalidArguments(String),

    /// An error occurred during command execution
    #[error("Error executing command: {0}")]
    ExecutionError(String),

    /// An underlying Discord API error occurred
    #[error("Discord API error: {0}")]
    SerenityError(#[from] SerenityError),
}

/// Result type for command operations
pub type CommandResult<T> = Result<T, CommandError>;

/// Trait defining common properties and metadata for all commands
#[async_trait]
pub trait CommandInfo: Send + Sync {
    /// Returns the command's name, used for invocation
    fn name(&self) -> &'static str;

    /// Returns a brief description of what the command does
    fn description(&self) -> &'static str;

    /// Returns the category this command belongs to (e.g., "Utility", "Moderation")
    fn category(&self) -> &'static str;

    /// Returns the permissions required to use this command
    fn required_permissions(&self) -> Permissions;

    /// Returns whether this command can only be used by bot owners
    /// Defaults to false if not overridden
    fn owner_only(&self) -> bool {
        false
    }
}

/// Trait for implementing message-based commands
#[async_trait]
pub trait MessageCommand: CommandInfo {
    /// Executes the command with the given context, message, and arguments
    ///
    /// # Arguments
    ///
    /// * `ctx` - The Serenity context
    /// * `msg` - The message that triggered the command
    /// * `args` - Vector of command arguments (space-separated)
    ///
    /// # Returns
    ///
    /// * `CommandResult<()>` - Result indicating success or the specific error that occurred
    async fn execute(&self, ctx: &Context, msg: &Message, args: Vec<&str>) -> CommandResult<()>;
}

/// Helper function to check if a user has the required permissions
///
/// # Arguments
///
/// * `ctx` - The Serenity context
/// * `msg` - The message to check permissions for
/// * `required` - The permissions to check for
///
/// # Returns
///
/// * `CommandResult<()>` - Ok if the user has the permissions, Err otherwise
pub async fn check_permissions(
    ctx: &Context,
    msg: &Message,
    required: Permissions,
) -> CommandResult<()> {
    if let Some(member) = &msg.member {
        if let Some(permissions) = member.permissions {
            if !permissions.contains(required) {
                return Err(CommandError::MissingPermissions(required));
            }
        }
    }
    Ok(())
}
