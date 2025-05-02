# Serenity Framework Documentation - Page 3

## Advanced Features and Modern Discord Integration

Building on the foundations covered in Pages 1 and 2, this final page explores advanced integration with Discord's modern features, database connectivity, web interaction, and more specialized bot functionality.

## Table of Contents

1. Parallel Tasks and Background Loops
2. Slash Commands and Interactions
3. Web Dashboards
4. Database Integration
5. Message Components
6. Webhooks
7. Interactions Endpoint
8. Best Practices

---

## Parallel Tasks and Background Loops

Discord bots often need to perform periodic tasks such as updating status messages, cleaning databases, or fetching external data. Serenity enables this through Tokio's async runtime.

### Setting Up Background Tasks

```rust
// Create a flag to ensure loops start only once
struct Handler {
    is_loop_running: AtomicBool,
}

#[async_trait]
impl EventHandler for Handler {
    // cache_ready is a good event to start background loops
    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        // Prevent multiple instances of the loop
        if !self.is_loop_running.load(Ordering::Relaxed) {
            // Clone context for thread safety
            let ctx = Arc::new(ctx);
            let ctx_clone = Arc::clone(&ctx);

            // Start the first background task
            tokio::spawn(async move {
                loop {
                    // Do periodic work here
                    perform_task(&ctx_clone).await;
                    tokio::time::sleep(Duration::from_secs(120)).await;
                }
            });

            // Set the flag to true to prevent duplicate loops
            self.is_loop_running.swap(true, Ordering::Relaxed);
        }
    }
}
```

### Common Background Tasks

#### Status Rotation

```rust
// Rotate through different status messages
let ctx_clone = Arc::clone(&ctx);
tokio::spawn(async move {
    let statuses = vec![
        "Type !help",
        "Serving servers",
        "Version 1.0.0",
    ];
    let mut counter = 0;

    loop {
        let status = statuses[counter % statuses.len()];
        ctx_clone.set_activity(Some(ActivityData::playing(status)));

        counter += 1;
        tokio::time::sleep(Duration::from_secs(30)).await;
    }
});
```

#### System Monitoring

```rust
async fn log_system_load(ctx: &Context) {
    // Get system metrics
    let cpu_load = sys_info::loadavg().unwrap();
    let mem_use = sys_info::mem_info().unwrap();

    // Create an embed with the data
    let embed = CreateEmbed::new()
        .title("System Resource Load")
        .field("CPU Load Average", format!("{:.2}%", cpu_load.one * 10.0), false)
        .field(
            "Memory Usage",
            format!(
                "{:.2} MB Free out of {:.2} MB",
                mem_use.free as f32 / 1000.0,
                mem_use.total as f32 / 1000.0
            ),
            false,
        );

    // Send to a specific channel
    let builder = CreateMessage::new().embed(embed);
    let log_channel_id = ChannelId::new(123456789);

    if let Err(why) = log_channel_id.send_message(ctx, builder).await {
        eprintln!("Error sending log message: {why:?}");
    }
}
```

### Task Cancellation

For clean shutdown, you should provide a way to cancel background tasks:

```rust
// Use a watch channel for graceful termination
let (shutdown_send, mut shutdown_recv) = tokio::sync::watch::channel(false);

// In your background task
tokio::spawn(async move {
    loop {
        tokio::select! {
            // Check for shutdown signal
            _ = shutdown_recv.changed() => {
                if *shutdown_recv.borrow() {
                    println!("Shutting down background task");
                    break;
                }
            }
            // Normal task work
            _ = async {
                perform_task().await;
                tokio::time::sleep(Duration::from_secs(60)).await;
            } => {}
        }
    }
});

// When shutting down
shutdown_send.send(true).unwrap();
```

> **Note**: Always consider rate limits when performing periodic operations. Discord enforces strict rate limits, and exceeding them can result in temporary IP bans.

---

## Slash Commands and Interactions

Discord's modern API emphasizes slash commands and interactive components. This section covers how to implement them with Serenity.

### Registering Slash Commands

There are two types of slash commands in Discord:

- **Global commands**: Available in all servers where your bot has permissions
- **Guild-specific commands**: Only available in specific servers (useful for testing)

```rust
async fn ready(&self, ctx: Context, ready: Ready) {
    println!("{} is connected!", ready.user.name);

    // Register guild-specific commands (faster updates, good for testing)
    let guild_id = GuildId::new(123456789);
    let commands = guild_id
        .set_commands(&ctx.http, vec![
            commands::ping::register(),
            commands::welcome::register(),
        ])
        .await
        .unwrap();

    // Register global commands (can take up to an hour to propagate)
    let global_command =
        Command::create_global_command(&ctx.http, commands::help::register())
            .await
            .unwrap();
}
```

### Command Registration Structure

Each command is typically defined in its own module with registration and execution functions:

```rust
// In commands/ping.rs
pub fn register() -> CreateCommand {
    CreateCommand::new("ping")
        .description("Checks the bot's latency")
}

pub fn run(_options: &[CommandDataOption]) -> String {
    "Pong!".to_string()
}
```

### Handling Command Interactions

Listen for interactions in the `interaction_create` event handler:

```rust
async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
    if let Interaction::Command(command) = interaction {
        println!("Received command interaction: {command:#?}");

        let content = match command.data.name.as_str() {
            "ping" => Some(commands::ping::run(&command.data.options())),
            "welcome" => Some(commands::welcome::run(&command.data.options())),
            _ => Some("Command not implemented".to_string()),
        };

        if let Some(content) = content {
            // Create and send the response
            let data = CreateInteractionResponseMessage::new().content(content);
            let builder = CreateInteractionResponse::Message(data);

            if let Err(why) = command.create_response(&ctx.http, builder).await {
                println!("Cannot respond to slash command: {why}");
            }
        }
    }
}
```

### Command Options and Arguments

Discord slash commands support various option types which are set during registration:

```rust
pub fn register() -> CreateCommand {
    CreateCommand::new("welcome")
        .description("Welcome a user")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::User,
                "user",
                "The user to welcome"
            )
            .required(true)
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "message",
                "The welcome message"
            )
            .required(false)
            .add_string_choice("Formal", "Welcome to our server!")
            .add_string_choice("Casual", "Hey there, welcome!")
        )
}
```

Parsing options in the command handler:

```rust
pub fn run(options: &[CommandDataOption]) -> String {
    let user = options
        .iter()
        .find(|opt| opt.name == "user")
        .and_then(|opt| opt.resolved.as_ref())
        .and_then(|resolved| {
            if let CommandDataOptionValue::User(user, _) = resolved {
                Some(user)
            } else {
                None
            }
        })
        .unwrap();

    let message = options
        .iter()
        .find(|opt| opt.name == "message")
        .and_then(|opt| opt.resolved.as_ref())
        .and_then(|resolved| {
            if let CommandDataOptionValue::String(message) = resolved {
                Some(message.as_str())
            } else {
                None
            }
        })
        .unwrap_or("Welcome to the server!");

    format!("{}: {}", user.mention(), message)
}
```

### Modal Interactions

Modals are forms that can be presented to users when they use commands:

```rust
// Command registration
pub fn register() -> CreateCommand {
    CreateCommand::new("modal")
        .description("Shows a modal dialog")
}

// Command handler
pub async fn run(ctx: &Context, command: &CommandInteraction) -> Result<(), Error> {
    // Create a custom ID for the modal
    let custom_id = format!("modal_test_{}", command.id);

    // Build the modal with text input components
    let modal = CreateModal::new(custom_id, "My Cool Modal")
        .add_action_row(CreateActionRow::new().add_text_input(
            CreateInputText::new(
                TextInputStyle::Short,
                "name",
                "Name"
            )
            .placeholder("Enter your name")
            .min_length(1)
            .max_length(32)
            .required(true),
        ))
        .add_action_row(CreateActionRow::new().add_text_input(
            CreateInputText::new(
                TextInputStyle::Paragraph,
                "feedback",
                "Feedback"
            )
            .placeholder("Your thoughts here...")
            .min_length(10)
            .max_length(1000)
            .required(true),
        ));

    // Present the modal to the user
    command
        .create_response(
            ctx,
            CreateInteractionResponse::Modal(modal),
        )
        .await?;

    Ok(())
}
```

### Responding to Interactions

There are several ways to respond to interactions:

1. **Immediate Response** (required within 3 seconds):

```rust
command
    .create_response(
        &ctx,
        CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .content("Processing your request...")
                .ephemeral(true), // Only visible to the user who triggered it
        ),
    )
    .await?;
```

2. **Deferred Response** (gives you up to 15 minutes to respond):

```rust
// First defer the response
command
    .create_response(
        &ctx,
        CreateInteractionResponse::Defer(
            CreateInteractionResponseMessage::new().ephemeral(true)
        ),
    )
    .await?;

// Perform long operation
tokio::time::sleep(Duration::from_secs(5)).await;

// Now send the follow-up
command
    .create_followup(
        &ctx,
        CreateInteractionFollowup::new()
            .content("Here's your processed result!")
            .ephemeral(true),
    )
    .await?;
```

> **Important**: You must respond to an interaction within 3 seconds, or Discord will consider the interaction failed. If you need more time, use a deferred response.

---

## Web Dashboards

For advanced bots, a web dashboard can provide monitoring and configuration capabilities. Serenity integrates well with the `rillrate` crate for this purpose.

### Setting Up RillRate

```rust
// Install RillRate and open the dashboard automatically
fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Configure logging
    env::set_var(
        "RUST_LOG",
        "info,my_bot=trace,meio=warn,rate_core=warn,rill_engine=warn",
    );
    tracing_subscriber::fmt::init();

    // Start RillRate server on http://localhost:6361/
    rillrate::install("my_bot")?;

    // Open the dashboard in browser
    let _ = webbrowser::open("http://localhost:6361");

    // Rest of bot initialization...
}
```

### Creating Dashboard Elements

RillRate provides various components for visualizing data:

```rust
// For TypeMap storage of dashboard components
struct RillRateComponents;

impl TypeMapKey for RillRateComponents {
    // RillRate elements have internal mutability
    type Value = Arc<Components>;
}

struct Components {
    // Various dashboards and metrics
    ws_ping_history: Pulse,
    command_usage_table: Table,
    config_switch: Switch,
    value_slider: Slider,
}

// Initialize components
let components = Arc::new(Components {
    // Create a graph/chart for websocket ping
    ws_ping_history: Pulse::new(
        ["Bot Dashboards", "Statistics", "Latency", "Websocket Ping"],
        Default::default(),
        PulseOpts::default()
            .retain(1800) // 30 minutes of data
            .min(0)
            .max(200)
            .suffix("ms".to_string())
    ),

    // Create a table for command usage
    command_usage_table: Table::new(
        ["Bot Dashboards", "Statistics", "Commands", "Usage"],
        Default::default(),
        TableOpts::default().columns(vec![
            (0, "Command".to_string()),
            (1, "Uses".to_string())
        ])
    ),

    // Create interactive config elements
    config_switch: Switch::new(
        ["Bot Dashboards", "Config", "Features", "Toggle Feature"],
        SwitchOpts::default().label("Enable feature")
    ),

    value_slider: Slider::new(
        ["Bot Dashboards", "Config", "Parameters", "Message Limit"],
        SliderOpts::default()
            .min(1.0)
            .max(100.0)
            .step(1.0)
    ),
});

// Add to client
let mut client = Client::builder(token, intents)
    .event_handler(Handler)
    .type_map_insert::<RillRateComponents>(components)
    .await?;
```

### Updating Dashboard Data

```rust
// Update a graph with new data
let ctx_clone = ctx.clone();
tokio::spawn(async move {
    let elements = {
        let data_read = ctx_clone.data.read().await;
        data_read.get::<RillRateComponents>().unwrap().clone()
    };

    loop {
        // Get websocket latency
        let ws_latency = {
            let data_read = ctx_clone.data.read().await;
            let shard_manager = data_read.get::<ShardManagerContainer>().unwrap();
            let runners = shard_manager.runners.lock().await;
            let runner = runners.get(&ctx_clone.shard_id).unwrap();

            runner.latency.map_or(f64::NAN, |d| d.as_millis() as f64)
        };

        // Push to the graph
        elements.ws_ping_history.push(ws_latency);

        tokio::time::sleep(Duration::from_secs(30)).await;
    }
});
```

### Handling Dashboard Interactions

When users interact with dashboard controls, you need to handle those events:

```rust
let switch = Switch::new(
    ["Bot Dashboards", "Config", "Features", "Toggle Feature"],
    SwitchOpts::default().label("Feature toggle")
);
let switch_clone = switch.clone();

let ctx_clone = ctx.clone();
tokio::spawn(async move {
    let elements = {
        let data_read = ctx_clone.data.read().await;
        data_read.get::<RillRateComponents>().unwrap().clone()
    };

    // Handle interactions with the switch
    switch.sync_callback(move |envelope| {
        if let Some(action) = envelope.action {
            // Update the internal state
            elements.feature_enabled.store(action, Ordering::Relaxed);

            // Update the UI to reflect the state
            switch_clone.apply(action);

            println!("Feature toggled to: {}", action);
        }

        Ok(())
    });
});
```

> **Note**: RillRate is a good option for personal or development dashboards, but for production bots serving many users, consider a more robust web framework with proper authentication.

---

## Database Integration

Most bots require persistent storage. Serenity works well with various database libraries, with SQLx being a common choice for SQL databases.

### Setting Up SQLx with SQLite

```rust
// In Cargo.toml
// sqlx = { version = "0.6", features = ["runtime-tokio-rustls", "sqlite", "migrate"] }

// Bot structure with database connection
struct Bot {
    database: sqlx::SqlitePool,
}

#[async_trait]
impl EventHandler for Bot {
    // Event handlers with database access
}

async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize database connection
    let database = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename("database.sqlite")
                .create_if_missing(true),
        )
        .await?;

    // Run migrations from ./migrations directory
    sqlx::migrate!("./migrations").run(&database).await?;

    // Create bot with database access
    let bot = Bot { database };

    // Initialize client with our bot as the event handler
    let mut client = Client::builder(&token, intents)
        .event_handler(bot)
        .await?;

    client.start().await?;

    Ok(())
}
```

### Database Migrations

SQLx supports migrations for schema changes. Create a `migrations` directory with numbered SQL files:

```
migrations/
  20220101000000_initial.sql
  20220201000000_add_user_preferences.sql
```

Example migration file:

```sql
-- 20220101000000_initial.sql
-- Create initial tables
CREATE TABLE IF NOT EXISTS todo (
    rowid INTEGER PRIMARY KEY AUTOINCREMENT,
    task TEXT NOT NULL,
    user_id INTEGER NOT NULL
);

-- Add indexes
CREATE INDEX IF NOT EXISTS todo_user_id_idx ON todo(user_id);
```

### Executing Queries

```rust
async fn message(&self, ctx: Context, msg: Message) {
    let user_id = msg.author.id.get() as i64;

    if let Some(task_description) = msg.content.strip_prefix("~todo add") {
        let task_description = task_description.trim();

        // Insert into database
        sqlx::query!(
            "INSERT INTO todo (task, user_id) VALUES (?, ?)",
            task_description,
            user_id,
        )
        .execute(&self.database)
        .await
        .unwrap();

        msg.channel_id.say(&ctx, format!(
            "Successfully added `{task_description}` to your todo list"
        )).await.unwrap();
    }
}
```

### Reading from Database

```rust
// Fetch a single row
let entry = sqlx::query!(
    "SELECT rowid, task FROM todo WHERE user_id = ? ORDER BY rowid LIMIT 1 OFFSET ?",
    user_id,
    task_index,
)
.fetch_one(&self.database) // Returns a single row
.await
.unwrap();

// Fetch multiple rows
let todos = sqlx::query!(
    "SELECT task FROM todo WHERE user_id = ? ORDER BY rowid",
    user_id
)
.fetch_all(&self.database) // Returns all matching rows
.await
.unwrap();

// Process results
let mut response = format!("You have {} pending tasks:\n", todos.len());
for (i, todo) in todos.iter().enumerate() {
    response.push_str(&format!("{}. {}\n", i + 1, todo.task));
}
```

### Transactions

For multiple related operations, use transactions:

```rust
// Start a transaction
let mut tx = self.database.begin().await?;

// Perform multiple operations
sqlx::query!(
    "INSERT INTO inventory (user_id, item_id, quantity) VALUES (?, ?, ?)",
    user_id,
    item_id,
    1
)
.execute(&mut tx)
.await?;

sqlx::query!(
    "UPDATE users SET money = money - ? WHERE id = ?",
    item_price,
    user_id
)
.execute(&mut tx)
.await?;

// Commit the transaction (or it will be rolled back if dropped)
tx.commit().await?;
```

> **Important**: Always use parameterized queries to prevent SQL injection attacks. Never concatenate user input directly into SQL strings.

---

## Message Components

Message components make Discord bots interactive with buttons, select menus, and more.

### Buttons

Create messages with button components:

```rust
// Create a message with buttons
let message = msg
    .channel_id
    .send_message(
        &ctx,
        CreateMessage::new()
            .content("Please make a selection")
            .button(CreateButton::new("btn_accept").label("Accept").style(ButtonStyle::Success))
            .button(CreateButton::new("btn_decline").label("Decline").style(ButtonStyle::Danger))
            .button(
                CreateButton::new("btn_maybe")
                    .label("Maybe")
                    .style(ButtonStyle::Secondary)
                    .emoji('🤔')
            )
    )
    .await?;
```

### Select Menus

```rust
// Create a message with a select menu
let message = msg
    .channel_id
    .send_message(
        &ctx,
        CreateMessage::new()
            .content("Choose your favorite animal")
            .select_menu(
                CreateSelectMenu::new(
                    "animal_select",
                    CreateSelectMenuKind::String {
                        options: vec![
                            CreateSelectMenuOption::new("🐈 Cat", "cat"),
                            CreateSelectMenuOption::new("🐕 Dog", "dog"),
                            CreateSelectMenuOption::new("🐎 Horse", "horse"),
                            CreateSelectMenuOption::new("🦙 Alpaca", "alpaca"),
                            CreateSelectMenuOption::new("🦀 Ferris", "ferris"),
                        ],
                    }
                )
                .placeholder("Select an animal")
            )
    )
    .await?;
```

### Listening for Component Interactions

Using collectors to wait for user interaction:

```rust
// Wait for a button click
if let Some(interaction) = message
    .await_component_interaction(&ctx.shard)
    .timeout(Duration::from_secs(60))
    .await
{
    // Get the custom ID of the clicked button
    let custom_id = &interaction.data.custom_id;

    // Respond to the interaction
    interaction
        .create_response(
            &ctx,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(format!("You clicked {}", custom_id))
                    .ephemeral(true)
            )
        )
        .await?;
}
```

### Waiting for Select Menu Choice

```rust
// Wait for a selection
if let Some(interaction) = message
    .await_component_interaction(&ctx.shard)
    .timeout(Duration::from_secs(60))
    .await
{
    // Extract the selected value
    let selected = match &interaction.data.kind {
        ComponentInteractionDataKind::StringSelect { values } => &values[0],
        _ => panic!("Expected string select menu interaction"),
    };

    // Acknowledge the interaction and update the message
    interaction
        .create_response(
            &ctx,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new()
                    .content(format!("You selected: {}", selected))
            )
        )
        .await?;
}
```

### Multiple Interactions

For handling multiple sequential interactions:

```rust
// Create a stream of interactions
let mut interaction_stream = message
    .await_component_interactions(&ctx.shard)
    .timeout(Duration::from_secs(120))
    .stream();

// Process interactions as they arrive
while let Some(interaction) = interaction_stream.next().await {
    match interaction.data.custom_id.as_str() {
        "btn_next" => {
            // Show next item
            interaction
                .create_response(
                    &ctx,
                    CreateInteractionResponse::UpdateMessage(
                        CreateInteractionResponseMessage::new()
                            .content("Next page")
                            .button(CreateButton::new("btn_prev").label("Previous"))
                            .button(CreateButton::new("btn_next").label("Next"))
                    )
                )
                .await?;
        },
        "btn_prev" => {
            // Show previous item
            interaction
                .create_response(
                    &ctx,
                    CreateInteractionResponse::UpdateMessage(
                        CreateInteractionResponseMessage::new()
                            .content("Previous page")
                            .button(CreateButton::new("btn_prev").label("Previous"))
                            .button(CreateButton::new("btn_next").label("Next"))
                    )
                )
                .await?;
        },
        "btn_cancel" => {
            // End the interaction
            interaction
                .create_response(
                    &ctx,
                    CreateInteractionResponse::UpdateMessage(
                        CreateInteractionResponseMessage::new()
                            .content("Interaction cancelled")
                            .components(vec![]) // Remove all components
                    )
                )
                .await?;
            break; // Exit the loop
        },
        _ => continue,
    }
}
```

> **Important**: Always clean up component messages when done by either deleting them or clearing the components, otherwise users might click on stale components and receive an error.

---

## Webhooks

Webhooks provide a way to send messages to Discord channels without a bot user being visible.

### Using Existing Webhooks

```rust
use serenity::model::webhook::Webhook;
use serenity::builder::ExecuteWebhook;

async fn send_webhook_message() -> Result<(), Box<dyn std::error::Error>> {
    // Create HTTP client (no token needed for webhooks)
    let http = Http::new("");

    // Load a webhook from its URL
    let webhook = Webhook::from_url(
        &http,
        "https://discord.com/api/webhooks/123456789/your-webhook-token"
    ).await?;

    // Execute the webhook
    let builder = ExecuteWebhook::new()
        .content("Hello from a webhook!")
        .username("Custom Name")
        .avatar_url("https://example.com/avatar.png");

    webhook.execute(&http, false, builder).await?;

    Ok(())
}
```

### Creating New Webhooks

```rust
async fn create_webhook(ctx: &Context, channel_id: ChannelId) -> Result<Webhook, SerenityError> {
    channel_id.create_webhook(&ctx.http, "My Bot Webhook").await
}
```

### Webhook with Embeds and Files

```rust
// Create webhook message with embed and file
let builder = ExecuteWebhook::new()
    .content("Check out this data!")
    .username("Analytics Bot")
    .embed(CreateEmbed::new()
        .title("Weekly Statistics")
        .description("Here are your weekly stats")
        .field("Users", "1,234", true)
        .field("Messages", "5,678", true)
        .color(0x3498db)
    )
    .add_file(CreateAttachment::path("./stats.png").await?);

webhook.execute(&http, false, builder).await?;
```

---

## Interactions Endpoint

For certain applications (especially those using serverless platforms), you may want to handle Discord interactions via HTTP endpoint rather than through the gateway.

### Setting Up an HTTP Server

```rust
use serenity::interactions_endpoint::Verifier;
use serenity::model::application::Interaction;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Create a verifier with your application's public key
    let verifier = Verifier::new("your_public_key_from_discord_developer_portal");

    // Setup HTTP server
    let server = tiny_http::Server::http("0.0.0.0:8000")?;
    let mut body = Vec::new();

    println!("Server listening on port 8000");

    loop {
        let request = server.recv()?;
        if let Err(e) = handle_request(request, &mut body, &verifier) {
            eprintln!("Error handling request: {}", e);
        }
    }
}
```

### Handling Interaction Requests

```rust
fn handle_request(
    mut request: tiny_http::Request,
    body: &mut Vec<u8>,
    verifier: &Verifier,
) -> Result<(), Error> {
    println!("Received request from {:?}", request.remote_addr());

    // Read the request body
    body.clear();
    request.as_reader().read_to_end(body)?;

    // Verify the request is from Discord
    let find_header = |name| {
        Some(request.headers().iter().find(|h| h.field.equiv(name))?.value.as_str())
    };
    let signature = find_header("X-Signature-Ed25519").ok_or("missing signature")?;
    let timestamp = find_header("X-Signature-Timestamp").ok_or("missing timestamp")?;

    if verifier.verify(signature, timestamp, body).is_err() {
        request.respond(tiny_http::Response::empty(401))?;
        return Ok(());
    }

    // Parse the interaction
    let response = match json::from_slice::<Interaction>(body)? {
        // Must acknowledge ping interactions for Discord to verify endpoint
        Interaction::Ping(_) => CreateInteractionResponse::Pong,

        // Handle command interactions
        Interaction::Command(interaction) => handle_command(interaction),

        // Handle other interaction types
        _ => return Ok(()),
    };

    // Send response back to Discord
    request.respond(
        tiny_http::Response::from_data(json::to_vec(&response)?)
            .with_header("Content-Type: application/json".parse::<tiny_http::Header>().unwrap()),
    )?;

    Ok(())
}
```

### Command Handler

```rust
fn handle_command(interaction: CommandInteraction) -> CreateInteractionResponse {
    match interaction.data.name.as_str() {
        "ping" => CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .content("Pong!")
        ),
        "echo" => {
            let content = interaction.data.options
                .iter()
                .find(|opt| opt.name == "message")
                .and_then(|opt| opt.resolved.as_ref())
                .and_then(|resolved| {
                    if let CommandDataOptionValue::String(content) = resolved {
                        Some(content.clone())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "No message provided".to_string());

            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(content)
            )
        },
        _ => CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .content("Unknown command")
                .ephemeral(true)
        ),
    }
}
```

> **Important**: When using an HTTP endpoint, you need to configure your application in the Discord Developer Portal with the URL of your endpoint.

---

## Best Practices

### Error Handling

Implement thorough error handling to prevent crashes:

```rust
async fn handle_command(ctx: &Context, msg: &Message) -> CommandResult {
    match do_something_risky().await {
        Ok(result) => {
            msg.channel_id.say(&ctx.http, format!("Success: {}", result)).await?;
        }
        Err(e) => {
            // Log detailed error internally
            error!("Command failed: {:?}", e);

            // Give user friendly message
            msg.channel_id
                .say(&ctx.http, "Sorry, something went wrong. Please try again later.")
                .await?;
        }
    }

    Ok(())
}
```

### Rate Limit Handling

Respect Discord's rate limits, especially for bulk operations:

```rust
async fn send_messages_to_users(ctx: &Context, users: Vec<UserId>, content: &str) {
    for user in users {
        match user.create_dm_channel(&ctx.http).await {
            Ok(channel) => {
                if let Err(e) = channel.say(&ctx.http, content).await {
                    error!("Failed to send DM to {}: {:?}", user, e);
                }

                // Add delay between messages to avoid rate limits
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
            Err(e) => error!("Failed to create DM channel for {}: {:?}", user, e),
        }
    }
}
```

### Memory Management

For large bots, consider memory usage with caches:

```rust
// Configure client with cache settings
let mut client = ClientBuilder::new(&token, intents)
    .event_handler(Handler)
    .cache_settings({
        let mut settings = CacheSettings::default();
        settings.max_messages = 100; // Only cache 100 messages per channel
        settings.message_ttl = Some(Duration::from_secs(3600)); // 1 hour TTL
        settings
    })
    .await?;
```

### Structured Logging

Use tracing for structured and contextual logs:

```rust
#[instrument(skip(ctx, msg), fields(user = %msg.author.name, channel = %msg.channel_id))]
async fn process_command(ctx: &Context, msg: &Message) -> CommandResult {
    debug!("Processing command: {}", msg.content);

    // Command logic
    let result = perform_operation().await;

    if result.is_ok() {
        info!("Command completed successfully");
    } else {
        error!("Command failed: {:?}", result.unwrap_err());
    }

    Ok(())
}
```

### Managing Privileged Intents

For bots in 100+ servers, you'll need to request privileged intents:

```rust
// Only request what you need
let intents = GatewayIntents::GUILDS
    | GatewayIntents::GUILD_MESSAGES
    | GatewayIntents::MESSAGE_CONTENT; // Privileged

// Adapt your code to work even if intents are denied
if !ctx.cache.guild_members.is_empty() {
    // We have member data, do precise operations
} else {
    // We don't have member data, use fallback logic
}
```

### Sharding Considerations

For large bots, plan your sharding strategy:

```rust
// Automatic sharding based on bot size
client.start_autosharded().await?;

// Manual sharding for special needs
let shard_count = 10;
client.start_shards(shard_count).await?;
```

### Application Architecture

Separate your bot into logical modules:

```
src/
  main.rs           # Bot initialization
  commands/         # Command implementations
    mod.rs          # Module exports
    admin.rs        # Admin commands
    fun.rs          # Entertainment commands
    utility.rs      # Utility commands
  services/         # Business logic
    database.rs     # Database interactions
    external_api.rs # External API calls
  models/           # Data structures
  utils/            # Helper functions
```

### Security Practices

1. **Environment Variables**: Never hardcode tokens, use environment variables or secure vaults
2. **Input Validation**: Always validate user input before processing
3. **Permission Checks**: Check permissions before executing privileged operations
4. **Secure Storage**: Use secure methods for storing sensitive user data

### Performance Optimization

1. **Cache Usage**: Utilize Serenity's cache for frequently accessed data
2. **Batch Operations**: Group database operations into batches
3. **Async Efficiency**: Use `join_all` for parallel operations
4. **Connection Pooling**: Use connection pools for databases
5. **Resource Limiting**: Implement timeouts and resource limits for user operations

---
