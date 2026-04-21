use crate::config::Config;
use serenity::all::{CommandInteraction, Context, CreateCommand};

pub mod ping;
pub mod votekick;

/// Trait for slash commands.
#[serenity::async_trait]
pub trait SlashCommand: Send + Sync {
    /// The command name (must match Discord command name).
    fn name(&self) -> &'static str;

    /// Register the command with Discord.
    fn register(&self) -> CreateCommand;

    /// Execute the command.
    async fn run(&self, ctx: Context, command: CommandInteraction, config: Config);
}

/// Get all available commands.
pub fn get_commands() -> Vec<Box<dyn SlashCommand>> {
    vec![
        Box::new(ping::PingCommand),
        Box::new(votekick::VotekickCommand),
    ]
}
