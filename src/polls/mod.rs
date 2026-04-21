//! Poll functionality for the Streamocracy bot

use serenity::all::{CommandInteraction, Context, CreateEmbed, ReactionType, UserId};
use std::collections::HashMap;
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
    pub channel_id: u64,
}

/// Thread-safe storage for active polls
type ActivePolls = Arc<Mutex<HashMap<u64, PollInfo>>>;

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
    /// yes_votes and no_votes are counts excluding the bot.
    async fn on_complete(&self, ctx: &Context, message_id: u64, yes_votes: u32, no_votes: u32);

    /// Start the poll by sending the embed and adding reactions.
    /// Returns the message ID of the created poll.
    async fn start(&self, ctx: &Context, command: &CommandInteraction) -> anyhow::Result<u64> {
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

        let message_id = message.id.get();

        {
            let mut active = ACTIVE_POLLS.lock().await;
            active.insert(
                message_id,
                PollInfo {
                    channel_id: message.channel_id.get(),
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
    message_id: u64,
    duration_secs: u64,
) {
    let ctx_clone = ctx.clone();
    tokio::spawn(async move {
        sleep(Duration::from_secs(duration_secs)).await;
        complete_poll(&poll, &ctx_clone, message_id).await;
    });
}

/// Complete a poll by counting votes and calling on_complete.
async fn complete_poll<P: Poll>(poll: &P, ctx: &Context, message_id: u64) {
    let poll_info = {
        let mut active = ACTIVE_POLLS.lock().await;
        match active.remove(&message_id) {
            Some(info) => info,
            None => {
                warn!("No active poll found for message {}", message_id);
                return;
            }
        }
    };

    let channel_id = serenity::all::ChannelId::new(poll_info.channel_id);
    let message = match channel_id.message(&ctx.http, message_id).await {
        Ok(m) => m,
        Err(e) => {
            error!("Failed to fetch poll message: {}", e);
            return;
        }
    };

    let yes_reaction = poll.yes_reaction();
    let no_reaction = poll.no_reaction();
    let yes_votes = get_reaction_count(&ctx.http, &message, &yes_reaction).await;
    let no_votes = get_reaction_count(&ctx.http, &message, &no_reaction).await;

    info!(
        "Poll results for message {}: Yes={}, No={}",
        message_id, yes_votes, no_votes
    );

    if let Err(e) = channel_id.delete_message(&ctx.http, message_id).await {
        warn!("Failed to delete poll message: {}", e);
    }

    poll.on_complete(ctx, message_id, yes_votes, no_votes).await;
}

/// Count users who reacted with a specific emoji, excluding the bot.
async fn get_reaction_count(
    http: &serenity::all::Http,
    message: &serenity::all::Message,
    reaction_type: &ReactionType,
) -> u32 {
    let mut count = 0u32;
    let mut after: Option<UserId> = None;

    loop {
        let users = match message
            .reaction_users(http, reaction_type.clone(), Some(100u8), after)
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
            if user.id != message.author.id {
                count += 1;
            }
        }

        if users.len() < 100 {
            break;
        }

        after = users.last().map(|u| u.id);
    }

    count
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
