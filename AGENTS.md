# REDHORSE KNOWLEDGE BASE

## OVERVIEW
Redhorse is a Rust-first agent runtime repo with four real engineering lanes: Rust core, embedded firmware, Python companion tooling, and a React/Vite web surface.
Use this file as the repo router; once work enters `src/`, `firmware/`, `sdk/python/`, or `apps/web/`, the nearest child `AGENTS.md` wins.

## STRUCTURE
```text
redhorse/
|- src/            # Rust runtime, gateway, providers, tools, security
|- firmware/       # embedded targets (ESP32, Nucleo, Arduino, bridge)
|- sdk/python/     # redhorse-tools companion package
|- apps/web/       # React/Vite dashboard
|- modules/robot-kit/ # workspace helper crate during migration
|- docs/           # ops, security, maintainers, reference, hardware
`- dev/            # CI helpers and local verification scripts
```

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| Runtime core | `src/AGENTS.md` | command routing, gateway, tools, security |
| Risk tiers and workflow | `CLAUDE.md` | branch rules, risk classification, validation depth |
| Embedded work | `firmware/AGENTS.md` | target-specific firmware crates and mixed tooling |
| Python companion tools | `sdk/python/AGENTS.md` | package metadata, pytest/ruff, CLI entry |
| Web UI | `apps/web/AGENTS.md` | Vite/React frontend lane |
| Repo-wide contracts | `docs/`, `SECURITY.md`, `README.md` | architecture, policies, operator guidance |

## CONVENTIONS
- Treat the four lanes as separate stacks; do not assume Rust-core commands or patterns apply to firmware, Python, or web work.
- Keep trait-driven Rust extension points in `src/` aligned with the workflow and risk guidance in `CLAUDE.md`.
- Prefer child `AGENTS.md` files for lane-local commands and anti-patterns instead of bloating this router.

## ANTI-PATTERNS
- Do not leave this repo root as a placeholder redirect; the router must explain the lane split.
- Do not mix firmware, Python, or web implementation details into `src/` guidance or vice versa.
- Do not treat `modules/robot-kit` as a full crate workspace map; it is a helper crate, not the primary runtime lane.

## COMMANDS
```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
./dev/ci.sh all
```

## NOTES
- `src/AGENTS.md` is the deepest existing runtime guide; start there for Rust behavior changes.
- `CLAUDE.md` remains the detailed source for risk tiers, PR discipline, and linked contributing docs.
