//! Configuration management for Streamocracy
//!
//! Loads configuration from environment variables.
//! A `.env` file can be used for local development.

use anyhow::Context;
use serenity::all::GuildId;

/// Bot configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    /// Discord bot token (required)
    pub discord_token: String,
    /// Optional guild ID for instant command registration
    pub guild_id: Option<u64>,
    /// Log level filter (default: info)
    pub log_level: String,
    /// Default votekick duration in seconds (default: 60)
    pub default_votekick_duration: u64,
    /// Minimum votekick duration in seconds (default: 10)
    pub min_votekick_duration: u64,
    /// Maximum votekick duration in seconds (default: 300)
    pub max_votekick_duration: u64,
}

impl Config {
    /// Load configuration from environment variables.
    pub fn from_env() -> anyhow::Result<Self> {
        let discord_token = std::env::var("DISCORD_TOKEN")
            .context("DISCORD_TOKEN environment variable is required")?;

        let guild_id = std::env::var("GUILD_ID").ok().and_then(|v| v.parse().ok());

        let log_level = std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());

        let default_votekick_duration = std::env::var("DEFAULT_VOTEKICK_DURATION")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(60);

        let min_votekick_duration = std::env::var("MIN_VOTEKICK_DURATION")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10);

        let max_votekick_duration = std::env::var("MAX_VOTEKICK_DURATION")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(300);

        Ok(Self {
            discord_token,
            guild_id,
            log_level,
            default_votekick_duration,
            min_votekick_duration,
            max_votekick_duration,
        })
    }

    /// Get the guild ID as an Option<serenity::all::GuildId>
    pub fn guild_id(&self) -> Option<GuildId> {
        self.guild_id.map(GuildId::new)
    }
}

/// Load config and set up logging
pub fn init() -> anyhow::Result<Config> {
    let config = Config::from_env()?;
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&config.log_level)),
        )
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .context("Failed to set global default subscriber")?;

    Ok(config)
}
