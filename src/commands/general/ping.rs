//! Ping command to check the bot's latency.

use async_trait::async_trait;
use std::time::Instant;

use crate::framework::command_handler::{Command, CommandContext, CommandResult};

/// A simple ping command that responds with the bot's latency.
pub struct PingCommand;

#[async_trait]
impl Command for PingCommand {
    fn name(&self) -> &str {
        "ping"
    }

    fn description(&self) -> &str {
        "Check the bot's latency"
    }

    async fn execute(&self, ctx: CommandContext<'_>) -> CommandResult {
        let msg = ctx.msg;
        let start = Instant::now();

        // Send an initial message
        let mut response = msg.channel_id.say(&ctx.ctx.http, "Pinging...").await?;

        // Calculate the time it took to send the message
        let latency = start.elapsed().as_millis();

        // Edit the message with the latency information
        response
            .edit(&ctx.ctx.http, |m| {
                m.content("");
                m.embed(|e| {
                    e.title("🏓 Pong!")
                        .description(format!("Latency: {}ms", latency))
                        .color(0x7289DA)
                })
            })
            .await?;

        Ok(())
    }
}
