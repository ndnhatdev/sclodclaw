#!/bin/bash
# RedClaw Docker Quickstart
# 
# Usage: ./examples/docker-quickstart.sh

set -e

echo "=== RedClaw Docker Quickstart ==="
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Check Docker
if ! command -v docker &> /dev/null; then
    echo -e "${RED}✗ Docker is not installed${NC}"
    echo "Please install Docker: https://docs.docker.com/get-docker/"
    exit 1
fi

if ! docker info &> /dev/null; then
    echo -e "${RED}✗ Docker daemon is not running${NC}"
    echo "Please start Docker Desktop"
    exit 1
fi

echo -e "${GREEN}✓ Docker is ready${NC}"
echo ""

# Build
echo -e "${YELLOW}Building RedClaw image...${NC}"
docker build -t redclaw:dev --target dev .

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Build complete${NC}"
else
    echo -e "${RED}✗ Build failed${NC}"
    exit 1
fi

echo ""
echo -e "${YELLOW}Starting container...${NC}"
docker run -d \
    --name redclaw-dev \
    -p 42617:42617 \
    -v redclaw-data:/redclaw-data \
    -e REDCLAW_CONFIG_DIR=/redclaw-data/.redclaw \
    -e REDCLAW_WORKSPACE=/redclaw-data/workspace \
    -e PROVIDER=ollama \
    -e REDCLAW_MODEL=llama3.2 \
    redclaw:dev

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ RedClaw started${NC}"
    echo ""
    echo "Access points:"
    echo "  Gateway: http://localhost:42617"
    echo ""
    echo "Commands:"
    echo "  View logs:     docker logs -f redclaw-dev"
    echo "  CLI access:    docker exec -it redclaw-dev redclaw --help"
    echo "  Test status:   docker exec redclaw-dev redclaw status"
    echo "  Stop:          docker stop redclaw-dev"
    echo "  Remove:        docker rm redclaw-dev"
    echo ""
else
    echo -e "${RED}✗ Failed to start${NC}"
    exit 1
fi

# Wait for health check
echo -e "${YELLOW}Waiting for container to be ready...${NC}"
sleep 5

# Test
if docker exec redclaw-dev redclaw --help &> /dev/null; then
    echo -e "${GREEN}✓ RedClaw is responding${NC}"
else
    echo -e "${YELLOW}⚠ Container started but not responding yet${NC}"
    echo "Check logs: docker logs redclaw-dev"
fi
