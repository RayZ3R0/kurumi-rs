// Re-export command traits and types
pub mod command_trait;
pub use command_trait::{CommandError, CommandInfo, CommandResult, MessageCommand};

// Command modules
pub mod message;
pub mod slash;

// Command registry implementation
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Global command registry for storing and retrieving commands
pub struct CommandRegistry {
    message_commands: HashMap<String, Box<dyn MessageCommand>>,
}

impl CommandRegistry {
    /// Creates a new empty command registry
    pub fn new() -> Self {
        Self {
            message_commands: HashMap::new(),
        }
    }

    /// Registers a new message command
    pub fn register_message_command(&mut self, command: Box<dyn MessageCommand>) {
        self.message_commands
            .insert(command.name().to_string(), command);
    }

    /// Gets a message command by name
    pub fn get_message_command(&self, name: &str) -> Option<&Box<dyn MessageCommand>> {
        self.message_commands.get(name)
    }

    /// Returns a list of all registered message commands
    pub fn list_message_commands(&self) -> Vec<&Box<dyn MessageCommand>> {
        self.message_commands.values().collect()
    }
}

// Create a global command registry
lazy_static::lazy_static! {
    pub static ref REGISTRY: Arc<RwLock<CommandRegistry>> = Arc::new(RwLock::new(CommandRegistry::new()));
}
