//! Poll functionality for the Streamocracy bot

use crate::polls::votekick::{VotekickMetadata, VotekickPoll};
use serenity::all::{
    ChannelId, CommandInteraction, Context, CreateEmbed, MessageId, ReactionType, UserId,
};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Arc;
use std::sync::LazyLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::{error, info, warn};

pub mod votekick;

/// Poll metadata stored while poll is active.
#[derive(Clone)]
pub struct PollInfo {
    /// Channel where poll was created
    pub channel_id: ChannelId,
    /// Votekick-specific metadata, if this poll is a votekick
    pub votekick: Option<votekick::VotekickMetadata>,
}

/// Thread-safe storage for active polls
type ActivePolls = Arc<Mutex<HashMap<MessageId, PollInfo>>>;

static ACTIVE_POLLS: LazyLock<ActivePolls> = LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));

/// Trait for reaction-based polls.
#[serenity::async_trait]
pub trait Poll: Send + Sync {
    /// The poll title displayed in the embed.
    fn title(&self) -> String;

    /// The poll description/question.
    fn description(&self) -> String;

    /// Duration of the poll in seconds.
    fn duration(&self) -> u64;

    /// The yes/positive reaction emoji.
    fn yes_reaction(&self) -> ReactionType {
        ReactionType::Unicode("✅".to_string())
    }

    /// The no/negative reaction emoji.
    fn no_reaction(&self) -> ReactionType {
        ReactionType::Unicode("❌".to_string())
    }

    /// Build the embed shown for the poll.
    fn build_embed(&self) -> CreateEmbed {
        CreateEmbed::default()
            .title(self.title())
            .description(self.description())
            .field("Duration", format!("{} seconds", self.duration()), false)
    }

    /// Called when the poll ends with results.
    /// `yes_votes` and `no_votes` are counts excluding the bot.
    async fn on_complete(
        &self,
        ctx: &Context,
        message_id: MessageId,
        yes_votes: u32,
        no_votes: u32,
        info: PollInfo,
    );

    /// Optional poll-specific metadata stored alongside `PollInfo`.
    fn metadata(&self) -> Option<votekick::VotekickMetadata> {
        None
    }

    /// Determine whether a user is eligible to vote in this poll.
    /// Defaults to `true`; votekick polls override this to restrict by voice channel.
    async fn is_eligible_voter(&self, _ctx: &Context, _user_id: UserId, _info: &PollInfo) -> bool {
        true
    }

    /// Start the poll by sending the embed and adding reactions.
    /// Returns the message ID of the created poll.
    async fn start(
        &self,
        ctx: &Context,
        command: &CommandInteraction,
    ) -> anyhow::Result<MessageId> {
        let embed = self.build_embed();

        command
            .create_response(
                &ctx.http,
                serenity::all::CreateInteractionResponse::Message(
                    serenity::all::CreateInteractionResponseMessage::new().embed(embed),
                ),
            )
            .await?;

        let message = command.get_response(&ctx.http).await?;
        let yes = self.yes_reaction();
        let no = self.no_reaction();

        if let Err(e) = message.react(&ctx.http, yes).await {
            error!("Failed to add yes reaction: {}", e);
        }
        if let Err(e) = message.react(&ctx.http, no).await {
            error!("Failed to add no reaction: {}", e);
        }

        let message_id = message.id;

        {
            let metadata = self.metadata();
            let mut active = ACTIVE_POLLS.lock().await;
            active.insert(
                message_id,
                PollInfo {
                    channel_id: message.channel_id,
                    votekick: metadata,
                },
            );
        }

        info!("Poll started (message_id: {})", message_id);
        Ok(message_id)
    }
}

/// Persisted record of an active poll.
#[derive(serde::Serialize, serde::Deserialize)]
struct PersistedPoll {
    /// The Discord message ID hosting the poll
    message_id: u64,
    /// The channel the poll message was posted in
    channel_id: u64,
    /// Unix timestamp (seconds) when the poll should complete
    end_time: u64,
    /// Votekick metadata, if this is a votekick poll
    votekick: Option<votekick::VotekickMetadata>,
}

/// Persist active poll state so it can survive a restart.
pub async fn persist_active_poll(
    message_id: MessageId,
    metadata: VotekickMetadata,
    duration_secs: u64,
) {
    let path = active_polls_path();
    let mut state = load_persisted_state(&path).await.unwrap_or_default();

    let end_time = unix_now() + duration_secs;
    state.push(PersistedPoll {
        message_id: message_id.get(),
        channel_id: metadata.channel_id.get(),
        end_time,
        votekick: Some(metadata),
    });

    if let Err(e) = write_persisted_state(&path, &state).await {
        error!("Failed to persist active poll state: {}", e);
    }
}

/// Remove a poll from persistent state once it has completed.
pub async fn remove_persisted_poll(message_id: MessageId) {
    let path = active_polls_path();
    let state = match load_persisted_state(&path).await {
        Ok(s) => s,
        Err(e) => {
            warn!("Failed to load persisted poll state for removal: {}", e);
            return;
        }
    };

    let id = message_id.get();
    let retained: Vec<_> = state.into_iter().filter(|p| p.message_id != id).collect();

    if let Err(e) = write_persisted_state(&path, &retained).await {
        error!("Failed to write persisted poll state after removal: {}", e);
    }
}

/// Resume polls from persisted state and schedule any that have not yet ended.
pub async fn resume_polls(ctx: &Context) {
    let path = active_polls_path();
    let state = match load_persisted_state(&path).await {
        Ok(s) => s,
        Err(e) => {
            warn!("Failed to load persisted poll state: {}", e);
            return;
        }
    };

    let now = unix_now();
    let mut retained = Vec::new();

    for record in state {
        let Some(ref metadata) = record.votekick else {
            continue;
        };
        let poll = VotekickPoll::from_metadata(metadata, metadata.results_delete_delay_secs);

        let message_id = MessageId::new(record.message_id);
        let channel_id = ChannelId::new(record.channel_id);

        if record.end_time <= now {
            info!(
                "Completing overdue votekick poll (message_id: {})",
                message_id
            );
            {
                let mut active = ACTIVE_POLLS.lock().await;
                active.insert(
                    message_id,
                    PollInfo {
                        channel_id,
                        votekick: Some(metadata.clone()),
                    },
                );
            }
            complete_poll(&poll, ctx, message_id).await;
        } else {
            info!(
                "Resuming votekick poll (message_id: {}, remaining: {}s)",
                message_id,
                record.end_time - now
            );
            {
                let mut active = ACTIVE_POLLS.lock().await;
                active.insert(
                    message_id,
                    PollInfo {
                        channel_id,
                        votekick: Some(metadata.clone()),
                    },
                );
            }
            schedule_poll_completion(poll, ctx.clone(), message_id, record.end_time - now).await;
            retained.push(record);
        }
    }

    if let Err(e) = write_persisted_state(&path, &retained).await {
        error!("Failed to write persisted poll state after resume: {}", e);
    }
}

fn active_polls_path() -> std::path::PathBuf {
    std::env::var("POLL_STATE_FILE").map_or_else(
        |_| std::path::PathBuf::from("poll_state.json"),
        std::path::PathBuf::from,
    )
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

async fn load_persisted_state(path: &Path) -> anyhow::Result<Vec<PersistedPoll>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let contents = tokio::fs::read_to_string(path).await?;
    Ok(serde_json::from_str(&contents)?)
}

async fn write_persisted_state(path: &Path, state: &[PersistedPoll]) -> anyhow::Result<()> {
    let contents = serde_json::to_string_pretty(state)?;
    tokio::fs::write(path, contents).await?;
    Ok(())
}

/// Schedule a poll to complete after its duration.
pub async fn schedule_poll_completion<P: Poll + 'static>(
    poll: P,
    ctx: Context,
    message_id: MessageId,
    duration_secs: u64,
) {
    let ctx_clone = ctx.clone();
    tokio::spawn(async move {
        sleep(Duration::from_secs(duration_secs)).await;
        complete_poll(&poll, &ctx_clone, message_id).await;
    });
}

/// Complete a poll by counting votes and calling `on_complete`.
async fn complete_poll<P: Poll>(poll: &P, ctx: &Context, message_id: MessageId) {
    let poll_info = {
        let mut active = ACTIVE_POLLS.lock().await;
        if let Some(info) = active.remove(&message_id) {
            info
        } else {
            warn!("No active poll found for message {}", message_id);
            return;
        }
    };

    let channel_id = poll_info.channel_id;
    let message = match channel_id.message(&ctx.http, message_id).await {
        Ok(m) => m,
        Err(e) => {
            error!("Failed to fetch poll message: {}", e);
            return;
        }
    };

    let (yes_votes, no_votes) = count_eligible_votes(poll, ctx, &message, &poll_info).await;

    info!(
        "Poll results for message {}: Yes={}, No={}",
        message_id, yes_votes, no_votes
    );

    if let Err(e) = channel_id.delete_message(&ctx.http, message_id).await {
        warn!("Failed to delete poll message: {}", e);
    }

    remove_persisted_poll(message_id).await;

    poll.on_complete(ctx, message_id, yes_votes, no_votes, poll_info)
        .await;
}

/// Count eligible yes/no votes for a poll, excluding the bot and preventing duplicate votes.
async fn count_eligible_votes<P: Poll>(
    poll: &P,
    ctx: &Context,
    message: &serenity::all::Message,
    info: &PollInfo,
) -> (u32, u32) {
    let bot_id = message.author.id;

    let mut voters: HashSet<UserId> = HashSet::new();
    let mut yes_votes = 0u32;
    let mut no_votes = 0u32;

    count_eligible_reactions(
        poll,
        ctx,
        message,
        info,
        bot_id,
        &mut voters,
        &mut yes_votes,
        poll.yes_reaction(),
    )
    .await;
    count_eligible_reactions(
        poll,
        ctx,
        message,
        info,
        bot_id,
        &mut voters,
        &mut no_votes,
        poll.no_reaction(),
    )
    .await;

    (yes_votes, no_votes)
}

/// Iterate through all pages of a reaction and count eligible, non-duplicate voters.
#[allow(clippy::too_many_arguments)]
async fn count_eligible_reactions<P: Poll>(
    poll: &P,
    ctx: &Context,
    message: &serenity::all::Message,
    info: &PollInfo,
    bot_id: UserId,
    voters: &mut HashSet<UserId>,
    counter: &mut u32,
    reaction_type: ReactionType,
) {
    let mut after: Option<UserId> = None;

    loop {
        let users = match message
            .reaction_users(&ctx.http, reaction_type.clone(), Some(100u8), after)
            .await
        {
            Ok(u) => u,
            Err(e) => {
                error!("Failed to get reaction users: {}", e);
                break;
            }
        };

        if users.is_empty() {
            break;
        }

        for user in &users {
            if user.id == bot_id || voters.contains(&user.id) {
                continue;
            }
            if poll.is_eligible_voter(ctx, user.id, info).await {
                voters.insert(user.id);
                *counter += 1;
            }
        }

        if users.len() < 100 {
            break;
        }

        after = users.last().map(|u| u.id);
    }
}

/// Send a message that auto-deletes after a specified number of seconds.
pub async fn send_temporary_message(
    ctx: &Context,
    channel_id: serenity::all::ChannelId,
    content: impl Into<String>,
    delete_after_secs: u64,
) {
    let content = content.into();
    let http = ctx.http.clone();

    match channel_id.say(&http, content).await {
        Ok(message) => {
            let message_id = message.id;
            tokio::spawn(async move {
                sleep(Duration::from_secs(delete_after_secs)).await;
                let _ = channel_id.delete_message(&http, message_id).await;
            });
        }
        Err(e) => {
            error!("Failed to send temporary message: {}", e);
        }
    }
}
