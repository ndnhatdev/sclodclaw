# REDHORSE WEB KNOWLEDGE BASE

## OVERVIEW
`apps/web/` is the React 19 + Vite frontend lane for Redhorse.
It is a separate TypeScript app with its own `package.json`, lockfile, and build pipeline; do not treat it as an extension of the Rust runtime tree.

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| App boot | `src/main.tsx`, `src/App.tsx` | frontend entry and top-level shell |
| Screen and shared UI | `src/pages/`, `src/components/` | route surfaces and reusable components |
| Browser logic | `src/hooks/`, `src/lib/`, `src/types/` | hooks, helpers, app types |
| Build config | `package.json`, `vite.config.ts`, `tsconfig*.json` | commands and bundler/type settings |

## CONVENTIONS
- Use the local `package-lock.json` and `package.json` workflow for this lane.
- Keep frontend assumptions here; Rust runtime behavior belongs in `src/AGENTS.md`, not in this child file.
- Treat Vite build and TypeScript project config as the source of truth for frontend verification.

## ANTI-PATTERNS
- Do not run repo-root cargo commands and assume the web app was validated.
- Do not move web-only state or component logic into Rust docs or firmware guidance.
- Do not mix package-manager expectations from other TS repos in this workspace into this app without checking `package.json` first.

## COMMANDS
```bash
npm install
npm run build
npm run dev
```

## NOTES
- This subtree currently defines dev/build scripts only; if test or lint scripts are added later, document them here instead of at repo root.
- Use repo-root `AGENTS.md` only for lane routing once you are already inside `apps/web/`.
