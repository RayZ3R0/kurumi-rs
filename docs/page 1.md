# Serenity Framework Documentation - Page 1

## Introduction to Serenity

Serenity is a powerful Rust library for creating Discord bots, providing a high-level, async interface to the Discord API. This documentation covers the core concepts, architecture, and usage patterns of the Serenity framework.

## Table of Contents

1. Setting Up a Serenity Bot
2. Core Concepts
3. Event Handling
4. Command Framework
5. Advanced Features

---

## Setting Up a Serenity Bot

### Dependencies

To use Serenity, add it to your Cargo.toml:

```toml
[dependencies]
serenity = { version = "0.11", features = ["client", "gateway", "rustls_backend", "model"] }
tokio = { version = "1", features = ["full"] }
dotenv = "0.15"
tracing = "0.1"
tracing-subscriber = "0.2"
```

### Minimum Viable Bot

Here's the minimal code needed to create a functioning Discord bot:

```rust
use std::env;
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    // Load token from environment or .env file
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // Define gateway intents
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create client
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    // Start the client
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
```

### Environment Setup

Create a .env file in your project root:

```
DISCORD_TOKEN=your_bot_token_here
```

Add this file to .gitignore to prevent committing your token.

---

## Core Concepts

### Client

The `Client` is the main interface to Discord. It:

- Establishes and maintains the WebSocket connection
- Manages sharding
- Processes events
- Provides access to the HTTP API and cache

### Context

The `Context` struct provides access to various parts of the bot:

- `ctx.http` - HTTP client for making API requests
- `ctx.cache` - In-memory cache of Discord objects
- `ctx.data` - Shared state container for your application

### TypeMap and Data Sharing

Serenity provides a thread-safe way to share data across your bot using `TypeMap`:

```rust
// Define type for shared data
struct Counter;
impl TypeMapKey for Counter {
    type Value = Arc<RwLock<i32>>;
}

// Initialize in main()
{
    let mut data = client.data.write().await;
    data.insert::<Counter>(Arc::new(RwLock::new(0)));
}

// Access in event handlers or commands
async fn some_command(ctx: &Context, _: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let counter = data.get::<Counter>().unwrap().read().unwrap();
    println!("Counter: {}", *counter);
    Ok(())
}
```

### Gateway Intents

Gateway Intents control what events your bot receives from Discord:

```rust
let intents = GatewayIntents::GUILD_MESSAGES  // Receive messages in servers
    | GatewayIntents::DIRECT_MESSAGES         // Receive direct messages
    | GatewayIntents::MESSAGE_CONTENT         // Access to message content (privileged)
    | GatewayIntents::GUILD_MEMBERS;          // Member events (privileged)
```

Privileged intents must be enabled in the Discord Developer Portal.

---

## Event Handling

### Event Handler

The `EventHandler` trait defines methods that are called when Discord events occur:

```rust
#[async_trait]
impl EventHandler for Handler {
    // Called when a message is received
    async fn message(&self, ctx: Context, msg: Message) {
        // Handle message
    }

    // Called when bot connects successfully
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }

    // Called when a user's presence updates
    async fn presence_update(&self, ctx: Context, new_data: PresenceUpdate) {
        // Handle presence update
    }

    // Many more event handlers available
}
```

### Important Events

- `ready` - Bot connected and ready
- `message` - Message created
- `reaction_add` - Reaction added to message
- `guild_member_addition` - User joins a server
- `voice_state_update` - Voice channel status changes
- `interaction_create` - Slash command or button interaction

---

## Command Framework

Serenity offers two command frameworks:

1. Standard Framework (older, shown in examples)
2. Poise (newer, recommended)

### Standard Framework Setup

```rust
use serenity::framework::standard::{
    macros::{command, group},
    CommandResult, StandardFramework,
};

#[group]
#[commands(ping, echo)]
struct General;

// Create the framework
let framework = StandardFramework::new()
    .configure(|c| c
        .prefix("!")
        .owners(owners)
        .allow_dm(true)
    )
    .group(&GENERAL_GROUP);

// Add to client
let mut client = Client::builder(&token, intents)
    .framework(framework)
    .event_handler(Handler)
    .await
    .expect("Err creating client");
```

### Creating Commands

```rust
#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "Pong!").await?;
    Ok(())
}

#[command]
async fn echo(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let content = args.rest();
    if content.is_empty() {
        msg.reply(&ctx.http, "Please provide some text to echo!").await?;
    } else {
        msg.channel_id.say(&ctx.http, content).await?;
    }
    Ok(())
}
```

### Command Attributes

Commands can be customized with attributes:

```rust
#[command]
#[description("Shows help information")]
#[only_in(guilds)]  // Only usable in servers, not DMs
#[aliases("h", "commands")]  // Alternative command names
#[usage = "!help [command]"]  // Usage information
#[example = "!help ping"]  // Example usage
#[bucket = "general"]  // Rate limit bucket
async fn help(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    // Command implementation
    Ok(())
}
```

### Command Groups

Groups organize commands and can have settings:

```rust
#[group]
#[prefixes("math", "m")]  // Access via !math or !m
#[description = "Math commands"]
#[default_command(calculate)]  // Used when no subcommand specified
#[commands(add, subtract, multiply, divide)]
struct Math;
```

### Custom Checks

Checks control whether commands can be executed:

```rust
#[check]
#[name = "Admin"]
async fn admin_check(
    ctx: &Context,
    msg: &Message,
    _: &mut Args,
    _: &CommandOptions
) -> Result<(), Reason> {
    // Check if user has admin role
    if let Some(member) = &msg.member {
        for role in &member.roles {
            if role.name.to_lowercase().contains("admin") {
                return Ok(());
            }
        }
    }

    Err(Reason::User("You need admin permissions".to_string()))
}

// Apply check to command
#[command]
#[checks(Admin)]
async fn restricted_command(ctx: &Context, msg: &Message) -> CommandResult {
    // Only admins can run this
    Ok(())
}
```

### Hooks

Hooks are special functions that run at different points in command processing:

```rust
#[hook]
async fn before(ctx: &Context, msg: &Message, command_name: &str) -> bool {
    println!("Running command: {}", command_name);
    true // Continue command execution
}

#[hook]
async fn after(ctx: &Context, msg: &Message, cmd_name: &str, result: CommandResult) {
    match result {
        Ok(()) => println!("Command '{}' executed successfully", cmd_name),
        Err(why) => println!("Command '{}' returned error: {:?}", cmd_name, why),
    }
}

// Add to framework
let framework = StandardFramework::new()
    .before(before)
    .after(after)
    .unrecognised_command(unknown_command)
    .group(&GENERAL_GROUP);
```

### Rate Limiting

Control command usage with buckets:

```rust
// Create bucket
let framework = StandardFramework::new()
    .bucket("emoji", |b| b.delay(5)).await // 5 second cooldown
    .bucket("complex", |b| b
        .limit(2)       // Max 2 uses
        .time_span(30)  // Per 30 seconds
        .delay(5)       // 5 second cooldown between uses
        .limit_for(LimitedFor::User) // Per user
    ).await
    .group(&GENERAL_GROUP);

// Apply to command
#[command]
#[bucket = "emoji"]
async fn cat(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, ":cat:").await?;
    Ok(())
}
```

---

## Advanced Features

### Sharding

For larger bots (reaching many servers), Discord requires sharding:

```rust
// Automatically manage 2 shards
if let Err(why) = client.start_shards(2).await {
    println!("Client error: {:?}", why);
}
```

Serenity handles sharding transparently, distributing guilds across shards.

### Message Building

Create rich messages with `MessageBuilder`:

```rust
let response = MessageBuilder::new()
    .push("Hello, ")
    .mention(&msg.author)
    .push("!")
    .push_line("")
    .push_bold("Welcome to our server!")
    .push_line("")
    .push_italic("Check out our rules channel.")
    .build();

msg.channel_id.say(&ctx.http, response).await?;
```

### Content Safety

Safely echo user input without triggering mentions:

```rust
let content = content_safe(
    &ctx.cache,
    user_input,
    &ContentSafeOptions::default()
        .clean_role(true)
        .clean_user(true)
        .clean_channel(false),
    &msg.mentions
);
```

### Help Command

Customize the help command:

```rust
#[help]
#[individual_command_tip = "Hello! Type `!help command` for more info on a command."]
#[command_not_found_text = "Could not find: `{}`."]
#[max_levenshtein_distance(3)]
#[indention_prefix = "+"]
#[lacking_permissions = "Hide"]
#[lacking_role = "Nothing"]
#[wrong_channel = "Strike"]
async fn my_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>
) -> CommandResult {
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}

// Add to framework
let framework = StandardFramework::new()
    .help(&MY_HELP)
    .group(&GENERAL_GROUP);
```

### Permissions

Check for Discord permissions:

```rust
if let Some(member) = &msg.member {
    if member.permissions(&ctx).await?.administrator() {
        msg.reply(ctx, "You are an administrator!").await?;
    } else if member.permissions(&ctx).await?.manage_messages() {
        msg.reply(ctx, "You can manage messages!").await?;
    } else {
        msg.reply(ctx, "You don't have special permissions.").await?;
    }
}
```

### Shutdown Handling

Gracefully shutdown on Ctrl+C:

```rust
let shard_manager = client.shard_manager.clone();

tokio::spawn(async move {
    tokio::signal::ctrl_c().await.expect("Could not register ctrl+c handler");
    shard_manager.shutdown_all().await;
});
```

### Owner Commands

Create commands only bot owners can use:

```rust
#[group]
#[owners_only]
#[summary = "Commands for bot owners"]
#[commands(shutdown)]
struct Owner;

#[command]
async fn shutdown(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Shutting down...").await?;

    let data = ctx.data.read().await;
    let shard_manager = data.get::<ShardManagerContainer>().unwrap();
    shard_manager.shutdown_all().await;

    Ok(())
}
```
