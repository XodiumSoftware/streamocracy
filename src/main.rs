//! Streamocracy - A simple Discord bot
//!
//! A single-purpose Discord bot built with Serenity using slash commands.
//!
//! ## Environment Variables
//!
//! - `DISCORD_TOKEN` - Your Discord bot token (required)
//! - `GUILD_ID` - Optional: Set to register commands in a specific guild for faster testing

use anyhow::Context as AnyhowContext;
use serenity::all::{Client, Context, EventHandler, GatewayIntents, Interaction, Ready};
use std::env;
use tracing::{error, info};

use crate::utils::Utils;

mod ping {
    pub mod cmd;
}

mod votekick {
    pub mod cmd;
    pub mod poll;
}

mod utils;

struct Bot;

#[serenity::async_trait]
impl EventHandler for Bot {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("Bot is connected as {}", ready.user.name);

        let guild_id = Utils::guild_id_from_env("GUILD_ID");
        let commands = Utils::create_command_list();

        Utils::register_commands(&ctx, guild_id, commands).await;
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            match command.data.name.as_str() {
                "ping" => ping::cmd::run(&ctx, &command).await,
                "votekick" | "vk" => votekick::cmd::run(&ctx, &command).await,
                _ => {
                    Utils::ephemeral_response(&ctx.http, &command, "Unknown command").await;
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new("info"))
        .init();
    dotenvy::dotenv().ok();

    let token =
        env::var("DISCORD_TOKEN").context("DISCORD_TOKEN environment variable must be set")?;

    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_VOICE_STATES
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::GUILD_MESSAGE_REACTIONS;

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
