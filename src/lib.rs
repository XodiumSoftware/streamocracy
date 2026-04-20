//! Streamocracy - A simple Discord bot
//!
//! This crate provides a minimal Discord bot built with Serenity.
//!
//! The bot handles basic events and responds to simple commands.
//!
//! ## Usage
//!
//! Set the `DISCORD_TOKEN` environment variable and run the bot:
//!
//! ```bash
//! export DISCORD_TOKEN="your-bot-token"
//! cargo run
//! ```
//!
//! ## Commands
//!
//! - `!ping` - Bot responds with "Pong! 🏓"

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
