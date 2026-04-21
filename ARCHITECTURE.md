# ARCHITECTURE.md

This file provides guidance when working with code in this repository.

## Project Overview

Streamocracy is a Discord bot built with Rust and the Serenity framework. It provides votekick functionality for voice channels, allowing users to vote on kicking someone who is screensharing.

## Build & Run Commands

```bash
# Build the bot (debug)
cargo build

# Build the bot (release, with LTO + strip)
cargo build --release

# First run - creates a default config.toml
# Edit config.toml and set your discord_token
cargo run

# Subsequent runs
cargo run
```

## Architecture

### Entry Point

- **`main.rs`** — contains the main function and bot implementation.
    - Loads configuration from `config.toml`
    - Sets up logging with configured log level
    - Creates a Serenity `Client` with the `Bot` event handler
    - Starts the async runtime and connects to Discord

### Configuration

- **`config.rs`** — configuration management module.
    - Loads settings from `config.toml` in the executable directory
    - Creates a default config file if one doesn't exist
    - Validates required settings (discord_token)
    - Handles log level initialization

**Example `config.toml`:**

```toml
# Discord bot token (required)
discord_token = "your_token_here"

# Optional guild ID for instant command registration during testing
guild_id = null

# Log level filter (error, warn, info, debug, trace)
log_level = "info"

# Votekick duration settings (in seconds)
default_votekick_duration = 60
min_votekick_duration = 10
max_votekick_duration = 300

# How long to display results before auto-deleting (seconds)
results_delete_delay = 10
```

### Event Handler

**`Bot`** — implements `EventHandler` from `serenity`:

- `ready()` — logs when the bot successfully connects and registers slash commands
- `interaction_create()` — handles slash command interactions

### Commands

The bot uses Discord slash commands:

| Command     | Arguments                           | Description                                               |
|-------------|-------------------------------------|-----------------------------------------------------------|
| `/ping`     | None                                | Responds with `Pong! 🏓` to verify bot is responsive      |
| `/votekick` | `user` (required), `duration` (opt) | Start a votekick poll against a user who is screensharing |
| `/vk`       | `user` (required), `duration` (opt) | Alias for `/votekick`                                     |

### Project Structure

```
src/
├── main.rs              # Main entry point, bot event handler
├── config.rs            # Configuration management (TOML file)
├── utils.rs             # Utility functions for command registration and responses
├── commands/
│   ├── mod.rs           # SlashCommand trait definition and command registry
│   ├── ping.rs          # Ping command implementation
│   └── votekick.rs      # Votekick command handler (validates and starts poll)
└── polls/
    ├── mod.rs           # Poll trait and shared poll infrastructure
    └── votekick.rs      # VotekickPoll struct implementing Poll trait
```

### Votekick Flow

1. User runs `/votekick @user` or `/vk @user`
2. `commands::votekick::VotekickCommand::run()` validates:
    - Command is used in a guild (not DM)
    - Invoker is in a voice channel
    - Target user is in the same voice channel
    - Target user is currently screensharing
3. `polls::votekick::VotekickPoll::start()` (via `Poll` trait):
    - Creates embed with votekick details
    - Adds ✅ and ❌ reactions for voting
    - Schedules completion via `polls::schedule_poll_completion()`
4. `polls::votekick::VotekickPoll::on_complete()` (via `Poll` trait):
    - Deletes the poll message
    - Counts reactions (excluding the bot)
    - Requires minimum 2 yes votes and majority to pass
    - Disconnects user from voice channel if passed
    - Sends temporary results message (auto-deletes after configured delay)

### Poll System

The bot uses a generic `Poll` trait for reaction-based voting:

- **`polls::Poll`** — trait defining poll lifecycle:
    - `title()` / `description()` — embed content
    - `duration()` — poll duration in seconds
    - `start()` — sends embed and schedules completion
    - `on_complete()` — called when poll ends with vote counts

- **`polls::VotekickPoll`** — struct implementing `Poll` for votekick functionality:
    - Stores target user, guild, channel metadata
    - Implements votekick-specific completion logic (disconnecting users)

### Dependencies

| Crate                | Purpose                          |
|----------------------|----------------------------------|
| `serenity`           | Discord API client and framework |
| `tokio`              | Async runtime                    |
| `tracing`            | Logging and diagnostics          |
| `tracing-subscriber` | Log formatting and output        |
| `toml`               | TOML configuration file parsing  |
| `serde`              | Serialization/deserialization    |
| `anyhow`             | Error handling                   |

### Key Conventions

- `unsafe_code` is forbidden project-wide (`[lints.rust] unsafe_code = "forbid"`).
- All Clippy warnings are enabled (`[lints.clippy] all = "warn"`).
- The release profile enables LTO and strips symbols for minimal binary size.
- Configuration is loaded from `config.toml` in the executable directory (or path specified by `STREAMOCRACY_CONFIG` env var).
- The bot creates a default config file on first run if one doesn't exist.

## Docker

The bot can be run in a Docker container:

```bash
# Build the image
docker build -t streamocracy .

# Run with config mounted
docker run -v $(pwd)/config.toml:/app/config/config.toml:ro streamocracy

# Or use docker-compose
docker-compose up -d
```

### Environment Variables

| Variable | Description |
|----------|-------------|
| `STREAMOCRACY_CONFIG` | Path to config file (default: `/app/config/config.toml`) |
| `RUST_LOG` | Log level filter (default: `info`) |

### GitHub Container Registry

The image is automatically published to GHCR on every push to main (nightly builds):

```bash
# Pull nightly
docker pull ghcr.io/illyrius666/streamocracy:nightly
```
