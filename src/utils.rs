//! Utility functions and helpers for the Streamocracy bot

use serenity::all::{
    Command, CommandInteraction, ComponentInteraction, Context, CreateCommand,
    CreateInteractionResponse, CreateInteractionResponseMessage, GuildId,
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

    /// Send an ephemeral response to a component interaction
    pub async fn ephemeral_component_response(
        http: &serenity::all::Http,
        interaction: &ComponentInteraction,
        content: impl Into<String>,
    ) {
        if let Err(e) = interaction
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
            error!("Failed to send component response: {}", e);
        }
    }

    /// Log command invocation with user and guild info
    pub fn log_command(command: &CommandInteraction) {
        let user = &command.user;
        let guild_id = command
            .guild_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| "DM".to_string());

        info!(
            "Command '{}' invoked by {} ({}) in {}",
            command.data.name, user.name, user.id, guild_id
        );
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

    /// Get the guild ID from an environment variable
    /// Returns None if the variable is not set or invalid
    pub fn guild_id_from_env(var_name: &str) -> Option<GuildId> {
        std::env::var(var_name)
            .ok()
            .and_then(|id| id.parse::<u64>().ok())
            .map(GuildId::new)
    }

    /// Create a standard set of bot commands
    pub fn create_command_list() -> Vec<CreateCommand> {
        vec![
            CreateCommand::new("ping").description("Check if bot is responsive"),
            CreateCommand::new("votekick").description("Start a votekick"),
            CreateCommand::new("vk").description("Alias for votekick"),
        ]
    }
}
