//! Votekick poll implementation

use crate::polls::{Poll, PollInfo, schedule_poll_completion, send_temporary_message};
use anyhow::Context as AnyhowContext;
use serenity::all::{
    ChannelId, CommandInteraction, Context, CreateEmbed, GuildId, MessageId, Permissions, UserId,
};
use serenity::prelude::Mentionable;
use tracing::{error, info, warn};

/// Metadata for active votekicks.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[allow(clippy::struct_field_names)]
pub struct VotekickMetadata {
    /// Target user ID
    pub target_user_id: UserId,
    /// Guild where the votekick was started
    pub guild_id: GuildId,
    /// Voice channel the target was in
    pub channel_id: ChannelId,
    /// Name of the votekick initiator
    pub initiator_name: String,
    /// Name of the target user
    pub target_name: String,
    /// Poll duration in seconds
    pub duration_secs: u64,
    /// How long result messages remain before deletion, in seconds
    pub results_delete_delay_secs: u64,
    /// Minimum yes votes needed for the votekick to pass
    pub min_yes_votes: u32,
}

/// A poll for voting to kick a user from a voice channel.
pub struct VotekickPoll {
    /// The user who initiated the votekick
    pub initiator_name: String,
    /// Target user's display name
    pub target_name: String,
    /// Poll duration in seconds
    pub duration_secs: u64,
    /// How long result messages remain before deletion, in seconds
    pub results_delete_delay_secs: u64,
    /// Target user ID
    pub target_user_id: UserId,
    /// Guild where the votekick was started
    pub guild_id: GuildId,
    /// Voice channel the target was in
    pub channel_id: ChannelId,
    /// Minimum yes votes needed for the votekick to pass
    pub min_yes_votes: u32,
}
impl VotekickPoll {
    /// Create a new votekick poll.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        initiator_name: String,
        target_name: String,
        duration_secs: u64,
        results_delete_delay_secs: u64,
        target_user_id: UserId,
        guild_id: GuildId,
        channel_id: ChannelId,
        min_yes_votes: u32,
    ) -> Self {
        Self {
            initiator_name,
            target_name,
            duration_secs,
            results_delete_delay_secs,
            target_user_id,
            guild_id,
            channel_id,
            min_yes_votes,
        }
    }

    /// Reconstruct a votekick poll from persisted metadata.
    pub fn from_metadata(
        metadata: &VotekickMetadata,
        results_delete_delay_secs: u64,
    ) -> VotekickPoll {
        VotekickPoll::new(
            metadata.initiator_name.clone(),
            metadata.target_name.clone(),
            metadata.duration_secs,
            results_delete_delay_secs,
            metadata.target_user_id,
            metadata.guild_id,
            metadata.channel_id,
            metadata.min_yes_votes,
        )
    }
}

#[allow(clippy::too_many_lines)]
#[serenity::async_trait]
impl Poll for VotekickPoll {
    /// Provide votekick metadata so it is stored with the active poll record.
    fn metadata(&self) -> Option<VotekickMetadata> {
        Some(VotekickMetadata {
            target_user_id: self.target_user_id,
            guild_id: self.guild_id,
            channel_id: self.channel_id,
            initiator_name: self.initiator_name.clone(),
            target_name: self.target_name.clone(),
            duration_secs: self.duration_secs,
            results_delete_delay_secs: self.results_delete_delay_secs,
            min_yes_votes: self.min_yes_votes,
        })
    }

    /// Only users in the same voice channel as the target may vote.
    async fn is_eligible_voter(&self, ctx: &Context, user_id: UserId, info: &PollInfo) -> bool {
        let Some(ref metadata) = info.votekick else {
            return false;
        };
        let Some(guild) = ctx.cache.guild(metadata.guild_id) else {
            return false;
        };
        let Some(vs) = guild.voice_states.get(&user_id) else {
            return false;
        };
        vs.channel_id == Some(metadata.channel_id)
    }

    fn title(&self) -> String {
        "📊 Votekick Started".to_string()
    }

    fn description(&self) -> String {
        format!(
            "Vote to kick **{}** from the voice channel?\n\nReact with ✅ to vote **Yes**\nReact with ❌ to vote **No**",
            self.target_name
        )
    }

    fn duration(&self) -> u64 {
        self.duration_secs
    }

    fn build_embed(&self) -> CreateEmbed {
        CreateEmbed::default()
            .title(self.title())
            .description(self.description())
            .field("Duration", format!("{} seconds", self.duration()), false)
            .footer(serenity::all::CreateEmbedFooter::new(format!(
                "Initiated by {}",
                self.initiator_name
            )))
    }

    async fn on_complete(
        &self,
        ctx: &Context,
        _message_id: MessageId,
        yes_votes: u32,
        no_votes: u32,
        info: PollInfo,
    ) {
        let total_votes = yes_votes + no_votes;
        let Some(ref metadata) = info.votekick else {
            warn!("No votekick metadata found");
            return;
        };

        let guild_id = metadata.guild_id;
        let target_user_id = metadata.target_user_id;
        let channel_id = metadata.channel_id;

        if yes_votes < self.min_yes_votes {
            info!(
                "Votekick did not pass - need minimum 2 yes votes (got {})",
                yes_votes
            );
            self.send_results_message(
                ctx,
                channel_id,
                format!(
                    "📊 Votekick results: **Did not pass**\nNeed at least {} ✅ votes.\nResults: ✅ {yes_votes} | ❌ {no_votes} (Total votes: {total_votes})",
                    self.min_yes_votes,
                ),
            )
                .await;
            return;
        }

        if yes_votes <= no_votes {
            info!(
                "Votekick did not pass (yes: {}, no: {})",
                yes_votes, no_votes
            );
            self.send_results_message(
                ctx,
                channel_id,
                format!(
                    "📊 Votekick results: **Did not pass**\n✅ {yes_votes} | ❌ {no_votes} (Total votes: {total_votes})\n\nYes votes needed to exceed No votes.",
                ),
            )
            .await;
            return;
        }

        info!(
            "Votekick passed (yes: {}, no: {}) - kicking {}",
            yes_votes, no_votes, target_user_id
        );

        let target_member = match guild_id.member(&ctx.http, target_user_id).await {
            Ok(m) => m,
            Err(e) => {
                error!("Failed to get target member for kick: {}", e);
                self.send_results_message(
                    ctx,
                    channel_id,
                    "📊 Votekick passed but I couldn't fetch the target member. Please verify my permissions.",
                )
                .await;
                return;
            }
        };

        let can_disconnect = Self::can_disconnect_member(ctx, guild_id, channel_id).await;
        if !can_disconnect {
            warn!("Bot lacks Move Members permission in guild {}", guild_id);
            self.send_results_message(
                ctx,
                channel_id,
                format!(
                    "📊 Votekick passed (✅ {yes_votes} | ❌ {no_votes}) but I don't have permission to disconnect members. An admin needs to grant me **Move Members**.",
                ),
            )
            .await;
            return;
        }

        let guild_cache = ctx.cache.guild(guild_id);
        let in_voice = guild_cache.is_some_and(|g| g.voice_states.contains_key(&target_user_id));

        if !in_voice {
            info!(
                "Target user {} is no longer in a voice channel",
                target_user_id
            );
            self.send_results_message(
                ctx,
                channel_id,
                format!(
                    "📊 Votekick passed (✅ {} | ❌ {}) but {} is no longer in the voice channel.",
                    yes_votes,
                    no_votes,
                    target_member.user.mention()
                ),
            )
            .await;
            return;
        }

        if let Err(e) = guild_id.disconnect_member(&ctx.http, target_user_id).await {
            error!("Failed to disconnect member: {}", e);
            self.send_results_message(
                ctx,
                channel_id,
                format!(
                    "📊 Votekick passed (✅ {} | ❌ {}) but failed to kick {}: {}",
                    yes_votes,
                    no_votes,
                    target_member.user.mention(),
                    e
                ),
            )
            .await;
        } else {
            info!(
                "Successfully disconnected {} from voice channel",
                target_user_id
            );

            self.send_results_message(
                ctx,
                channel_id,
                format!(
                    "👢 **{}** was kicked from the voice channel!\n\n📊 Results: ✅ {} | ❌ {} (Total votes: {})",
                    target_member.user.mention(),
                    yes_votes,
                    no_votes,
                    total_votes
                ),
            )
            .await;
        }
    }
}

impl VotekickPoll {
    /// Send a temporary results message using the configured delete delay.
    async fn send_results_message(
        &self,
        ctx: &Context,
        channel_id: ChannelId,
        content: impl Into<String> + Send,
    ) {
        send_temporary_message(ctx, channel_id, content, self.results_delete_delay_secs).await;
    }

    /// Check whether the bot has permission to disconnect members in the channel.
    async fn can_disconnect_member(
        ctx: &Context,
        guild_id: GuildId,
        channel_id: ChannelId,
    ) -> bool {
        let bot_user_id = ctx.cache.current_user().id;
        let Ok(bot_member) = guild_id.member(&ctx.http, bot_user_id).await else {
            return false;
        };

        let permissions = {
            let Some(guild) = ctx.cache.guild(guild_id) else {
                return false;
            };
            let Some(channel) = guild.channels.get(&channel_id) else {
                return false;
            };
            guild.user_permissions_in(channel, &bot_member)
        };

        permissions.contains(Permissions::MOVE_MEMBERS)
    }
}

/// Start a new votekick poll.
/// This is the public interface used by the command handler.
pub async fn start_votekick(
    ctx: &Context,
    command: &CommandInteraction,
    target_user_id: UserId,
    channel_id: ChannelId,
    duration_secs: u64,
    results_delete_delay_secs: u64,
    min_yes_votes: u32,
) -> anyhow::Result<()> {
    let Some(guild_id) = command.guild_id else {
        anyhow::bail!("Votekick used outside guild");
    };

    let target_member = guild_id
        .member(&ctx.http, target_user_id)
        .await
        .context("Failed to get target member")?;

    let poll = VotekickPoll::new(
        command.user.name.clone(),
        target_member.user.name,
        duration_secs,
        results_delete_delay_secs,
        target_user_id,
        guild_id,
        channel_id,
        min_yes_votes,
    );

    let message_id = Poll::start(&poll, ctx, command)
        .await
        .context("Failed to start votekick poll")?;

    info!(
        "Votekick poll created for {} (message_id: {})",
        poll.target_name, message_id
    );

    crate::polls::persist_active_poll(
        message_id,
        poll.metadata().expect("votekick metadata"),
        duration_secs,
    )
    .await;

    schedule_poll_completion(poll, ctx.clone(), message_id, duration_secs).await;

    Ok(())
}
