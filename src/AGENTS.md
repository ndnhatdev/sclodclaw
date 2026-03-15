# REDHORSE SRC KNOWLEDGE BASE

## OVERVIEW
`src/` is the Rust runtime core for Redhorse: CLI entry, runtime loop, gateway, channels, providers, tools, security, and observability.
This subtree is the high-change, higher-risk boundary compared with `firmware/`, `sdk/python/`, and `apps/web/`.

## STRUCTURE
```text
src/
|- main.rs         # CLI entry and command routing
|- config/         # schema, loading, merge, export
|- agent/          # orchestration loop
|- gateway/        # HTTP/WS server
|- providers/      # model adapters
|- channels/       # Telegram/Discord/Slack/etc.
|- tools/          # tool execution surfaces
|- memory/         # sqlite/markdown/vector storage
|- security/       # policy, pairing, secret store
`- observability/  # tracing and metrics
```

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| CLI and commands | `main.rs`, `lib.rs` | command enums and startup wiring |
| Runtime behavior | `agent/`, `runtime/`, `service/` | loop, adapters, service orchestration |
| Network surfaces | `gateway/`, `tunnel/`, `hooks/` | inbound control and integration edges |
| Model and channel integration | `providers/`, `channels/` | external platform boundaries |
| Tooling and memory | `tools/`, `memory/`, `rag/`, `skillforge/` | execution and retrieval paths |
| Risk-heavy security | `security/`, `auth/`, `approval/` | access control and secrets |

## CONVENTIONS
- Extend existing trait and factory-style boundaries before creating new top-level modules.
- Config changes usually require schema, defaults, CLI exposure, and serialization updates as one unit.
- When touching risky surfaces, inspect adjacent docs and tests first; `security/`, `gateway/`, and `tools/` are not safe for guesswork.

## ANTI-PATTERNS
- Do not silently weaken security or access constraints.
- Do not add speculative config or feature flags without a concrete use path.
- Do not hide behavior-changing side effects inside broad refactors.

## COMMANDS
```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## NOTES
- Repo-root `AGENTS.md` delegates to `CLAUDE.md`; read `../CLAUDE.md` for risk tiers, branch rules, and deeper workflow guidance.
- `apps/web/`, `sdk/python/`, and `firmware/` are separate lanes; keep their commands and assumptions out of `src/` guidance.
