//! Streamocracy - A simple Discord bot
//!
//! A single-purpose Discord bot built with Serenity using slash commands.
//!
//! ## Configuration
//!
//! Configuration is loaded from `config.toml` in the executable directory.
//! The bot will create a default config file on first run.

use anyhow::Context as AnyhowContext;
use serenity::all::{Client, Context, EventHandler, GatewayIntents, Interaction, Ready};
use tracing::{error, info};

use crate::config::Config;
use crate::utils::Utils;

mod config;

mod ping {
    pub mod cmd;
}

mod votekick {
    pub mod cmd;
    pub mod poll;
}

mod utils;

struct Bot {
    config: Config,
}

#[serenity::async_trait]
impl EventHandler for Bot {
    async fn ready(&self,
        ctx: Context,
        ready: Ready,
    ) {
        info!("Bot is connected as {}", ready.user.name);

        let commands = Utils::create_command_list(&self.config);

        Utils::register_commands(&ctx, self.config.guild_id(), commands).await;
    }

    async fn interaction_create(&self,
        ctx: Context,
        interaction: Interaction,
    ) {
        if let Interaction::Command(command) = interaction {
            match command.data.name.as_str() {
                "ping" => ping::cmd::run(&ctx, &command).await,
                "votekick" | "vk" => {
                    votekick::cmd::run(&ctx, &command, &self.config).await
                }
                _ => {
                    Utils::ephemeral_response(
                        &ctx.http, &command, "Unknown command"
                    ).await;
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration and initialize logging
    let config = config::init().context("Failed to initialize configuration")?;

    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_VOICE_STATES
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::GUILD_MESSAGE_REACTIONS;

    let mut client = Client::builder(&config.discord_token, intents
        )
        .event_handler(Bot { config })
        .await
        .context("Failed to create Discord client")?;

    info!("Starting Streamocracy Discord Bot...");

    if let Err(e) = client.start().await {
        error!("Client error: {:?}", e);
    }

    Ok(())
}
