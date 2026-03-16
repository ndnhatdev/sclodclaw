# RedClaw Commands Reference

This reference is derived from the current CLI surface (`redclaw --help`).

Last verified: **March 16, 2026**.

## Top-Level Commands

Maturity labels used here:

- `stable` - real command family with user-facing value today
- `experimental` - real but narrow or still evolving
- `scaffolded/redirected` - exposed on the surface, but not yet a full first-class flow

| Command | Purpose | Maturity | Notes |
|---|---|---|---|
| `onboard` | Initialize workspace/config quickly or interactively | `stable` | some setup debt remains |
| `agent` | Run interactive chat or single-message mode | `stable` | |
| `gateway` | Start webhook and WhatsApp HTTP gateway | `stable` | |
| `daemon` | Start supervised runtime (gateway + channels + optional heartbeat/scheduler) | `stable` | |
| `service` | Manage daemon service lifecycle across supported init systems | `stable` | RedClaw naming is canonical; legacy Redhorse names remain compatibility aliases |
| `doctor` | Run diagnostics and freshness checks | `stable` | baseline `doctor --json` is now available |
| `status` | Print runtime/configuration summary for the current environment | `stable` | `status --json` is now available |
| `estop` | Engage/resume emergency stop levels and inspect estop state | `stable` | terse operator naming remains |
| `cron` | Manage scheduled tasks | `stable` | |
| `models` | Refresh provider model catalogs | `stable` | |
| `providers` | List provider IDs, aliases, and active provider | `experimental` | thin read-only catalog; `providers --json` is available |
| `channel` | Manage channels and channel health checks | `stable` | `add/remove` are still redirected setup paths |
| `integrations` | Inspect one integration and setup hints | `experimental` | narrow-but-real (`info <name>` only) |
| `skills` | List/install/remove skills | `stable` | |
| `migrate` | Import from external runtimes (currently OpenClaw) | `experimental` | partial migration surface |
| `auth` | Manage provider authentication profiles | `stable` | previously missing from this top-level table |
| `hardware` | Experimental hardware discovery surface | `scaffolded/redirected` | command family is visible for discoverability but currently scaffolded |
| `peripheral` | Experimental peripheral management surface | `scaffolded/redirected` | command family is visible for discoverability but currently scaffolded |
| `memory` | List/get/stats/clear stored memory entries | `stable` | pragmatic read/write utility surface |
| `config` | Export machine-readable config schema | `stable` | intentionally focused (`schema` only) |
| `modules` | Manage install/update/doctor flows for modules | `stable` | parser/help normalization still in progress |
| `completions` | Generate shell completion scripts to stdout | `stable` | stdout-only by contract |

## Command Groups

### `onboard`

- `redclaw onboard`
- `redclaw onboard --interactive`
- `redclaw onboard --channels-only`
- `redclaw onboard --force`
- `redclaw onboard --api-key <KEY> --provider <ID> --memory <sqlite|lucid|markdown|none>`
- `redclaw onboard --api-key <KEY> --provider <ID> --model <MODEL_ID> --memory <sqlite|lucid|markdown|none>`
- `redclaw onboard --api-key <KEY> --provider <ID> --model <MODEL_ID> --memory <sqlite|lucid|markdown|none> --force`
- `redclaw onboard --reinit --interactive`

`onboard` safety behavior:

- If `config.toml` already exists and you run `--interactive`, onboarding now offers two modes:
  - Full onboarding (overwrite `config.toml`)
  - Provider-only update (update provider/model/API key while preserving existing channels, tunnel, memory, hooks, and other settings)
- In non-interactive environments, existing `config.toml` causes a safe refusal unless `--force` is passed.
- Use `redclaw onboard --channels-only` when you only need to rotate channel tokens/allowlists.
- Use `redclaw onboard --reinit --interactive` to start fresh. This backs up your existing config directory with a timestamp suffix and creates a new configuration from scratch. Requires `--interactive`.

### `agent`

- `redclaw agent`
- `redclaw agent -m "Hello"`
- `redclaw agent --provider <ID> --model <MODEL> --temperature <0.0-2.0>`
- `redclaw agent --peripheral <board:path>`

Tip:

- In interactive chat, you can ask for route changes in natural language (for example “conversation uses kimi, coding uses gpt-5.3-codex”); the assistant can persist this via tool `model_routing_config`.

### `gateway` / `daemon`

- `redclaw gateway [--host <HOST>] [--port <PORT>]`
- `redclaw daemon [--host <HOST>] [--port <PORT>]`

### `estop`

- `redclaw estop` (engage `kill-all`)
- `redclaw estop --level network-kill`
- `redclaw estop --level domain-block --domain "*.chase.com" [--domain "*.paypal.com"]`
- `redclaw estop --level tool-freeze --tool shell [--tool browser]`
- `redclaw estop status`
- `redclaw estop resume`
- `redclaw estop resume --network`
- `redclaw estop resume --domain "*.chase.com"`
- `redclaw estop resume --tool shell`
- `redclaw estop resume --otp <123456>`

Notes:

- `estop` commands require `[security.estop].enabled = true`.
- When `[security.estop].require_otp_to_resume = true`, `resume` requires OTP validation.
- OTP prompt appears automatically if `--otp` is omitted.

### `service`

- `redclaw service install`
- `redclaw service start`
- `redclaw service stop`
- `redclaw service restart`
- `redclaw service status`
- `redclaw service uninstall`

RedClaw-facing service names are canonical in help/docs. If an older installation still uses a legacy service identifier, the runtime treats it as a compatibility alias rather than the public contract.

### `cron`

- `redclaw cron list`
- `redclaw cron add <expr> [--tz <IANA_TZ>] <command>`
- `redclaw cron add-at <rfc3339_timestamp> <command>`
- `redclaw cron add-every <every_ms> <command>`
- `redclaw cron once <delay> <command>`
- `redclaw cron remove <id>`
- `redclaw cron pause <id>`
- `redclaw cron resume <id>`

Notes:

- Mutating schedule/cron actions require `cron.enabled = true`.
- Shell command payloads for schedule creation (`create` / `add` / `once`) are validated by security command policy before job persistence.

### `models`

- `redclaw models refresh`
- `redclaw models refresh --provider <ID>`
- `redclaw models refresh --force`

`models refresh` currently supports live catalog refresh for provider IDs: `openrouter`, `openai`, `anthropic`, `groq`, `mistral`, `deepseek`, `xai`, `together-ai`, `gemini`, `ollama`, `llamacpp`, `sglang`, `vllm`, `astrai`, `venice`, `fireworks`, `cohere`, `moonshot`, `glm`, `zai`, `qwen`, and `nvidia`.

### `doctor`

- `redclaw doctor`
- `redclaw doctor --json`
- `redclaw doctor models [--provider <ID>] [--use-cache]`
- `redclaw doctor traces [--limit <N>] [--event <TYPE>] [--contains <TEXT>]`
- `redclaw doctor traces --id <TRACE_ID>`

`doctor traces` reads runtime tool/model diagnostics from `observability.runtime_trace_path`.

`doctor --json` prints the baseline diagnostics as structured JSON. Subcommands keep their existing human-oriented output today.

### `status`

- `redclaw status`
- `redclaw status --json`

`status --json` prints a machine-readable snapshot of the same runtime/config summary shown in human mode.

### `providers`

- `redclaw providers`
- `redclaw providers --json`

`providers` is real and useful today, but it remains a thin read-only surface. `providers --json` prints the provider catalog and active marker as structured JSON.

### `channel`

- `redclaw channel list`
- `redclaw channel start`
- `redclaw channel doctor`
- `redclaw channel bind-telegram <IDENTITY>`
- `redclaw channel add <type> <json>`
- `redclaw channel remove <name>`

Runtime in-chat commands (Telegram/Discord while channel server is running):

- `/models`
- `/models <provider>`
- `/model`
- `/model <model-id>`
- `/new`

Channel runtime also watches `config.toml` and hot-applies updates to:
- `default_provider`
- `default_model`
- `default_temperature`
- `api_key` / `api_url` (for the default provider)
- `reliability.*` provider retry settings

`add/remove` currently route you back to managed setup/manual config paths (not full declarative mutators yet).

### `integrations`

- `redclaw integrations info <name>`

`integrations` is currently a narrow-but-real surface: `info <name>` works, but there is no list/search flow yet.

### `skills`

- `redclaw skills list`
- `redclaw skills audit <source_or_name>`
- `redclaw skills install <source>`
- `redclaw skills remove <name>`

`<source>` accepts git remotes (`https://...`, `http://...`, `ssh://...`, and `git@host:owner/repo.git`) or a local filesystem path.

`skills install` always runs a built-in static security audit before the skill is accepted. The audit blocks:
- symlinks inside the skill package
- script-like files (`.sh`, `.bash`, `.zsh`, `.ps1`, `.bat`, `.cmd`)
- high-risk command snippets (for example pipe-to-shell payloads)
- markdown links that escape the skill root, point to remote markdown, or target script files

Use `skills audit` to manually validate a candidate skill directory (or an installed skill by name) before sharing it.

Skill manifests (`SKILL.toml`) support `prompts` and `[[tools]]`; both are injected into the agent system prompt at runtime, so the model can follow skill instructions without manually reading skill files.

### `migrate`

- `redclaw migrate openclaw [--source <path>] [--dry-run]`

### `auth`

- `redclaw auth login --provider <openai-codex|gemini> [--profile <name>] [--device-code]`
- `redclaw auth paste-redirect --provider <openai-codex> [--profile <name>] [--input <value>]`
- `redclaw auth paste-token --provider <anthropic> [--profile <name>] [--token <value>] [--auth-kind <kind>]`
- `redclaw auth setup-token --provider <anthropic> [--profile <name>]`
- `redclaw auth refresh --provider <openai-codex> [--profile <name>]`
- `redclaw auth logout --provider <id> [--profile <name>]`
- `redclaw auth use --provider <id> --profile <name>`
- `redclaw auth list`
- `redclaw auth status`

### `config`

- `redclaw config schema`

`config` is intentionally focused today: `schema` prints a JSON Schema (draft 2020-12) for the full `config.toml` contract to stdout.

### `completions`

- `redclaw completions bash`
- `redclaw completions fish`
- `redclaw completions zsh`
- `redclaw completions powershell`
- `redclaw completions elvish`

`completions` is stdout-only by design so scripts can be sourced directly without log/warning contamination.

### `hardware`

- `redclaw hardware discover`
- `redclaw hardware introspect <path>`
- `redclaw hardware info [--chip <chip_name>]`

This command family is currently scaffolded. The surface remains visible for discoverability, but handlers may still return placeholder output rather than a full hardware workflow.

### `peripheral`

- `redclaw peripheral list`
- `redclaw peripheral add <board> <path>`
- `redclaw peripheral flash [--port <serial_port>]`
- `redclaw peripheral setup-uno-q [--host <ip_or_host>]`
- `redclaw peripheral flash-nucleo`

This command family is currently scaffolded. The surface remains visible for discoverability, but handlers may still return placeholder output rather than a full peripheral workflow.

### `memory`

- `redclaw memory list [--category <name>] [--session <id>] [--limit <n>] [--offset <n>]`
- `redclaw memory get <key>`
- `redclaw memory stats`
- `redclaw memory clear [--key <prefix>] [--category <name>] [--yes]`

`memory` is a stable utility surface for inspecting and pruning stored entries. It is intentionally pragmatic (operator-facing text output), not yet a full machine-readable contract for every subcommand.

### `modules`

- `redclaw modules list`
- `redclaw modules info <module-id>`
- `redclaw modules install <source> [--enable]`
- `redclaw modules remove <module-id>`
- `redclaw modules enable <module-id>`
- `redclaw modules disable <module-id>`
- `redclaw modules update [<module-id>] [--all]`
- `redclaw modules doctor`

`modules` is a real command family today, but its help/parser path is still being normalized into the main CLI pipeline.

## Validation Tip

To verify docs against your current binary quickly:

```bash
redclaw --help
redclaw <command> --help
```
