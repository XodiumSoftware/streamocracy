use crate::commands::SlashCommand;
use crate::config::Config;
use serenity::all::{
    ChannelId, CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    CreateInteractionResponse, CreateInteractionResponseMessage, UserId,
};
use tracing::{error, info, warn};

/// Slash command for starting a votekick poll against a user.
pub struct VotekickCommand;

#[serenity::async_trait]
impl SlashCommand for VotekickCommand {
    fn name(&self) -> &'static str {
        "votekick"
    }

    fn register(&self) -> CreateCommand {
        CreateCommand::new(self.name())
            .description("Start a votekick poll against a user")
            .add_option(
                CreateCommandOption::new(CommandOptionType::User, "user", "The user to votekick")
                    .required(true),
            )
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::Integer,
                    "duration",
                    "Poll duration in seconds",
                )
                .required(false),
            )
    }

    async fn run(&self, ctx: Context, command: CommandInteraction, config: Config) {
        if let Err(e) = self.run_internal(&ctx, &command, &config).await {
            error!("Votekick error: {}", e);
        }
    }
}

impl VotekickCommand {
    /// Internal implementation of the votekick command.
    /// Validates preconditions and starts the poll if all checks pass.
    async fn run_internal(
        &self,
        ctx: &Context,
        command: &CommandInteraction,
        config: &Config,
    ) -> anyhow::Result<()> {
        let user = &command.user;
        let guild_id = command
            .guild_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| "DM".to_string());

        info!(
            "Command 'votekick' invoked by {} ({}) in {}",
            user.name, user.id, guild_id,
        );

        let Some(guild_id) = command.guild_id else {
            warn!("votekick used in DM by {}", user.name);
            self.send_error(ctx, command, "This command can only be used in a server.")
                .await;
            return Ok(());
        };

        let target_user_id = command
            .data
            .options
            .first()
            .and_then(|opt| opt.value.as_user_id())
            .expect("User option is required");

        let duration = command
            .data
            .options
            .get(1)
            .and_then(|opt| opt.value.as_i64())
            .map(|v| {
                v.clamp(
                    config.min_votekick_duration as i64,
                    config.max_votekick_duration as i64,
                ) as u64
            })
            .unwrap_or(config.default_votekick_duration);

        let user_channel_id = match self.get_user_voice_channel(ctx, guild_id, user.id) {
            Some(cid) => cid,
            None => {
                warn!("{} tried votekick but is not in a voice channel", user.name);
                self.send_error(
                    ctx,
                    command,
                    "You must be in a voice channel to use this command.",
                )
                .await;
                return Ok(());
            }
        };

        let (target_in_same_channel, target_screensharing) =
            self.check_target_user(ctx, guild_id, target_user_id, user_channel_id);

        if !target_in_same_channel {
            warn!(
                "Target user {} is not in the same voice channel as {}",
                target_user_id, user.name
            );
            self.send_error(
                ctx,
                command,
                "The target user must be in the same voice channel as you.",
            )
            .await;
            return Ok(());
        }

        if !target_screensharing {
            warn!("Target user {} is not screensharing", target_user_id);
            self.send_error(
                ctx,
                command,
                "The target user must be screensharing to start a votekick.",
            )
            .await;
            return Ok(());
        }

        info!(
            "Votekick starting by {} targeting {} in channel {} (duration: {}s)",
            user.name, target_user_id, user_channel_id, duration
        );

        crate::polls::votekick::start_votekick(
            ctx,
            command,
            target_user_id,
            user_channel_id,
            duration,
        )
        .await;

        Ok(())
    }

    /// Get the voice channel ID for a user in a guild.
    fn get_user_voice_channel(
        &self,
        ctx: &Context,
        guild_id: serenity::all::GuildId,
        user_id: UserId,
    ) -> Option<ChannelId> {
        let guild = ctx.cache.guild(guild_id)?;
        let vs = guild.voice_states.get(&user_id)?;
        vs.channel_id
    }

    /// Check if target user is in the same channel and screensharing.
    /// Returns (in_same_channel, is_screensharing).
    fn check_target_user(
        &self,
        ctx: &Context,
        guild_id: serenity::all::GuildId,
        target_user_id: UserId,
        user_channel_id: ChannelId,
    ) -> (bool, bool) {
        let Some(guild) = ctx.cache.guild(guild_id) else {
            return (false, false);
        };
        let Some(vs) = guild.voice_states.get(&target_user_id) else {
            return (false, false);
        };
        let in_same = vs.channel_id == Some(user_channel_id);
        let screensharing = vs.self_stream.unwrap_or(false);
        (in_same, screensharing)
    }

    /// Send an ephemeral error response to the user.
    async fn send_error(&self, ctx: &Context, command: &CommandInteraction, message: &str) {
        if let Err(e) = command
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content(message)
                        .ephemeral(true),
                ),
            )
            .await
        {
            error!("Failed to send error response: {}", e);
        }
    }
}
