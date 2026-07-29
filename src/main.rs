//! Streamocracy - A simple Discord bot
//!
//! A single-purpose Discord bot built with Serenity using slash commands.
//!
//! ## Configuration
//!
//! Configuration is loaded from environment variables.
//! A `.env` file can be used for local development.

use anyhow::Context as AnyhowContext;
use serenity::all::{Client, Context, EventHandler, GatewayIntents, Interaction, Ready};
use tracing::{error, info};

use crate::commands::get_commands;
use crate::config::Config;
use crate::utils::Utils;

mod commands;
mod config;
mod polls;
mod utils;

struct Bot {
    config: Config,
}

#[serenity::async_trait]
impl EventHandler for Bot {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("Bot is connected as {}", ready.user.name);

        let config_ref = &self.config;
        let commands: Vec<_> = get_commands()
            .iter()
            .map(|cmd| cmd.register(config_ref))
            .collect();

        Utils::register_commands(&ctx, self.config.guild_id(), commands).await;

        crate::polls::resume_polls(&ctx).await;
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            let command_name = command.data.name.clone();
            let config = self.config.clone();

            for cmd in commands::get_commands() {
                if cmd.name() == command_name.as_str() {
                    cmd.run(ctx, command, config).await;
                    return;
                }
            }

            Utils::ephemeral_response(&ctx.http, &command, "Unknown command").await;
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let config = config::init().context("Failed to initialize configuration")?;
    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_VOICE_STATES
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::GUILD_MESSAGE_REACTIONS;

    let mut client = Client::builder(&config.discord_token, intents)
        .event_handler(Bot { config })
        .await
        .context("Failed to create Discord client")?;

    info!("Starting Streamocracy Discord Bot...");

    tokio::select! {
        res = client.start() => {
            if let Err(e) = res {
                error!("Client error: {:?}", e);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Shutdown signal received, stopping bot...");
        }
    }

    Ok(())
}
