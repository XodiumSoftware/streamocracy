use crate::commands::SlashCommand;
use crate::config::Config;
use crate::utils::Utils;
use serenity::all::{
    ChannelId, CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    UserId,
};
use tracing::{error, info, warn};

/// Slash command for starting a votekick poll against a user.
pub struct VotekickCommand;

#[serenity::async_trait]
impl SlashCommand for VotekickCommand {
    fn name(&self) -> &'static str {
        "votekick"
    }

    fn register(&self, config: &Config) -> CreateCommand {
        #[allow(clippy::cast_precision_loss)]
        let min_duration = config.min_votekick_duration as f64;
        #[allow(clippy::cast_precision_loss)]
        let max_duration = config.max_votekick_duration as f64;

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
                .required(false)
                .min_number_value(min_duration)
                .max_number_value(max_duration),
            )
    }

    async fn run(&self, ctx: Context, command: CommandInteraction, config: Config) {
        if let Err(e) = self.run_internal(&ctx, &command, &config).await {
            error!("Votekick error: {}", e);
            Utils::ephemeral_response(
                &ctx.http,
                &command,
                "Failed to start the votekick. Please try again later.",
            )
            .await;
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
            .map_or_else(|| "DM".to_string(), |id| id.to_string());

        info!(
            "Command 'votekick' invoked by {} ({}) in {}",
            user.name, user.id, guild_id,
        );

        let Some(guild_id) = command.guild_id else {
            warn!("votekick used in DM by {}", user.name);
            Utils::ephemeral_response(
                &ctx.http,
                command,
                "This command can only be used in a server.",
            )
            .await;
            return Ok(());
        };

        let target_user_id = command
            .data
            .options
            .first()
            .and_then(|opt| opt.value.as_user_id())
            .expect("User option is required");

        let duration = Self::resolve_duration(
            command
                .data
                .options
                .get(1)
                .and_then(|opt| opt.value.as_i64()),
            config,
        );

        let Some(user_channel_id) = Self::get_user_voice_channel(ctx, guild_id, user.id) else {
            warn!("{} tried votekick but is not in a voice channel", user.name);
            Utils::ephemeral_response(
                &ctx.http,
                command,
                "You must be in a voice channel to use this command.",
            )
            .await;
            return Ok(());
        };

        let (target_in_same_channel, target_screensharing) =
            Self::check_target_user(ctx, guild_id, target_user_id, user_channel_id);

        if !target_in_same_channel {
            warn!(
                "Target user {} is not in the same voice channel as {}",
                target_user_id, user.name
            );
            Utils::ephemeral_response(
                &ctx.http,
                command,
                "The target user must be in the same voice channel as you.",
            )
            .await;
            return Ok(());
        }

        if !target_screensharing {
            warn!("Target user {} is not screensharing", target_user_id);
            Utils::ephemeral_response(
                &ctx.http,
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
            config.results_delete_delay,
            config.min_votekick_yes_votes,
        )
        .await?;

        Ok(())
    }

    /// Resolve the poll duration from the optional integer command argument.
    /// Falls back to the configured default and clamps to [min, max].
    fn resolve_duration(raw: Option<i64>, config: &Config) -> u64 {
        raw.map_or(config.default_votekick_duration, |v| {
            let v = u64::try_from(v.max(0)).unwrap_or(config.default_votekick_duration);
            v.clamp(config.min_votekick_duration, config.max_votekick_duration)
        })
    }

    /// Get the voice channel ID for a user in a guild.
    fn get_user_voice_channel(
        ctx: &Context,
        guild_id: serenity::all::GuildId,
        user_id: UserId,
    ) -> Option<ChannelId> {
        let guild = ctx.cache.guild(guild_id)?;
        let vs = guild.voice_states.get(&user_id)?;
        vs.channel_id
    }

    /// Check if target user is in the same channel and screensharing.
    /// Returns (`in_same_channel`, `is_screensharing`).
    fn check_target_user(
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    fn test_config() -> Config {
        Config {
            discord_token: "test".to_string(),
            guild_id: None,
            log_level: "info".to_string(),
            log_format: "pretty".to_string(),
            default_votekick_duration: 60,
            min_votekick_duration: 10,
            max_votekick_duration: 300,
            results_delete_delay: 10,
            min_votekick_yes_votes: 2,
        }
    }

    #[test]
    fn resolve_duration_uses_default_when_missing() {
        let config = test_config();
        assert_eq!(VotekickCommand::resolve_duration(None, &config), 60);
    }

    #[test]
    fn resolve_duration_clamps_to_minimum() {
        let config = test_config();
        assert_eq!(VotekickCommand::resolve_duration(Some(5), &config), 10);
        assert_eq!(VotekickCommand::resolve_duration(Some(0), &config), 10);
        assert_eq!(VotekickCommand::resolve_duration(Some(-5), &config), 10);
    }

    #[test]
    fn resolve_duration_clamps_to_maximum() {
        let config = test_config();
        assert_eq!(VotekickCommand::resolve_duration(Some(500), &config), 300);
    }

    #[test]
    fn resolve_duration_keeps_values_in_range() {
        let config = test_config();
        assert_eq!(VotekickCommand::resolve_duration(Some(30), &config), 30);
        assert_eq!(VotekickCommand::resolve_duration(Some(10), &config), 10);
        assert_eq!(VotekickCommand::resolve_duration(Some(300), &config), 300);
    }
}
