//! Event dispatching system for handling Discord events.

use async_trait::async_trait;
use serenity::model::gateway::Ready;
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error};

/// A trait for event handlers.
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// The event type this handler responds to.
    fn event_type(&self) -> &'static str;

    /// Handle the ready event.
    async fn on_ready(&self, _ctx: Context, _ready: &Ready) {}

    /// Handle message creation.
    async fn on_message(&self, _ctx: Context, _msg: &Message) {}

    /// Handle reaction addition.
    async fn on_reaction_add(&self, _ctx: Context, _reaction: &Reaction) {}

    /// Handle guild member join.
    async fn on_guild_member_add(&self, _ctx: Context, _guild_id: GuildId, _member: &Member) {}

    /// Handle an interaction.
    async fn on_interaction(&self, _ctx: Context, _interaction: &Interaction) {}

    // Add more event handlers as needed
}

/// Dispatches events to registered handlers.
pub struct EventDispatcher {
    /// Maps event types to their handlers.
    handlers: HashMap<&'static str, Vec<Arc<dyn EventHandler>>>,
}

impl EventDispatcher {
    /// Creates a new EventDispatcher.
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Registers an event handler.
    pub fn register_handler(&mut self, handler: impl EventHandler + 'static) {
        let handler = Arc::new(handler);
        let event_type = handler.event_type();

        self.handlers
            .entry(event_type)
            .or_insert_with(Vec::new)
            .push(handler);

        debug!("Registered handler for event type: {}", event_type);
    }

    /// Dispatches the ready event to registered handlers.
    pub async fn dispatch_ready(&self, ctx: Context, ready: &Ready) {
        if let Some(handlers) = self.handlers.get("ready") {
            for handler in handlers {
                let handler_clone = handler.clone(); // Clone the Arc to move it into the task
                let ctx_clone = ctx.clone();
                let ready_owned = ready.clone(); // Clone to owned Ready

                match tokio::spawn(
                    async move { handler_clone.on_ready(ctx_clone, &ready_owned).await },
                )
                .await
                {
                    Ok(_) => debug!("Ready event handler completed"),
                    Err(e) => error!("Ready event handler panicked: {}", e),
                }
            }
        }
    }

    /// Dispatches message events to registered handlers.
    pub async fn dispatch_message(&self, ctx: Context, msg: &Message) {
        if let Some(handlers) = self.handlers.get("message") {
            for handler in handlers {
                let handler_clone = handler.clone();
                let ctx_clone = ctx.clone();
                let msg_clone = msg.clone();

                match tokio::spawn(
                    async move { handler_clone.on_message(ctx_clone, &msg_clone).await },
                )
                .await
                {
                    Ok(_) => debug!("Message event handler completed"),
                    Err(e) => error!("Message event handler panicked: {}", e),
                }
            }
        }
    }

    /// Dispatches reaction events to registered handlers.
    pub async fn dispatch_reaction_add(&self, ctx: Context, reaction: &Reaction) {
        if let Some(handlers) = self.handlers.get("reaction_add") {
            for handler in handlers {
                let handler_clone = handler.clone();
                let ctx_clone = ctx.clone();
                let reaction_clone = reaction.clone();

                match tokio::spawn(async move {
                    handler_clone
                        .on_reaction_add(ctx_clone, &reaction_clone)
                        .await
                })
                .await
                {
                    Ok(_) => debug!("Reaction add event handler completed"),
                    Err(e) => error!("Reaction add event handler panicked: {}", e),
                }
            }
        }
    }

    /// Dispatches guild member add events to registered handlers.
    pub async fn dispatch_guild_member_add(
        &self,
        ctx: Context,
        guild_id: GuildId,
        member: &Member,
    ) {
        if let Some(handlers) = self.handlers.get("guild_member_add") {
            for handler in handlers {
                let handler_clone = handler.clone();
                let ctx_clone = ctx.clone();
                let guild_id_clone = guild_id;
                let member_clone = member.clone();

                match tokio::spawn(async move {
                    handler_clone
                        .on_guild_member_add(ctx_clone, guild_id_clone, &member_clone)
                        .await
                })
                .await
                {
                    Ok(_) => debug!("Guild member add event handler completed"),
                    Err(e) => error!("Guild member add event handler panicked: {}", e),
                }
            }
        }
    }

    /// Dispatches interaction events to registered handlers.
    pub async fn dispatch_interaction(&self, ctx: Context, interaction: &Interaction) {
        if let Some(handlers) = self.handlers.get("interaction") {
            for handler in handlers {
                let handler_clone = handler.clone();
                let ctx_clone = ctx.clone();
                let interaction_clone = interaction.clone();

                match tokio::spawn(async move {
                    handler_clone
                        .on_interaction(ctx_clone, &interaction_clone)
                        .await
                })
                .await
                {
                    Ok(_) => debug!("Interaction event handler completed"),
                    Err(e) => error!("Interaction event handler panicked: {}", e),
                }
            }
        }
    }

    // Add more dispatch methods as needed
}
