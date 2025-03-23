use crate::commands::{CommandInfo, CommandResult, MessageCommand};
use async_trait::async_trait;
use serenity::{
    client::Context,
    model::{channel::Message, permissions::Permissions},
};

/// A simple ping command to test bot responsiveness
#[derive(Default)]
pub struct PingCommand;

#[async_trait]
impl CommandInfo for PingCommand {
    fn name(&self) -> &'static str {
        "ping"
    }

    fn description(&self) -> &'static str {
        "Check the bot's response time"
    }

    fn category(&self) -> &'static str {
        "Utility"
    }

    fn required_permissions(&self) -> Permissions {
        Permissions::empty() // No special permissions required
    }
}

#[async_trait]
impl MessageCommand for PingCommand {
    async fn execute(&self, ctx: &Context, msg: &Message, _args: Vec<&str>) -> CommandResult<()> {
        // Get current timestamp for latency calculation
        let start_time = msg.timestamp;

        // Send initial response
        let response = msg.channel_id.say(&ctx.http, "Pong!").await?;

        // Calculate latency
        let latency = response.timestamp.timestamp_millis() - start_time.timestamp_millis();

        // Edit message to include latency
        response
            .edit(&ctx.http, |m| {
                m.content(format!("Pong! Latency: {}ms", latency))
            })
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_info() {
        let cmd = PingCommand::default();
        assert_eq!(cmd.name(), "ping");
        assert_eq!(cmd.category(), "Utility");
        assert!(cmd.description().contains("response time"));
        assert!(cmd.required_permissions().is_empty());
        assert!(!cmd.owner_only());
    }
}
