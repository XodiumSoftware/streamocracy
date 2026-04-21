# Installation

## Prerequisites

- [Rust](https://rustup.rs/) (latest stable version)
- A Discord bot token from the [Discord Developer Portal](https://discord.com/developers/applications)

## Setup

1. Clone the repository
2. Set up your Discord bot token:
   ```bash
   export DISCORD_TOKEN="your-bot-token"
   ```
   Or create a `.env` file with:
   ```
   DISCORD_TOKEN=your-bot-token
   ```
3. Optional: Set a guild ID for instant command updates during testing:
   ```bash
   export GUILD_ID="your-guild-id"
   ```
4. Build and run:
   ```bash
   cargo run --release
   ```

## Required Discord Intents

Enable these intents in your [Discord Developer Portal](https://discord.com/developers/applications):

- **Server Members** - Required for accessing voice states and member info
- **Message Content** - Required for command handling

## Configuration

On first run, the bot creates a `config.toml` file with default settings. Edit this file to customize:

- `discord_token` - Your bot token (can also use env var)
- `guild_id` - Optional guild ID for instant command registration
- `log_level` - Logging verbosity (error, warn, info, debug, trace)
- `default_votekick_duration` - Default poll duration in seconds
- `min_votekick_duration` / `max_votekick_duration` - Duration bounds

## Usage

The bot uses Discord slash commands. Type `/` in chat to see available commands:

| Command     | Description                                                          |
|-------------|----------------------------------------------------------------------|
| `/ping`     | Bot responds with "Pong! 🏓"                                         |
| `/votekick` | Start a votekick against someone screensharing in your voice channel |
| `/vk`       | Alias for `/votekick`                                                |

### Votekick

The votekick command allows server members to vote on kicking a user from a voice channel:

1. User must be in a voice channel
2. Someone in that channel must be screensharing
3. Command displays a dropdown with screensharers to select
4. Other members can vote; majority vote disconnects the target
