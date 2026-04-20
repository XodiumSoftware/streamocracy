# ARCHITECTURE.md

This file provides guidance when working with code in this repository.

## Project Overview

Streamocracy is a simple Discord bot built with Rust and the Serenity framework. It is a small, single-purpose bot without complex configuration or module systems.

## Build & Run Commands

```bash
# Build the bot (debug)
cargo build

# Build the bot (release, with LTO + strip)
cargo build --release

# Run the bot
export DISCORD_TOKEN="your-bot-token"
cargo run
```

## Architecture

### Entry Point

- **`main.rs`** — contains the main function and bot implementation.
    - Sets up logging with `tracing_subscriber`
    - Loads environment variables from `.env` file (optional)
    - Creates a Serenity `Client` with the `Bot` event handler
    - Starts the async runtime and connects to Discord

### Event Handler

**`Bot`** — implements `EventHandler` from `serenity`:

- `ready()` — logs when the bot successfully connects
- `message()` — handles incoming messages, ignoring bot messages

### Commands

The bot responds to simple text commands:

| Command | Response |
|---------|----------|
| `!ping` | `Pong! 🏓` |

### Package Structure

| Path         | Contents                                           |
|--------------|----------------------------------------------------|
| `src/main.rs` | Main entry point, bot event handler, command logic |
| `src/lib.rs`  | Library exports and documentation                  |

### Dependencies

| Crate           | Purpose                                   |
|-----------------|-------------------------------------------|
| `serenity`      | Discord API client and framework          |
| `tokio`         | Async runtime                             |
| `tracing`       | Logging and diagnostics                   |
| `dotenvy`       | Environment variable loading from `.env`  |
| `anyhow`        | Error handling                            |

### Key Conventions

- `unsafe_code` is forbidden project-wide (`[lints.rust] unsafe_code = "forbid"`).
- All Clippy warnings are enabled (`[lints.clippy] all = "warn"`).
- The release profile enables LTO and strips symbols for minimal binary size.
- Bot token is read from the `DISCORD_TOKEN` environment variable (required).
