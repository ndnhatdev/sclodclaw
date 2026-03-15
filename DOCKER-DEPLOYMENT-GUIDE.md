# RedClaw Docker Deployment Guide

## 🚀 Quick Start (3 Steps)

### Option 1: PowerShell Script (Windows - Recommended)

```powershell
cd D:/tools/zeroclaw
.\scripts\run-docker.ps1 -WithOllama
```

### Option 2: Docker Compose

```bash
cd D:/tools/zeroclaw
docker-compose -f docker-compose.dev.yml up -d
```

### Option 3: Bash Script (Linux/Mac)

```bash
cd /path/to/zeroclaw
./examples/docker-quickstart.sh
```

---

## 📋 Prerequisites

### Required:
- ✅ Docker Desktop installed and running
- ✅ 4GB+ free disk space
- ✅ Port 42617 available

### Optional (for local LLM):
- ✅ Port 11434 available (for Ollama)

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────┐
│         Your Machine                    │
│                                         │
│  ┌─────────────────────────────────┐   │
│  │  RedClaw Container              │   │
│  │  - Image: redclaw:dev          │   │
│  │  - Binary: /usr/local/bin/redclaw │
│  │  - Port: 42617                 │   │
│  │  - Volume: /redclaw-data       │   │
│  └─────────────────────────────────┘   │
│                                         │
│  ┌─────────────────────────────────┐   │
│  │  Ollama Container (optional)    │   │
│  │  - Image: ollama/ollama:latest │   │
│  │  - Port: 11434                 │   │
│  │  - Models: llama3.2, etc.      │   │
│  └─────────────────────────────────┘   │
└─────────────────────────────────────────┘
         ↓
┌─────────────────────────────────────────┐
│         Access via:                     │
│  - http://localhost:42617 (Gateway)    │
│  - docker exec redclaw-dev redclaw     │
└─────────────────────────────────────────┘
```

---

## 🔧 Configuration

### Environment Variables

| Variable | Description | Default | Example |
|----------|-------------|---------|---------|
| `PROVIDER` | LLM provider | `ollama` | `openrouter`, `ollama`, `openai` |
| `REDCLAW_MODEL` | Model name | `llama3.2` | `anthropic/claude-sonnet-4-20250514` |
| `API_KEY` | Provider API key | - | `sk-...` |
| `REDCLAW_GATEWAY_PORT` | Gateway port | `42617` | `42617` |
| `REDCLAW_WORKSPACE` | Workspace path | `/redclaw-data/workspace` | - |

### Provider Examples

#### Ollama (Local, Free)
```yaml
environment:
  - PROVIDER=ollama
  - REDCLAW_MODEL=llama3.2
```

#### OpenRouter (Cloud, Paid)
```yaml
environment:
  - PROVIDER=openrouter
  - REDCLAW_MODEL=anthropic/claude-sonnet-4-20250514
  - API_KEY=your-api-key-here
```

#### OpenAI (Cloud, Paid)
```yaml
environment:
  - PROVIDER=openai
  - REDCLAW_MODEL=gpt-4o
  - API_KEY=sk-your-api-key
```

---

## 📦 Build & Run

### Build Image

```bash
cd D:/tools/zeroclaw

# Development build
docker build -t redclaw:dev --target dev .

# Production build (smaller, distroless)
docker build -t redclaw:prod --target release .
```

### Run Container

```bash
# Basic run
docker run -d \
  --name redclaw-dev \
  -p 42617:42617 \
  -v redclaw-data:/redclaw-data \
  -e PROVIDER=ollama \
  -e REDCLAW_MODEL=llama3.2 \
  redclaw:dev

# With custom config
docker run -d \
  --name redclaw-dev \
  -p 42617:42617 \
  -v redclaw-data:/redclaw-data \
  -v ./workspace:/redclaw-data/workspace \
  -e API_KEY=sk-xxx \
  -e PROVIDER=openrouter \
  -e REDCLAW_MODEL=anthropic/claude-sonnet-4-20250514 \
  redclaw:dev
```

### With Docker Compose

```bash
# Start
docker-compose -f docker-compose.dev.yml up -d

# View logs
docker-compose -f docker-compose.dev.yml logs -f

# Stop
docker-compose -f docker-compose.dev.yml down

# With Ollama
docker-compose -f docker-compose.dev.yml --profile ollama up -d
```

---

## 🧪 Testing

### CLI Commands

```bash
# Help
docker exec redclaw-dev redclaw --help

# Status
docker exec redclaw-dev redclaw status

# Version
docker exec redclaw-dev redclaw --version

# Modules
docker exec redclaw-dev redclaw modules --help
docker exec redclaw-dev redclaw modules list
docker exec redclaw-dev redclaw modules install ./my-module
```

### Gateway API

```bash
# Health check
curl http://localhost:42617/health

# Get paircode
curl http://localhost:42617/gateway/paircode

# Gateway status
curl http://localhost:42617/gateway/status
```

### Interactive Mode

```bash
# Run interactive session
docker run -it --rm redclaw:dev --help

# Test specific command
docker run -it --rm redclaw:dev status
```

---

## 🗂️ Volume Management

### Persistent Data

```yaml
volumes:
  - redclaw-data:/redclaw-data      # Config + workspace
  - ./workspace:/redclaw-data/workspace  # Your project files
```

### Volume Commands

```bash
# List volumes
docker volume ls | grep redclaw

# Inspect volume
docker volume inspect redclaw-data

# Backup volume
docker run --rm \
  -v redclaw-data:/source \
  -v $(pwd):/backup \
  alpine tar czf /backup/redclaw-backup.tar.gz -C /source .

# Restore volume
docker run --rm \
  -v redclaw-data:/target \
  -v $(pwd):/backup \
  alpine tar xzf /backup/redclaw-backup.tar.gz -C /target
```

---

## 🔍 Troubleshooting

### Container Won't Start

```bash
# Check logs
docker logs redclaw-dev

# Check if port is in use
netstat -ano | findstr :42617

# Remove and recreate
docker rm -f redclaw-dev
docker run -d --name redclaw-dev ... (your run command)
```

### Build Fails

```bash
# Clean build
docker build --no-cache -t redclaw:dev --target dev .

# Check Dockerfile syntax
docker build -t redclaw:dev --target dev . 2>&1 | Select-String "error"
```

### Permission Issues

```bash
# Remove volumes and recreate
docker-compose down -v
docker-compose up -d
```

### Out of Disk Space

```bash
# Clean Docker
docker system prune -a

# Remove old images
docker rmi $(docker images -q redclaw)

# Remove orphaned volumes
docker volume prune
```

---

## 🚢 Production Deployment

### Production Image

```bash
# Build production (distroless, smaller)
docker build -t redclaw:prod --target release .

# Run with API key
docker run -d \
  --name redclaw-prod \
  -p 42617:42617 \
  -v redclaw-data:/redclaw-data \
  -e API_KEY=your-api-key \
  -e PROVIDER=openrouter \
  redclaw:prod
```

### Resource Limits

```yaml
deploy:
  resources:
    limits:
      cpus: '2'
      memory: 2G
    reservations:
      cpus: '0.5'
      memory: 512M
```

### Health Check

```yaml
healthcheck:
  test: ["CMD", "redclaw", "status"]
  interval: 60s
  timeout: 10s
  retries: 3
  start_period: 10s
```

---

## 📊 Monitoring

### Logs

```bash
# View logs
docker logs redclaw-dev

# Follow logs
docker logs -f redclaw-dev

# Last 100 lines
docker logs --tail 100 redclaw-dev
```

### Stats

```bash
# Container stats
docker stats redclaw-dev

# Inspect container
docker inspect redclaw-dev
```

---

## 🧹 Cleanup

```bash
# Stop and remove container
docker stop redclaw-dev
docker rm redclaw-dev

# Remove image
docker rmi redclaw:dev

# Remove volumes
docker volume rm redclaw-data

# Remove everything
docker-compose down -v --rmi all --remove-orphans
```

---

## 📚 Next Steps

After successful deployment:

1. **Configure provider**: Edit config at `/redclaw-data/.redclaw/config.toml`
2. **Install modules**: `docker exec redclaw-dev redclaw modules install <path>`
3. **Start gateway**: `docker exec redclaw-dev redclaw gateway start`
4. **Connect clients**: WebSocket at `ws://localhost:42617`

---

## 🆘 Support

### Common Issues

| Issue | Solution |
|-------|----------|
| Port 42617 in use | Change `HOST_PORT` in docker-compose.yml |
| API key required | Set `API_KEY` environment variable |
| Model not found | Pull model: `docker exec ollama-dev ollama pull llama3.2` |
| Container exits immediately | Check logs: `docker logs redclaw-dev` |

### Get Help

1. Check logs: `docker logs redclaw-dev`
2. Verify config: `docker exec redclaw-dev cat /redclaw-data/.redclaw/config.toml`
3. Test CLI: `docker exec redclaw-dev redclaw status`
4. Review docs: `D:/tools/zeroclaw/DOCKER-DEPLOYMENT-GUIDE.md`

---

## ✅ Verification Checklist

After deployment, verify:

- [ ] Container is running: `docker ps | grep redclaw`
- [ ] Port 42617 is accessible: `curl http://localhost:42617/health`
- [ ] CLI works: `docker exec redclaw-dev redclaw --help`
- [ ] Status command: `docker exec redclaw-dev redclaw status`
- [ ] Modules command: `docker exec redclaw-dev redclaw modules --help`
- [ ] Logs are clean: `docker logs redclaw-dev --tail 20`

---

**🎉 RedClaw is now running in Docker!**

For more info:
- Source: `D:/tools/zeroclaw`
- Docs: `D:/tools/docs`
- Issues: https://github.com/redclaw-labs/redclaw
