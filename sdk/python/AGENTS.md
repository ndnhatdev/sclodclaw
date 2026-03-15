# REDHORSE PYTHON KNOWLEDGE BASE

## OVERVIEW
`sdk/python/` is the companion Python package lane for Redhorse.
It ships `redclaw-tools`, a LangGraph-based tool-calling package with its own packaging, test, and lint workflow.

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| Package metadata | `pyproject.toml` | dependencies, optional extras, ruff and pytest config |
| CLI entry | `redclaw_tools/__main__.py` | `redclaw-tools` console script |
| Core package | `redclaw_tools/agent.py`, `redclaw_tools/tools/`, `redclaw_tools/integrations/` | runtime behavior and integrations |
| Tests | `tests/` | pytest coverage for package behavior |

## CONVENTIONS
- Treat `pyproject.toml` as the source of truth for Python dependencies, dev extras, and tool config.
- Keep package changes scoped to `redclaw_tools/`; this lane should not mirror Rust runtime structure one-for-one.
- Use pytest and ruff from the Python dev extras rather than inventing a parallel lint/test flow.

## ANTI-PATTERNS
- Do not assume repo-root Rust commands validate this package.
- Do not add Python-only behavior to the Rust runtime docs instead of documenting it here.
- Do not bypass `pyproject.toml` when changing package metadata or tool configuration.

## COMMANDS
```bash
python -m pytest tests
python -m ruff check .
python -m redclaw_tools
```

## NOTES
- `requires-python` is `>=3.10`; keep version-sensitive syntax aligned with that floor.
- Optional extras exist for `discord` and `telegram`; keep integrations isolated behind those package boundaries.
