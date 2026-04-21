use serenity::all::{
    ChannelId, CommandInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, ReactionType, UserId,
};
use serenity::prelude::Mentionable;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::LazyLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::{error, info, warn};

/// (target_user_id, guild_id, channel_id, end_timestamp)
type VotekickInfo = (u64, u64, u64, u64);

/// Thread-safe storage for active votekicks
type ActiveVotekicks = Arc<Mutex<HashMap<u64, VotekickInfo>>>;

static ACTIVE_VOTEKICKS: LazyLock<ActiveVotekicks> =
    LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));

/// Start a new votekick poll for the target user.
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
            let _ = command
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content("Failed to get target user information.")
                            .ephemeral(true),
                    ),
                )
                .await;
            return;
        }
    };
    let target_name = &target_member.user.name;

    let end_time = SystemTime::now() + Duration::from_secs(duration_secs);
    let end_timestamp = end_time
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let embed = CreateEmbed::default()
        .title("📊 Votekick Started")
        .description(format!(
            "Vote to kick **{}** from the voice channel?\n\nReact with ✅ to vote **Yes**\nReact with ❌ to vote **No**",
            target_name
        ))
        .field("Duration", format!("{} seconds", duration_secs), false)
        .footer(serenity::all::CreateEmbedFooter::new(format!(
            "Initiated by {}",
            command.user.name
        )));

    // Send the poll as the command response
    if let Err(e) = command
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new().embed(embed),
            ),
        )
        .await
    {
        error!("Failed to send poll message: {}", e);
        return;
    }

    // Get the message ID from the interaction response
    let message = match command.get_response(&ctx.http).await {
        Ok(m) => m,
        Err(e) => {
            error!("Failed to get response message: {}", e);
            return;
        }
    };

    let yes_reaction = ReactionType::Unicode("✅".to_string());
    let no_reaction = ReactionType::Unicode("❌".to_string());

    if let Err(e) = message.react(&ctx.http, yes_reaction).await {
        error!("Failed to add yes reaction: {}", e);
    }
    if let Err(e) = message.react(&ctx.http, no_reaction).await {
        error!("Failed to add no reaction: {}", e);
    }

    let message_id = message.id.get();
    let guild_id_u64 = guild_id.get();
    let channel_id_u64 = channel_id.get();
    {
        let mut active = ACTIVE_VOTEKICKS.lock().await;
        active.insert(
            message_id,
            (
                target_user_id.get(),
                guild_id_u64,
                channel_id_u64,
                end_timestamp,
            ),
        );
    }

    let ctx_clone = ctx.clone();
    tokio::spawn(async move {
        sleep(Duration::from_secs(duration_secs)).await;
        check_poll_results(&ctx_clone, message_id).await;
    });

    info!(
        "Votekick poll created for {} (message_id: {})",
        target_name, message_id
    );
}

/// Check poll results and execute the votekick if passed.
async fn check_poll_results(ctx: &Context, message_id: u64) {
    let (target_user_id, guild_id, channel_id, _end_timestamp) = {
        let mut active = ACTIVE_VOTEKICKS.lock().await;
        match active.remove(&message_id) {
            Some(info) => info,
            None => {
                warn!("No active votekick found for message {}", message_id);
                return;
            }
        }
    };
    let guild_id = serenity::all::GuildId::new(guild_id);
    let target_user_id = UserId::new(target_user_id);
    let channel_id = ChannelId::new(channel_id);
    let message = match channel_id.message(&ctx.http, message_id).await {
        Ok(m) => m,
        Err(e) => {
            error!("Failed to fetch poll message: {}", e);
            return;
        }
    };
    let yes_reaction = ReactionType::Unicode("✅".to_string());
    let no_reaction = ReactionType::Unicode("❌".to_string());
    let yes_votes = get_reaction_count(&ctx.http, &message, &yes_reaction).await;
    let no_votes = get_reaction_count(&ctx.http, &message, &no_reaction).await;
    let total_votes = yes_votes + no_votes;

    info!(
        "Poll results for message {}: Yes={}, No={}, Total={}",
        message_id, yes_votes, no_votes, total_votes
    );

    if let Err(e) = channel_id.delete_message(&ctx.http, message_id).await {
        warn!("Failed to delete poll message: {}", e);
    }

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

/// Count users who reacted with a specific emoji, excluding the bot
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

/// Send a message that auto-deletes after a specified number of seconds
async fn send_temporary_message(
    ctx: &Context,
    channel_id: ChannelId,
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
