//! Helper functions for common operations.

use chrono::Utc;
use serenity::model::channel::Message;
use serenity::model::id::UserId;
use serenity::model::timestamp::Timestamp;
use serenity::prelude::*;
use std::fmt::Display;
use std::time::{Duration, SystemTime};

use crate::utils::constants::{DEFAULT_COLOR, ERROR_COLOR, SUCCESS_COLOR, WARNING_COLOR};

// Create a wrapper struct to implement TypeMapKey for BotConfig
pub struct BotConfigKey;

impl TypeMapKey for BotConfigKey {
    type Value = crate::models::config::BotConfig;
}

/// Check if a user is a bot owner.
pub async fn is_owner(ctx: &Context, user_id: UserId) -> bool {
    let data = ctx.data.read().await;

    if let Some(config) = data.get::<BotConfigKey>() {
        return config.owners.contains(&user_id.0);
    }

    false
}

/// Format a duration into a human-readable string (e.g., "2h 15m 30s").
pub fn format_duration(duration: Duration) -> String {
    let seconds = duration.as_secs();
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;

    match (days, hours, minutes, seconds) {
        (0, 0, 0, s) => format!("{}s", s),
        (0, 0, m, s) => format!("{}m {}s", m, s),
        (0, h, m, s) => format!("{}h {}m {}s", h, m, s),
        (d, h, m, s) => format!("{}d {}h {}m {}s", d, h, m, s),
    }
}

/// Get the current timestamp as seconds since the Unix epoch.
pub fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_secs()
}

/// Get the current date as a formatted string.
pub fn current_date_formatted(format: &str) -> String {
    Utc::now().format(format).to_string()
}

/// Convert chrono::DateTime to serenity::Timestamp
fn datetime_to_timestamp(dt: chrono::DateTime<Utc>) -> Timestamp {
    // Convert to secs and nanos
    let secs = dt.timestamp();
    match Timestamp::from_unix_timestamp(secs) {
        Ok(timestamp) => timestamp,
        Err(_) => Timestamp::now(), // Use current time as fallback
    }
}

/// Send an info embed to a channel.
pub async fn send_info(
    ctx: &Context,
    msg: &Message,
    title: impl Display,
    description: impl Display,
) -> Result<Message, SerenityError> {
    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title(title)
                    .description(description)
                    .color(DEFAULT_COLOR)
                    .footer(|f| f.text(format!("Requested by {}", msg.author.tag())))
                    .timestamp(datetime_to_timestamp(Utc::now()))
            })
        })
        .await
}

/// Send a success embed to a channel.
pub async fn send_success(
    ctx: &Context,
    msg: &Message,
    description: impl Display,
) -> Result<Message, SerenityError> {
    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title("Success")
                    .description(description)
                    .color(SUCCESS_COLOR)
                    .footer(|f| f.text(format!("Requested by {}", msg.author.tag())))
                    .timestamp(datetime_to_timestamp(Utc::now()))
            })
        })
        .await
}

/// Send an error embed to a channel.
pub async fn send_error(
    ctx: &Context,
    msg: &Message,
    description: impl Display,
) -> Result<Message, SerenityError> {
    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title("Error")
                    .description(description)
                    .color(ERROR_COLOR)
                    .footer(|f| f.text(format!("Requested by {}", msg.author.tag())))
                    .timestamp(datetime_to_timestamp(Utc::now()))
            })
        })
        .await
}

/// Send a warning embed to a channel.
pub async fn send_warning(
    ctx: &Context,
    msg: &Message,
    description: impl Display,
) -> Result<Message, SerenityError> {
    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title("Warning")
                    .description(description)
                    .color(WARNING_COLOR)
                    .footer(|f| f.text(format!("Requested by {}", msg.author.tag())))
                    .timestamp(datetime_to_timestamp(Utc::now()))
            })
        })
        .await
}

/// Truncate a string to a maximum length, appending an ellipsis if necessary.
pub fn truncate(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let mut result = String::with_capacity(max_chars + 3);
        for (i, c) in s.chars().enumerate() {
            if i >= max_chars {
                break;
            }
            result.push(c);
        }
        result.push_str("...");
        result
    }
}
