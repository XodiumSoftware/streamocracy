# Arch Linux Packaging for Streamocracy

This directory contains the files needed to build and install Streamocracy on Arch Linux.

## Files

| File                    | Description                                |
|-------------------------|--------------------------------------------|
| `PKGBUILD`              | Build script for creating the Arch package |
| `streamocracy.service`  | Systemd service file for running the bot   |
| `streamocracy.sysusers` | Creates the `streamocracy` system user     |

## Building Locally

```bash
cd pkg
makepkg -si
```

## Publishing to AUR

1. Update `pkgver` in `PKGBUILD` when releasing a new version
2. Generate checksums: `updpkgsums`
3. Generate .SRCINFO: `makepkg --printsrcinfo > .SRCINFO`
4. Push to AUR:
   ```bash
   git clone ssh://aur@aur.archlinux.org/streamocracy.git
   cp PKGBUILD .SRCINFO streamocracy.service streamocracy.sysusers streamocracy/
   cd streamocracy
   git add .
   git commit -m "Update to vX.Y.Z"
   git push
   ```

## Installation

After publishing to AUR, users can install via:

```bash
# Using yay (or any AUR helper)
yay -S streamocracy

# Or manually
git clone https://aur.archlinux.org/streamocracy.git
cd streamocracy
makepkg -si
```

## Configuration

1. Create the config directory and copy the example config:
   ```bash
   sudo mkdir -p /etc/streamocracy /var/lib/streamocracy
   sudo cp /etc/streamocracy/config.toml.example /etc/streamocracy/config.toml
   sudo chown streamocracy:streamocracy /etc/streamocracy/config.toml
   sudo chown streamocracy:streamocracy /var/lib/streamocracy
   ```

2. Edit the config and add your Discord bot token:
   ```bash
   sudo nano /etc/streamocracy/config.toml
   ```

3. Start the service:
   ```bash
   sudo systemctl enable --now streamocracy
   ```

4. View logs:
   ```bash
   sudo journalctl -u streamocracy -f
   ```

## Service Management

| Command                               | Description               |
|---------------------------------------|---------------------------|
| `sudo systemctl start streamocracy`   | Start the bot             |
| `sudo systemctl stop streamocracy`    | Stop the bot              |
| `sudo systemctl restart streamocracy` | Restart the bot           |
| `sudo systemctl enable streamocracy`  | Enable auto-start on boot |
| `sudo systemctl status streamocracy`  | Check service status      |

## Automated Publishing

A GitHub Actions workflow (`.github/workflows/publish-aur.yml`) automatically publishes to AUR on releases.

### Setup

1. **Generate SSH key pair** (without passphrase):
   ```bash
   ssh-keygen -t ed25519 -C "aur@streamocracy" -f ~/.ssh/aur
   ```

2. **Add public key to AUR**:
    - Go to https://aur.archlinux.org/
    - Login and go to "My Account"
    - Paste the contents of `~/.ssh/aur.pub` into "SSH Public Key"

3. **Add private key to GitHub Secrets**:
    - Go to Repository Settings → Secrets and variables → Actions
    - Create secret named `AUR_SSH_PRIVATE_KEY`
    - Paste the contents of `~/.ssh/aur`

### Manual Trigger

You can also trigger manually from the Actions tab if needed.
