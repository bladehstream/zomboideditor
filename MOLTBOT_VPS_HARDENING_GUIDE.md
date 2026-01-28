# Moltbot VPS Setup & Hardening Guide

A comprehensive step-by-step guide for deploying Moltbot on an Ubuntu VPS with security hardening.

**Target Environment:**
- Ubuntu 22.04/24.04 LTS VPS
- Remote access via Tailscale only
- Messaging via Telegram
- No inbound internet connectivity

**Estimated Time:** 2-3 hours

---

## Table of Contents

1. [Phase 1: Initial VPS Hardening](#phase-1-initial-vps-hardening)
2. [Phase 2: Install Core Dependencies](#phase-2-install-core-dependencies)
3. [Phase 3: Create Dedicated Moltbot User](#phase-3-create-dedicated-moltbot-user)
4. [Phase 4: Install and Configure Tailscale](#phase-4-install-and-configure-tailscale)
5. [Phase 5: Install Moltbot](#phase-5-install-moltbot)
6. [Phase 6: Configure Moltbot Securely](#phase-6-configure-moltbot-securely)
7. [Phase 7: Set Up Telegram Integration](#phase-7-set-up-telegram-integration)
8. [Phase 8: Implement Docker Sandboxing](#phase-8-implement-docker-sandboxing)
9. [Phase 9: Configure AppArmor Mandatory Access Control](#phase-9-configure-apparmor-mandatory-access-control)
10. [Phase 10: Set Up Audit Logging](#phase-10-set-up-audit-logging)
11. [Phase 11: Configure Network Egress Filtering](#phase-11-configure-network-egress-filtering)
12. [Phase 12: Create Systemd Service with Hardening](#phase-12-create-systemd-service-with-hardening)
13. [Phase 13: Final Verification & Testing](#phase-13-final-verification--testing)
14. [Appendix: Maintenance & Monitoring](#appendix-maintenance--monitoring)

---

## Phase 1: Initial VPS Hardening

### Step 1.1: Connect to Your Fresh VPS

```bash
# Connect via your VPS provider's console or initial SSH
ssh root@your-vps-ip
```

### Step 1.2: Update the System

```bash
# Update package lists and upgrade all packages
apt update && apt upgrade -y

# Install essential security tools
apt install -y \
    ufw \
    fail2ban \
    unattended-upgrades \
    apt-listchanges \
    needrestart \
    curl \
    wget \
    git \
    jq \
    htop \
    tree
```

### Step 1.3: Configure Automatic Security Updates

```bash
# Enable unattended upgrades
dpkg-reconfigure -plow unattended-upgrades
```

When prompted, select **Yes** to enable automatic updates.

Now configure the update settings:

```bash
cat > /etc/apt/apt.conf.d/20auto-upgrades << 'EOF'
APT::Periodic::Update-Package-Lists "1";
APT::Periodic::Unattended-Upgrade "1";
APT::Periodic::AutocleanInterval "7";
APT::Periodic::Download-Upgradeable-Packages "1";
EOF
```

Configure what gets updated:

```bash
cat > /etc/apt/apt.conf.d/50unattended-upgrades << 'EOF'
Unattended-Upgrade::Allowed-Origins {
    "${distro_id}:${distro_codename}";
    "${distro_id}:${distro_codename}-security";
    "${distro_id}ESMApps:${distro_codename}-apps-security";
    "${distro_id}ESM:${distro_codename}-infra-security";
};

Unattended-Upgrade::Package-Blacklist {
};

Unattended-Upgrade::DevRelease "false";
Unattended-Upgrade::Remove-Unused-Kernel-Packages "true";
Unattended-Upgrade::Remove-New-Unused-Dependencies "true";
Unattended-Upgrade::Remove-Unused-Dependencies "true";
Unattended-Upgrade::Automatic-Reboot "false";
Unattended-Upgrade::Automatic-Reboot-WithUsers "false";
EOF
```

### Step 1.4: Configure Timezone

```bash
# Set your timezone (adjust as needed)
timedatectl set-timezone UTC

# Verify
timedatectl status
```

### Step 1.5: Configure Firewall (UFW)

```bash
# Reset UFW to defaults
ufw --force reset

# Set default policies - deny ALL inbound, allow outbound
ufw default deny incoming
ufw default allow outgoing

# DO NOT allow SSH from internet - we'll use Tailscale
# ufw allow ssh  # <-- DO NOT RUN THIS

# Enable the firewall
ufw --force enable

# Verify status
ufw status verbose
```

**Expected output:**
```
Status: active
Logging: on (low)
Default: deny (incoming), allow (outgoing), disabled (routed)
```

> **WARNING:** At this point, if you disconnect, you may lose SSH access until Tailscale is configured. Keep your VPS provider's console access ready as a backup.

### Step 1.6: Harden SSH Configuration

Even though we'll use Tailscale, let's harden SSH:

```bash
# Backup original config
cp /etc/ssh/sshd_config /etc/ssh/sshd_config.backup

# Create hardened SSH config
cat > /etc/ssh/sshd_config.d/hardening.conf << 'EOF'
# Disable root login
PermitRootLogin no

# Disable password authentication (use keys only)
PasswordAuthentication no
PermitEmptyPasswords no

# Use only SSH Protocol 2
Protocol 2

# Limit authentication attempts
MaxAuthTries 3
MaxSessions 2

# Disconnect idle sessions after 5 minutes
ClientAliveInterval 300
ClientAliveCountMax 0

# Disable X11 forwarding
X11Forwarding no

# Disable TCP forwarding (can re-enable if needed)
AllowTcpForwarding no

# Log more verbosely
LogLevel VERBOSE

# Restrict to specific users (we'll add moltbot-admin later)
# AllowUsers moltbot-admin
EOF
```

> **NOTE:** Don't restart SSH yet - we need to create an admin user first.

### Step 1.7: Create an Administrative User

```bash
# Create admin user for your own access
useradd -m -s /bin/bash -G sudo moltbot-admin

# Set a strong password
passwd moltbot-admin

# Create SSH directory
mkdir -p /home/moltbot-admin/.ssh
chmod 700 /home/moltbot-admin/.ssh

# Add your SSH public key (paste your public key here)
cat > /home/moltbot-admin/.ssh/authorized_keys << 'EOF'
ssh-ed25519 YOUR_PUBLIC_KEY_HERE your-email@example.com
EOF

# Set permissions
chmod 600 /home/moltbot-admin/.ssh/authorized_keys
chown -R moltbot-admin:moltbot-admin /home/moltbot-admin/.ssh

# Enable the AllowUsers directive
echo "AllowUsers moltbot-admin" >> /etc/ssh/sshd_config.d/hardening.conf

# Restart SSH
systemctl restart sshd
```

### Step 1.8: Verify Admin Access

**In a NEW terminal window** (keep the root session open):

```bash
# Test SSH access via your VPS IP (while still open)
ssh moltbot-admin@your-vps-ip

# Verify sudo works
sudo whoami
# Should output: root
```

If this works, you can proceed. If not, use the root terminal to fix issues.

### Verification Checkpoint 1

Run these commands to verify Phase 1:

```bash
# Check firewall is active
sudo ufw status
# Expected: Status: active, deny incoming

# Check auto-updates are enabled
systemctl status unattended-upgrades
# Expected: active (running)

# Check SSH hardening
sudo sshd -T | grep -E "permitrootlogin|passwordauthentication"
# Expected: permitrootlogin no, passwordauthentication no
```

---

## Phase 2: Install Core Dependencies

### Step 2.1: Install Node.js 22.x (LTS)

```bash
# Switch to admin user if not already
sudo -i

# Install Node.js 22.x repository
curl -fsSL https://deb.nodesource.com/setup_22.x | bash -

# Install Node.js
apt install -y nodejs

# Verify version (must be 22.12.0 or later for security patches)
node --version
# Expected: v22.x.x (at least v22.12.0)

npm --version
# Expected: 10.x.x
```

### Step 2.2: Install Docker

```bash
# Install prerequisites
apt install -y \
    ca-certificates \
    curl \
    gnupg \
    lsb-release

# Add Docker's official GPG key
install -m 0755 -d /etc/apt/keyrings
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg
chmod a+r /etc/apt/keyrings/docker.gpg

# Add the repository
echo \
  "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu \
  $(. /etc/os-release && echo "$VERSION_CODENAME") stable" | \
  tee /etc/apt/sources.list.d/docker.list > /dev/null

# Install Docker
apt update
apt install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin

# Verify Docker
docker --version
# Expected: Docker version 24.x.x or later

# Enable Docker to start on boot
systemctl enable docker
systemctl start docker
```

### Step 2.3: Install Additional Security Tools

```bash
# Install AppArmor utilities
apt install -y apparmor apparmor-utils

# Install audit daemon
apt install -y auditd audispd-plugins

# Install encryption tools
apt install -y ecryptfs-utils cryptsetup

# Enable services
systemctl enable apparmor auditd
systemctl start apparmor auditd

# Verify AppArmor is running
aa-status
# Expected: apparmor module is loaded, profiles are in enforce mode
```

### Verification Checkpoint 2

```bash
# Verify Node.js
node --version && npm --version
# Expected: v22.x.x and 10.x.x

# Verify Docker
docker run hello-world
# Expected: "Hello from Docker!" message

# Verify AppArmor
sudo aa-status | head -5
# Expected: apparmor module is loaded

# Verify auditd
sudo systemctl status auditd
# Expected: active (running)
```

---

## Phase 3: Create Dedicated Moltbot User

### Step 3.1: Create the Moltbot Service User

```bash
# Create moltbot user with restricted shell
# -r: system account
# -m: create home directory
# -d: home directory path
# -s: login shell (rbash = restricted bash)
useradd -r -m -d /home/moltbot -s /bin/rbash -c "Moltbot Service Account" moltbot

# Verify user creation
id moltbot
# Expected: uid=xxx(moltbot) gid=xxx(moltbot) groups=xxx(moltbot)
```

### Step 3.2: Set Up Restricted Shell Environment

```bash
# Create bin directory for allowed commands
mkdir -p /home/moltbot/bin

# Link only necessary commands to the restricted bin
ln -s /usr/bin/node /home/moltbot/bin/node
ln -s /usr/bin/npm /home/moltbot/bin/npm
ln -s /usr/bin/npx /home/moltbot/bin/npx
ln -s /usr/bin/git /home/moltbot/bin/git
ln -s /bin/ls /home/moltbot/bin/ls
ln -s /bin/cat /home/moltbot/bin/cat
ln -s /bin/mkdir /home/moltbot/bin/mkdir
ln -s /bin/rm /home/moltbot/bin/rm
ln -s /bin/cp /home/moltbot/bin/cp
ln -s /bin/mv /home/moltbot/bin/mv
ln -s /usr/bin/docker /home/moltbot/bin/docker

# Create restricted .bashrc
cat > /home/moltbot/.bashrc << 'EOF'
# Restricted bash configuration for moltbot
export PATH="/home/moltbot/bin"
export HOME="/home/moltbot"
export SHELL="/bin/rbash"

# Prevent modification of PATH
readonly PATH
readonly HOME
readonly SHELL

# Disable potentially dangerous operations
set -r
EOF

# Create .profile
cat > /home/moltbot/.profile << 'EOF'
# Moltbot restricted profile
if [ -f "$HOME/.bashrc" ]; then
    . "$HOME/.bashrc"
fi
EOF

# Lock down permissions
chmod 755 /home/moltbot
chmod 755 /home/moltbot/bin
chmod 644 /home/moltbot/.bashrc
chmod 644 /home/moltbot/.profile
chown -R moltbot:moltbot /home/moltbot
```

### Step 3.3: Add Moltbot to Docker Group (Limited)

```bash
# Add moltbot to docker group so it can run containers
usermod -aG docker moltbot

# Verify
groups moltbot
# Expected: moltbot : moltbot docker
```

### Step 3.4: Create Directory Structure

```bash
# Create moltbot directories
mkdir -p /home/moltbot/{.clawdbot,moltbot,workspace,logs}
mkdir -p /home/moltbot/.clawdbot/{agents,memory,metrics}

# Set ownership
chown -R moltbot:moltbot /home/moltbot

# Set restrictive permissions on sensitive directories
chmod 700 /home/moltbot/.clawdbot
chmod 700 /home/moltbot/workspace
chmod 755 /home/moltbot/logs
```

### Step 3.5: Configure Limited Sudo Access (Optional)

Only add this if you need the bot to perform specific privileged operations:

```bash
# Create sudoers file with VERY limited permissions
cat > /etc/sudoers.d/moltbot << 'EOF'
# Moltbot limited sudo access
# ONLY these specific commands are allowed

# Allow restarting its own service (no args)
moltbot ALL=(root) NOPASSWD: /usr/bin/systemctl restart moltbot.service
moltbot ALL=(root) NOPASSWD: /usr/bin/systemctl status moltbot.service

# Allow reading system logs (read-only operations)
moltbot ALL=(root) NOPASSWD: /usr/bin/journalctl -u moltbot.service -n *

# DENY everything else explicitly
moltbot ALL=(ALL) !ALL
EOF

# Set correct permissions
chmod 440 /etc/sudoers.d/moltbot

# Validate sudoers file
visudo -c
# Expected: /etc/sudoers.d/moltbot: parsed OK
```

### Verification Checkpoint 3

```bash
# Verify user exists
id moltbot

# Verify restricted shell
getent passwd moltbot | cut -d: -f7
# Expected: /bin/rbash

# Verify directory structure
ls -la /home/moltbot/
# Expected: directories with correct permissions

# Verify sudo limits (should fail)
sudo -u moltbot sudo ls /root
# Expected: moltbot is not allowed to run 'sudo' as root
```

---

## Phase 4: Install and Configure Tailscale

### Step 4.1: Install Tailscale

```bash
# Add Tailscale repository
curl -fsSL https://pkgs.tailscale.com/stable/ubuntu/$(lsb_release -cs).noarmor.gpg | \
    tee /usr/share/keyrings/tailscale-archive-keyring.gpg >/dev/null

curl -fsSL https://pkgs.tailscale.com/stable/ubuntu/$(lsb_release -cs).tailscale-keyring.list | \
    tee /etc/apt/sources.list.d/tailscale.list

# Install
apt update
apt install -y tailscale

# Enable service
systemctl enable tailscaled
systemctl start tailscaled
```

### Step 4.2: Authenticate Tailscale

```bash
# Start authentication (this will print a URL)
tailscale up --ssh

# You'll see output like:
# To authenticate, visit:
#   https://login.tailscale.com/a/xxxxxxxxxxxxx
```

Open the URL in your browser and authenticate with your Tailscale account.

### Step 4.3: Verify Tailscale Connection

```bash
# Check Tailscale status
tailscale status
# Expected: Your machine and other devices listed

# Get your Tailscale IP
tailscale ip -4
# Note this IP - you'll use it for SSH access
```

### Step 4.4: Configure Firewall to Allow Tailscale

```bash
# Allow SSH only over Tailscale interface
ufw allow in on tailscale0 to any port 22 proto tcp comment 'SSH over Tailscale'

# Verify
ufw status numbered
```

### Step 4.5: Test Tailscale SSH Access

From another device on your Tailnet:

```bash
# SSH using Tailscale IP
ssh moltbot-admin@100.x.x.x  # Your Tailscale IP

# Or using Tailscale MagicDNS name
ssh moltbot-admin@your-vps-name
```

### Step 4.6: Disable Direct Internet SSH (Final Lockdown)

Once Tailscale SSH works:

```bash
# Remove any internet-facing SSH rules
ufw delete allow ssh 2>/dev/null || true
ufw delete allow 22/tcp 2>/dev/null || true

# Verify only Tailscale SSH remains
ufw status
# Expected: Only "22/tcp on tailscale0" should show for SSH
```

### Verification Checkpoint 4

```bash
# Verify Tailscale is connected
tailscale status | head -5
# Expected: Shows your machine as connected

# Verify firewall
sudo ufw status
# Expected: Port 22 only allowed on tailscale0

# Test SSH over Tailscale (from another device)
ssh moltbot-admin@your-tailscale-ip
# Expected: Successful login
```

---

## Phase 5: Install Moltbot

### Step 5.1: Clone Moltbot Repository

```bash
# Switch to moltbot user's directory
cd /home/moltbot

# Clone as root (moltbot user has restricted shell)
git clone https://github.com/moltbot/moltbot.git /home/moltbot/moltbot

# Set ownership
chown -R moltbot:moltbot /home/moltbot/moltbot
```

### Step 5.2: Install Dependencies

```bash
# Navigate to moltbot directory
cd /home/moltbot/moltbot

# Install npm dependencies as moltbot user
sudo -u moltbot npm install

# Build the project
sudo -u moltbot npm run build
```

### Step 5.3: Verify Installation

```bash
# Check that build succeeded
ls -la /home/moltbot/moltbot/dist/
# Expected: JavaScript files present

# Test that node can run the app (will fail without config, but validates install)
sudo -u moltbot node /home/moltbot/moltbot/dist/index.js --help 2>&1 | head -5
# Expected: Help output or config error (not "module not found")
```

### Step 5.4: Pull Docker Sandbox Image

```bash
# Pull the official sandbox image
docker pull moltbot/sandbox:latest

# Verify
docker images | grep moltbot
# Expected: moltbot/sandbox listed
```

### Verification Checkpoint 5

```bash
# Verify moltbot installation
ls /home/moltbot/moltbot/dist/index.js
# Expected: File exists

# Verify npm packages
ls /home/moltbot/moltbot/node_modules | wc -l
# Expected: Large number (hundreds of packages)

# Verify docker image
docker images moltbot/sandbox
# Expected: Image listed with tag "latest"
```

---

## Phase 6: Configure Moltbot Securely

### Step 6.1: Create Secrets Storage

```bash
# Create secure secrets directory (root-owned)
mkdir -p /etc/moltbot
chmod 700 /etc/moltbot

# Create secrets file
cat > /etc/moltbot/secrets.env << 'EOF'
# Moltbot Secrets - DO NOT COMMIT TO VERSION CONTROL
# Generated: $(date)

# Anthropic API Key (get from console.anthropic.com)
ANTHROPIC_API_KEY=sk-ant-api03-REPLACE_WITH_YOUR_KEY

# Telegram Bot Token (get from @BotFather)
TELEGRAM_BOT_TOKEN=REPLACE_WITH_YOUR_TOKEN

# Gateway authentication token (generate a random string)
MOLTBOT_GATEWAY_TOKEN=REPLACE_WITH_RANDOM_STRING

# OpenAI API Key (optional, for embeddings)
# OPENAI_API_KEY=sk-REPLACE_IF_NEEDED
EOF

# Set strict permissions
chmod 600 /etc/moltbot/secrets.env
chown root:moltbot /etc/moltbot/secrets.env

# Allow moltbot group to read (but not write)
chmod 640 /etc/moltbot/secrets.env
```

### Step 6.2: Generate Gateway Token

```bash
# Generate a secure random token
GATEWAY_TOKEN=$(openssl rand -base64 32 | tr -d '/+=' | head -c 32)
echo "Generated Gateway Token: $GATEWAY_TOKEN"

# Update the secrets file
sed -i "s/MOLTBOT_GATEWAY_TOKEN=REPLACE_WITH_RANDOM_STRING/MOLTBOT_GATEWAY_TOKEN=$GATEWAY_TOKEN/" /etc/moltbot/secrets.env

# Verify
grep MOLTBOT_GATEWAY_TOKEN /etc/moltbot/secrets.env
```

### Step 6.3: Create Main Configuration File

```bash
# Create the moltbot configuration
cat > /home/moltbot/.clawdbot/moltbot.json << 'JSONEOF'
{
  "$schema": "https://moltbot.github.io/schema/config.json",

  "gateway": {
    "bind": "loopback",
    "port": 18789,
    "auth": {
      "required": true,
      "token": "${MOLTBOT_GATEWAY_TOKEN}"
    },
    "rateLimit": {
      "enabled": true,
      "maxRequestsPerMinute": 30
    }
  },

  "agents": {
    "defaults": {
      "model": "claude-sonnet-4-20250514",
      "contextWindow": 128000,

      "sandbox": {
        "mode": "always",
        "docker": {
          "image": "moltbot/sandbox:latest",
          "readOnly": false,
          "capDrop": ["ALL"],
          "capAdd": [],
          "securityOpt": ["no-new-privileges:true"],
          "networkMode": "bridge",
          "memoryLimit": "512m",
          "cpuLimit": "1.0",
          "tmpfs": {
            "/tmp": "size=100m,noexec,nosuid"
          },
          "timeout": 300000
        }
      },

      "tools": {
        "allowlist": [
          "bash",
          "read",
          "write",
          "edit",
          "glob",
          "grep",
          "memory_search",
          "memory_get",
          "sessions_list",
          "sessions_history"
        ],
        "denylist": [
          "browser",
          "canvas",
          "nodes",
          "cron",
          "gateway",
          "discord",
          "slack"
        ]
      },

      "memorySearch": {
        "enabled": true,
        "provider": "local",
        "fallback": "none",
        "sources": ["memory"],
        "query": {
          "maxResults": 6,
          "minScore": 0.35,
          "hybrid": {
            "enabled": true,
            "vectorWeight": 0.7,
            "textWeight": 0.3
          }
        },
        "sync": {
          "watch": true,
          "watchDebounceMs": 2000
        }
      },

      "compaction": {
        "reserveTokensFloor": 20000,
        "memoryFlush": {
          "enabled": true,
          "softThresholdTokens": 4000
        }
      },

      "workspaceAccess": "rw",
      "workspacePath": "/home/moltbot/workspace"
    }
  },

  "sessions": {
    "keyScope": "main",
    "reset": {
      "daily": {
        "enabled": true,
        "hour": 4,
        "timezone": "UTC"
      },
      "idle": {
        "enabled": true,
        "minutes": 60
      }
    }
  },

  "channels": {
    "telegram": {
      "enabled": true,
      "botToken": "${TELEGRAM_BOT_TOKEN}",
      "allowedChatIds": [],
      "dmPolicy": "reject",
      "groupPolicy": "ignore",
      "rateLimit": {
        "messagesPerMinute": 20
      }
    }
  },

  "logging": {
    "level": "info",
    "file": "/home/moltbot/logs/moltbot.log",
    "maxSize": "50m",
    "maxFiles": 5,
    "compress": true
  },

  "metrics": {
    "enabled": true,
    "path": "/home/moltbot/.clawdbot/metrics/quality.jsonl"
  }
}
JSONEOF

# Set ownership and permissions
chown moltbot:moltbot /home/moltbot/.clawdbot/moltbot.json
chmod 600 /home/moltbot/.clawdbot/moltbot.json
```

### Step 6.4: Update Your API Keys

Now edit the secrets file with your actual credentials:

```bash
# Edit secrets file
nano /etc/moltbot/secrets.env

# Replace:
# - ANTHROPIC_API_KEY with your key from console.anthropic.com
# - TELEGRAM_BOT_TOKEN with your token from @BotFather (next section)
```

### Verification Checkpoint 6

```bash
# Verify secrets file permissions
ls -la /etc/moltbot/secrets.env
# Expected: -rw-r----- 1 root moltbot

# Verify config file exists and is valid JSON
sudo -u moltbot cat /home/moltbot/.clawdbot/moltbot.json | jq . > /dev/null && echo "Valid JSON"
# Expected: "Valid JSON"

# Verify config permissions
ls -la /home/moltbot/.clawdbot/moltbot.json
# Expected: -rw------- 1 moltbot moltbot
```

---

## Phase 7: Set Up Telegram Integration

### Step 7.1: Create Telegram Bot

1. Open Telegram and search for `@BotFather`
2. Send `/newbot`
3. Follow prompts to name your bot
4. Copy the bot token provided

### Step 7.2: Get Your Chat ID

1. Search for `@userinfobot` on Telegram
2. Send `/start`
3. It will reply with your user ID (a number like `123456789`)
4. Copy this number

### Step 7.3: Update Configuration with Chat ID

```bash
# Edit the moltbot config to add your chat ID
nano /home/moltbot/.clawdbot/moltbot.json

# Find the "allowedChatIds" line and add your ID:
# "allowedChatIds": [123456789],
```

Or use sed:

```bash
# Replace YOUR_CHAT_ID with your actual numeric chat ID
YOUR_CHAT_ID=123456789
sed -i "s/\"allowedChatIds\": \[\]/\"allowedChatIds\": [$YOUR_CHAT_ID]/" /home/moltbot/.clawdbot/moltbot.json

# Verify
grep allowedChatIds /home/moltbot/.clawdbot/moltbot.json
```

### Step 7.4: Update Secrets with Bot Token

```bash
# Edit secrets file
nano /etc/moltbot/secrets.env

# Replace TELEGRAM_BOT_TOKEN with your actual token from BotFather
# Example: TELEGRAM_BOT_TOKEN=7123456789:AAHxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
```

### Step 7.5: Configure Telegram Privacy Settings

Send these commands to `@BotFather`:

```
/setprivacy
Select your bot
Disable  (so bot only sees messages directed at it in groups)

/setjoingroups
Select your bot
Disable  (prevent bot from being added to groups)

/setcommands
Select your bot
Send:
start - Start conversation
help - Show help
status - Check bot status
clear - Clear conversation history
```

### Verification Checkpoint 7

```bash
# Verify Telegram config
grep -A5 '"telegram"' /home/moltbot/.clawdbot/moltbot.json
# Expected: Shows your config with allowedChatIds populated

# Verify bot token is set (partial match for security)
grep TELEGRAM_BOT_TOKEN /etc/moltbot/secrets.env | cut -c1-30
# Expected: TELEGRAM_BOT_TOKEN=7xxxxxxx (shows beginning of token)
```

---

## Phase 8: Implement Docker Sandboxing

### Step 8.1: Create Custom Sandbox Network

```bash
# Create isolated Docker network for sandboxes
docker network create \
    --driver bridge \
    --subnet 172.30.0.0/24 \
    --opt com.docker.network.bridge.enable_ip_masquerade=false \
    moltbot-sandbox-net

# Verify
docker network ls | grep moltbot
```

### Step 8.2: Configure Docker Daemon Security

```bash
# Create Docker daemon configuration
cat > /etc/docker/daemon.json << 'EOF'
{
  "live-restore": true,
  "userland-proxy": false,
  "no-new-privileges": true,
  "seccomp-profile": "/etc/docker/seccomp-default.json",
  "log-driver": "json-file",
  "log-opts": {
    "max-size": "10m",
    "max-file": "3"
  },
  "storage-driver": "overlay2",
  "default-ulimits": {
    "nofile": {
      "Name": "nofile",
      "Hard": 1024,
      "Soft": 1024
    },
    "nproc": {
      "Name": "nproc",
      "Hard": 256,
      "Soft": 256
    }
  }
}
EOF

# Restart Docker to apply
systemctl restart docker

# Verify
docker info | grep -E "Security|Seccomp"
```

### Step 8.3: Create Seccomp Profile

```bash
# Download default seccomp profile
curl -L https://raw.githubusercontent.com/moby/moby/master/profiles/seccomp/default.json \
    -o /etc/docker/seccomp-default.json

# Verify
ls -la /etc/docker/seccomp-default.json
```

### Step 8.4: Test Sandbox Container

```bash
# Test running a sandbox container as moltbot
sudo -u moltbot docker run --rm \
    --security-opt no-new-privileges:true \
    --cap-drop ALL \
    --memory 512m \
    --cpus 1.0 \
    --read-only \
    --tmpfs /tmp:size=100m,noexec,nosuid \
    moltbot/sandbox:latest \
    echo "Sandbox test successful"

# Expected: "Sandbox test successful"
```

### Verification Checkpoint 8

```bash
# Verify Docker network
docker network inspect moltbot-sandbox-net | jq '.[0].IPAM'
# Expected: Shows subnet 172.30.0.0/24

# Verify Docker security settings
docker info 2>/dev/null | grep -E "Security|Seccomp"
# Expected: Shows security options enabled

# Verify moltbot can run containers
sudo -u moltbot docker ps
# Expected: Empty list (no error)
```

---

## Phase 9: Configure AppArmor Mandatory Access Control

### Step 9.1: Create Moltbot AppArmor Profile

```bash
cat > /etc/apparmor.d/moltbot << 'EOF'
#include <tunables/global>

profile moltbot /usr/bin/node {
  #include <abstractions/base>
  #include <abstractions/nameservice>
  #include <abstractions/openssl>
  #include <abstractions/ssl_certs>

  # Node.js binary and libraries
  /usr/bin/node mr,
  /usr/lib/node_modules/** r,
  /usr/share/nodejs/** r,

  # Moltbot installation (read-only for code)
  /home/moltbot/moltbot/** r,
  /home/moltbot/moltbot/node_modules/** r,
  /home/moltbot/moltbot/dist/** r,

  # Moltbot data directories (read-write)
  /home/moltbot/.clawdbot/** rw,
  /home/moltbot/.clawdbot/agents/**/sessions/** rw,
  /home/moltbot/.clawdbot/memory/** rw,
  /home/moltbot/.clawdbot/metrics/** rw,
  /home/moltbot/workspace/** rw,
  /home/moltbot/logs/** rw,

  # Temporary files
  /tmp/** rw,
  /var/tmp/** rw,

  # Node.js needs these
  /proc/*/maps r,
  /proc/sys/kernel/random/uuid r,
  /sys/devices/system/cpu/** r,

  # Network access (required for Telegram/API)
  network inet stream,
  network inet6 stream,
  network inet dgram,
  network inet6 dgram,

  # Docker socket (for sandbox management)
  /var/run/docker.sock rw,

  # Secrets file (read-only)
  /etc/moltbot/secrets.env r,

  # Deny dangerous paths explicitly
  deny /etc/shadow r,
  deny /etc/gshadow r,
  deny /etc/passwd w,
  deny /etc/group w,
  deny /etc/sudoers* rw,
  deny /etc/ssh/** w,
  deny /root/** rwx,
  deny /home/moltbot-admin/** rwx,
  deny /boot/** rwx,
  deny /usr/bin/* w,
  deny /usr/sbin/* w,
  deny /bin/* w,
  deny /sbin/* w,

  # Deny access to other users' homes
  deny /home/*/.ssh/** rwx,
  deny /home/*/.gnupg/** rwx,
  deny /home/*/.aws/** rwx,

  # Deny raw device access
  deny /dev/sd* rwx,
  deny /dev/nvme* rwx,
  deny /dev/mem rwx,
  deny /dev/kmem rwx,
}
EOF
```

### Step 9.2: Load and Enforce the Profile

```bash
# Parse and load the profile
apparmor_parser -r /etc/apparmor.d/moltbot

# Set to enforce mode
aa-enforce moltbot

# Verify
aa-status | grep moltbot
# Expected: moltbot (enforce)
```

### Step 9.3: Test AppArmor Restrictions

```bash
# Test that moltbot cannot read shadow file
sudo -u moltbot cat /etc/shadow 2>&1
# Expected: Permission denied (enforced by AppArmor)

# Test that moltbot can read its own config
sudo -u moltbot cat /home/moltbot/.clawdbot/moltbot.json | head -3
# Expected: Shows JSON content
```

### Verification Checkpoint 9

```bash
# Verify AppArmor profile is loaded
sudo aa-status | grep -A2 "profiles are in enforce mode"
# Expected: moltbot listed

# Verify profile denies sensitive access
sudo -u moltbot cat /etc/shadow 2>&1 | grep -i denied
# Expected: Shows permission denied

# Check AppArmor logs
sudo dmesg | grep -i apparmor | tail -5
# Expected: May show DENIED entries for blocked access
```

---

## Phase 10: Set Up Audit Logging

### Step 10.1: Configure Auditd Rules

```bash
# Create moltbot-specific audit rules
cat > /etc/audit/rules.d/moltbot.rules << 'EOF'
# Moltbot Audit Rules
# Monitor all activity by the moltbot user

# Delete all existing rules (clean slate for this file)
-D

# Set buffer size
-b 8192

# Log all commands executed by moltbot
-a always,exit -F arch=b64 -F uid=moltbot -S execve -k moltbot_exec

# Log file modifications in sensitive areas
-w /etc/passwd -p wa -k moltbot_sensitive
-w /etc/shadow -p wa -k moltbot_sensitive
-w /etc/sudoers -p wa -k moltbot_sensitive
-w /etc/ssh/sshd_config -p wa -k moltbot_sensitive

# Log moltbot data directory changes
-w /home/moltbot/.clawdbot -p wa -k moltbot_data

# Log moltbot config changes
-w /home/moltbot/.clawdbot/moltbot.json -p wa -k moltbot_config

# Log secrets file access
-w /etc/moltbot/secrets.env -p r -k moltbot_secrets

# Log network connections by moltbot
-a always,exit -F arch=b64 -F uid=moltbot -S connect -k moltbot_network
-a always,exit -F arch=b64 -F uid=moltbot -S socket -k moltbot_network

# Log privilege escalation attempts
-a always,exit -F arch=b64 -F uid=moltbot -S setuid -k moltbot_privesc
-a always,exit -F arch=b64 -F uid=moltbot -S setgid -k moltbot_privesc

# Log Docker operations
-w /var/run/docker.sock -p rwxa -k moltbot_docker

# Make the configuration immutable (requires reboot to change)
-e 2
EOF

# Load the rules
augenrules --load

# Verify rules are loaded
auditctl -l | grep moltbot
```

### Step 10.2: Configure Log Rotation for Audit Logs

```bash
cat > /etc/audit/auditd.conf << 'EOF'
log_file = /var/log/audit/audit.log
log_format = ENRICHED
log_group = adm
priority_boost = 4
flush = INCREMENTAL_ASYNC
freq = 50
num_logs = 10
max_log_file = 50
max_log_file_action = ROTATE
space_left = 75
space_left_action = SYSLOG
admin_space_left = 50
admin_space_left_action = SUSPEND
disk_full_action = SUSPEND
disk_error_action = SUSPEND
tcp_listen_queue = 5
tcp_max_per_addr = 1
tcp_client_max_idle = 0
enable_krb5 = no
krb5_principal = auditd
distribute_network = no
EOF

# Restart auditd
systemctl restart auditd
```

### Step 10.3: Create Audit Log Search Script

```bash
cat > /usr/local/bin/moltbot-audit << 'EOF'
#!/bin/bash
# Search moltbot audit logs
# Usage: moltbot-audit [hours]

HOURS=${1:-24}
START=$(date -d "$HOURS hours ago" '+%m/%d/%Y %H:%M:%S')

echo "=== Moltbot Audit Log (last $HOURS hours) ==="
echo ""

echo "--- Commands Executed ---"
ausearch -k moltbot_exec --start "$START" 2>/dev/null | aureport -x --summary

echo ""
echo "--- Sensitive File Access ---"
ausearch -k moltbot_sensitive --start "$START" 2>/dev/null | aureport -f --summary

echo ""
echo "--- Network Connections ---"
ausearch -k moltbot_network --start "$START" 2>/dev/null | head -20

echo ""
echo "--- Docker Operations ---"
ausearch -k moltbot_docker --start "$START" 2>/dev/null | aureport -x --summary

echo ""
echo "--- Full log available at: /var/log/audit/audit.log ---"
EOF

chmod +x /usr/local/bin/moltbot-audit
```

### Verification Checkpoint 10

```bash
# Verify audit rules
sudo auditctl -l | grep moltbot | wc -l
# Expected: 10+ rules

# Generate test event
sudo -u moltbot ls /etc/ > /dev/null

# Check for audit event
sudo ausearch -k moltbot_exec --raw | tail -1
# Expected: Shows recent event

# Test the audit script
sudo moltbot-audit 1
# Expected: Shows audit summary
```

---

## Phase 11: Configure Network Egress Filtering

### Step 11.1: Identify Required Destinations

Moltbot needs to reach:
- `api.anthropic.com` (Claude API)
- `api.telegram.org` (Telegram Bot API)
- `github.com` / `raw.githubusercontent.com` (for updates, optional)

### Step 11.2: Create IP Sets for Allowed Destinations

```bash
# Install ipset
apt install -y ipset

# Create IP set for allowed destinations
ipset create moltbot_allowed hash:net

# Resolve and add Anthropic API IPs
for ip in $(dig +short api.anthropic.com); do
    ipset add moltbot_allowed $ip
done

# Resolve and add Telegram API IPs
for ip in $(dig +short api.telegram.org); do
    ipset add moltbot_allowed $ip
done

# Add Telegram CDN ranges (149.154.160.0/20, 91.108.4.0/22)
ipset add moltbot_allowed 149.154.160.0/20
ipset add moltbot_allowed 91.108.4.0/22

# Add GitHub (optional, for updates)
for ip in $(dig +short github.com); do
    ipset add moltbot_allowed $ip
done

# Verify
ipset list moltbot_allowed
```

### Step 11.3: Create iptables Rules

```bash
# Get moltbot UID
MOLTBOT_UID=$(id -u moltbot)

# Allow established connections
iptables -A OUTPUT -m owner --uid-owner $MOLTBOT_UID -m state --state ESTABLISHED,RELATED -j ACCEPT

# Allow DNS (required for API resolution)
iptables -A OUTPUT -m owner --uid-owner $MOLTBOT_UID -p udp --dport 53 -j ACCEPT
iptables -A OUTPUT -m owner --uid-owner $MOLTBOT_UID -p tcp --dport 53 -j ACCEPT

# Allow localhost (for Gateway)
iptables -A OUTPUT -m owner --uid-owner $MOLTBOT_UID -d 127.0.0.0/8 -j ACCEPT

# Allow Docker bridge networks
iptables -A OUTPUT -m owner --uid-owner $MOLTBOT_UID -d 172.16.0.0/12 -j ACCEPT

# Allow HTTPS to permitted destinations
iptables -A OUTPUT -m owner --uid-owner $MOLTBOT_UID -p tcp --dport 443 -m set --match-set moltbot_allowed dst -j ACCEPT

# Log blocked connections
iptables -A OUTPUT -m owner --uid-owner $MOLTBOT_UID -j LOG --log-prefix "MOLTBOT_BLOCKED: " --log-level 4

# Block everything else from moltbot
iptables -A OUTPUT -m owner --uid-owner $MOLTBOT_UID -j DROP
```

### Step 11.4: Persist iptables Rules

```bash
# Install iptables-persistent
apt install -y iptables-persistent

# Save rules
netfilter-persistent save

# Verify persistence
cat /etc/iptables/rules.v4 | grep moltbot
```

### Step 11.5: Create IP Set Update Script

API IPs can change, so create a refresh script:

```bash
cat > /usr/local/bin/moltbot-update-ipset << 'EOF'
#!/bin/bash
# Update moltbot allowed IP set with current API IPs

set -e

# Flush existing entries (keep the set)
ipset flush moltbot_allowed

# Re-add API endpoints
for host in api.anthropic.com api.telegram.org github.com; do
    for ip in $(dig +short $host 2>/dev/null); do
        [[ $ip =~ ^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+$ ]] && ipset add moltbot_allowed $ip 2>/dev/null || true
    done
done

# Static Telegram ranges
ipset add moltbot_allowed 149.154.160.0/20
ipset add moltbot_allowed 91.108.4.0/22

echo "Updated moltbot_allowed ipset at $(date)"
ipset list moltbot_allowed | head -10
EOF

chmod +x /usr/local/bin/moltbot-update-ipset

# Add to cron (run daily)
echo "0 4 * * * root /usr/local/bin/moltbot-update-ipset >> /var/log/moltbot-ipset.log 2>&1" > /etc/cron.d/moltbot-ipset
```

### Verification Checkpoint 11

```bash
# Verify ipset
sudo ipset list moltbot_allowed | head -15
# Expected: Shows IP addresses

# Verify iptables rules
sudo iptables -L OUTPUT -n -v | grep moltbot
# Expected: Shows rules for moltbot UID

# Test allowed connection (as moltbot)
sudo -u moltbot curl -s -o /dev/null -w "%{http_code}" https://api.anthropic.com/v1/messages
# Expected: 401 (unauthorized, but connection allowed)

# Test blocked connection (should fail)
sudo -u moltbot curl -s --connect-timeout 5 https://evil-site.example.com 2>&1
# Expected: Connection timeout or refused

# Check for blocked log entries
sudo dmesg | grep MOLTBOT_BLOCKED | tail -3
```

---

## Phase 12: Create Systemd Service with Hardening

### Step 12.1: Create the Service File

```bash
cat > /etc/systemd/system/moltbot.service << 'EOF'
[Unit]
Description=Moltbot AI Assistant
Documentation=https://docs.molt.bot
After=network-online.target docker.service
Wants=network-online.target
Requires=docker.service

[Service]
Type=simple
User=moltbot
Group=moltbot
WorkingDirectory=/home/moltbot/moltbot

# Environment
EnvironmentFile=/etc/moltbot/secrets.env
Environment="NODE_ENV=production"
Environment="HOME=/home/moltbot"

# Execute
ExecStart=/usr/bin/node /home/moltbot/moltbot/dist/index.js
ExecReload=/bin/kill -HUP $MAINPID

# Restart policy
Restart=on-failure
RestartSec=10
StartLimitIntervalSec=300
StartLimitBurst=5

# Resource limits
LimitNOFILE=4096
LimitNPROC=256
MemoryMax=1G
CPUQuota=100%

# Security hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=read-only
PrivateTmp=true
PrivateDevices=true
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectControlGroups=true
RestrictAddressFamilies=AF_INET AF_INET6 AF_UNIX AF_NETLINK
RestrictNamespaces=true
RestrictRealtime=true
RestrictSUIDSGID=true
LockPersonality=true
MemoryDenyWriteExecute=false
SystemCallArchitectures=native

# Allow read-write to specific paths
ReadWritePaths=/home/moltbot/.clawdbot
ReadWritePaths=/home/moltbot/workspace
ReadWritePaths=/home/moltbot/logs
ReadWritePaths=/var/run/docker.sock

# AppArmor profile
AppArmorProfile=moltbot

# Logging
StandardOutput=append:/home/moltbot/logs/moltbot-stdout.log
StandardError=append:/home/moltbot/logs/moltbot-stderr.log
SyslogIdentifier=moltbot

[Install]
WantedBy=multi-user.target
EOF
```

### Step 12.2: Create Log Rotation

```bash
cat > /etc/logrotate.d/moltbot << 'EOF'
/home/moltbot/logs/*.log {
    daily
    rotate 14
    compress
    delaycompress
    missingok
    notifempty
    create 640 moltbot moltbot
    sharedscripts
    postrotate
        systemctl reload moltbot 2>/dev/null || true
    endscript
}

/home/moltbot/.clawdbot/agents/*/sessions/*.jsonl {
    weekly
    rotate 8
    compress
    delaycompress
    missingok
    notifempty
    create 600 moltbot moltbot
}
EOF
```

### Step 12.3: Reload and Enable Service

```bash
# Reload systemd
systemctl daemon-reload

# Enable service to start on boot
systemctl enable moltbot.service

# Verify service file
systemd-analyze verify moltbot.service
# Expected: No errors (warnings about AppArmor are OK)
```

### Step 12.4: Start Moltbot

```bash
# Start the service
systemctl start moltbot.service

# Check status
systemctl status moltbot.service

# View logs
journalctl -u moltbot.service -f
```

### Verification Checkpoint 12

```bash
# Verify service is running
systemctl is-active moltbot.service
# Expected: active

# Verify service hardening
systemctl show moltbot.service | grep -E "NoNewPrivileges|ProtectSystem|PrivateTmp"
# Expected: All set to yes/strict

# Check logs
tail -20 /home/moltbot/logs/moltbot-stdout.log
# Expected: Moltbot startup messages

# Verify process is running as moltbot
ps aux | grep -E "[n]ode.*moltbot"
# Expected: Shows process running as moltbot user
```

---

## Phase 13: Final Verification & Testing

### Step 13.1: Complete Security Checklist

Run this comprehensive verification script:

```bash
cat > /usr/local/bin/moltbot-security-check << 'EOF'
#!/bin/bash
# Moltbot Security Verification Script

echo "=========================================="
echo "  MOLTBOT SECURITY VERIFICATION"
echo "  $(date)"
echo "=========================================="
echo ""

PASS=0
FAIL=0

check() {
    if eval "$2" > /dev/null 2>&1; then
        echo "[PASS] $1"
        ((PASS++))
    else
        echo "[FAIL] $1"
        ((FAIL++))
    fi
}

echo "--- USER & PERMISSIONS ---"
check "Moltbot user exists" "id moltbot"
check "Moltbot has restricted shell" "[[ \$(getent passwd moltbot | cut -d: -f7) == '/bin/rbash' ]]"
check "Moltbot not in sudo group" "! groups moltbot | grep -q sudo"
check "Config file permissions (600)" "[[ \$(stat -c %a /home/moltbot/.clawdbot/moltbot.json) == '600' ]]"
check "Secrets file permissions (640)" "[[ \$(stat -c %a /etc/moltbot/secrets.env) == '640' ]]"

echo ""
echo "--- FIREWALL ---"
check "UFW is active" "ufw status | grep -q 'Status: active'"
check "Default incoming: deny" "ufw status verbose | grep -q 'deny (incoming)'"
check "SSH only on Tailscale" "ufw status | grep -q 'tailscale0'"

echo ""
echo "--- APPARMOR ---"
check "AppArmor is loaded" "aa-status | grep -q 'profiles are loaded'"
check "Moltbot profile enforced" "aa-status | grep -q 'moltbot'"

echo ""
echo "--- DOCKER ---"
check "Docker is running" "systemctl is-active docker"
check "Moltbot can access Docker" "sudo -u moltbot docker ps"
check "Sandbox image exists" "docker images | grep -q 'moltbot/sandbox'"

echo ""
echo "--- NETWORK ---"
check "Gateway on localhost only" "ss -tlnp | grep 18789 | grep -q '127.0.0.1'"
check "Egress filtering active" "iptables -L OUTPUT -n | grep -q 'moltbot'"
check "Tailscale connected" "tailscale status | grep -q 'offers exit'"

echo ""
echo "--- AUDIT ---"
check "Auditd is running" "systemctl is-active auditd"
check "Moltbot audit rules loaded" "auditctl -l | grep -q moltbot"

echo ""
echo "--- SERVICE ---"
check "Moltbot service is active" "systemctl is-active moltbot"
check "Service has NoNewPrivileges" "systemctl show moltbot | grep -q 'NoNewPrivileges=yes'"
check "Service has ProtectSystem" "systemctl show moltbot | grep -q 'ProtectSystem=strict'"

echo ""
echo "=========================================="
echo "  RESULTS: $PASS passed, $FAIL failed"
echo "=========================================="

exit $FAIL
EOF

chmod +x /usr/local/bin/moltbot-security-check

# Run the check
moltbot-security-check
```

### Step 13.2: Test Telegram Communication

1. Open Telegram
2. Find your bot (search for the name you gave it)
3. Send `/start`
4. Send a test message: "Hello, what time is it?"
5. Verify you receive a response

### Step 13.3: Test Security Boundaries

```bash
# Test 1: Verify moltbot cannot read shadow file
echo "Test 1: Shadow file access"
sudo -u moltbot cat /etc/shadow 2>&1 | head -1
# Expected: Permission denied

# Test 2: Verify moltbot cannot modify system files
echo "Test 2: System file modification"
sudo -u moltbot touch /etc/test-file 2>&1
# Expected: Permission denied

# Test 3: Verify network egress filtering
echo "Test 3: Blocked network destination"
sudo -u moltbot curl -s --connect-timeout 5 https://httpbin.org/ip 2>&1 | head -1
# Expected: Timeout or connection refused

# Test 4: Verify allowed network destination
echo "Test 4: Allowed network destination"
sudo -u moltbot curl -s -o /dev/null -w "%{http_code}" https://api.telegram.org/bot123/getMe
# Expected: 401 or 200 (connection allowed)

# Test 5: Verify sandbox container works
echo "Test 5: Sandbox container"
sudo -u moltbot docker run --rm moltbot/sandbox:latest echo "Sandbox OK"
# Expected: "Sandbox OK"
```

### Step 13.4: Monitor Initial Operation

```bash
# Watch logs in real-time
journalctl -u moltbot.service -f &

# In another terminal, watch audit logs
tail -f /var/log/audit/audit.log | grep moltbot &

# Send some test messages via Telegram and observe
```

### Final Verification Checkpoint

```bash
# Run final security check
sudo moltbot-security-check
# Expected: All checks pass

# Verify no unexpected listening ports
ss -tlnp | grep -v -E "(127.0.0.1|::1|tailscale)"
# Expected: Only expected services

# Check for any failed services
systemctl --failed
# Expected: 0 loaded units listed
```

---

## Appendix: Maintenance & Monitoring

### A.1: Daily Monitoring Commands

```bash
# Check service health
systemctl status moltbot.service

# View recent logs
journalctl -u moltbot.service --since "1 hour ago"

# Check audit summary
moltbot-audit 24

# Check resource usage
ps aux | grep moltbot
docker stats --no-stream
```

### A.2: Weekly Maintenance Tasks

```bash
# Update system packages
apt update && apt upgrade -y

# Update Moltbot
cd /home/moltbot/moltbot
git pull
sudo -u moltbot npm install
sudo -u moltbot npm run build
systemctl restart moltbot

# Update Docker image
docker pull moltbot/sandbox:latest

# Review audit logs
moltbot-audit 168  # Last week

# Check disk usage
df -h /home/moltbot
du -sh /home/moltbot/.clawdbot/*
```

### A.3: Backup Script

```bash
cat > /usr/local/bin/moltbot-backup << 'EOF'
#!/bin/bash
# Moltbot backup script

BACKUP_DIR="/var/backups/moltbot"
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="$BACKUP_DIR/moltbot_$DATE.tar.gz"

mkdir -p "$BACKUP_DIR"

# Stop service briefly for consistent backup
systemctl stop moltbot.service

# Create backup
tar -czf "$BACKUP_FILE" \
    /home/moltbot/.clawdbot \
    /home/moltbot/workspace \
    /etc/moltbot/secrets.env \
    --exclude='*.log'

# Restart service
systemctl start moltbot.service

# Remove backups older than 30 days
find "$BACKUP_DIR" -name "moltbot_*.tar.gz" -mtime +30 -delete

echo "Backup created: $BACKUP_FILE"
ls -lh "$BACKUP_FILE"
EOF

chmod +x /usr/local/bin/moltbot-backup

# Add to cron (weekly backup)
echo "0 3 * * 0 root /usr/local/bin/moltbot-backup >> /var/log/moltbot-backup.log 2>&1" > /etc/cron.d/moltbot-backup
```

### A.4: Emergency Procedures

**If you suspect compromise:**

```bash
# 1. Immediately stop the service
systemctl stop moltbot.service

# 2. Block all moltbot network access
iptables -I OUTPUT -m owner --uid-owner $(id -u moltbot) -j DROP

# 3. Capture forensic data
moltbot-audit 720 > /root/moltbot-audit-dump.txt
cp -r /home/moltbot/.clawdbot /root/moltbot-forensics/
docker ps -a > /root/moltbot-docker-state.txt

# 4. Review recent commands
ausearch -k moltbot_exec --start today | aureport -x

# 5. Check for persistence mechanisms
crontab -u moltbot -l
ls -la /home/moltbot/.bashrc /home/moltbot/.profile

# 6. Consider rebuilding the VPS from scratch
```

### A.5: Useful Aliases

Add to `/root/.bashrc`:

```bash
# Moltbot management aliases
alias mb-status='systemctl status moltbot.service'
alias mb-logs='journalctl -u moltbot.service -f'
alias mb-restart='systemctl restart moltbot.service'
alias mb-audit='moltbot-audit'
alias mb-check='moltbot-security-check'
alias mb-backup='moltbot-backup'
```

---

## Quick Reference Card

| Task | Command |
|------|---------|
| Check status | `systemctl status moltbot` |
| View logs | `journalctl -u moltbot -f` |
| Restart service | `systemctl restart moltbot` |
| Security check | `moltbot-security-check` |
| Audit last 24h | `moltbot-audit 24` |
| Update IPs | `moltbot-update-ipset` |
| Backup | `moltbot-backup` |
| SSH access | `ssh moltbot-admin@<tailscale-ip>` |

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Service won't start | Check `journalctl -u moltbot -e` for errors |
| Telegram not responding | Verify `TELEGRAM_BOT_TOKEN` in secrets.env |
| API errors | Verify `ANTHROPIC_API_KEY` in secrets.env |
| Network blocked | Run `moltbot-update-ipset` to refresh IPs |
| AppArmor denials | Check `dmesg \| grep apparmor` |
| Docker permission denied | Verify moltbot is in docker group |
| Audit floods | Adjust rules in `/etc/audit/rules.d/moltbot.rules` |

---

**Guide Version:** 1.0
**Last Updated:** January 2026
**Author:** Generated for secure Moltbot deployment
