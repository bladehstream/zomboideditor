# Moltbot VPS Setup & Hardening Guide

A simple, linear guide for deploying Moltbot on an Ubuntu VPS with security hardening.

**What you'll end up with:**
- Moltbot running in Docker on your VPS
- Only accessible via Tailscale (no public internet exposure)
- Telegram as your messaging interface
- All data persists across container restarts

**Time required:** ~1 hour

---

## Before You Start

You'll need:
- A fresh Ubuntu 22.04/24.04 VPS (Hetzner, DigitalOcean, etc.)
- A Tailscale account (free at tailscale.com)
- A Telegram account
- An Anthropic API key (from console.anthropic.com)

---

## Phase 1: Secure Your VPS

### 1.1 Connect to your VPS

Use your VPS provider's console or SSH:
```bash
ssh root@your-vps-ip
```

### 1.2 Update the system

```bash
apt update && apt upgrade -y
```

### 1.3 Create your admin user

```bash
# Create user
useradd -m -s /bin/bash -G sudo moltbot-admin
passwd moltbot-admin

# Set up SSH key
mkdir -p /home/moltbot-admin/.ssh
chmod 700 /home/moltbot-admin/.ssh

# Paste YOUR public SSH key here (replace the example)
echo "ssh-ed25519 AAAA... your-email@example.com" > /home/moltbot-admin/.ssh/authorized_keys

chmod 600 /home/moltbot-admin/.ssh/authorized_keys
chown -R moltbot-admin:moltbot-admin /home/moltbot-admin/.ssh
```

### 1.4 Harden SSH

```bash
cat > /etc/ssh/sshd_config.d/hardening.conf << 'EOF'
PermitRootLogin no
PasswordAuthentication no
MaxAuthTries 3
AllowUsers moltbot-admin
EOF

systemctl restart sshd
```

### 1.5 Set up firewall

```bash
apt install -y ufw
ufw default deny incoming
ufw default allow outgoing
ufw --force enable
```

> **Note:** We're blocking ALL incoming traffic for now. We'll allow Tailscale SSH later.

---

## Phase 2: Install Tailscale

This gives you secure remote access without exposing SSH to the internet.

### 2.1 Install Tailscale

```bash
curl -fsSL https://tailscale.com/install.sh | sh
```

### 2.2 Connect to your Tailnet

```bash
tailscale up --ssh
```

This prints a URL. Open it in your browser and log in to Tailscale.

### 2.3 Get your Tailscale IP

```bash
tailscale ip -4
```

Note this IP (like `100.x.x.x`) - you'll use it to connect from now on.

### 2.4 Allow SSH only over Tailscale

```bash
ufw allow in on tailscale0 to any port 22
```

### 2.5 Test Tailscale SSH (important!)

**Open a NEW terminal** and test:
```bash
ssh moltbot-admin@100.x.x.x  # Use YOUR Tailscale IP
```

If this works, you're good. If not, keep your root session open and troubleshoot.

---

## Phase 3: Install Docker

### 3.1 Install Docker

```bash
curl -fsSL https://get.docker.com | sh
```

### 3.2 Add your user to Docker group

```bash
usermod -aG docker moltbot-admin
```

### 3.3 Verify Docker works

```bash
docker run --rm hello-world
```

---

## Phase 4: Create Your Telegram Bot

Do these steps on your phone or computer in the Telegram app.

### 4.1 Get your Chat ID

1. Open Telegram
2. Search for `@userinfobot`
3. Tap **Start**
4. It replies with your ID - **write this number down**

Example:
```
Id: 123456789    <-- This is your Chat ID
```

### 4.2 Create your bot

1. Search for `@BotFather`
2. Send `/newbot`
3. Answer the prompts (give it a name and username)
4. BotFather gives you a token - **copy and save it securely**

Example token:
```
7123456789:AAHxxxxxxxxxxxxxxxxxxxxxxxxxxx
```

### 4.3 Harden your bot (optional but recommended)

Send these to @BotFather:
- `/setjoingroups` → select your bot → **Disable** (prevents strangers adding it to groups)

---

## Phase 5: Install Moltbot

Now we'll use Moltbot's official setup script.

### 5.1 Switch to your admin user

```bash
su - moltbot-admin
```

### 5.2 Clone Moltbot

```bash
cd ~
git clone https://github.com/moltbot/moltbot.git
cd moltbot
```

### 5.3 Run the setup wizard

```bash
./docker-setup.sh
```

The wizard will ask you for:
- **Anthropic API Key** - paste your key from console.anthropic.com
- **Telegram Bot Token** - paste the token from BotFather

It will:
- Generate a gateway token automatically
- Build the Docker image
- Create all config files
- Start the container

### 5.4 Add your Chat ID to the config

The setup wizard doesn't ask for your Telegram Chat ID, so we need to add it:

```bash
nano ~/.clawdbot/moltbot.json
```

Find the `telegram` section and add your chat ID:
```json
"telegram": {
  "enabled": true,
  "allowedChatIds": [123456789],
  "dmPolicy": "reject",
  "groupPolicy": "ignore"
}
```

Replace `123456789` with YOUR Chat ID from step 4.1.

Save and exit (Ctrl+X, Y, Enter).

### 5.5 Restart to apply changes

```bash
cd ~/moltbot
docker compose restart
```

---

## Phase 6: Test Your Bot

1. Open Telegram
2. Find your bot (search for the username you created)
3. Send `/start`
4. Send a message like "Hello! What's 2+2?"

You should get a response within a few seconds.

**Not working?** Check the logs:
```bash
cd ~/moltbot
docker compose logs --tail 50
```

---

## Phase 7: Enable Auto-Start

Make sure Moltbot starts automatically if your VPS reboots.

```bash
cd ~/moltbot
docker compose down
docker compose up -d
```

The `-d` flag runs it in the background with auto-restart enabled.

---

## Phase 8: Optional Hardening

These steps add extra security but aren't required.

### 8.1 Enable automatic security updates

```bash
sudo apt install -y unattended-upgrades
sudo dpkg-reconfigure -plow unattended-upgrades
# Select: Yes
```

### 8.2 Set up a basic firewall for Docker

```bash
# Only allow outbound connections to known APIs
sudo apt install -y ipset

# Create allowed destinations list
sudo ipset create moltbot_allowed hash:net

# Add Anthropic and Telegram
for ip in $(dig +short api.anthropic.com api.telegram.org); do
  sudo ipset add moltbot_allowed $ip 2>/dev/null || true
done

# Add Telegram CDN ranges
sudo ipset add moltbot_allowed 149.154.160.0/20
sudo ipset add moltbot_allowed 91.108.4.0/22
```

---

## Quick Reference

| Task | Command |
|------|---------|
| View logs | `cd ~/moltbot && docker compose logs -f` |
| Restart | `cd ~/moltbot && docker compose restart` |
| Stop | `cd ~/moltbot && docker compose down` |
| Start | `cd ~/moltbot && docker compose up -d` |
| Update | `cd ~/moltbot && git pull && docker compose pull && docker compose up -d` |
| SSH to VPS | `ssh moltbot-admin@<tailscale-ip>` |

---

## Troubleshooting

| Problem | Solution |
|---------|----------|
| Bot doesn't respond | Check `allowedChatIds` has your Chat ID |
| 401 Unauthorized in logs | Telegram token is wrong - get a new one from @BotFather |
| Can't SSH after reboot | Use VPS provider console, check Tailscale is running |
| Container keeps restarting | Check logs: `docker compose logs --tail 100` |

---

## Where Your Data Lives

| What | Location |
|------|----------|
| Config & sessions | `~/.clawdbot/` |
| Moltbot code | `~/moltbot/` |
| Docker data | Managed by Docker |

Your conversations and memory persist across container restarts.

---

## Summary

You now have:
- ✅ Moltbot running in Docker
- ✅ Secure SSH via Tailscale only (no public exposure)
- ✅ Telegram as your interface
- ✅ Automatic restarts on reboot

**Questions?** Check the [official docs](https://docs.molt.bot) or the [GitHub repo](https://github.com/moltbot/moltbot).
