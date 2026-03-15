# Open-Source Fortress Setup

This document is the maintainer checklist for making `ndnhatdev/sclodclaw` safe for public contributions while keeping `main` protected.

## 1. Repository baseline

- repository visibility: public
- default branch: `main`
- required root docs present: `README.md`, `LICENSE-*`, `CONTRIBUTING.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md`, `TRADEMARK.md`
- governance files present: `.github/CODEOWNERS`, `.github/dependabot.yml`, `.github/pull_request_template.md`, issue templates, CI workflows

## 2. Branch protection for `main`

Configure GitHub branch protection on `main` with all of the following enabled:

1. require a pull request before merging
2. require approvals: at least `1` (recommend `2` for security/CI/runtime changes)
3. require status checks to pass before merging
4. require branches to be up to date before merging
5. do not allow bypassing the above settings
6. restrict direct pushes to maintainers only (or nobody, if PR-only)

Recommended required checks:

- `CI Required Gate`
- `Security Required Gate`

## 3. GitHub security settings

Enable these in `Settings -> Code security and analysis`:

- Dependabot alerts
- Dependabot security updates
- Secret scanning
- Push protection for secrets
- Private vulnerability reporting

Recommended extra:

- Code scanning alerts (CodeQL workflow)

## 4. Contributor workflow

Contributor path must be:

1. fork repository
2. create feature branch from `main`
3. run local checks before opening PR
4. open PR against `main`
5. wait for CI + maintainer approval

The project-level reminder lives in `CONTRIBUTING.md` and `.github/pull_request_template.md`.

## 5. Required local/CI checks

Contributors should run:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --locked
```

GitHub Actions should enforce:

- lint
- tests
- build
- dependency audit / deny
- docs quality (where enabled)

## 6. Release/install publish checklist

Before advertising one-command install, verify:

1. `install.sh` raw URL points to `https://raw.githubusercontent.com/ndnhatdev/sclodclaw/main/install.sh`
2. release assets exist with names `redclaw-<target>.tar.gz`
3. release archive contains executable `redclaw`
4. GHCR image exists at `ghcr.io/ndnhatdev/sclodclaw:latest` if Docker fallback is expected

## 7. Recommended one-line install commands

Machine already has Rust:

```bash
curl -fsSL https://raw.githubusercontent.com/ndnhatdev/sclodclaw/main/install.sh | bash
```

Fresh Linux/macOS machine:

```bash
curl -fsSL https://raw.githubusercontent.com/ndnhatdev/sclodclaw/main/install.sh | bash -s -- --install-system-deps --install-rust
```

## 8. Maintainer acceptance packet for every PR

Before merge, verify the PR contains:

- clear summary and scope boundary
- validation evidence
- security impact statement
- rollback plan
- no secrets or personal data

The PR template already requires this, so maintainers should reject incomplete PRs instead of filling gaps manually.
