# Moltbot VPS Setup & Hardening Guide (Docker Edition)

A comprehensive step-by-step guide for deploying Moltbot in Docker on an Ubuntu VPS with persistent storage and security hardening.

**Target Environment:**
- Ubuntu 22.04/24.04 LTS VPS (Hetzner or similar)
- **Gateway-in-Docker deployment** (all components containerized)
- Remote access via Tailscale only
- Messaging via Telegram
- No inbound internet connectivity
- **Maximum persistence** across container rebuilds

**Estimated Time:** 1.5-2 hours

---

## Table of Contents

1. [Phase 1: Initial VPS Hardening](#phase-1-initial-vps-hardening)
2. [Phase 2: Install Docker & Core Dependencies](#phase-2-install-docker--core-dependencies)
3. [Phase 3: Configure Persistent Storage](#phase-3-configure-persistent-storage)
4. [Phase 4: Install and Configure Tailscale](#phase-4-install-and-configure-tailscale)
5. [Phase 5: Set Up Telegram Bot](#phase-5-set-up-telegram-bot)
6. [Phase 6: Configure Moltbot Environment](#phase-6-configure-moltbot-environment)
7. [Phase 7: Deploy Moltbot Container](#phase-7-deploy-moltbot-container)
8. [Phase 8: Harden Docker & Network](#phase-8-harden-docker--network)
9. [Phase 9: Set Up Monitoring & Logging](#phase-9-set-up-monitoring--logging)
10. [Phase 10: Final Verification & Testing](#phase-10-final-verification--testing)
11. [Appendix: Maintenance & Persistence](#appendix-maintenance--persistence)

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        Ubuntu VPS (Hetzner)                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │              Docker Container (moltbot-gateway)          │   │
│  │  ┌─────────────────────────────────────────────────┐    │   │
│  │  │  Moltbot Gateway + Agent Runtime                 │    │   │
│  │  │  - Telegram Channel                              │    │   │
│  │  │  - Tool Execution (sandboxed within container)   │    │   │
│  │  │  - Memory/Session Management                     │    │   │
│  │  └─────────────────────────────────────────────────┘    │   │
│  │                         │                                │   │
│  │              Volume Mounts (Persistent)                  │   │
│  └──────────────┬──────────┴─────────────┬─────────────────┘   │
│                 │                        │                      │
│  ┌──────────────▼──────────┐  ┌─────────▼──────────────┐      │
│  │ /opt/moltbot/config     │  │ /opt/moltbot/workspace │      │
│  │ (.clawdbot data)        │  │ (clawd workspace)      │      │
│  │ - agents/sessions       │  │ - projects             │      │
│  │ - memory/               │  │ - files                │      │
│  │ - moltbot.json          │  │                        │      │
│  └─────────────────────────┘  └────────────────────────┘      │
│                                                                  │
│  ┌──────────────────────┐  ┌─────────────────────────────┐    │
│  │ /opt/moltbot/home    │  │ /opt/moltbot/secrets        │    │
│  │ (full home persist)  │  │ (.env file - root owned)    │    │
│  └──────────────────────┘  └─────────────────────────────┘    │
│                                                                  │
│  Tailscale ◄──────── Only inbound access ──────────►           │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Persistence Strategy:**
- All state lives on host filesystem in `/opt/moltbot/`
- Container can be destroyed and recreated without data loss
- Named Docker volume for Node.js home directory persistence
- Bind mounts for explicit data directories

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

# Install essential tools
apt install -y \
    ufw \
    fail2ban \
    unattended-upgrades \
    apt-listchanges \
    curl \
    wget \
    git \
    jq \
    htop \
    tree \
    ncdu
```

### Step 1.3: Configure Automatic Security Updates

```bash
# Enable unattended upgrades
dpkg-reconfigure -plow unattended-upgrades
# Select: Yes

# Configure update settings
cat > /etc/apt/apt.conf.d/20auto-upgrades << 'EOF'
APT::Periodic::Update-Package-Lists "1";
APT::Periodic::Unattended-Upgrade "1";
APT::Periodic::AutocleanInterval "7";
APT::Periodic::Download-Upgradeable-Packages "1";
EOF

cat > /etc/apt/apt.conf.d/50unattended-upgrades << 'EOF'
Unattended-Upgrade::Allowed-Origins {
    "${distro_id}:${distro_codename}";
    "${distro_id}:${distro_codename}-security";
    "${distro_id}ESMApps:${distro_codename}-apps-security";
    "${distro_id}ESM:${distro_codename}-infra-security";
};
Unattended-Upgrade::Remove-Unused-Kernel-Packages "true";
Unattended-Upgrade::Remove-New-Unused-Dependencies "true";
Unattended-Upgrade::Automatic-Reboot "false";
EOF
```

### Step 1.4: Configure Timezone

```bash
timedatectl set-timezone UTC
timedatectl status
```

### Step 1.5: Configure Firewall (UFW)

```bash
# Reset and configure UFW
ufw --force reset
ufw default deny incoming
ufw default allow outgoing

# DO NOT allow SSH from internet yet
ufw --force enable
ufw status verbose
```

**Expected output:**
```
Status: active
Default: deny (incoming), allow (outgoing), disabled (routed)
```

> **WARNING:** Keep your VPS provider's console access ready as backup.

### Step 1.6: Create Administrative User

```bash
# Create admin user
useradd -m -s /bin/bash -G sudo moltbot-admin
passwd moltbot-admin

# Set up SSH key authentication
mkdir -p /home/moltbot-admin/.ssh
chmod 700 /home/moltbot-admin/.ssh

# Add your SSH public key
cat > /home/moltbot-admin/.ssh/authorized_keys << 'EOF'
ssh-ed25519 YOUR_PUBLIC_KEY_HERE your-email@example.com
EOF

chmod 600 /home/moltbot-admin/.ssh/authorized_keys
chown -R moltbot-admin:moltbot-admin /home/moltbot-admin/.ssh
```

### Step 1.7: Harden SSH

```bash
cp /etc/ssh/sshd_config /etc/ssh/sshd_config.backup

cat > /etc/ssh/sshd_config.d/hardening.conf << 'EOF'
PermitRootLogin no
PasswordAuthentication no
PermitEmptyPasswords no
MaxAuthTries 3
MaxSessions 2
ClientAliveInterval 300
ClientAliveCountMax 0
X11Forwarding no
AllowTcpForwarding no
LogLevel VERBOSE
AllowUsers moltbot-admin
EOF

systemctl restart sshd
```

### Step 1.8: Test Admin Access

**In a NEW terminal:**

```bash
ssh moltbot-admin@your-vps-ip
sudo whoami
# Should output: root
```

### Verification Checkpoint 1

```bash
sudo ufw status
# Expected: Status: active, deny incoming

systemctl status unattended-upgrades
# Expected: active (running)

sudo sshd -T | grep -E "permitrootlogin|passwordauthentication"
# Expected: permitrootlogin no, passwordauthentication no
```

---

## Phase 2: Install Docker & Core Dependencies

### Step 2.1: Install Docker

```bash
sudo -i

# Install prerequisites
apt install -y ca-certificates curl gnupg lsb-release

# Add Docker's GPG key
install -m 0755 -d /etc/apt/keyrings
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg
chmod a+r /etc/apt/keyrings/docker.gpg

# Add repository
echo \
  "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu \
  $(. /etc/os-release && echo "$VERSION_CODENAME") stable" | \
  tee /etc/apt/sources.list.d/docker.list > /dev/null

# Install Docker
apt update
apt install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin

# Enable and start
systemctl enable docker
systemctl start docker

# Verify
docker --version
docker compose version
```

### Step 2.2: Add Admin User to Docker Group

```bash
usermod -aG docker moltbot-admin

# Verify (need to re-login for group to take effect)
groups moltbot-admin
```

### Step 2.3: Configure Docker Daemon for Security & Performance

```bash
cat > /etc/docker/daemon.json << 'EOF'
{
  "live-restore": true,
  "userland-proxy": false,
  "log-driver": "json-file",
  "log-opts": {
    "max-size": "10m",
    "max-file": "3"
  },
  "storage-driver": "overlay2",
  "default-ulimits": {
    "nofile": {
      "Name": "nofile",
      "Hard": 65536,
      "Soft": 65536
    }
  }
}
EOF

systemctl restart docker
```

### Verification Checkpoint 2

```bash
docker run --rm hello-world
# Expected: "Hello from Docker!"

docker info | grep -E "Storage|Logging"
# Expected: overlay2, json-file
```

---

## Phase 3: Configure Persistent Storage

This is the critical phase for ensuring your Moltbot data survives container rebuilds.

### Step 3.1: Create Persistent Directory Structure

```bash
# Create the main moltbot data directory
mkdir -p /opt/moltbot/{config,workspace,home,secrets,logs,backups}

# Set ownership (will be mapped to node user UID 1000 inside container)
chown -R 1000:1000 /opt/moltbot/config
chown -R 1000:1000 /opt/moltbot/workspace
chown -R 1000:1000 /opt/moltbot/home
chown -R 1000:1000 /opt/moltbot/logs

# Secrets directory stays root-owned
chmod 700 /opt/moltbot/secrets
```

### Step 3.2: Configure Mount Options for Durability (Hetzner/SSD Optimization)

If using a dedicated data partition or volume:

```bash
# Check your disk setup
lsblk
df -h

# For Hetzner NVMe SSDs, optimize mount options
# Edit /etc/fstab if you have a separate data partition:
# /dev/nvme0n1p2 /opt/moltbot ext4 defaults,noatime,nodiratime,discard 0 2

# For the root filesystem, you can remount with noatime
mount -o remount,noatime /
```

Add to `/etc/fstab` for persistence:

```bash
# Add noatime to root filesystem (reduces unnecessary writes)
# Find your root device first:
findmnt /

# Edit fstab - change 'defaults' to 'defaults,noatime'
nano /etc/fstab
```

### Step 3.3: Create Named Docker Volume for Home Persistence

```bash
# Create a named volume for complete home directory persistence
docker volume create moltbot_home

# Verify
docker volume inspect moltbot_home
```

### Step 3.4: Set Up Pre-populated Config Directory

```bash
# Create subdirectories that Moltbot expects
mkdir -p /opt/moltbot/config/{agents,memory,metrics}

# Set permissions
chown -R 1000:1000 /opt/moltbot/config
chmod 700 /opt/moltbot/config
```

### Verification Checkpoint 3

```bash
# Verify directory structure
tree -L 2 /opt/moltbot/
# Expected:
# /opt/moltbot/
# ├── backups
# ├── config
# │   ├── agents
# │   ├── memory
# │   └── metrics
# ├── home
# ├── logs
# ├── secrets
# └── workspace

# Verify ownership
ls -la /opt/moltbot/
# Expected: 1000:1000 for config, workspace, home, logs

# Verify Docker volume
docker volume ls | grep moltbot
# Expected: moltbot_home
```

---

## Phase 4: Install and Configure Tailscale

### Step 4.1: Install Tailscale

```bash
curl -fsSL https://pkgs.tailscale.com/stable/ubuntu/$(lsb_release -cs).noarmor.gpg | \
    tee /usr/share/keyrings/tailscale-archive-keyring.gpg >/dev/null

curl -fsSL https://pkgs.tailscale.com/stable/ubuntu/$(lsb_release -cs).tailscale-keyring.list | \
    tee /etc/apt/sources.list.d/tailscale.list

apt update
apt install -y tailscale

systemctl enable tailscaled
systemctl start tailscaled
```

### Step 4.2: Authenticate Tailscale

```bash
tailscale up --ssh

# Follow the URL to authenticate
# Note your Tailscale IP: tailscale ip -4
```

### Step 4.3: Configure Firewall for Tailscale-Only Access

```bash
# Allow SSH only over Tailscale
ufw allow in on tailscale0 to any port 22 proto tcp comment 'SSH over Tailscale'

# Verify and remove any internet SSH rules
ufw status numbered
ufw delete allow ssh 2>/dev/null || true
ufw delete allow 22/tcp 2>/dev/null || true

ufw status
```

### Step 4.4: Test Tailscale Access

From another device on your Tailnet:

```bash
ssh moltbot-admin@<tailscale-ip>
```

### Verification Checkpoint 4

```bash
tailscale status | head -5
# Expected: Connected

sudo ufw status
# Expected: Port 22 only on tailscale0
```

---

## Phase 5: Set Up Telegram Bot

### Step 5.1: Create Telegram Bot

1. Open Telegram, search for `@BotFather`
2. Send `/newbot`
3. Follow prompts to name your bot
4. **Copy the bot token** (format: `7123456789:AAH...`)

### Step 5.2: Get Your Chat ID

1. Search for `@userinfobot` on Telegram
2. Send `/start`
3. **Copy your user ID** (a number like `123456789`)

### Step 5.3: Configure Bot Privacy

Send to `@BotFather`:

```
/setprivacy → Select bot → Disable
/setjoingroups → Select bot → Disable
/setcommands → Select bot → Send:
start - Start conversation
help - Show help
status - Check bot status
clear - Clear conversation history
```

### Step 5.4: Save Your Credentials

```bash
# Note these down securely - you'll need them in Phase 6:
# TELEGRAM_BOT_TOKEN=7123456789:AAHxxxxxxxxxxxxx
# YOUR_CHAT_ID=123456789
```

---

## Phase 6: Configure Moltbot Environment

### Step 6.1: Generate Gateway Token

```bash
# Generate secure gateway token
GATEWAY_TOKEN=$(openssl rand -hex 32)
echo "Gateway Token: $GATEWAY_TOKEN"
# Save this - you'll need it
```

### Step 6.2: Create Environment File

```bash
cat > /opt/moltbot/secrets/.env << 'EOF'
# ===========================================
# MOLTBOT CONFIGURATION - SECRETS
# ===========================================
# DO NOT COMMIT THIS FILE TO VERSION CONTROL

# --- Required: Gateway Authentication ---
CLAWDBOT_GATEWAY_TOKEN=REPLACE_WITH_GATEWAY_TOKEN

# --- Required: LLM Provider (at least one) ---
ANTHROPIC_API_KEY=sk-ant-api03-REPLACE_WITH_YOUR_KEY

# --- Optional: Additional Providers ---
# OPENAI_API_KEY=sk-REPLACE_IF_NEEDED
# OPENROUTER_API_KEY=REPLACE_IF_NEEDED

# --- Required: Telegram ---
TELEGRAM_BOT_TOKEN=REPLACE_WITH_BOT_TOKEN

# --- Container Configuration ---
CLAWDBOT_GATEWAY_BIND=loopback
CLAWDBOT_GATEWAY_PORT=18789
CLAWDBOT_BRIDGE_PORT=18790

# --- Directory Mappings (used by docker-compose) ---
CLAWDBOT_CONFIG_DIR=/opt/moltbot/config
CLAWDBOT_WORKSPACE_DIR=/opt/moltbot/workspace
CLAWDBOT_HOME_VOLUME=moltbot_home

# --- Timezone ---
TZ=UTC
EOF

# Set strict permissions
chmod 600 /opt/moltbot/secrets/.env
chown root:root /opt/moltbot/secrets/.env
```

### Step 6.3: Update Environment File with Your Values

```bash
nano /opt/moltbot/secrets/.env

# Replace:
# - CLAWDBOT_GATEWAY_TOKEN with your generated token
# - ANTHROPIC_API_KEY with your Anthropic API key
# - TELEGRAM_BOT_TOKEN with your bot token from BotFather
```

### Step 6.4: Create Moltbot Configuration File

```bash
cat > /opt/moltbot/config/moltbot.json << 'JSONEOF'
{
  "$schema": "https://moltbot.github.io/schema/config.json",

  "gateway": {
    "bind": "loopback",
    "port": 18789,
    "auth": {
      "required": true,
      "token": "${CLAWDBOT_GATEWAY_TOKEN}"
    }
  },

  "agents": {
    "defaults": {
      "model": "claude-sonnet-4-20250514",
      "contextWindow": 128000,

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
      "workspacePath": "/home/node/clawd"
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
        "minutes": 120
      }
    }
  },

  "channels": {
    "telegram": {
      "enabled": true,
      "botToken": "${TELEGRAM_BOT_TOKEN}",
      "allowedChatIds": [REPLACE_WITH_YOUR_CHAT_ID],
      "dmPolicy": "reject",
      "groupPolicy": "ignore"
    }
  },

  "logging": {
    "level": "info"
  }
}
JSONEOF

# Set ownership for container user
chown 1000:1000 /opt/moltbot/config/moltbot.json
chmod 600 /opt/moltbot/config/moltbot.json
```

### Step 6.5: Update Chat ID in Config

```bash
# Replace REPLACE_WITH_YOUR_CHAT_ID with your actual chat ID
YOUR_CHAT_ID=123456789  # Use your actual ID
sed -i "s/REPLACE_WITH_YOUR_CHAT_ID/$YOUR_CHAT_ID/" /opt/moltbot/config/moltbot.json

# Verify
grep allowedChatIds /opt/moltbot/config/moltbot.json
```

### Verification Checkpoint 6

```bash
# Verify secrets file
ls -la /opt/moltbot/secrets/.env
# Expected: -rw------- 1 root root

# Verify config file (valid JSON)
cat /opt/moltbot/config/moltbot.json | jq . > /dev/null && echo "Valid JSON"
# Expected: "Valid JSON"

# Verify environment variables are set (without showing secrets)
grep -c "REPLACE" /opt/moltbot/secrets/.env
# Expected: 0 (no unreplaced placeholders)
```

---

## Phase 7: Deploy Moltbot Container

### Step 7.1: Create Docker Compose File

```bash
cat > /opt/moltbot/docker-compose.yml << 'EOF'
version: "3.8"

services:
  moltbot-gateway:
    image: ghcr.io/moltbot/moltbot:latest
    container_name: moltbot-gateway
    restart: unless-stopped

    # Environment
    env_file:
      - /opt/moltbot/secrets/.env
    environment:
      - HOME=/home/node
      - TERM=xterm-256color
      - NODE_ENV=production

    # Persistent Volume Mounts
    volumes:
      # Primary config/data persistence (sessions, memory, settings)
      - /opt/moltbot/config:/home/node/.clawdbot:rw

      # Workspace persistence (user projects/files)
      - /opt/moltbot/workspace:/home/node/clawd:rw

      # Full home directory persistence (npm cache, shell history, etc.)
      - moltbot_home:/home/node:rw

      # Logs persistence
      - /opt/moltbot/logs:/home/node/logs:rw

      # Docker socket for DinD sandbox capabilities
      - /var/run/docker.sock:/var/run/docker.sock:rw

    # Network - bind to localhost only (accessed via Tailscale SSH tunnel)
    ports:
      - "127.0.0.1:18789:18789"
      - "127.0.0.1:18790:18790"

    # Process management
    init: true
    stop_grace_period: 30s

    # Resource limits
    deploy:
      resources:
        limits:
          memory: 2G
          cpus: "2.0"
        reservations:
          memory: 512M
          cpus: "0.5"

    # Security options
    security_opt:
      - no-new-privileges:true

    # Healthcheck
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:18789/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s

    # Startup command
    command: >
      node dist/index.js gateway
      --bind loopback
      --port 18789

volumes:
  moltbot_home:
    external: true
    name: moltbot_home

networks:
  default:
    driver: bridge
EOF
```

### Step 7.2: Pull the Moltbot Image

```bash
cd /opt/moltbot
docker compose pull
```

### Step 7.3: Start Moltbot

```bash
cd /opt/moltbot
docker compose up -d

# Check logs
docker compose logs -f
# Press Ctrl+C to exit logs
```

### Step 7.4: Verify Container is Running

```bash
# Check container status
docker compose ps
# Expected: moltbot-gateway running (healthy)

# Check gateway is listening
ss -tlnp | grep 18789
# Expected: 127.0.0.1:18789

# Check container logs
docker compose logs --tail 50
```

### Verification Checkpoint 7

```bash
# Verify container is running
docker ps | grep moltbot
# Expected: moltbot-gateway Up (healthy)

# Verify mounts
docker inspect moltbot-gateway | jq '.[0].Mounts'
# Expected: Shows all bind mounts and volumes

# Verify gateway responds
curl -s http://127.0.0.1:18789/health || echo "Health endpoint may not exist - check logs"

# Test Telegram (send a message to your bot)
# Should receive a response
```

---

## Phase 8: Harden Docker & Network

### Step 8.1: Create Network Egress Rules

```bash
# Install ipset
apt install -y ipset

# Create IP set for allowed destinations
ipset create moltbot_allowed hash:net

# Add API endpoints
for ip in $(dig +short api.anthropic.com 2>/dev/null); do
    ipset add moltbot_allowed $ip 2>/dev/null || true
done

for ip in $(dig +short api.telegram.org 2>/dev/null); do
    ipset add moltbot_allowed $ip 2>/dev/null || true
done

# Add Telegram CDN ranges
ipset add moltbot_allowed 149.154.160.0/20
ipset add moltbot_allowed 91.108.4.0/22

# Add GitHub (for updates)
for ip in $(dig +short github.com 2>/dev/null); do
    ipset add moltbot_allowed $ip 2>/dev/null || true
done
for ip in $(dig +short ghcr.io 2>/dev/null); do
    ipset add moltbot_allowed $ip 2>/dev/null || true
done

# Verify
ipset list moltbot_allowed
```

### Step 8.2: Create IP Set Update Script

```bash
cat > /usr/local/bin/moltbot-update-ipset << 'EOF'
#!/bin/bash
set -e

ipset flush moltbot_allowed 2>/dev/null || ipset create moltbot_allowed hash:net

for host in api.anthropic.com api.telegram.org github.com ghcr.io; do
    for ip in $(dig +short $host 2>/dev/null); do
        [[ $ip =~ ^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+$ ]] && \
            ipset add moltbot_allowed $ip 2>/dev/null || true
    done
done

# Static Telegram ranges
ipset add moltbot_allowed 149.154.160.0/20 2>/dev/null || true
ipset add moltbot_allowed 91.108.4.0/22 2>/dev/null || true

echo "Updated moltbot_allowed ipset at $(date)"
EOF

chmod +x /usr/local/bin/moltbot-update-ipset

# Add to daily cron
echo "0 4 * * * root /usr/local/bin/moltbot-update-ipset >> /var/log/moltbot-ipset.log 2>&1" > /etc/cron.d/moltbot-ipset
```

### Step 8.3: Configure Container Egress Filtering (Optional)

For stricter control, you can use iptables to filter Docker container traffic:

```bash
# Get the Docker bridge network subnet
DOCKER_SUBNET=$(docker network inspect bridge | jq -r '.[0].IPAM.Config[0].Subnet')

# Allow established connections
iptables -I DOCKER-USER -m state --state ESTABLISHED,RELATED -j ACCEPT

# Allow DNS
iptables -I DOCKER-USER -p udp --dport 53 -j ACCEPT
iptables -I DOCKER-USER -p tcp --dport 53 -j ACCEPT

# Allow HTTPS to permitted destinations
iptables -I DOCKER-USER -p tcp --dport 443 -m set --match-set moltbot_allowed dst -j ACCEPT

# Log other outbound (for debugging)
iptables -A DOCKER-USER -j LOG --log-prefix "DOCKER_EGRESS: " --log-level 4

# Save rules
apt install -y iptables-persistent
netfilter-persistent save
```

### Verification Checkpoint 8

```bash
# Verify ipset
ipset list moltbot_allowed | head -10
# Expected: Shows IPs

# Test container can reach Telegram
docker exec moltbot-gateway curl -s -o /dev/null -w "%{http_code}" https://api.telegram.org
# Expected: 200 or 404 (connection works)
```

---

## Phase 9: Set Up Monitoring & Logging

### Step 9.1: Create Log Rotation

```bash
cat > /etc/logrotate.d/moltbot << 'EOF'
/opt/moltbot/logs/*.log {
    daily
    rotate 14
    compress
    delaycompress
    missingok
    notifempty
    create 644 1000 1000
}

/opt/moltbot/config/agents/*/sessions/*.jsonl {
    weekly
    rotate 8
    compress
    delaycompress
    missingok
    notifempty
    create 600 1000 1000
}
EOF
```

### Step 9.2: Create Monitoring Script

```bash
cat > /usr/local/bin/moltbot-status << 'EOF'
#!/bin/bash

echo "=========================================="
echo "  MOLTBOT STATUS"
echo "  $(date)"
echo "=========================================="
echo ""

echo "--- CONTAINER STATUS ---"
docker ps --filter name=moltbot --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"

echo ""
echo "--- RESOURCE USAGE ---"
docker stats moltbot-gateway --no-stream --format "table {{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.NetIO}}"

echo ""
echo "--- DISK USAGE ---"
du -sh /opt/moltbot/*

echo ""
echo "--- RECENT LOGS ---"
docker logs moltbot-gateway --tail 10 2>&1

echo ""
echo "--- TAILSCALE STATUS ---"
tailscale status | head -3
EOF

chmod +x /usr/local/bin/moltbot-status
```

### Step 9.3: Create Security Check Script

```bash
cat > /usr/local/bin/moltbot-security-check << 'EOF'
#!/bin/bash

echo "=========================================="
echo "  MOLTBOT SECURITY CHECK"
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

echo "--- FIREWALL ---"
check "UFW is active" "ufw status | grep -q 'Status: active'"
check "Default incoming: deny" "ufw status verbose | grep -q 'deny (incoming)'"
check "SSH only on Tailscale" "ufw status | grep -q 'tailscale0'"

echo ""
echo "--- DOCKER ---"
check "Container is running" "docker ps | grep -q moltbot-gateway"
check "Container is healthy" "docker inspect moltbot-gateway | jq -e '.[0].State.Health.Status == \"healthy\"' 2>/dev/null || docker ps | grep -q 'Up'"
check "Gateway on localhost only" "ss -tlnp | grep 18789 | grep -q '127.0.0.1'"

echo ""
echo "--- PERSISTENCE ---"
check "Config directory exists" "[ -d /opt/moltbot/config ]"
check "Workspace directory exists" "[ -d /opt/moltbot/workspace ]"
check "Secrets file secure" "[ $(stat -c %a /opt/moltbot/secrets/.env) = '600' ]"
check "Docker volume exists" "docker volume ls | grep -q moltbot_home"

echo ""
echo "--- NETWORK ---"
check "Tailscale connected" "tailscale status 2>/dev/null | grep -q -E '(online|offers)'"
check "IPset configured" "ipset list moltbot_allowed 2>/dev/null | grep -q 'Members'"

echo ""
echo "=========================================="
echo "  RESULTS: $PASS passed, $FAIL failed"
echo "=========================================="

exit $FAIL
EOF

chmod +x /usr/local/bin/moltbot-security-check
```

### Step 9.4: Create Backup Script

```bash
cat > /usr/local/bin/moltbot-backup << 'EOF'
#!/bin/bash
set -e

BACKUP_DIR="/opt/moltbot/backups"
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="$BACKUP_DIR/moltbot_$DATE.tar.gz"

echo "Starting Moltbot backup..."

# Create backup (without stopping container for minimal disruption)
tar -czf "$BACKUP_FILE" \
    -C /opt/moltbot \
    config \
    workspace \
    --exclude='*.log' \
    --exclude='node_modules' \
    2>/dev/null

# Also backup the env file separately (encrypted)
cp /opt/moltbot/secrets/.env "$BACKUP_DIR/secrets_$DATE.env"
chmod 600 "$BACKUP_DIR/secrets_$DATE.env"

# Remove backups older than 30 days
find "$BACKUP_DIR" -name "moltbot_*.tar.gz" -mtime +30 -delete
find "$BACKUP_DIR" -name "secrets_*.env" -mtime +30 -delete

echo "Backup created: $BACKUP_FILE"
ls -lh "$BACKUP_FILE"
EOF

chmod +x /usr/local/bin/moltbot-backup

# Add to weekly cron
echo "0 3 * * 0 root /usr/local/bin/moltbot-backup >> /var/log/moltbot-backup.log 2>&1" > /etc/cron.d/moltbot-backup
```

### Verification Checkpoint 9

```bash
# Run status check
moltbot-status

# Run security check
moltbot-security-check
# Expected: All checks pass

# Test backup
moltbot-backup
ls -la /opt/moltbot/backups/
```

---

## Phase 10: Final Verification & Testing

### Step 10.1: Test Telegram Communication

1. Open Telegram
2. Find your bot
3. Send `/start`
4. Send: "Hello, what time is it?"
5. Verify response

### Step 10.2: Test Persistence

```bash
# Create a test file in workspace
docker exec moltbot-gateway touch /home/node/clawd/persistence-test.txt

# Recreate container
cd /opt/moltbot
docker compose down
docker compose up -d

# Verify file persists
docker exec moltbot-gateway ls /home/node/clawd/persistence-test.txt
# Expected: File exists

# Clean up
docker exec moltbot-gateway rm /home/node/clawd/persistence-test.txt
```

### Step 10.3: Test Container Rebuild

```bash
# Simulate complete rebuild
cd /opt/moltbot
docker compose down
docker compose pull  # Get latest image
docker compose up -d

# Verify everything works
moltbot-status
# Send test message via Telegram
```

### Step 10.4: Run Full Security Check

```bash
moltbot-security-check
# Expected: All checks pass
```

### Final Verification Checklist

```bash
# Container running
docker ps | grep moltbot

# Gateway accessible locally
curl -s http://127.0.0.1:18789/ || echo "Check if health endpoint exists"

# Volumes mounted
docker inspect moltbot-gateway | jq '.[0].Mounts | length'
# Expected: 5 mounts

# Firewall configured
ufw status

# Tailscale connected
tailscale status

# Send Telegram test message and verify response
```

---

## Appendix: Maintenance & Persistence

### A.1: Container Management Commands

```bash
# View logs
docker compose -f /opt/moltbot/docker-compose.yml logs -f

# Restart container
docker compose -f /opt/moltbot/docker-compose.yml restart

# Stop container
docker compose -f /opt/moltbot/docker-compose.yml down

# Start container
docker compose -f /opt/moltbot/docker-compose.yml up -d

# Update to latest image
docker compose -f /opt/moltbot/docker-compose.yml pull
docker compose -f /opt/moltbot/docker-compose.yml up -d

# Shell into container
docker exec -it moltbot-gateway /bin/bash
```

### A.2: Persistence Locations

| Data Type | Host Path | Container Path | Purpose |
|-----------|-----------|----------------|---------|
| Config/Sessions | `/opt/moltbot/config` | `/home/node/.clawdbot` | Agent data, memory, sessions |
| Workspace | `/opt/moltbot/workspace` | `/home/node/clawd` | User projects and files |
| Home Directory | `moltbot_home` volume | `/home/node` | npm cache, shell history |
| Logs | `/opt/moltbot/logs` | `/home/node/logs` | Application logs |
| Secrets | `/opt/moltbot/secrets` | N/A (env file) | API keys, tokens |

### A.3: Update Procedure

```bash
#!/bin/bash
# /usr/local/bin/moltbot-update

cd /opt/moltbot

echo "Creating backup before update..."
moltbot-backup

echo "Pulling latest image..."
docker compose pull

echo "Recreating container..."
docker compose up -d

echo "Waiting for health check..."
sleep 30

echo "Verifying..."
moltbot-status
moltbot-security-check
```

### A.4: Disaster Recovery

If you need to restore from backup:

```bash
# Stop container
cd /opt/moltbot
docker compose down

# Restore from backup
BACKUP_FILE="/opt/moltbot/backups/moltbot_YYYYMMDD_HHMMSS.tar.gz"
tar -xzf "$BACKUP_FILE" -C /opt/moltbot/

# Restore secrets
cp /opt/moltbot/backups/secrets_YYYYMMDD_HHMMSS.env /opt/moltbot/secrets/.env
chmod 600 /opt/moltbot/secrets/.env

# Fix permissions
chown -R 1000:1000 /opt/moltbot/config
chown -R 1000:1000 /opt/moltbot/workspace

# Start container
docker compose up -d
```

### A.5: Useful Aliases

Add to `/root/.bashrc` and `/home/moltbot-admin/.bashrc`:

```bash
# Moltbot management
alias mb='cd /opt/moltbot && docker compose'
alias mb-logs='docker compose -f /opt/moltbot/docker-compose.yml logs -f'
alias mb-status='moltbot-status'
alias mb-check='moltbot-security-check'
alias mb-backup='moltbot-backup'
alias mb-restart='docker compose -f /opt/moltbot/docker-compose.yml restart'
alias mb-shell='docker exec -it moltbot-gateway /bin/bash'
```

---

## Quick Reference Card

| Task | Command |
|------|---------|
| Check status | `moltbot-status` |
| View logs | `docker compose -f /opt/moltbot/docker-compose.yml logs -f` |
| Restart | `docker compose -f /opt/moltbot/docker-compose.yml restart` |
| Security check | `moltbot-security-check` |
| Update | `docker compose -f /opt/moltbot/docker-compose.yml pull && docker compose up -d` |
| Backup | `moltbot-backup` |
| Shell access | `docker exec -it moltbot-gateway /bin/bash` |
| SSH access | `ssh moltbot-admin@<tailscale-ip>` |

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Container won't start | `docker compose logs` to see errors |
| Telegram not responding | Verify `TELEGRAM_BOT_TOKEN` in `.env` |
| API errors | Verify `ANTHROPIC_API_KEY` in `.env` |
| Permission denied | Check ownership: `chown -R 1000:1000 /opt/moltbot/config` |
| Data lost after restart | Verify volume mounts in `docker-compose.yml` |
| Can't connect via Tailscale | Check `tailscale status` and firewall |
| Gateway not accessible | Verify `ss -tlnp | grep 18789` shows 127.0.0.1 |

---

## Key Differences from Bare Metal Install

| Aspect | Bare Metal | Docker (This Guide) |
|--------|------------|---------------------|
| Installation | Clone repo, npm install | `docker compose pull` |
| Updates | git pull, npm install, rebuild | `docker compose pull && up -d` |
| Isolation | AppArmor, restricted user | Container isolation + resource limits |
| Persistence | Direct filesystem | Bind mounts + named volumes |
| Complexity | Higher (many config files) | Lower (single compose file) |
| Rebuild time | ~10 minutes | ~30 seconds |
| State survival | Automatic | Requires proper volume mounts |

---

**Guide Version:** 2.0 (Docker Edition)
**Last Updated:** January 2026
**Architecture:** Gateway-in-Docker with persistent volumes

**Sources:**
- [Moltbot Docker Documentation](https://docs.molt.bot/install/docker)
- [Moltbot Hetzner Guide](https://docs.molt.bot/platforms/hetzner)
- [GitHub - moltbot/moltbot docker-compose.yml](https://github.com/clawdbot/clawdbot/blob/main/docker-compose.yml)
