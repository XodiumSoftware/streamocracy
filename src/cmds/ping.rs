use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use tracing::{error, info};

pub async fn run(ctx: &Context, command: &CommandInteraction) {
    let user = &command.user;
    let guild_id = command
        .guild_id
        .map(|id| id.to_string())
        .unwrap_or_else(|| "DM".to_string());

    info!(
        "Command 'ping' invoked by {} ({}) in {}",
        user.name,
        user.id,
        guild_id
    );

    if let Err(e) = command
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content("Pong! 🏓")
                    .ephemeral(true),
            ),
        )
        .await
    {
        error!("Failed to respond to ping command: {}", e);
    }
}
