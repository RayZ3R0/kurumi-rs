//! Command registration and execution system.

use async_trait::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, instrument};

use crate::utils::constants::DEFAULT_PREFIX;

/// Result type for command functions.
pub type CommandResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

/// Context passed to command execution functions.
pub struct CommandContext<'a> {
    /// The Serenity context.
    pub ctx: &'a Context,
    /// The message that triggered the command.
    pub msg: &'a Message,
    /// Command arguments (space-separated words after the command).
    pub args: Vec<String>,
    /// Data passed from the framework.
    pub data: &'a TypeMap,
}

/// Trait for implementing commands.
#[async_trait]
pub trait Command: Send + Sync {
    /// The command name (used for invocation).
    fn name(&self) -> &str;

    /// Optional description of the command.
    fn description(&self) -> &str {
        ""
    }

    /// Optional usage information.
    fn usage(&self) -> &str {
        ""
    }

    /// Optional list of aliases for the command.
    fn aliases(&self) -> Vec<&str> {
        vec![]
    }

    /// Execute the command.
    async fn execute(&self, ctx: CommandContext<'_>) -> CommandResult;
}

/// Handles command registration and execution.
pub struct CommandHandler {
    /// Maps command names to command implementations.
    commands: HashMap<String, Arc<dyn Command>>,
    /// Maps command aliases to their primary name.
    aliases: HashMap<String, String>,
    /// Command prefix.
    prefix: String,
}

impl CommandHandler {
    /// Creates a new CommandHandler with the default prefix.
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
            aliases: HashMap::new(),
            prefix: DEFAULT_PREFIX.to_string(),
        }
    }

    /// Sets the command prefix.
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self
    }

    /// Registers a command.
    pub fn register_command(&mut self, command: impl Command + 'static) {
        let command = Arc::new(command);
        let name = command.name().to_lowercase();

        // Register main command
        self.commands.insert(name.clone(), command.clone()); // Fixed: Using clone() directly

        // Register aliases
        for alias in command.aliases() {
            let alias = alias.to_lowercase();
            self.aliases.insert(alias, name.clone());
        }

        debug!(
            "Registered command: {} with {} aliases",
            name,
            command.aliases().len()
        );
    }

    /// Checks if a message is a command and executes it.
    #[instrument(skip(self, ctx, msg), fields(command))]
    pub async fn handle_message(&self, ctx: &Context, msg: &Message) -> CommandResult {
        // Skip messages from bots
        if msg.author.bot {
            return Ok(());
        }

        // Check if message starts with prefix
        if !msg.content.starts_with(&self.prefix) {
            return Ok(());
        }

        // Parse command name and arguments
        let content = msg.content.trim_start_matches(&self.prefix);
        let mut args = content.split_whitespace();

        let cmd_name = match args.next() {
            Some(name) => name.to_lowercase(),
            None => return Ok(()),
        };

        // Find command by name or alias
        let command_name = self.aliases.get(&cmd_name).unwrap_or(&cmd_name);
        let command = match self.commands.get(command_name) {
            Some(cmd) => cmd,
            None => return Ok(()), // Command not found
        };

        // Collect remaining arguments
        let arguments: Vec<String> = args.map(String::from).collect();

        // Create command context
        let data = ctx.data.read().await;
        let cmd_ctx = CommandContext {
            ctx,
            msg,
            args: arguments,
            data: &data,
        };

        // Execute command
        debug!("Executing command: {}", command_name);
        match command.execute(cmd_ctx).await {
            Ok(()) => {
                debug!("Command {} executed successfully", command_name);
            }
            Err(e) => {
                error!("Command {} failed with error: {:?}", command_name, e);
                // You could send an error message to the channel here
            }
        }

        Ok(())
    }

    /// Get a list of all registered command names.
    pub fn command_names(&self) -> Vec<String> {
        self.commands.keys().cloned().collect()
    }

    /// Get the current command prefix.
    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    /// Get a command by name.
    pub fn get_command(&self, name: &str) -> Option<Arc<dyn Command>> {
        let name = name.to_lowercase();

        // Check for direct command
        if let Some(cmd) = self.commands.get(&name) {
            return Some(Arc::clone(cmd));
        }

        // Check aliases
        if let Some(primary_name) = self.aliases.get(&name) {
            if let Some(cmd) = self.commands.get(primary_name) {
                return Some(Arc::clone(cmd));
            }
        }

        None
    }
}
