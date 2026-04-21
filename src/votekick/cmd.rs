use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use tracing::{error, info, warn};

/// Send an ephemeral error response to the user.
async fn send_error(ctx: &Context, command: &CommandInteraction, message: &str) {
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

/// Get the voice channel ID for a user in a guild.
fn get_user_voice_channel(
    ctx: &Context,
    guild_id: serenity::all::GuildId,
    user_id: serenity::all::UserId,
) -> Option<serenity::all::ChannelId> {
    let guild = ctx.cache.guild(guild_id)?;
    let vs = guild.voice_states.get(&user_id)?;
    vs.channel_id
}

/// Check if target user is in the same channel and screensharing.
fn check_target_user(
    ctx: &Context,
    guild_id: serenity::all::GuildId,
    target_user_id: serenity::all::UserId,
    user_channel_id: serenity::all::ChannelId,
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

/// Handle the votekick command.
pub async fn run(ctx: &Context, command: &CommandInteraction) {
    let user = &command.user;
    let guild_id = command
        .guild_id
        .map(|id| id.to_string())
        .unwrap_or_else(|| "DM".to_string());

    info!(
        "Command '{} (votekick)' invoked by {} ({}) in {}",
        command.data.name, user.name, user.id, guild_id,
    );

    let Some(guild_id) = command.guild_id else {
        warn!("votekick used in DM by {}", user.name);
        send_error(ctx, command, "This command can only be used in a server.").await;
        return;
    };

    let target_user_id = command
        .data
        .options
        .first()
        .and_then(|opt| opt.value.as_user_id())
        .expect("User option is required");

    let user_channel_id = match get_user_voice_channel(ctx, guild_id, user.id) {
        Some(cid) => cid,
        None => {
            warn!("{} tried votekick but is not in a voice channel", user.name);
            send_error(
                ctx,
                command,
                "You must be in a voice channel to use this command.",
            )
            .await;
            return;
        }
    };

    let (target_in_same_channel, target_screensharing) =
        check_target_user(ctx, guild_id, target_user_id, user_channel_id);

    if !target_in_same_channel {
        warn!(
            "Target user {} is not in the same voice channel as {}",
            target_user_id, user.name
        );
        send_error(
            ctx,
            command,
            "The target user must be in the same voice channel as you.",
        )
        .await;
        return;
    }

    if !target_screensharing {
        warn!("Target user {} is not screensharing", target_user_id);
        send_error(
            ctx,
            command,
            "The target user must be screensharing to start a votekick.",
        )
        .await;
        return;
    }

    info!(
        "Votekick starting by {} targeting {} in channel {}",
        user.name, target_user_id, user_channel_id
    );

    super::poll::start_votekick(ctx, command, target_user_id, user_channel_id).await;
}
