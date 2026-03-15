# RedClaw Docker Runbook

## Quick Start

### Option 1: Docker Compose (Recommended)

```bash
# 1. Navigate to zeroclaw directory
cd D:/tools/zeroclaw

# 2. Build and run RedClaw
docker-compose up -d

# 3. View logs
docker-compose logs -f redclaw

# 4. Test the gateway
curl http://localhost:42617/health

# 5. Stop
docker-compose down
```

### Option 2: Docker Build & Run

```bash
# 1. Build the image
docker build -t redclaw:latest --target dev .

# 2. Run the container
docker run -d \
  --name redclaw-dev \
  -p 42617:42617 \
  -v redclaw-data:/redclaw-data \
  -v ${PWD}/workspace:/redclaw-data/workspace \
  -e REDCLAW_WORKSPACE=/redclaw-data/workspace \
  -e PROVIDER=ollama \
  -e REDCLAW_MODEL=llama3.2 \
  redclaw:latest

# 3. View logs
docker logs -f redclaw-dev

# 4. Test
docker exec -it redclaw-dev redclaw --help

# 5. Stop
docker stop redclaw-dev
docker rm redclaw-dev
```

### Option 3: Interactive Mode

```bash
# Run interactively for testing
docker run -it --rm \
  -p 42617:42617 \
  -v redclaw-data:/redclaw-data \
  redclaw:latest \
  --help

# Test specific commands
docker run -it --rm redclaw:latest status
docker run -it --rm redclaw:latest modules --help
```

## Configuration

### Provider Setup

#### Ollama (Local LLM - Free)

```bash
# 1. Start Ollama first
docker-compose --profile ollama up -d

# 2. Pull a model
docker exec ollama ollama pull llama3.2

# 3. Start RedClaw
docker-compose up -d redclaw
```

#### OpenRouter (Cloud LLM - Paid)

```bash
# Set environment variables
export API_KEY="your-openrouter-api-key"

# Start with OpenRouter
docker-compose up -d
# Or set in docker-compose.yml:
# - PROVIDER=openrouter
# - API_KEY=your-api-key
# - REDCLAW_MODEL=anthropic/claude-sonnet-4-20250514
```

### Volume Mounts

| Volume | Purpose | Default Location |
|--------|---------|-----------------|
| `redclaw-data` | Config and workspace | Docker managed volume |
| `./workspace` | Your project files | `D:/tools/zeroclaw/workspace` |
| `ollama-data` | Ollama models | Docker managed volume |

## Testing

### Test CLI Commands

```bash
# Help
docker exec redclaw-dev redclaw --help

# Status
docker exec redclaw-dev redclaw status

# Modules
docker exec redclaw-dev redclaw modules --help
docker exec redclaw-dev redclaw modules list

# Gateway
docker exec redclaw-dev redclaw gateway --help
```

### Test Gateway API

```bash
# Health check
curl http://localhost:42617/health

# Get paircode
curl http://localhost:42617/gateway/paircode
```

### Test with Python SDK

```python
import requests

# Connect to RedClaw gateway
response = requests.get('http://localhost:42617/health')
print(response.json())
```

## Troubleshooting

### Container Won't Start

```bash
# Check logs
docker-compose logs redclaw

# Check if port is in use
netstat -ano | findstr :42617

# Restart
docker-compose down
docker-compose up -d
```

### Permission Issues

```bash
# Fix volume permissions
docker-compose down
docker volume rm zeroclaw_redclaw-data
docker-compose up -d
```

### Build Fails

```bash
# Clean build
docker-compose build --no-cache redclaw

# Or with docker build
docker build -t redclaw:latest --target dev --no-cache .
```

### Check Running Containers

```bash
# List containers
docker ps

# List images
docker images redclaw

# List volumes
docker volume ls | grep redclaw
```

## Production Deployment

### Use Production Image

```bash
# Build production (distroless) image
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

### Docker Swarm

```bash
# Deploy to Swarm
docker stack deploy -c docker-compose.yml redclaw
```

### Kubernetes

See `k8s/` directory for Kubernetes manifests (TODO: create).

## Cleanup

```bash
# Stop and remove all
docker-compose down -v

# Remove images
docker rmi redclaw:latest

# Remove all RedClaw volumes
docker volume rm $(docker volume ls -q -f name=redclaw)
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `REDCLAW_WORKSPACE` | Workspace path | `/redclaw-data/workspace` |
| `REDCLAW_GATEWAY_PORT` | Gateway port | `42617` |
| `PROVIDER` | LLM provider | `ollama` |
| `REDCLAW_MODEL` | Model name | `llama3.2` |
| `API_KEY` | API key for cloud providers | (none) |
| `LANG` | Locale | `C.UTF-8` |

## Next Steps

After running successfully:

1. **Configure provider**: Edit `/redclaw-data/.redclaw/config.toml`
2. **Add modules**: Use `redclaw modules install <path>`
3. **Start gateway**: `redclaw gateway start`
4. **Connect clients**: Use WebSocket at `ws://localhost:42617`

## Support

For issues:
1. Check logs: `docker-compose logs redclaw`
2. Verify config: `docker exec redclaw-dev cat /redclaw-data/.redclaw/config.toml`
3. Test CLI: `docker exec redclaw-dev redclaw status`
