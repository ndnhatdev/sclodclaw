# Zeroclaw Cleanup Checklist

Last reviewed: 2026-03-14
Scope: `D:/tools/redclaw`

This checklist separates safe cleanup from migration-sensitive cleanup so Redhorse
development can continue on a cleaner repo without deleting active transition
evidence.

## 1. Canonical lanes to keep

- [x] `src/` - primary Rust runtime lane
- [x] `src/core/` - canonical Redhorse core lane
- [x] `src/modules/` - canonical bundled-module lane
- [x] `firmware/` - embedded lane
- [x] `sdk/python/` - canonical Python lane
- [x] `apps/web/` - canonical web lane
- [x] `docs/`, `dev/`, `scripts/`, `tests/`, `examples/`, `fuzz/`, `benches/`
- [x] `modules/robot-kit/` - active workspace member; do not remove with legacy module cleanup

## 2. Safe cleanup to do now

- [x] Remove stale build cache `target_t0013/`
- [x] Remove stray system path `/%SystemDrive%/`
- [x] Remove empty helper directory `crates/`
- [x] Remove unused duplicate file `src/modules_command_handler.rs`
- [x] Update `.gitignore` to keep cleanup artifacts from reappearing
- [x] Update `build.rs` to use `apps/web/dist/` instead of legacy `web/dist/`
- [x] Update repo routing docs to current canonical paths
- [x] Update Docker build inputs to use `modules/` instead of deleted `crates/`

## 3. Keep for now, refactor later

- [x] Retire the former root transitional Rust files after confirming zero in-repo consumers:
  - `src/contracts.rs`
  - `src/lifecycle.rs`
  - `src/registry.rs`
  - `src/config/modules_lock.rs`
- [ ] Align package and lane identity after path migration settles:
  - `Cargo.toml`
  - `README.md`
  - `apps/web/package.json`
  - `sdk/python/pyproject.toml`
  - `apps/web/AGENTS.md`
  - `sdk/python/AGENTS.md`

## 4. Remove only after follow-up migration work

- [x] Remove legacy `web/` after no code or scripts create/use `web/dist/`
- [x] Remove duplicate docs tree `docs/vi/` after references point to `docs/i18n/vi/`
- [ ] Remove legacy `modules/*` entries only after any useful migration reference is archived and `modules/robot-kit/` has a settled home
- [ ] Finish tracked old-path deletion/new-path addition as one coherent migration change set in git

## 5. Execution order

1. Remove only obvious garbage and duplicate non-canonical files.
2. Fix path truth in routing/build/maintainer docs.
3. Stabilize current canonical lanes (`src/core`, `src/modules`, `apps/web`, `sdk/python`).
4. Complete migration commits for old paths vs new paths.
5. Remove legacy top-level scaffolding only after those migrations land cleanly.

## 5.5 Phase 4 status

- [x] `ActivationBootstrap` now lives in `src/core/lifecycle/bootstrap.rs` and `RegistrySnapshot` now lives in `src/core/registry/registry_snapshot.rs`.
- [x] `config/mod.rs` re-exports canonical core install-state types directly.
- [x] Internal module-system logic moved to canonical `crate::core::*` lanes before root wrapper retirement.

## 5.6 Phase 5 status

- [x] Removed root shim files `src/lifecycle.rs`, `src/registry.rs`, `src/contracts.rs`, and `src/config/modules_lock.rs` after confirming zero in-repo consumers.
- [x] `src/lib.rs` now exposes canonical lifecycle/registry surfaces through `src/core/*` instead of standalone shim modules.
- [x] `src/config/mod.rs` no longer exposes a duplicate `modules_lock` shim module.

## 5.7 Phase 6 status

- [x] Installer/security warning debt reduced by replacing placeholder imports and dead code paths with cleaner canonical implementations.
- [x] `artifact_fetch.rs` now stages archives into the returned directory and normalizes a single extracted root.
- [x] `installer/verification.rs` now validates artifact existence/entry presence and performs real SHA-256 checksum comparison.
- [x] `cargo check --all-targets` now passes after fixing lingering benchmark/test naming fallout.
- [x] Added unit coverage for archive-extension/flattening and checksum/signature-absent verification helpers.

## 6. Stop signs

- Do not delete `modules/robot-kit/` while it is still a Cargo workspace member.
- Do not delete `modules/` wholesale just because it is legacy; most of it is migration-sensitive.
- Do not treat path renames as complete until docs, build plumbing, and lane AGENTS agree.
