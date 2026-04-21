# Installation

## Table of Contents

- [Docker Compose (Recommended)](#docker-compose-recommended)
- [Standalone Executable](#standalone-executable)
- [Build from Source](#build-from-source)
- [Configuration](#configuration)
- [Discord Setup](#discord-setup)
- [Usage](#usage)

---

## Docker Compose (Recommended)

The easiest way to run Streamocracy is using Docker Compose.

### Prerequisites

- [Docker](https://docs.docker.com/get-docker/)
- [Docker Compose](https://docs.docker.com/compose/install/)
- A Discord bot token

### Setup

1. Create a directory for your bot:
   ```bash
   mkdir streamocracy
   cd streamocracy
   ```

2. Download the Docker Compose file:
   ```bash
   curl -O https://raw.githubusercontent.com/XodiumSoftware/streamocracy/main/docker-compose.yml
   ```

3. Create your configuration file:
   ```bash
   cat > config.toml << 'EOF'
   # Discord bot token (required)
   # Get this from https://discord.com/developers/applications
   discord_token = "YOUR_BOT_TOKEN_HERE"

   # Optional guild ID for instant command registration during testing
   # guild_id = 1234567890123456789

   log_level = "info"
   default_votekick_duration = 60
   min_votekick_duration = 10
   max_votekick_duration = 300
   results_delete_delay = 10
   EOF
   ```

4. Replace `YOUR_BOT_TOKEN_HERE` with your actual Discord bot token.

5. Start the bot:
   ```bash
   docker-compose up -d
   ```

6. View logs:
   ```bash
   docker-compose logs -f
   ```

### Updating

To update to the latest nightly build:

```bash
docker-compose pull
docker-compose up -d
```

### Stopping

```bash
docker-compose down
```

---

## Standalone Executable

Download pre-built binaries from GitHub releases.

### Prerequisites

- A Discord bot token

### Setup

1. Download the latest release:
   ```bash
   # Linux (x86_64)
   curl -L -o streamocracy https://github.com/XodiumSoftware/streamocracy/releases/download/nightly/streamocracy-linux-x64
   chmod +x streamocracy
   ```

2. Create a configuration file in the same directory:
   ```bash
   cat > config.toml << 'EOF'
   discord_token = "YOUR_BOT_TOKEN_HERE"
   log_level = "info"
   default_votekick_duration = 60
   min_votekick_duration = 10
   max_votekick_duration = 300
   results_delete_delay = 10
   EOF
   ```

3. Run the bot:
   ```bash
   ./streamocracy
   ```

### Systemd Service (Linux)

To run the bot as a systemd service:

1. Create a user for the bot:
   ```bash
   sudo useradd -r -s /bin/false streamocracy
   ```

2. Install the binary and config:
   ```bash
   sudo mv streamocracy /usr/local/bin/
   sudo mkdir -p /etc/streamocracy
   sudo mv config.toml /etc/streamocracy/
   sudo chown -R streamocracy:streamocracy /etc/streamocracy
   ```

3. Create a systemd service file:
   ```bash
   sudo tee /etc/systemd/system/streamocracy.service << 'EOF'
   [Unit]
   Description=Streamocracy Discord Bot
   After=network-online.target
   Wants=network-online.target

   [Service]
   Type=simple
   User=streamocracy
   Group=streamocracy
   Environment="RUST_LOG=info"
   Environment="STREAMOCRACY_CONFIG=/etc/streamocracy/config.toml"
   WorkingDirectory=/var/lib/streamocracy
   ExecStart=/usr/local/bin/streamocracy
   Restart=on-failure
   RestartSec=5

   [Install]
   WantedBy=multi-user.target
   EOF
   ```

4. Enable and start the service:
   ```bash
   sudo mkdir -p /var/lib/streamocracy
   sudo chown streamocracy:streamocracy /var/lib/streamocracy
   sudo systemctl daemon-reload
   sudo systemctl enable --now streamocracy
   ```

5. View logs:
   ```bash
   sudo journalctl -u streamocracy -f
   ```

---

## Build from Source

Build the bot yourself using Rust.

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable version)
- A Discord bot token

### Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/XodiumSoftware/streamocracy.git
   cd streamocracy
   ```

2. Set up your Discord bot token:
   ```bash
   export DISCORD_TOKEN="your-bot-token"
   ```

3. Optional: Set a guild ID for instant command updates during testing:
   ```bash
   export GUILD_ID="your-guild-id"
   ```

4. Build and run:
   ```bash
   cargo run --release
   ```

---

## Configuration

The bot uses a TOML configuration file. On first run (or if using Docker), create a `config.toml` file:

```toml
# Discord bot token (required)
# Get this from https://discord.com/developers/applications
discord_token = "your-bot-token-here"

# Optional guild ID for instant command registration during testing
# If set, commands register immediately in this guild
# If unset, commands register globally (takes up to 1 hour)
# guild_id = 1234567890123456789

log_level = "info"
default_votekick_duration = 60
min_votekick_duration = 10
max_votekick_duration = 300
results_delete_delay = 10
```

### Configuration Options

| Option                      | Description                        | Default    |
|-----------------------------|------------------------------------|------------|
| `discord_token`             | Your Discord bot token             | (required) |
| `guild_id`                  | Optional guild ID for testing      | `null`     |
| `log_level`                 | Logging verbosity                  | `info`     |
| `default_votekick_duration` | Default poll duration (seconds)    | `60`       |
| `min_votekick_duration`     | Minimum poll duration (seconds)    | `10`       |
| `max_votekick_duration`     | Maximum poll duration (seconds)    | `300`      |
| `results_delete_delay`      | Results message lifetime (seconds) | `10`       |

### Environment Variables

| Variable              | Description                                       |
|-----------------------|---------------------------------------------------|
| `DISCORD_TOKEN`       | Discord bot token (overrides config file)         |
| `GUILD_ID`            | Guild ID for testing (overrides config file)      |
| `RUST_LOG`            | Log level filter (e.g., `info`, `debug`, `trace`) |
| `STREAMOCRACY_CONFIG` | Path to config file                               |

---

## Discord Setup

### Creating a Bot Application

1. Go to the [Discord Developer Portal](https://discord.com/developers/applications)
2. Click "New Application" and give it a name
3. Go to the "Bot" section and click "Add Bot"
4. Copy the bot token (you'll need this for the config)
5. Enable these **Privileged Gateway Intents**:
    - **Server Members Intent** - Required for voice states
    - **Message Content Intent** - Required for command handling

### Inviting the Bot

1. Go to OAuth2 → URL Generator
2. Select scopes: `bot` and `applications.commands`
3. Select permissions:
    - **Connect** (to view voice channels)
    - **Speak** (for future voice features)
    - **Move Members** (required for votekick disconnect)
    - **View Channels** (to see voice channels)
4. Copy the generated URL and open it in your browser
5. Select your server and authorize

---

## Usage

The bot uses Discord slash commands. Type `/` in chat to see available commands:

| Command     | Description                  |
|-------------|------------------------------|
| `/ping`     | Bot responds with "Pong! 🏓" |
| `/votekick` | Start a votekick poll        |
| `/vk`       | Alias for `/votekick`        |

### Votekick

The votekick command allows server members to vote on kicking a user from a voice channel:

1. **Requirements**:
    - You must be in a voice channel
    - Someone in that channel must be screensharing

2. **Usage**:
    - Type `/votekick` or `/vk`
    - Select the user to kick from the dropdown
    - Optionally adjust the poll duration

3. **Voting**:
    - Members vote ✅ or ❌ on the poll
    - Requires at least 2 yes votes
    - Yes votes must exceed no votes

4. **Result**:
    - If passed: User is disconnected from voice
    - If failed: Results are shown and user stays

---

## Troubleshooting

### "Config file not found"

- Make sure `config.toml` exists in the working directory
- For Docker: Check the volume mount path
- For systemd: Verify `STREAMOCRACY_CONFIG` path

### "Failed to connect to Discord"

- Verify your `discord_token` is correct
- Check that all required intents are enabled

### Commands not appearing

- Global commands take up to 1 hour to propagate
- Set `guild_id` in config for instant testing

### Docker: "permission denied"

- Ensure the config file is readable: `chmod 644 config.toml`
- Check that the config.toml path matches the volume mount
