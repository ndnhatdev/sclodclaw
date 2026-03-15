# One-Click Bootstrap

This page defines the fastest supported path to install and initialize RedClaw.

Last verified: **February 20, 2026**.

## Option 0: Homebrew (macOS/Linuxbrew)

```bash
brew install redclaw
```

## Option A (Recommended): Clone + local script

```bash
git clone https://github.com/ndnhatdev/sclodclaw.git
cd redclaw
./install.sh
```

What it does by default:

1. `cargo build --release --locked`
2. `cargo install --path . --force --locked`

### Resource preflight and pre-built flow

Source builds typically require at least:

- **2 GB RAM + swap**
- **6 GB free disk**

When resources are constrained, bootstrap now attempts a pre-built binary first.

```bash
./install.sh --prefer-prebuilt
```

To require binary-only installation and fail if no compatible release asset exists:

```bash
./install.sh --prebuilt-only
```

To bypass pre-built flow and force source compilation:

```bash
./install.sh --force-source-build
```

## Dual-mode bootstrap

Default behavior is **app-only** (build/install RedClaw) and expects existing Rust toolchain.

For fresh machines, enable environment bootstrap explicitly:

```bash
./install.sh --install-system-deps --install-rust
```

Notes:

- `--install-system-deps` installs compiler/build prerequisites (may require `sudo`).
- `--install-rust` installs Rust via `rustup` when missing.
- `--prefer-prebuilt` tries release binary download first, then falls back to source build.
- `--prebuilt-only` disables source fallback.
- `--force-source-build` disables pre-built flow entirely.

## Option B: Remote one-liner

```bash
curl -fsSL https://raw.githubusercontent.com/ndnhatdev/sclodclaw/main/install.sh | bash
```

This path is best when Rust is already installed.

For fresh Linux/macOS machines, use:

```bash
curl -fsSL https://raw.githubusercontent.com/ndnhatdev/sclodclaw/main/install.sh | bash -s -- --install-system-deps --install-rust
```

For high-security environments, prefer Option A so you can review the script before execution.

If you run Option B outside a repository checkout, the install script automatically clones a temporary workspace, builds, installs, and then cleans it up.
If your current directory happens to contain an unrelated `Cargo.toml`, the installer now ignores it and clones RedClaw instead.

## Optional onboarding modes

### Containerized onboarding (Docker)

```bash
./install.sh --docker
```

This builds a local RedClaw image and launches onboarding inside a container while
persisting config/workspace to `./.redclaw-docker`.

Container CLI defaults to `docker`. If Docker CLI is unavailable and `podman` exists,
the installer auto-falls back to `podman`. You can also set `REDCLAW_CONTAINER_CLI`
explicitly (for example: `REDCLAW_CONTAINER_CLI=podman ./install.sh --docker`).

For Podman, the installer runs with `--userns keep-id` and `:Z` volume labels so
workspace/config mounts remain writable inside the container.

If you add `--skip-build`, the installer skips local image build. It first tries the local
Docker tag (`REDCLAW_DOCKER_IMAGE`, default: `redclaw-bootstrap:local`); if missing,
it pulls `ghcr.io/ndnhatdev/sclodclaw:latest` and tags it locally before running.

For a true one-command install after the repo is published, use either:

```bash
curl -fsSL https://raw.githubusercontent.com/ndnhatdev/sclodclaw/main/install.sh | bash
```

or include host bootstrap automatically:

```bash
curl -fsSL https://raw.githubusercontent.com/ndnhatdev/sclodclaw/main/install.sh | bash -s -- --install-system-deps --install-rust
```

### Quick onboarding (non-interactive)

```bash
./install.sh --onboard --api-key "sk-..." --provider openrouter
```

Or with environment variables:

```bash
REDCLAW_API_KEY="sk-..." REDCLAW_PROVIDER="openrouter" ./install.sh --onboard
```

### Interactive onboarding

```bash
./install.sh --interactive-onboard
```

## Useful flags

- `--install-system-deps`
- `--install-rust`
- `--skip-build` (in `--docker` mode: use local image if present, otherwise pull `ghcr.io/ndnhatdev/sclodclaw:latest`)
- `--skip-install`
- `--provider <id>`

See all options:

```bash
./install.sh --help
```

## Related docs

- [README.md](../README.md)
- [commands-reference.md](../reference/cli/commands-reference.md)
- [providers-reference.md](../reference/api/providers-reference.md)
- [channels-reference.md](../reference/api/channels-reference.md)
