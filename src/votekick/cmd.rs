use serenity::all::{
    CommandInteraction, Context, CreateActionRow, CreateInteractionResponse,
    CreateInteractionResponseMessage, CreateSelectMenu, CreateSelectMenuOption,
};
use tracing::{error, info, warn};

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

pub async fn run(ctx: &Context, command: &CommandInteraction) {
    let user = &command.user;
    let guild_id = command
        .guild_id
        .map(|id| id.to_string())
        .unwrap_or_else(|| "DM".to_string());

    info!(
        "Command '{} (votekick)' invoked by {} ({}) in {}",
        command.data.name,
        user.name,
        user.id,
        guild_id,
    );

    // Check if command is used in a guild (not DM)
    let Some(guild_id) = command.guild_id else {
        warn!("votekick used in DM by {}", user.name);
        send_error(ctx, command, "This command can only be used in a server.").await;
        return;
    };

    // Check if guild is in cache and clone it to avoid Send issues
    let guild = match ctx.cache.guild(guild_id) {
        Some(g) => g.clone(),
        None => {
            error!("Guild not in cache");
            return;
        }
    };

    // Check if user is in a voice channel
    let Some(user_voice_state) = guild.voice_states.get(&user.id) else {
        warn!("{} tried votekick but is not in a voice channel", user.name);
        send_error(ctx, command, "You must be in a voice channel to use this command.").await;
        return;
    };

    let Some(user_channel_id) = user_voice_state.channel_id else {
        warn!("{} has voice state but no channel", user.name);
        return;
    };

    // Check if anyone in the same voice channel is screensharing
    let has_screenshare = guild
        .voice_states
        .values()
        .filter(|vs| vs.channel_id == Some(user_channel_id))
        .any(|vs| vs.self_stream.unwrap_or(false));

    if !has_screenshare {
        warn!("No screenshares in voice channel for {}", user.name);
        send_error(
            ctx,
            command,
            "There are no active screenshares in this voice channel.",
        )
        .await;
        return;
    }

    // Get all users in the voice channel who are screensharing
    let screensharers: Vec<(&serenity::all::UserId, &str)> = guild
        .voice_states
        .iter()
        .filter(|(_, vs)| vs.channel_id == Some(user_channel_id) && vs.self_stream.unwrap_or(false))
        .filter_map(|(user_id, _)| {
            guild
                .members
                .get(user_id)
                .map(|m| (user_id, m.user.name.as_str()))
        })
        .collect();

    if screensharers.is_empty() {
        warn!("No screensharers found in voice channel for {}", user.name);
        send_error(
            ctx,
            command,
            "There are no active screenshares in this voice channel.",
        )
        .await;
        return;
    }

    info!(
        "Found {} screensharers for votekick by {}",
        screensharers.len(),
        user.name
    );

    // Create select menu options for each screensharer
    let options: Vec<CreateSelectMenuOption> = screensharers
        .into_iter()
        .map(|(user_id, name)| {
            CreateSelectMenuOption::new(name, user_id.to_string())
                .description(format!("Vote to kick {} from the voice channel", name))
        })
        .collect();

    let select_menu = CreateSelectMenu::new("votekick_select", serenity::all::CreateSelectMenuKind::String {
        options,
    })
    .placeholder("Select a user to votekick")
    .min_values(1)
    .max_values(1);

    // Create action row with the select menu
    let action_row = CreateActionRow::SelectMenu(select_menu);

    // Send response with the select menu
    if let Err(e) = command
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content("Select a user to start a votekick:")
                    .ephemeral(true)
                    .components(vec![action_row]),
            ),
        )
        .await
    {
        error!("Failed to respond with select menu: {}", e);
    }
}
