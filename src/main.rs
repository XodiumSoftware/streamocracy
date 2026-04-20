//! Streamocracy - A simple Discord bot
//!
//! A single-purpose Discord bot built with Serenity using slash commands.
//!
//! ## Environment Variables
//!
//! - `DISCORD_TOKEN` - Your Discord bot token (required)
//! - `GUILD_ID` - Optional: Set to register commands in a specific guild for faster testing

use anyhow::Context as AnyhowContext;
use serenity::all::{
    Client, Command, Context, CreateCommand, CreateInteractionResponse,
    CreateInteractionResponseMessage, EventHandler, GatewayIntents, GuildId, Interaction, Ready,
};
use std::env;
use tracing::{error, info};

mod cmds {
    pub mod ping;
}

struct Bot;

#[serenity::async_trait]
impl EventHandler for Bot {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("Bot is connected as {}", ready.user.name);

        let guild_id = env::var("GUILD_ID")
            .ok()
            .and_then(|id| id.parse::<u64>().ok())
            .map(GuildId::new);

        let commands = vec![CreateCommand::new("ping").description("Check if bot is responsive")];

        if let Some(guild_id) = guild_id {
            match guild_id.set_commands(&ctx.http, commands.clone()).await {
                Ok(cmds) => info!("Registered {} commands in guild {}", cmds.len(), guild_id),
                Err(e) => error!("Failed to register guild commands: {}", e),
            }
        } else {
            match Command::set_global_commands(&ctx.http, commands).await {
                Ok(cmds) => info!("Registered {} global commands", cmds.len()),
                Err(e) => error!("Failed to register global commands: {}", e),
            }
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            match command.data.name.as_str() {
                "ping" => cmds::ping::run(&ctx, &command).await,
                _ => {
                    if let Err(e) = command
                        .create_response(
                            &ctx.http,
                            CreateInteractionResponse::Message(
                                CreateInteractionResponseMessage::new()
                                    .content("Unknown command")
                                    .ephemeral(true),
                            ),
                        )
                        .await
                    {
                        error!("Failed to respond to unknown command: {}", e);
                    }
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

    let intents = GatewayIntents::GUILDS;

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
