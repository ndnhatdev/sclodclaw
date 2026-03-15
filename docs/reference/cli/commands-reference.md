# RedClaw Commands Reference

This reference is derived from the current CLI surface (`redclaw --help`).

Last verified: **February 21, 2026**.

## Top-Level Commands

| Command | Purpose |
|---|---|
| `onboard` | Initialize workspace/config quickly or interactively |
| `agent` | Run interactive chat or single-message mode |
| `gateway` | Start webhook and WhatsApp HTTP gateway |
| `daemon` | Start supervised runtime (gateway + channels + optional heartbeat/scheduler) |
| `service` | Manage user-level OS service lifecycle |
| `doctor` | Run diagnostics and freshness checks |
| `status` | Print current configuration and system summary |
| `estop` | Engage/resume emergency stop levels and inspect estop state |
| `cron` | Manage scheduled tasks |
| `models` | Refresh provider model catalogs |
| `providers` | List provider IDs, aliases, and active provider |
| `channel` | Manage channels and channel health checks |
| `integrations` | Inspect integration details |
| `skills` | List/install/remove skills |
| `migrate` | Import from external runtimes (currently OpenClaw) |
| `config` | Export machine-readable config schema |
| `completions` | Generate shell completion scripts to stdout |
| `hardware` | Discover and introspect USB hardware |
| `peripheral` | Configure and flash peripherals |

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
- `redclaw doctor models [--provider <ID>] [--use-cache]`
- `redclaw doctor traces [--limit <N>] [--event <TYPE>] [--contains <TEXT>]`
- `redclaw doctor traces --id <TRACE_ID>`

`doctor traces` reads runtime tool/model diagnostics from `observability.runtime_trace_path`.

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

### `config`

- `redclaw config schema`

`config schema` prints a JSON Schema (draft 2020-12) for the full `config.toml` contract to stdout.

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

### `peripheral`

- `redclaw peripheral list`
- `redclaw peripheral add <board> <path>`
- `redclaw peripheral flash [--port <serial_port>]`
- `redclaw peripheral setup-uno-q [--host <ip_or_host>]`
- `redclaw peripheral flash-nucleo`

## Validation Tip

To verify docs against your current binary quickly:

```bash
redclaw --help
redclaw <command> --help
```
