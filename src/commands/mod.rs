use crate::config::Config;
use serenity::all::{CommandInteraction, Context, CreateCommand};

pub mod ping;
pub mod votekick;

/// Trait for slash commands.
// The `double_must_use` lint misfires on the `async_trait` expansion, which adds
// `#[must_use]` to methods returning `Pin<Box<dyn Future>>` (already `must_use`).
#[allow(clippy::double_must_use)]
#[serenity::async_trait]
pub trait SlashCommand: Send + Sync {
    /// The command name (must match Discord command name).
    fn name(&self) -> &'static str;

    /// Register the command with Discord.
    ///
    /// Configuration values may influence command options such as min/max ranges.
    fn register(&self, config: &Config) -> CreateCommand;

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
