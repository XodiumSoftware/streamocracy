use serenity::all::{
    ChannelId, ComponentInteraction, Context, CreateInteractionResponse,
    CreateInteractionResponseMessage, CreateMessage, CreatePoll, CreatePollAnswer, UserId,
};
use serenity::prelude::Mentionable;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::{error, info, warn};

// Active votekicks: message_id -> (target_user_id, guild_id, channel_id)
use std::sync::LazyLock;
static ACTIVE_VOTEKICKS: LazyLock<Arc<Mutex<HashMap<u64, (u64, u64, u64)>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));

pub async fn handle_select(
    ctx: &Context,
    interaction: &ComponentInteraction,
    target_user_id: UserId,
) {
    info!(
        "Votekick select triggered by {} targeting {}",
        interaction.user.name, target_user_id
    );

    let guild_id = match interaction.guild_id {
        Some(id) => id,
        None => {
            error!("Votekick select used outside guild");
            return;
        }
    };

    // Get target user info
    let target_member = match guild_id.member(&ctx.http, target_user_id).await {
        Ok(m) => m,
        Err(e) => {
            error!("Failed to get target member: {}", e);
            return;
        }
    };

    let target_name = &target_member.user.name;

    // Acknowledge the select menu interaction (ephemeral)
    if let Err(e) = interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(format!(
                        "Starting votekick poll for **{}**...",
                        target_name
                    ))
                    .ephemeral(true),
            ),
        )
        .await
    {
        error!("Failed to acknowledge select: {}", e);
        return;
    }

    // Create the native Discord poll
    let yes_answer = CreatePollAnswer::new().text("✅ Yes");
    let no_answer = CreatePollAnswer::new().text("❌ No");

    let poll = CreatePoll::default()
        .question(format!(
            "Vote to kick {} from the voice channel?",
            target_name
        ))
        .answers(vec![yes_answer, no_answer])
        .duration(Duration::from_secs(60)); // TODO: Discord requires minimum 1 hour - need to investigate

    // Send public message with native poll
    let message = match interaction
        .channel_id
        .send_message(
            &ctx.http,
            CreateMessage::new()
                .content(format!(
                    "📊 **Votekick Started**\n\n*Initiated by {}*",
                    interaction.user.mention()
                ))
                .poll(poll),
        )
        .await
    {
        Ok(m) => m,
        Err(e) => {
            error!("Failed to send poll message: {}", e);
            return;
        }
    };

    // Store the votekick info with channel_id
    let message_id = message.id.get();
    let guild_id_u64 = guild_id.get();
    let channel_id_u64 = interaction.channel_id.get();
    {
        let mut active = ACTIVE_VOTEKICKS.lock().await;
        active.insert(message_id, (target_user_id.get(), guild_id_u64, channel_id_u64));
    }

    // Spawn background task to check results after poll ends
    let ctx_clone = ctx.clone();
    tokio::spawn(async move {
        sleep(Duration::from_secs(65)).await; // TODO: Wait time should match poll duration
        check_poll_results(&ctx_clone, message_id).await;
    });

    info!(
        "Votekick poll created for {} (message_id: {})",
        target_name, message_id
    );
}

async fn check_poll_results(ctx: &Context, message_id: u64) {
    // Get votekick info
    let (target_user_id, guild_id, channel_id) = {
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
    let target_user_id = serenity::all::UserId::new(target_user_id);
    let channel_id = ChannelId::new(channel_id);

    // Fetch the message to check poll results
    let message = match channel_id.message(&ctx.http, message_id).await {
        Ok(m) => m,
        Err(e) => {
            error!("Failed to fetch poll message: {}", e);
            return;
        }
    };

    // Check if poll exists and get results
    let poll = match message.poll {
        Some(p) => p,
        None => {
            error!("Message has no poll");
            return;
        }
    };

    // Get vote counts from answer_counts
    let results = match &poll.results {
        Some(r) => r,
        None => {
            error!("Poll has no results");
            return;
        }
    };

    // Get vote counts (answer id 1 = Yes, 2 = No)
    let yes_votes = results
        .answer_counts
        .iter()
        .find(|r| r.id.get() == 1)
        .map(|r| r.count)
        .unwrap_or(0);

    let no_votes = results
        .answer_counts
        .iter()
        .find(|r| r.id.get() == 2)
        .map(|r| r.count)
        .unwrap_or(0);

    let total_votes = yes_votes + no_votes;

    info!(
        "Poll results for message {}: Yes={}, No={}, Total={}",
        message_id, yes_votes, no_votes, total_votes
    );

    // Check minimum 2 votes requirement for yes
    if yes_votes < 2 {
        info!("Votekick did not pass - need minimum 2 yes votes (got {})", yes_votes);
        let _ = channel_id
            .say(
                &ctx.http,
                format!(
                    "📊 Votekick results: **Did not pass**\nNeed at least 2 ✅ votes.\nResults: ✅ {} | ❌ {} (Total votes: {})",
                    yes_votes, no_votes, total_votes
                ),
            )
            .await;
        return;
    }

    // Check if yes has majority
    if yes_votes <= no_votes {
        info!("Votekick did not pass (yes: {}, no: {})", yes_votes, no_votes);
        let _ = channel_id
            .say(
                &ctx.http,
                format!(
                    "📊 Votekick results: **Did not pass**\n✅ {} | ❌ {} (Total votes: {})\n\nYes votes needed to exceed No votes.",
                    yes_votes, no_votes, total_votes
                ),
            )
            .await;
        return;
    }

    // Yes has majority - kick the user
    info!(
        "Votekick passed (yes: {}, no: {}) - kicking {}",
        yes_votes, no_votes, target_user_id
    );

    // Get target member
    let target_member = match guild_id.member(&ctx.http, target_user_id).await {
        Ok(m) => m,
        Err(e) => {
            error!("Failed to get target member for kick: {}", e);
            return;
        }
    };

    // Check if user is still in a voice channel using cache
    let guild_cache = ctx.cache.guild(guild_id);
    let in_voice = guild_cache
        .map(|g| g.voice_states.contains_key(&target_user_id))
        .unwrap_or(false);

    if !in_voice {
        info!("Target user {} is no longer in a voice channel", target_user_id);
        let _ = channel_id
            .say(
                &ctx.http,
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

    // Disconnect the user from voice channel
    if let Err(e) = guild_id
        .disconnect_member(&ctx.http, target_user_id)
        .await
    {
        error!("Failed to disconnect member: {}", e);
        let _ = channel_id
            .say(
                &ctx.http,
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
        info!("Successfully disconnected {} from voice channel", target_user_id);

        // Send success message
        let _ = channel_id
            .say(
                &ctx.http,
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
