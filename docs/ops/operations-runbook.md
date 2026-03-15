# RedClaw Operations Runbook

This runbook is for operators who maintain availability, security posture, and incident response.

Last verified: **February 18, 2026**.

## Scope

Use this document for day-2 operations:

- starting and supervising runtime
- health checks and diagnostics
- safe rollout and rollback
- incident triage and recovery

For first-time installation, start from [one-click-bootstrap.md](../setup-guides/one-click-bootstrap.md).

## Runtime Modes

| Mode | Command | When to use |
|---|---|---|
| Foreground runtime | `redclaw daemon` | local debugging, short-lived sessions |
| Foreground gateway only | `redclaw gateway` | webhook endpoint testing |
| User service | `redclaw service install && redclaw service start` | persistent operator-managed runtime |

## Baseline Operator Checklist

1. Validate configuration:

```bash
redclaw status
```

2. Verify diagnostics:

```bash
redclaw doctor
redclaw channel doctor
```

3. Start runtime:

```bash
redclaw daemon
```

4. For persistent user session service:

```bash
redclaw service install
redclaw service start
redclaw service status
```

## Health and State Signals

| Signal | Command / File | Expected |
|---|---|---|
| Config validity | `redclaw doctor` | no critical errors |
| Channel connectivity | `redclaw channel doctor` | configured channels healthy |
| Runtime summary | `redclaw status` | expected provider/model/channels |
| Daemon heartbeat/state | `~/.redclaw/daemon_state.json` | file updates periodically |

## Logs and Diagnostics

### macOS / Windows (service wrapper logs)

- `~/.redclaw/logs/daemon.stdout.log`
- `~/.redclaw/logs/daemon.stderr.log`

### Linux (systemd user service)

```bash
journalctl --user -u redclaw.service -f
```

## Incident Triage Flow (Fast Path)

1. Snapshot system state:

```bash
redclaw status
redclaw doctor
redclaw channel doctor
```

2. Check service state:

```bash
redclaw service status
```

3. If service is unhealthy, restart cleanly:

```bash
redclaw service stop
redclaw service start
```

4. If channels still fail, verify allowlists and credentials in `~/.redclaw/config.toml`.

5. If gateway is involved, verify bind/auth settings (`[gateway]`) and local reachability.

## Safe Change Procedure

Before applying config changes:

1. backup `~/.redclaw/config.toml`
2. apply one logical change at a time
3. run `redclaw doctor`
4. restart daemon/service
5. verify with `status` + `channel doctor`

## Rollback Procedure

If a rollout regresses behavior:

1. restore previous `config.toml`
2. restart runtime (`daemon` or `service`)
3. confirm recovery via `doctor` and channel health checks
4. document incident root cause and mitigation

## Related Docs

- [one-click-bootstrap.md](../setup-guides/one-click-bootstrap.md)
- [troubleshooting.md](./troubleshooting.md)
- [config-reference.md](../reference/api/config-reference.md)
- [commands-reference.md](../reference/cli/commands-reference.md)
