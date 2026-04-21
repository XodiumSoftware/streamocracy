//! Utility functions and helpers for the Streamocracy bot

use serenity::all::{
    Command, CommandInteraction, Context, CreateCommand, CreateInteractionResponse,
    CreateInteractionResponseMessage, GuildId,
};
use tracing::{error, info};

/// Utility struct for common Discord bot operations
pub struct Utils;

impl Utils {
    /// Send an ephemeral (only visible to the user) response to a command interaction
    pub async fn ephemeral_response(
        http: &serenity::all::Http,
        command: &CommandInteraction,
        content: impl Into<String>,
    ) {
        if let Err(e) = command
            .create_response(
                http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content(content)
                        .ephemeral(true),
                ),
            )
            .await
        {
            error!("Failed to send ephemeral response: {}", e);
        }
    }

    /// Register slash commands with Discord
    /// If guild_id is provided, registers commands for that guild (instant update)
    /// Otherwise registers global commands (can take up to 1 hour to propagate)
    pub async fn register_commands(
        ctx: &Context,
        guild_id: Option<GuildId>,
        commands: Vec<CreateCommand>,
    ) {
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
}
