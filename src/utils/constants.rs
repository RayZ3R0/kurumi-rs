//! Constants used throughout the application.

/// The default command prefix.
pub const DEFAULT_PREFIX: &str = "!";

/// Bot version, pulled from Cargo.toml at compile time.
pub const BOT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Bot name.
pub const BOT_NAME: &str = "Kurumi";

/// Bot author(s).
pub const BOT_AUTHOR: &str = env!("CARGO_PKG_AUTHORS");

/// Default embed color (Discord blurple).
pub const DEFAULT_COLOR: u32 = 0x5865F2;

/// Error embed color (red).
pub const ERROR_COLOR: u32 = 0xED4245;

/// Success embed color (green).
pub const SUCCESS_COLOR: u32 = 0x57F287;

/// Warning embed color (yellow).
pub const WARNING_COLOR: u32 = 0xFEE75C;

/// Log date format for file names.
pub const LOG_DATE_FORMAT: &str = "%Y-%m-%d";

/// Maximum items to show in a paginated list.
pub const PAGINATION_MAX_ITEMS: usize = 10;

/// Default timeout for interactive components (in seconds).
pub const DEFAULT_COMPONENT_TIMEOUT: u64 = 60;
