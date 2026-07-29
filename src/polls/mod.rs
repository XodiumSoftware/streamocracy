//! Poll functionality for the Streamocracy bot

use serenity::all::{
    ChannelId, CommandInteraction, Context, CreateEmbed, MessageId, ReactionType, UserId,
};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::LazyLock;
use std::time::Duration;
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
