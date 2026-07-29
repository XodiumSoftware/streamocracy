<div id="readme-top"></div>

<h1 align="center">
  <br />
    <a href="https://xodium.org/">
        <img src="logo.svg" alt="IllyriaBridge Logo" width="200">
    </a>
  <br /><br />
  Streamocracy
  <br />
  <br />
</h1>

<h4 align="center">Democracy, but for your stream</h4><br />

<p align="center">
  A Discord bot for voice channel votekicking during screenshares
</p><br />

<div align="center">

[![Contributors][contributors_shield_url]][contributors_url]
[![Issues][issues_shield_url]][issues_url]
[![License][license_shield_url]][license_url]
[![Docs][docs_shield_url]][docs_url]
</div>

## Table of Contents

- [Features](#features)
- [Usage](#usage)
- [Configuration](#configuration)
- [Guide](GUIDE.md)
- [Built With](#built-with)
- [Code of Conduct][code_of_conduct_url]
- [Contributing][contributing_url]
- [License][license_url]
- [Security][security_url]

## Features

- **Slash commands** — `/ping` and `/votekick`
- **Voice-channel aware** — `/votekick` validates that the caller and target are in the same voice channel and that the target is screensharing
- **Reaction-based polls** — members vote ✅ or ❌; polls auto-complete and disconnect the target when the vote passes
- **Configurable durations** — default, minimum, and maximum poll durations are set via environment variables

## Usage

Streamocracy is a Discord bot driven entirely by slash commands. After inviting it to your server, type `/` in any channel to see the available commands.

| Command     | Arguments                           | Description                                               |
|-------------|-------------------------------------|-----------------------------------------------------------|
| `/ping`     | None                                | Responds with `Pong! 🏓` to verify the bot is responsive  |
| `/votekick` | `user` (required), `duration` (opt) | Start a votekick poll against a user who is screensharing |

### `/votekick` details

1. You must be in a voice channel.
2. The target user must be in the **same** voice channel and currently screensharing.
3. Optionally override the poll duration in seconds.
4. Members vote ✅ (yes) or ❌ (no).
5. The vote passes if there are **at least 2 yes votes** and **yes votes exceed no votes**. If it passes, the target is disconnected from the voice channel.

## Configuration

Configuration is loaded from environment variables. A `.env` file can be used for local development.

| Variable                    | Required | Default | Description                                         |
|-----------------------------|----------|---------|-----------------------------------------------------|
| `DISCORD_TOKEN`             | Yes      | -       | Discord bot token from Developer Portal             |
| `GUILD_ID`                  | No       | -       | Guild ID for instant command registration (testing) |
| `LOG_LEVEL`                 | No       | `info`  | Log level filter (trace, debug, info, warn, error)  |
| `LOG_FORMAT`                | No       | `pretty`| Log output format: `pretty` or `json`               |
| `DEFAULT_VOTEKICK_DURATION` | No       | `60`    | Default votekick duration in seconds                |
| `MIN_VOTEKICK_DURATION`     | No       | `10`    | Minimum votekick duration in seconds                |
| `MAX_VOTEKICK_DURATION`     | No       | `300`   | Maximum votekick duration in seconds                |
| `RESULTS_DELETE_DELAY`      | No       | `10`    | Results message deletion delay in seconds           |
| `MIN_VOTEKICK_YES_VOTES`    | No       | `2`     | Minimum ✅ votes needed for a votekick to pass        |
| `VOTEKICK_RATE_LIMIT_SECS`  | No       | `60`    | Cooldown per initiator in the same guild/channel      |

### Quick start (from source)

```bash
export DISCORD_TOKEN="your-bot-token"
export GUILD_ID="your-guild-id"   # optional, for instant slash command updates
cargo run --release
```

For Docker Compose, binary releases, and a full setup walkthrough, see [`GUIDE.md`](GUIDE.md).

## Built With

<div align="center">

[![Built With][built_with_shield_url]][built_with_url]
</div>

<p align="right"><a href="#readme-top">▲</a></p>

[built_with_shield_url]: https://skillicons.dev/icons?i=rust,github

[built_with_url]: https://skillicons.dev

[code_of_conduct_url]: https://github.com/XodiumSoftware/streamocracy?tab=coc-ov-file

[contributing_url]: https://github.com/XodiumSoftware/streamocracy?tab=contributing-ov-file

[contributors_shield_url]: https://img.shields.io/github/contributors/XodiumSoftware/streamocracy?style=for-the-badge&color=blue

[contributors_url]: https://github.com/XodiumSoftware/streamocracy/graphs/contributors

[issues_shield_url]: https://img.shields.io/github/issues/XodiumSoftware/streamocracy?style=for-the-badge&color=yellow

[issues_url]: https://github.com/XodiumSoftware/streamocracy/issues

[license_shield_url]: https://img.shields.io/github/license/XodiumSoftware/streamocracy?style=for-the-badge&color=green

[license_url]: https://github.com/XodiumSoftware/streamocracy?tab=AGPL-3.0-1-ov-file

[docs_shield_url]: https://img.shields.io/badge/docs-github--pages-blue?style=for-the-badge

[docs_url]: https://streamocracy.xodium.org/

[security_url]: https://github.com/XodiumSoftware/streamocracy?tab=security-ov-file
