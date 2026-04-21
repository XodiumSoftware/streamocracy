//! Configuration management for Streamocracy
//!
//! Loads configuration from `config.toml` in the executable directory.
//! Creates a default config file if one doesn't exist.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use tracing::info;

/// Bot configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// Results message deletion delay in seconds (default: 10)
    pub results_delete_delay: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            discord_token: String::new(),
            guild_id: None,
            log_level: "info".to_string(),
            default_votekick_duration: 60,
            min_votekick_duration: 10,
            max_votekick_duration: 300,
            results_delete_delay: 10,
        }
    }
}

impl Config {
    /// Load configuration from file or create default if missing
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            info!(
                "Config file not found, creating default at {:?}",
                config_path
            );
            Self::create_default(&config_path)?;
            anyhow::bail!(
                "Please edit {:?} and set your discord_token before running the bot",
                config_path
            );
        }

        let contents = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config file at {:?}", config_path))?;

        let config: Config = toml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file at {:?}", config_path))?;

        if config.discord_token.is_empty() {
            anyhow::bail!("discord_token is required in {:?}", config_path);
        }

        info!("Loaded configuration from {:?}", config_path);
        Ok(config)
    }

    /// Get the path to the config file (executable directory)
    fn config_path() -> Result<PathBuf> {
        let exe_path = env::current_exe().context("Failed to get current executable path")?;
        let exe_dir = exe_path
            .parent()
            .context("Failed to get executable directory")?;
        Ok(exe_dir.join("config.toml"))
    }

    /// Create a default config file
    fn create_default(path: &PathBuf) -> Result<()> {
        let config = Config::default();

        let toml = format!(
            r#"# Streamocracy Bot Configuration
# Place this file in the same directory as the executable

# Discord bot token (required)
# Get this from https://discord.com/developers/applications
discord_token = "{discord_token}"

# Optional guild ID for instant command registration during testing
# If set, commands register immediately in this guild
# If unset, commands register globally (takes up to 1 hour)
# guild_id = 1234567890123456789

log_level = "{log_level}"
default_votekick_duration = {default_votekick_duration}
min_votekick_duration = {min_votekick_duration}
max_votekick_duration = {max_votekick_duration}
results_delete_delay = {results_delete_delay}
"#,
            discord_token = config.discord_token,
            log_level = config.log_level,
            default_votekick_duration = config.default_votekick_duration,
            min_votekick_duration = config.min_votekick_duration,
            max_votekick_duration = config.max_votekick_duration,
            results_delete_delay = config.results_delete_delay,
        );

        fs::write(path, toml)
            .with_context(|| format!("Failed to write default config to {:?}", path))?;

        Ok(())
    }

    /// Get the guild ID as an Option<serenity::all::GuildId>
    pub fn guild_id(&self) -> Option<serenity::all::GuildId> {
        self.guild_id.map(serenity::all::GuildId::new)
    }
}

/// Load config and set up logging
pub fn init() -> Result<Config> {
    let config = Config::load()?;
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
