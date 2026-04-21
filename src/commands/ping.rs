use crate::commands::SlashCommand;
use crate::config::Config;
use serenity::all::{
    CommandInteraction, Context, CreateCommand, CreateInteractionResponse,
    CreateInteractionResponseMessage,
};
use tracing::info;

/// Slash command that responds with "Pong! 🏓" to verify bot responsiveness.
pub struct PingCommand;

#[serenity::async_trait]
impl SlashCommand for PingCommand {
    fn name(&self) -> &'static str {
        "ping"
    }

    fn register(&self) -> CreateCommand {
        CreateCommand::new(self.name()).description("Check if the bot is responsive")
    }

    async fn run(&self, ctx: Context, command: CommandInteraction, _config: Config) {
        let user = &command.user;
        info!("Command 'ping' invoked by {} ({})", user.name, user.id);

        if let Err(e) = command
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new().content("Pong! 🏓"),
                ),
            )
            .await
        {
            tracing::error!("Failed to respond to ping: {}", e);
        }
    }
}
