# RedClaw Docker Run Script
# Usage: .\scripts\run-docker.ps1

param(
    [string]$Provider = "ollama",
    [string]$Model = "llama3.2",
    [string]$ApiKey = "",
    [switch]$WithOllama,
    [switch]$Interactive
)

Write-Host "=== RedClaw Docker Deployment ===" -ForegroundColor Cyan
Write-Host ""

# Check if Docker is running
try {
    $dockerStatus = docker ps -ErrorAction Stop
    Write-Host "✓ Docker is running" -ForegroundColor Green
} catch {
    Write-Host "✗ Docker is not running or not accessible" -ForegroundColor Red
    Write-Host ""
    Write-Host "Please start Docker Desktop and try again." -ForegroundColor Yellow
    exit 1
}

# Check if image exists
$imageExists = docker images redclaw:dev --format "{{.Repository}}"
if (-not $imageExists) {
    Write-Host "Building RedClaw Docker image..." -ForegroundColor Yellow
    docker build -t redclaw:dev --target dev .
    if ($LASTEXITCODE -ne 0) {
        Write-Host "✗ Build failed" -ForegroundColor Red
        exit 1
    }
    Write-Host "✓ Build complete" -ForegroundColor Green
} else {
    Write-Host "✓ Docker image exists" -ForegroundColor Green
}

Write-Host ""
Write-Host "Starting RedClaw container..." -ForegroundColor Cyan

# Build docker run command
$runArgs = @(
    "run", "-d",
    "--name", "redclaw-dev",
    "-p", "42617:42617",
    "-v", "redclaw-data:/redclaw-data",
    "-e", "REDCLAW_CONFIG_DIR=/redclaw-data/.redclaw",
    "-e", "REDCLAW_WORKSPACE=/redclaw-data/workspace",
    "-e", "REDCLAW_GATEWAY_PORT=42617",
    "-e", "PROVIDER=$Provider",
    "-e", "REDCLAW_MODEL=$Model"
)

if ($ApiKey) {
    $runArgs += "-e", "API_KEY=$ApiKey"
}

if ($WithOllama) {
    Write-Host "Starting Ollama container..." -ForegroundColor Yellow
    docker run -d --name ollama-dev -p 11434:11434 -v ollama-data:/root/.ollama ollama/ollama:latest
    
    Write-Host "Pulling model $Model..." -ForegroundColor Yellow
    Start-Sleep -Seconds 5
    docker exec ollama-dev ollama pull $Model
}

if ($Interactive) {
    $runArgs = @(
        "run", "-it", "--rm",
        "-v", "redclaw-data:/redclaw-data",
        "-e", "REDCLAW_CONFIG_DIR=/redclaw-data/.redclaw",
        "-e", "REDCLAW_WORKSPACE=/redclaw-data/workspace",
        "redclaw:dev"
    )
    Write-Host "Starting interactive session..." -ForegroundColor Yellow
}

# Run container
docker @runArgs

if ($LASTEXITCODE -eq 0) {
    Write-Host "✓ RedClaw started successfully!" -ForegroundColor Green
    Write-Host ""
    Write-Host "Access points:" -ForegroundColor Cyan
    Write-Host "  Gateway: http://localhost:42617" -ForegroundColor White
    Write-Host "  Ollama:  http://localhost:11434" -ForegroundColor White
    Write-Host ""
    Write-Host "Commands:" -ForegroundColor Cyan
    Write-Host "  View logs:     docker logs -f redclaw-dev" -ForegroundColor White
    Write-Host "  CLI access:    docker exec -it redclaw-dev redclaw --help" -ForegroundColor White
    Write-Host "  Stop:          docker stop redclaw-dev" -ForegroundColor White
    Write-Host "  Remove:        docker rm redclaw-dev" -ForegroundColor White
} else {
    Write-Host "✗ Failed to start RedClaw" -ForegroundColor Red
}
