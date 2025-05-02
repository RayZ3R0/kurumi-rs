//! Extended context with additional functionality.

use serenity::builder::{CreateEmbed, CreateMessage};
use serenity::model::channel::{Channel, Message};
use serenity::model::id::ChannelId;
use serenity::prelude::*;
use std::fmt::Display;

/// Extended context for the bot with additional helper methods.
pub struct BotContext<'a> {
    /// The underlying Serenity context.
    pub ctx: &'a Context,
}

impl<'a> BotContext<'a> {
    /// Create a new BotContext wrapping a Serenity Context.
    pub fn new(ctx: &'a Context) -> Self {
        Self { ctx }
    }

    /// Replies to a message with text.
    pub async fn reply(
        &self,
        msg: &Message,
        content: impl Display,
    ) -> Result<Message, SerenityError> {
        msg.channel_id.say(&self.ctx.http, content).await
    }

    /// Sends a simple embed message to the specified channel.
    pub async fn send_embed(
        &self,
        channel_id: ChannelId,
        title: impl ToString + Display,
        description: impl ToString + Display,
        color: Option<u32>,
    ) -> Result<Message, SerenityError> {
        channel_id
            .send_message(&self.ctx.http, |m| {
                m.embed(|e| {
                    e.title(title).description(description);

                    if let Some(color_value) = color {
                        e.color(color_value);
                    }

                    e
                })
            })
            .await
    }

    /// Gets a channel by ID from the cache or API.
    pub async fn get_channel(&self, channel_id: ChannelId) -> Result<Channel, SerenityError> {
        channel_id.to_channel(&self.ctx).await
    }

    /// Gets the name of the bot.
    pub async fn bot_name(&self) -> String {
        match self.ctx.http.get_current_user().await {
            Ok(user) => user.name,
            Err(_) => "Unknown Bot".to_string(),
        }
    }
}
