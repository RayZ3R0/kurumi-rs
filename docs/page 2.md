# Serenity Framework Documentation - Page 2

## Advanced Concepts and Implementations

Building upon the foundational knowledge from Page 1, this documentation explores more advanced aspects of the Serenity framework for Discord bot development in Rust.

## Table of Contents

1. Global Data Management
2. Logging and Instrumentation
3. Shard Management
4. Rich Message Building
5. Collectors
6. Gateway Intents

---

## Global Data Management

One of the most powerful features of Serenity is its ability to share data between different parts of your bot using a thread-safe global state container.

### TypeMap Basics

Serenity uses a `TypeMap` to store shared data that can be accessed from any event handler or command:

```rust
// Define a type to use as a key for accessing data
struct CommandCounter;

// Implement the TypeMapKey trait
impl TypeMapKey for CommandCounter {
    // Define what type of value will be stored
    type Value = Arc<RwLock<HashMap<String, u64>>>;
}
```

### Data Storage Patterns

There are several common patterns for storing data in the TypeMap:

1. **Read/Write Data** (most common):

```rust
// For data that needs to be modified
struct Counter;
impl TypeMapKey for Counter {
    type Value = Arc<RwLock<HashMap<String, u64>>>;
}
```

2. **Atomic Data** (for simple counters):

```rust
// For simple atomic operations (no locks needed for modification)
struct MessageCount;
impl TypeMapKey for MessageCount {
    type Value = Arc<AtomicUsize>;
}
```

3. **Read-Only Data** (for configuration):

```rust
// For configuration data that doesn't change
struct Config;
impl TypeMapKey for Config {
    type Value = Arc<BotConfig>;
}
```

### Initializing Data

Initialize your global data when creating the client:

```rust
// In main()
{
    let mut data = client.data.write().await;
    data.insert::<CommandCounter>(Arc::new(RwLock::new(HashMap::default())));
    data.insert::<MessageCount>(Arc::new(AtomicUsize::new(0)));
}
```

Alternatively, use the builder pattern:

```rust
let mut client = Client::builder(&token, intents)
    .type_map_insert::<CommandCounter>(Arc::new(RwLock::new(HashMap::default())))
    .event_handler(Handler)
    .await
    .expect("Error creating client");
```

### Accessing Data

To access the data, follow these best practices to avoid deadlocks:

```rust
async fn some_function(ctx: &Context) {
    // Step 1: Get the data with a read lock, clone the Arc
    let counter_lock = {
        let data_read = ctx.data.read().await;
        data_read.get::<CommandCounter>().expect("Expected CommandCounter in TypeMap").clone()
    };

    // Step 2: Now we can acquire the inner lock and modify the data
    {
        let mut counter = counter_lock.write().await;
        let entry = counter.entry("command_name".to_string()).or_insert(0);
        *entry += 1;
    }
}
```

### Atomic Operations

For simple counters, use atomic operations to avoid locks entirely:

```rust
async fn increment_counter(ctx: &Context, amount: usize) {
    let counter = {
        let data_read = ctx.data.read().await;
        data_read.get::<MessageCount>().expect("Expected MessageCount in TypeMap").clone()
    };

    // No need for a write lock
    counter.fetch_add(amount, Ordering::SeqCst);
}
```

### Avoiding Deadlocks

- Keep locks open for the shortest time possible
- Use read locks when possible, only use write locks when necessary
- Use scoping blocks `{ }` to ensure locks are dropped when they're no longer needed
- Always acquire the outer TypeMap lock first, then inner locks

---

## Logging and Instrumentation

Serenity works well with the `tracing` ecosystem for structured logging.

### Setup

Include these dependencies in your Cargo.toml:

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = "0.2"
```

Initialize the tracing subscriber in your main function:

```rust
// Initialize the logger with environment variables
tracing_subscriber::fmt::init();

// Can be configured via the RUST_LOG environment variable
// e.g., RUST_LOG=info,serenity=debug
```

### Log Levels

Tracing provides several log levels:

- `error!()` - Error conditions that require attention
- `warn!()` - Warning conditions that might require attention
- `info!()` - Informational messages about normal operation
- `debug!()` - Detailed information for debugging
- `trace!()` - Very detailed trace information

### Usage Examples

```rust
// In event handlers
async fn ready(&self, _: Context, ready: Ready) {
    info!("{} is connected!", ready.user.name);
}

// In command handlers
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    debug!("Executing ping command");

    if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
        error!("Error sending message: {:?}", why);
    }

    Ok(())
}
```

### Instrumenting Functions

The `#[instrument]` attribute allows for detailed function tracing:

```rust
#[instrument(skip(self, ctx))]
async fn resume(&self, ctx: Context, resume: ResumedEvent) {
    debug!("Resumed");
}

#[hook]
#[instrument]
async fn before(_: &Context, msg: &Message, command_name: &str) -> bool {
    info!("Got command '{}' by user '{}'", command_name, msg.author.name);
    true
}
```

> Note: The `instrument` macro requires all parameters to implement Debug, or be explicitly skipped.

---

## Shard Management

Sharding is Discord's way of distributing the load of a bot across multiple connections. For bots in a large number of guilds (2500+), sharding is required.

### Understanding Shards

- Each shard handles a subset of guilds
- Shards operate on separate WebSocket connections
- Discord requires 1 shard per 2,500 guilds

### Basic Sharding

```rust
// Start 2 shards
if let Err(why) = client.start_shards(2).await {
    println!("Client error: {:?}", why);
}
```

### Transparent Sharding

Serenity implements transparent sharding, meaning you don't need to manually handle shard-specific logic:

```rust
// Accessing the current shard ID
async fn message(&self, ctx: Context, msg: Message) {
    println!("This message is being processed on shard {}", ctx.shard_id);
}
```

### ShardManager

The `ShardManager` allows runtime interaction with shards:

```rust
// Clone the ShardManager for use in a separate task
let shard_manager = client.shard_manager.clone();

// Spawn a task to monitor shard status
tokio::spawn(async move {
    loop {
        tokio::time::sleep(Duration::from_secs(30)).await;

        let shard_runners = shard_manager.runners.lock().await;

        for (id, runner) in shard_runners.iter() {
            println!(
                "Shard ID {} is {} with a latency of {:?}",
                id, runner.stage, runner.latency,
            );
        }
    }
});
```

### Shard Control

With the ShardManager, you can control individual shards:

```rust
// Restart a specific shard
shard_manager.restart(1).await;

// Shutdown all shards gracefully
shard_manager.shutdown_all().await;
```

### Shard Identification

When receiving events, you can check which shard the event came from:

```rust
async fn ready(&self, _: Context, ready: Ready) {
    if let Some(shard) = ready.shard {
        println!(
            "{} is connected on shard {}/{}!",
            ready.user.name,
            shard.id,
            shard.total
        );
    }
}
```

---

## Rich Message Building

Serenity provides builders for creating rich messages with embeds, files, and components.

### CreateMessage Builder

Use the `CreateMessage` builder for complete messages:

```rust
use serenity::builder::{CreateAttachment, CreateEmbed, CreateEmbedFooter, CreateMessage};
use serenity::model::Timestamp;

// Build a rich message with content, embed, and file attachment
let footer = CreateEmbedFooter::new("This is a footer");
let embed = CreateEmbed::new()
    .title("This is a title")
    .description("This is a description")
    .image("attachment://image.png")
    .fields(vec![
        ("Field 1", "This is field 1", true),
        ("Field 2", "This is field 2", true),
    ])
    .field("Field 3", "This is field 3", false)
    .footer(footer)
    .timestamp(Timestamp::now());

let message = CreateMessage::new()
    .content("Hello, World!")
    .embed(embed)
    .add_file(CreateAttachment::path("./image.png").await.unwrap());

// Send the message
channel_id.send_message(&ctx.http, message).await?;
```

### Embed Customization

Embeds can be extensively customized:

```rust
let embed = CreateEmbed::new()
    .title("Title")
    .description("Description")
    .url("https://example.com")
    .color(0x3498db)  // Blue color
    .author(|a| a
        .name("Author Name")
        .icon_url("https://example.com/avatar.png")
    )
    .thumbnail("https://example.com/thumbnail.png")
    .image("https://example.com/image.png")
    .fields(vec![
        ("Field 1", "Value 1", true),
        ("Field 2", "Value 2", true),
    ])
    .footer(|f| f
        .text("Footer text")
        .icon_url("https://example.com/footer-icon.png")
    )
    .timestamp(Timestamp::now());
```

### File Attachments

You can add file attachments to messages:

```rust
// Add a file from a path
let attachment = CreateAttachment::path("./file.txt").await.unwrap();

// Add a file from bytes
let bytes = vec![0, 1, 2, 3];
let attachment = CreateAttachment::bytes(bytes, "file.bin");

// Add to message
let message = CreateMessage::new()
    .content("Here's a file")
    .add_file(attachment);
```

### Message Reference (Replies)

Create a reply to a message:

```rust
let message = CreateMessage::new()
    .content("This is a reply")
    .reference_message(&original_message)
    .mention_replied_user(true);
```

---

## Collectors

Collectors allow you to await specific events like messages or reactions within your command flow.

### Message Collectors

Collect messages from a specific user:

```rust
use serenity::collector::MessageCollector;
use serenity::futures::stream::StreamExt;

// Collect up to 5 messages from a specific user in a specific channel within 30 seconds
let collector = MessageCollector::new(&ctx.shard)
    .author_id(user_id)
    .channel_id(channel_id)
    .timeout(Duration::from_secs(30))
    .build()
    .stream()
    .take(5);

// Process each collected message
let collected_messages: Vec<_> = collector.collect().await;

println!("Collected {} messages", collected_messages.len());
```

### Reaction Collectors

Collect reactions on a specific message:

```rust
// Wait for a reaction from a specific user on a specific message
let reaction = msg
    .await_reaction(&ctx.shard)
    .author_id(user_id)
    .timeout(Duration::from_secs(30))
    .await;

if let Some(reaction) = reaction {
    println!("Received reaction: {}", reaction.emoji.as_data());
}
```

### Reply Collectors

A convenience method to collect a single reply:

```rust
// Wait for a reply from the message author
let reply = msg
    .author
    .await_reply(&ctx.shard)
    .timeout(Duration::from_secs(10))
    .await;

if let Some(reply) = reply {
    println!("User replied: {}", reply.content);
} else {
    println!("User didn't reply within the timeout period");
}
```

### Generic Event Collectors

Collect any type of event:

```rust
use serenity::collector::collect;
use serenity::model::prelude::Event;

// Collect MessageUpdate events for specific message IDs
let mut collector = collect(&ctx.shard, move |event| match event {
    Event::MessageUpdate(event) if message_ids.contains(&event.id) => {
        Some(event.id)
    },
    _ => None,
})
.take_until(Box::pin(tokio::time::sleep(Duration::from_secs(20))));

// Process each collected event
while let Some(message_id) = collector.next().await {
    println!("Message {} was edited", message_id);
}
```

### Interactive Commands

Collectors are perfect for creating interactive commands:

```rust
async fn quiz(ctx: &Context, msg: &Message) -> CommandResult {
    let _ = msg.reply(ctx, "What is the capital of France?").await?;

    // Wait for the user's answer
    let reply = msg
        .author
        .await_reply(&ctx.shard)
        .timeout(Duration::from_secs(20))
        .await;

    if let Some(answer) = reply {
        if answer.content.to_lowercase() == "paris" {
            answer.reply(ctx, "Correct!").await?;
        } else {
            answer.reply(ctx, "Wrong! The answer is Paris.").await?;
        }
    } else {
        msg.reply(ctx, "You didn't answer in time.").await?;
    }

    Ok(())
}
```

---

## Gateway Intents

Gateway Intents control which events your bot receives from Discord, helping to reduce unnecessary traffic and improve performance.

### Basic Intents Setup

```rust
// Define which events your bot needs to receive
let intents = GatewayIntents::GUILD_MESSAGES
    | GatewayIntents::DIRECT_MESSAGES
    | GatewayIntents::MESSAGE_CONTENT;

// Use in client builder
let mut client = Client::builder(&token, intents)
    .event_handler(Handler)
    .await
    .expect("Error creating client");
```

### Common Intent Combinations

1. **Basic Bot** (message commands only):

```rust
let intents = GatewayIntents::GUILD_MESSAGES
    | GatewayIntents::DIRECT_MESSAGES
    | GatewayIntents::MESSAGE_CONTENT;
```

2. **Reaction-based Bot**:

```rust
let intents = GatewayIntents::GUILD_MESSAGES
    | GatewayIntents::GUILD_MESSAGE_REACTIONS;
```

3. **Member Tracking Bot**:

```rust
let intents = GatewayIntents::GUILDS
    | GatewayIntents::GUILD_MEMBERS // Privileged intent
    | GatewayIntents::GUILD_PRESENCES; // Privileged intent
```

4. **Full-featured Bot**:

```rust
let intents = GatewayIntents::GUILDS
    | GatewayIntents::GUILD_MESSAGES
    | GatewayIntents::DIRECT_MESSAGES
    | GatewayIntents::MESSAGE_CONTENT
    | GatewayIntents::GUILD_MESSAGE_REACTIONS
    | GatewayIntents::GUILD_MEMBERS;
```

### Privileged Intents

Some intents require approval from Discord for bots in 100+ servers:

- `GUILD_MEMBERS` - Required for accessing member lists and member events
- `GUILD_PRESENCES` - Required for tracking user presence updates
- `MESSAGE_CONTENT` - Required for accessing message content for bots verified after April 2022

To use these, you must enable them in the Discord Developer Portal under your application's "Bot" tab.

### Event Handlers and Intents

Events will only be dispatched if the bot has the corresponding intent:

```rust
#[async_trait]
impl EventHandler for Handler {
    // This requires GUILD_MESSAGES or DIRECT_MESSAGES intent
    async fn message(&self, ctx: Context, msg: Message) {
        println!("Received message: {}", msg.content);
    }

    // This requires GUILD_PRESENCES intent (privileged)
    async fn presence_update(&self, ctx: Context, new_data: Presence) {
        println!("Presence update for user: {}", new_data.user.id);
    }

    // This requires GUILD_MEMBERS intent (privileged)
    async fn guild_member_addition(&self, ctx: Context, new_member: Member) {
        println!("New member joined: {}", new_member.user.name);
    }
}
```

### Minimizing Intents

For better performance, only request the intents your bot needs:

```rust
// For a simple command bot that doesn't need to see every message
let intents = GatewayIntents::GUILDS // Server-related events
    | GatewayIntents::GUILD_MESSAGES // Only for server messages
    | GatewayIntents::MESSAGE_CONTENT; // Access to message content
```
