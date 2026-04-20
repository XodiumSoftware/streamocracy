//! Streamocracy - A simple Discord bot
//!
//! A single-purpose Discord bot built with Serenity.
//!
//! ## Environment Variables
//!
//! - `DISCORD_TOKEN` - Your Discord bot token (required)

use anyhow::Context as AnyhowContext;
use serenity::all::{Client, GatewayIntents, EventHandler, Context, Ready, Message};
use std::env;
use tracing::{error, info};

struct Bot;

#[serenity::async_trait]
impl EventHandler for Bot {
    async fn ready(&self,
        _ctx: Context,
        ready: Ready,
    ) {
        info!("Bot is connected as {}", ready.user.name);
    }

    async fn message(&self,
        ctx: Context,
        msg: Message,
    ) {
        // Ignore messages from bots
        if msg.author.bot {
            return;
        }

        // Simple ping command
        if msg.content == "!ping" {
            if let Err(e) = msg.channel_id.say(&ctx.http, "Pong! 🏓").await {
                error!("Failed to send message: {}", e);
            }
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load .env file if present
    dotenvy::dotenv().ok();

    // Get Discord token
    let token = env::var("DISCORD_TOKEN")
        .context("DISCORD_TOKEN environment variable must be set")?;

    // Configure bot intents
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create and start the client
    let mut client = Client::builder(&token, intents)
        .event_handler(Bot)
        .await
        .context("Failed to create Discord client")?;

    info!("Starting Streamocracy Discord Bot...");

    if let Err(e) = client.start().await {
        error!("Client error: {:?}", e);
    }

    Ok(())
}
