# Streamocracy — Claude Code Context

## Project at a Glance

- **Name:** Streamocracy
- **Type:** Discord bot
- **Language:** Rust
- **Build Tool:** Cargo
- **Framework:** Serenity 0.12
- **Output:** Binary executable

## APIs & Tools

| Category           | Technology                                               | Purpose                  |
|--------------------|----------------------------------------------------------|--------------------------|
| **Core API**       | [Serenity](https://github.com/serenity-rs/serenity) 0.12 | Discord bot framework    |
| **Language**       | Rust 2024 edition                                        | Systems language         |
| **Build Tool**     | Cargo                                                    | Build automation         |
| **Async Runtime**  | Tokio 1.43                                               | Async execution          |
| **Logging**        | Tracing + tracing-subscriber                             | Diagnostics and logging  |
| **Env Variables**  | dotenvy                                                  | `.env` file support      |
| **Error Handling** | anyhow                                                   | Ergonomic error handling |

### Serenity Resources

- **Documentation**: https://docs.rs/serenity/0.12.0/serenity/
- **GitHub**: https://github.com/serenity-rs/serenity
- **Examples**: https://github.com/serenity-rs/serenity/tree/current/examples

### Serenity Notes

- Uses async/await pattern with Tokio runtime
- Event-driven architecture with `EventHandler` trait
- Gateway intents must be explicitly declared for each feature
- Message content intent requires enabling in Discord Developer Portal

## Quick Commands

```bash
# Build the bot (debug)
cargo build

# Build the bot (release, with LTO + strip)
cargo build --release

# Run the bot
export DISCORD_TOKEN="your-bot-token"
cargo run
```

## Architecture Overview

### Entry Point

**`main.rs`** — Contains the main function and bot implementation:
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

| Command | Response   |
|---------|------------|
| `!ping` | `Pong! 🏓` |

### Project Structure

```
src/
├── main.rs          # Main entry point, bot event handler, command logic
└── lib.rs           # Library exports and documentation
```

### Key Conventions

- `unsafe_code` is forbidden project-wide (`[lints.rust] unsafe_code = "forbid"`)
- All Clippy warnings are enabled (`[lints.clippy] all = "warn"`)
- The release profile enables LTO and strips symbols for minimal binary size
- Bot token is read from the `DISCORD_TOKEN` environment variable (required)
- Use `tracing` macros (`info!`, `error!`, etc.) for logging
- Bot ignores messages from other bots (`msg.author.bot` check)

## Testing

- No automated tests in this project currently
- Test by running with a valid Discord token and verifying bot responds to `!ping`

## Important Notes

- Simple single-purpose bot — no module system or configuration files
- Intents must match those enabled in Discord Developer Portal
- Bot requires `Message Content` intent enabled for prefix commands

## Claude Code Workflow

### Task Management

**When creating tasks:**

- Number tasks in the name (e.g., "1. Add moderation commands", "2. Update event handler")
- This makes it easy to reference specific tasks in conversation

**After completing each task:**

- Ask the user if they want to git commit the changes or adjust before committing

**When all tasks in a worktree are complete:**

- Ask the user if they want to git publish (push) the changes or adjust before publishing

### After Making Edits

**Always update documentation when code changes:**

1. **ARCHITECTURE.md** — Update if you:
    - Add/remove commands or event handlers
    - Change the project structure
    - Add new Discord intents or features

2. **README.md** — Update if you:
    - Add/remove commands
    - Change installation or usage instructions
    - Modify environment variable requirements

**Rule of thumb:** If a code change would confuse someone reading the docs, update the docs.

## CI/CD

No CI/CD workflows are currently configured. Consider adding:
- GitHub Actions for automated builds on push/PR
- Release workflow for publishing binaries

## Adding Features

### Adding a New Command

1. Edit `src/main.rs`
2. Add command parsing in the `message()` event handler
3. Implement the command logic (e.g., API calls, calculations)
4. Send response using `msg.channel_id.say(&ctx.http, "response").await`
5. Handle errors with `tracing::error!`
6. Update `ARCHITECTURE.md` command table
7. Update `README.md` usage section

### Adding Event Handlers

1. Edit `src/main.rs`
2. Add method to `Bot` struct with `#[serenity::async_trait]`
3. Implement the `EventHandler` method (e.g., `reaction_add`, `guild_member_add`)
4. Add required GatewayIntents in `main()`
5. Enable the intent in Discord Developer Portal
6. Update `ARCHITECTURE.md` with new functionality

### Adding Dependencies

1. Add to `Cargo.toml` `[dependencies]` section
2. Run `cargo check` to verify compilation
3. Import with `use crate_name::...` in source files
4. Document purpose in `ARCHITECTURE.md` APIs table

## Memory System

This project uses Claude Code's persistent memory in `.claude/memory/`. These files persist across sessions and different PCs. Review `MEMORY.md` for existing context about the user and project.
