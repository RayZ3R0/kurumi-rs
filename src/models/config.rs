//! Configuration models for the bot.

use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::Path;

/// Main configuration for the bot.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BotConfig {
    /// Command-specific configuration.
    #[serde(default)]
    pub commands: CommandsConfig,

    /// Logging-specific configuration.
    #[serde(default)]
    pub logging: LoggingConfig,

    /// Default command prefix.
    #[serde(default = "default_prefix")]
    pub prefix: String,

    /// Owner user IDs who have special permissions.
    #[serde(default)]
    pub owners: Vec<u64>,

    /// Whether to respond to mentions.
    #[serde(default = "default_true")]
    pub respond_to_mentions: bool,
}

/// Configuration for command handling.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommandsConfig {
    /// Whether to enable case-insensitive commands.
    #[serde(default = "default_true")]
    pub case_insensitive: bool,

    /// List of disabled commands.
    #[serde(default)]
    pub disabled: Vec<String>,

    /// Command cooldown in seconds.
    #[serde(default = "default_cooldown")]
    pub cooldown: u64,
}

/// Configuration for logging.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error).
    #[serde(default = "default_log_level")]
    pub level: String,

    /// Whether to log to file.
    #[serde(default)]
    pub file_logging: bool,

    /// Log file path.
    #[serde(default = "default_log_path")]
    pub file_path: String,
}

impl Default for BotConfig {
    fn default() -> Self {
        Self {
            commands: CommandsConfig::default(),
            logging: LoggingConfig::default(),
            prefix: default_prefix(),
            owners: Vec::new(),
            respond_to_mentions: true,
        }
    }
}

impl Default for CommandsConfig {
    fn default() -> Self {
        Self {
            case_insensitive: true,
            disabled: Vec::new(),
            cooldown: default_cooldown(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            file_logging: false,
            file_path: default_log_path(),
        }
    }
}

impl BotConfig {
    /// Load configuration from a TOML file.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, io::Error> {
        let content = fs::read_to_string(path)?;
        toml::from_str(&content).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    /// Save configuration to a TOML file.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), io::Error> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(path, content)
    }
}

// Default values

fn default_prefix() -> String {
    "!".to_string()
}

fn default_true() -> bool {
    true
}

fn default_cooldown() -> u64 {
    3
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_path() -> String {
    "logs/bot.log".to_string()
}
