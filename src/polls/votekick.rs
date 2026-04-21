//! Votekick poll implementation

use crate::polls::{Poll, schedule_poll_completion, send_temporary_message};
use serenity::all::{ChannelId, CommandInteraction, Context, CreateEmbed, UserId};
use serenity::prelude::Mentionable;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::LazyLock;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

/// Metadata for active votekicks.
/// (target_user_id, guild_id, channel_id)
type VotekickMetadata = (u64, u64, u64);

/// Thread-safe storage for active votekick metadata.
type ActiveVotekicks = Arc<Mutex<HashMap<u64, VotekickMetadata>>>;

static ACTIVE_VOTEKICKS: LazyLock<ActiveVotekicks> =
    LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));

/// A poll for voting to kick a user from a voice channel.
pub struct VotekickPoll {
    /// The user who initiated the votekick
    pub initiator_name: String,
    /// Target user's display name
    pub target_name: String,
    /// Poll duration in seconds
    pub duration_secs: u64,
}

impl VotekickPoll {
    /// Create a new votekick poll.
    pub fn new(initiator_name: String, target_name: String, duration_secs: u64) -> Self {
        Self {
            initiator_name,
            target_name,
            duration_secs,
        }
    }
}

#[serenity::async_trait]
impl Poll for VotekickPoll {
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

    async fn on_complete(&self, ctx: &Context, message_id: u64, yes_votes: u32, no_votes: u32) {
        let total_votes = yes_votes + no_votes;
        let (target_user_id, guild_id, channel_id) = {
            let mut active = ACTIVE_VOTEKICKS.lock().await;
            match active.remove(&message_id) {
                Some(info) => info,
                None => {
                    warn!("No votekick metadata found for message {}", message_id);
                    return;
                }
            }
        };

        let guild_id = serenity::all::GuildId::new(guild_id);
        let target_user_id = UserId::new(target_user_id);
        let channel_id = ChannelId::new(channel_id);

        if yes_votes < 2 {
            info!(
                "Votekick did not pass - need minimum 2 yes votes (got {})",
                yes_votes
            );
            send_temporary_message(
                ctx,
                channel_id,
                format!(
                    "📊 Votekick results: **Did not pass**\nNeed at least 2 ✅ votes.\nResults: ✅ {} | ❌ {} (Total votes: {})",
                    yes_votes, no_votes, total_votes
                ),
                10,
            )
                .await;
            return;
        }

        if yes_votes <= no_votes {
            info!(
                "Votekick did not pass (yes: {}, no: {})",
                yes_votes, no_votes
            );
            send_temporary_message(
                ctx,
                channel_id,
                format!(
                    "📊 Votekick results: **Did not pass**\n✅ {} | ❌ {} (Total votes: {})\n\nYes votes needed to exceed No votes.",
                    yes_votes, no_votes, total_votes
                ),
                10,
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
                return;
            }
        };

        let guild_cache = ctx.cache.guild(guild_id);
        let in_voice = guild_cache
            .map(|g| g.voice_states.contains_key(&target_user_id))
            .unwrap_or(false);

        if !in_voice {
            info!(
                "Target user {} is no longer in a voice channel",
                target_user_id
            );
            send_temporary_message(
                ctx,
                channel_id,
                format!(
                    "📊 Votekick passed (✅ {} | ❌ {}) but {} is no longer in the voice channel.",
                    yes_votes,
                    no_votes,
                    target_member.user.mention()
                ),
                10,
            )
            .await;
            return;
        }

        if let Err(e) = guild_id.disconnect_member(&ctx.http, target_user_id).await {
            error!("Failed to disconnect member: {}", e);
            send_temporary_message(
                ctx,
                channel_id,
                format!(
                    "📊 Votekick passed (✅ {} | ❌ {}) but failed to kick {}: {}",
                    yes_votes,
                    no_votes,
                    target_member.user.mention(),
                    e
                ),
                10,
            )
            .await;
        } else {
            info!(
                "Successfully disconnected {} from voice channel",
                target_user_id
            );

            send_temporary_message(
                ctx,
                channel_id,
                format!(
                    "👢 **{}** was kicked from the voice channel!\n\n📊 Results: ✅ {} | ❌ {} (Total votes: {})",
                    target_member.user.mention(),
                    yes_votes,
                    no_votes,
                    total_votes
                ),
                10,
            )
                .await;
        }
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
) {
    let guild_id = match command.guild_id {
        Some(id) => id,
        None => {
            error!("Votekick used outside guild");
            return;
        }
    };

    let target_member = match guild_id.member(&ctx.http, target_user_id).await {
        Ok(m) => m,
        Err(e) => {
            error!("Failed to get target member: {}", e);
            return;
        }
    };

    let poll = VotekickPoll::new(
        command.user.name.clone(),
        target_member.user.name,
        duration_secs,
    );

    match Poll::start(&poll, ctx, command).await {
        Ok(message_id) => {
            {
                let mut active = ACTIVE_VOTEKICKS.lock().await;
                active.insert(
                    message_id,
                    (target_user_id.get(), guild_id.get(), channel_id.get()),
                );
            }

            info!(
                "Votekick poll created for {} (message_id: {})",
                poll.target_name, message_id
            );

            schedule_poll_completion(poll, ctx.clone(), message_id, duration_secs).await;
        }
        Err(e) => {
            error!("Failed to start votekick poll: {}", e);
        }
    }
}
