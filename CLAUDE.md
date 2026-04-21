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
| **Config**         | Environment variables                                    | Configuration source     |
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

- `ready()` — logs when the bot successfully connects and registers slash commands
- `interaction_create()` — handles slash command interactions

### Commands

The bot uses Discord slash commands:

| Command     | Arguments                           | Description                                               |
|-------------|-------------------------------------|-----------------------------------------------------------|
| `/ping`     | None                                | Responds with `Pong! 🏓` to verify bot is responsive      |
| `/votekick` | `user` (required), `duration` (opt) | Start a votekick poll against a user who is screensharing |

### Project Structure

```
src/
├── main.rs              # Main entry point, bot event handler
├── config.rs            # Configuration management (environment variables)
├── utils.rs             # Utility functions for command registration
├── commands/
│   ├── mod.rs           # SlashCommand trait and command registry
│   ├── ping.rs          # Ping command implementation
│   └── votekick.rs      # Votekick command validation
└── polls/
    └── votekick.rs      # Reaction-based poll logic
```

### Key Conventions

- `unsafe_code` is forbidden project-wide (`[lints.rust] unsafe_code = "forbid"`)
- All Clippy warnings are enabled (`[lints.clippy] all = "warn"`)
- The release profile enables LTO and strips symbols for minimal binary size
- Bot token is read from the `DISCORD_TOKEN` environment variable (required)
- Use `tracing` macros (`info!`, `error!`, etc.) for logging
- Slash commands are registered in `ready()` event
- Commands can be registered globally or in a specific guild (set `GUILD_ID` for instant testing)
- Handle interactions via `interaction_create()` event

### Documentation Guidelines

- **Public APIs** — All `pub` items (functions, structs, traits, modules) must have rustdoc comments (`///`)
- **Structs** — Document the purpose and any important invariants:
  ```rust
  /// Slash command for starting a votekick poll against a user.
  pub struct VotekickCommand;
  ```
- **Private functions** — Add rustdoc comments for non-trivial logic or when purpose isn't obvious from the name
- **Trait implementations** — Methods in `impl TraitName for Type` blocks inherit docs from the trait definition and don't need separate documentation unless behavior differs
- **Examples:**
  ```rust
  /// Start a new votekick poll for the target user.
  pub async fn start_votekick(...) { }

  /// Check if target user is in the same channel and screensharing.
  /// Returns (in_same_channel, is_screensharing).
  fn check_target_user(...) -> (bool, bool) { }
  ```

## Testing

- No automated tests in this project currently
- Test by running with a valid Discord token and verifying bot responds to `/ping`

## Configuration

Configuration is loaded from environment variables. A `.env` file can be used for local development.

| Variable                    | Required | Default | Description                                         |
|-----------------------------|----------|---------|-----------------------------------------------------|
| `DISCORD_TOKEN`             | Yes      | -       | Discord bot token from Developer Portal             |
| `GUILD_ID`                  | No       | -       | Guild ID for instant command registration (testing) |
| `LOG_LEVEL`                 | No       | `info`  | Log level filter (trace, debug, info, warn, error)  |
| `DEFAULT_VOTEKICK_DURATION` | No       | `60`    | Default votekick duration in seconds                |
| `MIN_VOTEKICK_DURATION`     | No       | `10`    | Minimum votekick duration in seconds                |
| `MAX_VOTEKICK_DURATION`     | No       | `300`   | Maximum votekick duration in seconds                |
| `RESULTS_DELETE_DELAY`      | No       | `10`    | Results message deletion delay in seconds           |

### Docker Compose Example

```yaml
services:
  bot:
    image: streamocracy:latest
    environment:
      - DISCORD_TOKEN=${DISCORD_TOKEN}
      - GUILD_ID=1234567890123456789
      - LOG_LEVEL=info
```

### Important Notes

- Intents must match those enabled in Discord Developer Portal

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

1. Create a new file in `src/commands/` (e.g., `src/commands/mycommand.rs`)
2. Implement the `SlashCommand` trait for your command struct
3. Add the command to `src/commands/mod.rs` in `get_commands()`
4. Handle errors with `tracing::error!`
5. Update `ARCHITECTURE.md` command table
6. Update `README.md` usage section

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
