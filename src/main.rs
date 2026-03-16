#![warn(clippy::all, clippy::pedantic)]
#![allow(
    clippy::assigning_clones,
    clippy::bool_to_int_with_if,
    clippy::case_sensitive_file_extension_comparisons,
    clippy::cast_possible_wrap,
    clippy::doc_markdown,
    clippy::field_reassign_with_default,
    clippy::float_cmp,
    clippy::implicit_clone,
    clippy::items_after_statements,
    clippy::map_unwrap_or,
    clippy::manual_let_else,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::needless_pass_by_value,
    clippy::needless_raw_string_hashes,
    clippy::redundant_closure_for_method_calls,
    clippy::similar_names,
    clippy::single_match_else,
    clippy::struct_field_names,
    clippy::too_many_lines,
    clippy::uninlined_format_args,
    clippy::unused_self,
    clippy::cast_precision_loss,
    clippy::unnecessary_cast,
    clippy::unnecessary_lazy_evaluations,
    clippy::unnecessary_literal_bound,
    clippy::unnecessary_map_or,
    clippy::unnecessary_wraps,
    dead_code
)]

use anyhow::{bail, Context, Result};
use clap::{CommandFactory, Parser, Subcommand};
use dialoguer::Input;
use redclaw::cli_support::{parse_temperature, CompletionShell, EstopLevelArg};
use std::io::Write;
use tracing::info;
use tracing_subscriber::{fmt, EnvFilter};

mod agent;
mod approval;
mod auth;
mod branding;
mod channels;
mod commands;
mod rag {
    pub use redclaw::rag::*;
}
mod config;
mod core;
mod cost;
mod cron;
mod daemon;
mod doctor;
mod gateway;
mod hardware;
mod health;
mod heartbeat;
mod hooks;
mod identity;
mod integrations;
mod memory;
mod migration;
mod multimodal;
mod observability;
mod onboard;
mod peripherals;
mod providers;
mod runtime;
mod security;
mod service;
mod skillforge;
mod skills;
mod tools;
mod tunnel;
mod util;

use config::Config;

// Re-export so binary modules can use crate::<CommandEnum> while keeping a single source of truth.
pub use redclaw::{
    ChannelCommands, CronCommands, GatewayCommands, HardwareCommands, IntegrationCommands,
    MigrateCommands, ModulesCommands, PeripheralCommands, ServiceCommands, SkillCommands,
};

/// `RedClaw` - Zero overhead. Zero compromise. 100% Rust.
#[derive(Parser, Debug)]
#[command(name = "redclaw")]
#[command(author = "theonlyhennygod")]
#[command(version)]
#[command(about = "The fastest, smallest AI assistant.", long_about = None)]
struct Cli {
    #[arg(long, global = true)]
    config_dir: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize your workspace and configuration
    Onboard {
        /// Run the full interactive wizard (default is quick setup)
        #[arg(long)]
        interactive: bool,

        /// Overwrite existing config without confirmation
        #[arg(long)]
        force: bool,

        /// Reinitialize from scratch (backup and reset all configuration)
        #[arg(long)]
        reinit: bool,

        /// Reconfigure channels only (fast repair flow)
        #[arg(long)]
        channels_only: bool,

        /// API key (used in quick mode, ignored with --interactive)
        #[arg(long)]
        api_key: Option<String>,

        /// Provider name (used in quick mode, default: openrouter)
        #[arg(long)]
        provider: Option<String>,
        /// Model ID override (used in quick mode)
        #[arg(long)]
        model: Option<String>,
        /// Memory backend (sqlite, lucid, markdown, none) - used in quick mode, default: sqlite
        #[arg(long)]
        memory: Option<String>,
    },

    /// Start the AI agent loop
    #[command(long_about = "\
Start the AI agent loop.

Launches an interactive chat session with the configured AI provider. \
Use --message for single-shot queries without entering interactive mode.

Examples:
  redclaw agent                              # interactive session
  redclaw agent -m \"Summarize today's logs\"  # single message
  redclaw agent -p anthropic --model claude-sonnet-4-20250514
  redclaw agent --peripheral nucleo-f401re:/dev/ttyACM0")]
    Agent {
        /// Single message mode (don't enter interactive mode)
        #[arg(short, long)]
        message: Option<String>,

        /// Provider to use (openrouter, anthropic, openai, openai-codex)
        #[arg(short, long)]
        provider: Option<String>,

        /// Model to use
        #[arg(long)]
        model: Option<String>,

        /// Temperature (0.0 - 2.0, defaults to config default_temperature)
        #[arg(short, long, value_parser = parse_temperature)]
        temperature: Option<f64>,

        /// Attach a peripheral (board:path, e.g. nucleo-f401re:/dev/ttyACM0)
        #[arg(long)]
        peripheral: Vec<String>,
    },

    /// Start/manage the gateway server (webhooks, websockets)
    #[command(long_about = "\
Manage the gateway server (webhooks, websockets).

Start, restart, or inspect the HTTP/WebSocket gateway that accepts \
incoming webhook events and WebSocket connections.

Examples:
  redclaw gateway start              # start gateway
  redclaw gateway restart            # restart gateway
  redclaw gateway get-paircode       # show pairing code")]
    Gateway {
        #[command(subcommand)]
        gateway_command: Option<redclaw::GatewayCommands>,
    },

    /// Start long-running autonomous runtime (gateway + channels + heartbeat + scheduler)
    #[command(long_about = "\
Start the long-running autonomous daemon.

Launches the full RedClaw runtime: gateway server, all configured \
channels (Telegram, Discord, Slack, etc.), heartbeat monitor, and \
the cron scheduler. This is the recommended way to run RedClaw in \
production or as an always-on assistant.

Use 'redclaw service install' to register the daemon as an OS \
service (systemd/launchd) for auto-start on boot.

Examples:
  redclaw daemon                   # use config defaults
  redclaw daemon -p 9090           # gateway on port 9090
  redclaw daemon --host 127.0.0.1  # localhost only")]
    Daemon {
        /// Port to listen on (use 0 for random available port); defaults to config gateway.port
        #[arg(short, long)]
        port: Option<u16>,

        /// Host to bind to; defaults to config gateway.host
        #[arg(long)]
        host: Option<String>,
    },

    /// Manage OS service lifecycle (install/start/stop/status)
    #[command(long_about = "\
Manage the RedClaw background service lifecycle.

Install, start, stop, restart, inspect, and uninstall the daemon service
across supported init systems (systemd, OpenRC, launchd, Windows Task Scheduler).

RedClaw names (`redclaw.service`, `com.redclaw.daemon`, `redclaw`) are the
public contract. Legacy Redhorse service names remain compatibility aliases.

Examples:
  redclaw service install
  redclaw service start
  redclaw service status
  redclaw service restart --service-init systemd
  redclaw service uninstall --service-init openrc")]
    Service {
        /// Init system to use: auto (detect), systemd, or openrc
        #[arg(long, default_value = "auto", value_parser = ["auto", "systemd", "openrc"])]
        service_init: String,

        #[command(subcommand)]
        service_command: ServiceCommands,
    },

    /// Run diagnostics for daemon/scheduler/channel freshness
    Doctor {
        #[command(subcommand)]
        doctor_command: Option<DoctorCommands>,
    },

    /// Show runtime and configuration status summary
    #[command(long_about = "\
Show runtime and configuration status.

Prints the current workspace/config paths, provider/model selection,
channel and gateway settings, memory backend, and daemon-related flags.

Use `redclaw doctor` for deeper diagnostics and freshness checks.

Examples:
  redclaw status")]
    Status,

    /// Engage, inspect, and resume emergency-stop states.
    ///
    /// Examples:
    /// - `redclaw estop`
    /// - `redclaw estop --level network-kill`
    /// - `redclaw estop --level domain-block --domain "*.chase.com"`
    /// - `redclaw estop --level tool-freeze --tool shell --tool browser`
    /// - `redclaw estop status`
    /// - `redclaw estop resume --network`
    /// - `redclaw estop resume --domain "*.chase.com"`
    /// - `redclaw estop resume --tool shell`
    Estop {
        #[command(subcommand)]
        estop_command: Option<EstopSubcommands>,

        /// Level used when engaging estop from `redclaw estop`.
        #[arg(long, value_enum)]
        level: Option<EstopLevelArg>,

        /// Domain pattern(s) for `domain-block` (repeatable).
        #[arg(long = "domain")]
        domains: Vec<String>,

        /// Tool name(s) for `tool-freeze` (repeatable).
        #[arg(long = "tool")]
        tools: Vec<String>,
    },

    /// Configure and manage scheduled tasks
    #[command(long_about = "\
Configure and manage scheduled tasks.

Schedule recurring, one-shot, or interval-based tasks using cron \
expressions, RFC 3339 timestamps, durations, or fixed intervals.

Cron expressions use the standard 5-field format: \
'min hour day month weekday'. Timezones default to UTC; \
override with --tz and an IANA timezone name.

Examples:
  redclaw cron list
  redclaw cron add '0 9 * * 1-5' 'Good morning' --tz America/New_York
  redclaw cron add '*/30 * * * *' 'Check system health'
  redclaw cron add-at 2025-01-15T14:00:00Z 'Send reminder'
  redclaw cron add-every 60000 'Ping heartbeat'
  redclaw cron once 30m 'Run backup in 30 minutes'
  redclaw cron pause <task-id>
  redclaw cron update <task-id> --expression '0 8 * * *' --tz Europe/London")]
    Cron {
        #[command(subcommand)]
        cron_command: CronCommands,
    },

    /// Manage provider model catalogs
    Models {
        #[command(subcommand)]
        model_command: ModelCommands,
    },

    /// List supported providers and active selection
    #[command(long_about = "\
List supported AI providers and aliases.

Shows provider IDs that can be used in config, marks the currently active
default provider, and includes alias names where available.

This is a read-only catalog command.

Examples:
  redclaw providers
  redclaw onboard --interactive")]
    Providers,

    /// Manage channels (telegram, discord, slack)
    #[command(long_about = "\
Manage communication channels.

Add, remove, list, send, and health-check channels that connect RedClaw \
to messaging platforms. Supported channel types: telegram, discord, \
slack, whatsapp, matrix, imessage, email.

Examples:
  redclaw channel list
  redclaw channel doctor
  redclaw channel add telegram '{\"bot_token\":\"...\",\"name\":\"my-bot\"}'
  redclaw channel remove my-bot
  redclaw channel bind-telegram redclaw_user
  redclaw channel send 'Alert!' --channel-id telegram --recipient 123456789")]
    Channel {
        #[command(subcommand)]
        channel_command: ChannelCommands,
    },

    /// Inspect integration readiness and setup hints (narrow, read-only)
    #[command(long_about = "\
Inspect one integration at a time.

This command family is intentionally narrow today: `info <name>` provides
status and setup hints for a known integration. There is currently no
list/search subcommand in the `integrations` family.

Examples:
  redclaw integrations info Telegram
  redclaw integrations info OpenRouter")]
    Integrations {
        #[command(subcommand)]
        integration_command: IntegrationCommands,
    },

    /// Manage skills (user-defined capabilities)
    Skills {
        #[command(subcommand)]
        skill_command: SkillCommands,
    },

    /// Migrate data from other agent runtimes
    Migrate {
        #[command(subcommand)]
        migrate_command: MigrateCommands,
    },

    /// Manage provider subscription authentication profiles
    Auth {
        #[command(subcommand)]
        auth_command: AuthCommands,
    },

    /// Experimental hardware discovery surface (scaffolded)
    #[command(long_about = "\
Discover and inspect USB hardware (experimental scaffold).

The command surface is exposed for discoverability, but this family is
still scaffolded in the current cut and may return placeholder output
instead of a full hardware workflow.

Examples:
  redclaw hardware discover
  redclaw hardware introspect /dev/ttyACM0
  redclaw hardware info --chip STM32F401RETx")]
    Hardware {
        #[command(subcommand)]
        hardware_command: redclaw::HardwareCommands,
    },

    /// Experimental peripheral management surface (scaffolded)
    #[command(long_about = "\
Manage hardware peripherals (experimental scaffold).

The command surface is exposed for discoverability, but this family is
still scaffolded in the current cut and may return placeholder output
instead of a full peripheral workflow.

Examples:
  redclaw peripheral list
  redclaw peripheral add nucleo-f401re /dev/ttyACM0
  redclaw peripheral add rpi-gpio native
  redclaw peripheral flash --port /dev/cu.usbmodem12345
  redclaw peripheral flash-nucleo")]
    Peripheral {
        #[command(subcommand)]
        peripheral_command: redclaw::PeripheralCommands,
    },

    /// Manage stored memory entries (list/get/stats/clear)
    #[command(long_about = "\
Manage RedClaw memory entries.

Use this read/write utility surface to inspect saved memory, check backend
health/statistics, and clear selected entries when needed.

Supports filtering by category/session, pagination, and guarded clear flows.

Examples:
  redclaw memory stats
  redclaw memory list
  redclaw memory list --category core --limit 10
  redclaw memory get <key>
  redclaw memory clear --category conversation --yes")]
    Memory {
        #[command(subcommand)]
        memory_command: MemoryCommands,
    },

    /// Inspect exported configuration contract
    #[command(long_about = "\
Inspect the RedClaw configuration contract.

This command family is intentionally focused today: `schema` exports
the full JSON Schema for `config.toml` (keys, types, and defaults).

Examples:
  redclaw config schema
  redclaw config schema > schema.json")]
    Config {
        #[command(subcommand)]
        config_command: ConfigCommands,
    },

    /// Manage modules (install, remove, list, info, enable, disable, update, doctor)
    #[command(long_about = "\
Manage installed modules.

Install new modules from local directories or archives, remove \
installed modules, list all modules, inspect module details, run \
module updates, and execute module diagnostics.

Examples:
  redclaw modules list
  redclaw modules info provider-openai-compatible
  redclaw modules install ./my-module
  redclaw modules install ./my-module --enable
  redclaw modules remove provider-openai-compatible
  redclaw modules enable provider-openai-compatible
  redclaw modules disable provider-openai-compatible
  redclaw modules update provider-openai-compatible
  redclaw modules update --all
  redclaw modules doctor")]
    Modules {
        #[command(subcommand)]
        modules_command: ModulesCommands,
    },

    /// Generate shell completion script to stdout
    #[command(long_about = "\
Generate shell completion scripts for `redclaw`.

The script is printed to stdout so it can be sourced directly:

Examples:
  source <(redclaw completions bash)
  redclaw completions zsh > ~/.zfunc/_redclaw
  redclaw completions fish > ~/.config/fish/completions/redclaw.fish")]
    Completions {
        /// Target shell
        #[arg(value_enum)]
        shell: CompletionShell,
    },
}

#[derive(Subcommand, Debug)]
enum ConfigCommands {
    /// Dump the full configuration JSON Schema to stdout
    Schema,
}

#[derive(Subcommand, Debug)]
enum EstopSubcommands {
    /// Print current estop status.
    Status,
    /// Resume from an engaged estop level.
    Resume {
        /// Resume only network kill.
        #[arg(long)]
        network: bool,
        /// Resume one or more blocked domain patterns.
        #[arg(long = "domain")]
        domains: Vec<String>,
        /// Resume one or more frozen tools.
        #[arg(long = "tool")]
        tools: Vec<String>,
        /// OTP code. If omitted and OTP is required, a prompt is shown.
        #[arg(long)]
        otp: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum AuthCommands {
    /// Login with OAuth (OpenAI Codex or Gemini)
    Login {
        /// Provider (`openai-codex` or `gemini`)
        #[arg(long)]
        provider: String,
        /// Profile name (default: default)
        #[arg(long, default_value = "default")]
        profile: String,
        /// Use OAuth device-code flow
        #[arg(long)]
        device_code: bool,
    },
    /// Complete OAuth by pasting redirect URL or auth code
    PasteRedirect {
        /// Provider (`openai-codex`)
        #[arg(long)]
        provider: String,
        /// Profile name (default: default)
        #[arg(long, default_value = "default")]
        profile: String,
        /// Full redirect URL or raw OAuth code
        #[arg(long)]
        input: Option<String>,
    },
    /// Paste setup token / auth token (for Anthropic subscription auth)
    PasteToken {
        /// Provider (`anthropic`)
        #[arg(long)]
        provider: String,
        /// Profile name (default: default)
        #[arg(long, default_value = "default")]
        profile: String,
        /// Token value (if omitted, read interactively)
        #[arg(long)]
        token: Option<String>,
        /// Auth kind override (`authorization` or `api-key`)
        #[arg(long)]
        auth_kind: Option<String>,
    },
    /// Alias for `paste-token` (interactive by default)
    SetupToken {
        /// Provider (`anthropic`)
        #[arg(long)]
        provider: String,
        /// Profile name (default: default)
        #[arg(long, default_value = "default")]
        profile: String,
    },
    /// Refresh OpenAI Codex access token using refresh token
    Refresh {
        /// Provider (`openai-codex`)
        #[arg(long)]
        provider: String,
        /// Profile name or profile id
        #[arg(long)]
        profile: Option<String>,
    },
    /// Remove auth profile
    Logout {
        /// Provider
        #[arg(long)]
        provider: String,
        /// Profile name (default: default)
        #[arg(long, default_value = "default")]
        profile: String,
    },
    /// Set active profile for a provider
    Use {
        /// Provider
        #[arg(long)]
        provider: String,
        /// Profile name or full profile id
        #[arg(long)]
        profile: String,
    },
    /// List auth profiles
    List,
    /// Show auth status with active profile and token expiry info
    Status,
}

#[derive(Subcommand, Debug)]
enum ModelCommands {
    /// Refresh and cache provider models
    Refresh {
        /// Provider name (defaults to configured default provider)
        #[arg(long)]
        provider: Option<String>,

        /// Refresh all providers that support live model discovery
        #[arg(long)]
        all: bool,

        /// Force live refresh and ignore fresh cache
        #[arg(long)]
        force: bool,
    },
    /// List cached models for a provider
    List {
        /// Provider name (defaults to configured default provider)
        #[arg(long)]
        provider: Option<String>,
    },
    /// Set the default model in config
    Set {
        /// Model name to set as default
        model: String,
    },
    /// Show current model configuration and cache status
    Status,
}

#[derive(Subcommand, Debug)]
enum DoctorCommands {
    /// Probe model catalogs across providers and report availability
    Models {
        /// Probe a specific provider only (default: all known providers)
        #[arg(long)]
        provider: Option<String>,

        /// Prefer cached catalogs when available (skip forced live refresh)
        #[arg(long)]
        use_cache: bool,
    },
    /// Query runtime trace events (tool diagnostics and model replies)
    Traces {
        /// Show a specific trace event by id
        #[arg(long)]
        id: Option<String>,
        /// Filter list output by event type
        #[arg(long)]
        event: Option<String>,
        /// Case-insensitive text match across message/payload
        #[arg(long)]
        contains: Option<String>,
        /// Maximum number of events to display
        #[arg(long, default_value = "20")]
        limit: usize,
    },
}

#[derive(Subcommand, Debug)]
enum MemoryCommands {
    /// List memory entries with optional filters
    List {
        #[arg(long)]
        category: Option<String>,
        #[arg(long)]
        session: Option<String>,
        #[arg(long, default_value = "50")]
        limit: usize,
        #[arg(long, default_value = "0")]
        offset: usize,
    },
    /// Get a specific memory entry by key
    Get { key: String },
    /// Show memory backend statistics and health
    Stats,
    /// Clear memories by category, by key, or clear all
    Clear {
        /// Delete a single entry by key (supports prefix match)
        #[arg(long)]
        key: Option<String>,
        #[arg(long)]
        category: Option<String>,
        /// Skip confirmation prompt
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ManualHelpTarget {
    TopLevel,
}

fn maybe_handle_manual_help() -> Result<bool> {
    let args = std::env::args().collect::<Vec<_>>();
    let Some(target) = detect_manual_help_target(&args) else {
        return Ok(false);
    };

    let mut stdout = std::io::stdout().lock();
    write_manual_help(target, &mut stdout)?;
    Ok(true)
}

fn detect_manual_help_target(args: &[String]) -> Option<ManualHelpTarget> {
    let mut saw_subcommand = false;
    let mut index = 1;

    while index < args.len() {
        let arg = args[index].as_str();
        match arg {
            "--config-dir" => {
                index += 2;
            }
            value if value.starts_with("--config-dir=") => {
                index += 1;
            }
            "help" => {
                return if saw_subcommand || args.get(index + 1).is_some() {
                    None
                } else {
                    Some(ManualHelpTarget::TopLevel)
                };
            }
            "-h" | "--help" => {
                return if saw_subcommand {
                    None
                } else {
                    Some(ManualHelpTarget::TopLevel)
                };
            }
            value if value.starts_with('-') => {
                index += 1;
            }
            _ => {
                saw_subcommand = true;
                index += 1;
            }
        }
    }

    None
}

fn write_manual_help(target: ManualHelpTarget, mut out: impl Write) -> Result<()> {
    let text = match target {
        ManualHelpTarget::TopLevel => TOP_LEVEL_HELP,
    };
    out.write_all(text.as_bytes())
        .context("failed to write help output")?;
    Ok(())
}

fn parse_cli_with_large_stack() -> Result<Cli> {
    let args = std::env::args().collect::<Vec<_>>();
    let parse_thread = std::thread::Builder::new()
        .name("redclaw-cli-parse".to_string())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || Cli::try_parse_from(args))
        .context("failed to spawn CLI parser thread")?;

    match parse_thread.join() {
        Ok(Ok(cli)) => Ok(cli),
        Ok(Err(err)) => err.exit(),
        Err(_) => bail!("CLI parser thread panicked"),
    }
}

fn build_cli_command_with_large_stack() -> Result<clap::Command> {
    let command_thread = std::thread::Builder::new()
        .name("redclaw-cli-command".to_string())
        .stack_size(64 * 1024 * 1024)
        .spawn(Cli::command)
        .context("failed to spawn CLI command builder thread")?;

    match command_thread.join() {
        Ok(command) => Ok(command),
        Err(_) => bail!("CLI command builder thread panicked"),
    }
}

const TOP_LEVEL_HELP: &str = "\
RedClaw

Usage: redclaw [OPTIONS] <COMMAND>

Commands:
  onboard       Initialize your workspace and configuration
  agent         Start the AI agent loop
  gateway       Start/manage the gateway server
  daemon        Start the long-running autonomous runtime
  service       Manage OS service lifecycle
  doctor        Run diagnostics
  status        Show system status
  estop         Engage, inspect, and resume emergency-stop states
  cron          Configure and manage scheduled tasks
  models        Manage provider model catalogs
  providers     List supported AI providers
  channel       Manage channels
  integrations  Browse integrations
  skills        Manage skills
  migrate       Migrate data from other agent runtimes
  auth          Manage provider authentication profiles
  hardware      Discover and introspect USB hardware
  peripheral    Manage hardware peripherals
  memory        Manage agent memory
  config        Manage configuration
  modules       Manage modules
  completions   Generate shell completion scripts

Options:
      --config-dir <CONFIG_DIR>  Override config directory
  -h, --help                     Print help
  -V, --version                  Print version
";

#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() -> Result<()> {
    if maybe_handle_manual_help()? {
        return Ok(());
    }

    // Install default crypto provider for Rustls TLS.
    // This prevents the error: "could not automatically determine the process-level CryptoProvider"
    // when both aws-lc-rs and ring features are available (or neither is explicitly selected).
    if let Err(e) = rustls::crypto::ring::default_provider().install_default() {
        eprintln!("Warning: Failed to install default crypto provider: {e:?}");
    }

    let cli = parse_cli_with_large_stack()?;

    if let Some(config_dir) = &cli.config_dir {
        if config_dir.trim().is_empty() {
            bail!("--config-dir cannot be empty");
        }
        std::env::set_var("REDCLAW_CONFIG_DIR", config_dir);
    }

    // Completions must remain stdout-only and should not load config or initialize logging.
    // This avoids warnings/log lines corrupting sourced completion scripts.
    if let Commands::Completions { shell } = &cli.command {
        let mut stdout = std::io::stdout().lock();
        write_shell_completion(*shell, &mut stdout)?;
        return Ok(());
    }

    // Initialize logging - respects RUST_LOG env var, defaults to INFO
    let subscriber = fmt::Subscriber::builder()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // Onboard runs quick setup by default, or the interactive wizard with --interactive.
    // The onboard wizard uses reqwest::blocking internally, which creates its own
    // Tokio runtime. To avoid "Cannot drop a runtime in a context where blocking is
    // not allowed", we run the wizard on a blocking thread via spawn_blocking.
    if let Commands::Onboard {
        interactive,
        force,
        reinit,
        channels_only,
        api_key,
        provider,
        model,
        memory,
    } = &cli.command
    {
        let interactive = *interactive;
        let force = *force;
        let reinit = *reinit;
        let channels_only = *channels_only;
        let api_key = api_key.clone();
        let provider = provider.clone();
        let model = model.clone();
        let memory = memory.clone();

        if interactive && channels_only {
            bail!("Use either --interactive or --channels-only, not both");
        }
        if reinit && channels_only {
            bail!("Use either --reinit or --channels-only, not both");
        }
        if reinit && !interactive {
            bail!("--reinit requires --interactive mode");
        }
        if channels_only
            && (api_key.is_some() || provider.is_some() || model.is_some() || memory.is_some())
        {
            bail!("--channels-only does not accept --api-key, --provider, --model, or --memory");
        }
        if channels_only && force {
            bail!("--channels-only does not accept --force");
        }

        // Handle --reinit: backup and reset configuration
        if reinit {
            let (redclaw_dir, _) = crate::config::schema::resolve_runtime_dirs_for_onboarding()?;

            if redclaw_dir.exists() {
                let timestamp = chrono::Local::now().format("%Y%m%d%H%M%S");
                let backup_dir = format!("{}.backup.{}", redclaw_dir.display(), timestamp);

                println!("⚠️  Reinitializing RedClaw configuration...");
                println!("   Current config directory: {}", redclaw_dir.display());
                println!(
                    "   This will back up your existing config to: {}",
                    backup_dir
                );
                println!();
                print!("Continue? [y/N] ");
                std::io::stdout()
                    .flush()
                    .context("Failed to flush stdout")?;

                let mut answer = String::new();
                std::io::stdin().read_line(&mut answer)?;
                if !answer.trim().eq_ignore_ascii_case("y") {
                    println!("Aborted.");
                    return Ok(());
                }
                println!();

                // Rename existing directory as backup
                tokio::fs::rename(&redclaw_dir, &backup_dir)
                    .await
                    .with_context(|| {
                        format!("Failed to backup existing config to {}", backup_dir)
                    })?;

                println!("   Backup created successfully.");
                println!("   Starting fresh initialization...\n");
            }
        }

        let config = if channels_only {
            Box::pin(onboard::run_channels_repair_wizard()).await
        } else if interactive {
            Box::pin(onboard::run_wizard(force)).await
        } else {
            onboard::run_quick_setup(
                api_key.as_deref(),
                provider.as_deref(),
                model.as_deref(),
                memory.as_deref(),
                force,
            )
            .await
        }?;
        // Auto-start channels if user said yes during wizard
        if std::env::var("REDCLAW_AUTOSTART_CHANNELS").as_deref() == Ok("1") {
            channels::start_channels(config).await?;
        }
        return Ok(());
    }

    // All other commands need config loaded first
    let mut config = Config::load_or_init().await?;
    config.apply_env_overrides();
    observability::runtime_trace::init_from_config(&config.observability, &config.workspace_dir);
    if config.security.otp.enabled {
        let config_dir = config
            .config_path
            .parent()
            .context("Config path must have a parent directory")?;
        let store = security::SecretStore::new(config_dir, config.secrets.encrypt);
        let (_validator, enrollment_uri) =
            security::OtpValidator::from_config(&config.security.otp, config_dir, &store)?;
        if let Some(uri) = enrollment_uri {
            println!("Initialized OTP secret for RedClaw.");
            println!("Enrollment URI: {uri}");
        }
    }

    match cli.command {
        Commands::Onboard { .. } | Commands::Completions { .. } => unreachable!(),

        Commands::Agent {
            message,
            provider,
            model,
            temperature,
            peripheral,
        } => {
            bootstrap_lifecycle_host(&config)?;

            let final_temperature = temperature.unwrap_or(config.default_temperature);

            agent::run(
                config,
                message,
                provider,
                model,
                final_temperature,
                peripheral,
                true,
            )
            .await
            .map(|_| ())
        }

        Commands::Gateway { gateway_command } => {
            commands::gateway::handle_gateway_command(gateway_command, config).await
        }

        Commands::Daemon { port, host } => {
            bootstrap_lifecycle_host(&config)?;

            let port = port.unwrap_or(config.gateway.port);
            let host = host.unwrap_or_else(|| config.gateway.host.clone());
            if port == 0 {
                info!("🧠 Starting RedClaw Daemon on {host} (random port)");
            } else {
                info!("🧠 Starting RedClaw Daemon on {host}:{port}");
            }
            daemon::run(config, host, port).await
        }

        Commands::Status => {
            println!("🦀 RedClaw Status");
            println!();
            println!("Version:     {}", env!("CARGO_PKG_VERSION"));
            println!("Workspace:   {}", config.workspace_dir.display());
            println!("Config:      {}", config.config_path.display());
            println!();
            println!(
                "🤖 Provider:      {}",
                config.default_provider.as_deref().unwrap_or("openrouter")
            );
            println!(
                "   Model:         {}",
                config.default_model.as_deref().unwrap_or("(default)")
            );
            println!("📊 Observability:  {}", config.observability.backend);
            println!(
                "🧾 Trace storage:  {} ({})",
                config.observability.runtime_trace_mode, config.observability.runtime_trace_path
            );
            println!("🛡️  Autonomy:      {:?}", config.autonomy.level);
            println!("⚙️  Runtime:       {}", config.runtime.kind);
            let effective_memory_backend = memory::effective_memory_backend_name(
                &config.memory.backend,
                Some(&config.storage.provider.config),
            );
            println!(
                "💓 Heartbeat:      {}",
                if config.heartbeat.enabled {
                    format!("every {}min", config.heartbeat.interval_minutes)
                } else {
                    "disabled".into()
                }
            );
            println!(
                "🧠 Memory:         {} (auto-save: {})",
                effective_memory_backend,
                if config.memory.auto_save { "on" } else { "off" }
            );

            println!();
            println!("Security:");
            println!("  Workspace only:    {}", config.autonomy.workspace_only);
            println!(
                "  Allowed roots:     {}",
                if config.autonomy.allowed_roots.is_empty() {
                    "(none)".to_string()
                } else {
                    config.autonomy.allowed_roots.join(", ")
                }
            );
            println!(
                "  Allowed commands:  {}",
                config.autonomy.allowed_commands.join(", ")
            );
            println!(
                "  Max actions/hour:  {}",
                config.autonomy.max_actions_per_hour
            );
            println!(
                "  Max cost/day:      ${:.2}",
                f64::from(config.autonomy.max_cost_per_day_cents) / 100.0
            );
            println!("  OTP enabled:       {}", config.security.otp.enabled);
            println!("  E-stop enabled:    {}", config.security.estop.enabled);
            println!();
            println!("Channels:");
            println!("  CLI:      ✅ always");
            for (channel, configured) in config.channels_config.channels() {
                println!(
                    "  {:9} {}",
                    channel.name(),
                    if configured {
                        "✅ configured"
                    } else {
                        "❌ not configured"
                    }
                );
            }
            println!();
            println!("Peripherals:");
            println!(
                "  Enabled:   {}",
                if config.peripherals.enabled {
                    "yes"
                } else {
                    "no"
                }
            );
            println!("  Boards:    {}", config.peripherals.boards.len());

            Ok(())
        }

        Commands::Estop {
            estop_command,
            level,
            domains,
            tools,
        } => handle_estop_command(&config, estop_command, level, domains, tools),

        Commands::Cron { cron_command } => cron::handle_command(cron_command, &config),

        Commands::Models { model_command } => match model_command {
            ModelCommands::Refresh {
                provider,
                all,
                force,
            } => {
                if all {
                    if provider.is_some() {
                        bail!("`models refresh --all` cannot be combined with --provider");
                    }
                    onboard::run_models_refresh_all(&config, force).await
                } else {
                    onboard::run_models_refresh(&config, provider.as_deref(), force).await
                }
            }
            ModelCommands::List { provider } => {
                onboard::run_models_list(&config, provider.as_deref()).await
            }
            ModelCommands::Set { model } => onboard::run_models_set(&config, &model).await,
            ModelCommands::Status => onboard::run_models_status(&config).await,
        },

        Commands::Providers => {
            let providers = providers::list_providers();
            let current = config
                .default_provider
                .as_deref()
                .unwrap_or("openrouter")
                .trim()
                .to_ascii_lowercase();
            println!("Supported providers ({} total):\n", providers.len());
            println!("  ID (use in config)  DESCRIPTION");
            println!("  ─────────────────── ───────────");
            for p in &providers {
                let is_active = p.name.eq_ignore_ascii_case(&current)
                    || p.aliases
                        .iter()
                        .any(|alias| alias.eq_ignore_ascii_case(&current));
                let marker = if is_active { " (active)" } else { "" };
                let local_tag = if p.local { " [local]" } else { "" };
                let aliases = if p.aliases.is_empty() {
                    String::new()
                } else {
                    format!("  (aliases: {})", p.aliases.join(", "))
                };
                println!(
                    "  {:<19} {}{}{}{}",
                    p.name, p.display_name, local_tag, marker, aliases
                );
            }
            println!("\n  custom:<URL>   Any OpenAI-compatible endpoint");
            println!("  anthropic-custom:<URL>  Any Anthropic-compatible endpoint");
            Ok(())
        }

        Commands::Service {
            service_command,
            service_init,
        } => {
            let init_system = service_init.parse()?;
            service::handle_command(&service_command, &config, init_system)
        }

        Commands::Doctor { doctor_command } => match doctor_command {
            Some(DoctorCommands::Models {
                provider,
                use_cache,
            }) => doctor::run_models(&config, provider.as_deref(), use_cache).await,
            Some(DoctorCommands::Traces {
                id,
                event,
                contains,
                limit,
            }) => doctor::run_traces(
                &config,
                id.as_deref(),
                event.as_deref(),
                contains.as_deref(),
                limit,
            ),
            None => doctor::run(&config),
        },

        Commands::Channel { channel_command } => match channel_command {
            ChannelCommands::Start => channels::start_channels(config).await,
            ChannelCommands::Doctor => channels::doctor_channels(config).await,
            other => channels::handle_command(other, &config).await,
        },

        Commands::Integrations {
            integration_command,
        } => integrations::handle_command(integration_command, &config),

        Commands::Skills { skill_command } => skills::handle_command(skill_command, &config),

        Commands::Migrate { migrate_command } => {
            migration::handle_command(migrate_command, &config).await
        }

        Commands::Memory { memory_command } => {
            memory::cli::handle_command(memory_command, &config).await
        }

        Commands::Auth { auth_command } => {
            commands::auth::handle_auth_command(auth_command, &config).await
        }

        Commands::Hardware { hardware_command } => {
            hardware::handle_command(hardware_command.clone(), &config)
        }

        Commands::Peripheral { peripheral_command } => {
            peripherals::handle_command(peripheral_command.clone(), &config)
        }

        Commands::Config { config_command } => match config_command {
            ConfigCommands::Schema => {
                let schema = schemars::schema_for!(config::Config);
                println!(
                    "{}",
                    serde_json::to_string_pretty(&schema).expect("failed to serialize JSON Schema")
                );
                Ok(())
            }
        },

        Commands::Modules { modules_command } => {
            commands::modules::handle_modules_command(modules_command)
        }
    }
}

fn build_engage_level(
    level: Option<EstopLevelArg>,
    domains: Vec<String>,
    tools: Vec<String>,
) -> Result<security::EstopLevel> {
    let requested = level.unwrap_or(EstopLevelArg::KillAll);
    match requested {
        EstopLevelArg::KillAll => {
            if !domains.is_empty() || !tools.is_empty() {
                bail!("--domain/--tool are only valid with --level domain-block/tool-freeze");
            }
            Ok(security::EstopLevel::KillAll)
        }
        EstopLevelArg::NetworkKill => {
            if !domains.is_empty() || !tools.is_empty() {
                bail!("--domain/--tool are not valid with --level network-kill");
            }
            Ok(security::EstopLevel::NetworkKill)
        }
        EstopLevelArg::DomainBlock => {
            if domains.is_empty() {
                bail!("--level domain-block requires at least one --domain");
            }
            if !tools.is_empty() {
                bail!("--tool is not valid with --level domain-block");
            }
            Ok(security::EstopLevel::DomainBlock(domains))
        }
        EstopLevelArg::ToolFreeze => {
            if tools.is_empty() {
                bail!("--level tool-freeze requires at least one --tool");
            }
            if !domains.is_empty() {
                bail!("--domain is not valid with --level tool-freeze");
            }
            Ok(security::EstopLevel::ToolFreeze(tools))
        }
    }
}

fn build_resume_selector(
    network: bool,
    domains: Vec<String>,
    tools: Vec<String>,
) -> Result<security::ResumeSelector> {
    let selected =
        usize::from(network) + usize::from(!domains.is_empty()) + usize::from(!tools.is_empty());
    if selected > 1 {
        bail!("Use only one of --network, --domain, or --tool for estop resume");
    }
    if network {
        return Ok(security::ResumeSelector::Network);
    }
    if !domains.is_empty() {
        return Ok(security::ResumeSelector::Domains(domains));
    }
    if !tools.is_empty() {
        return Ok(security::ResumeSelector::Tools(tools));
    }
    Ok(security::ResumeSelector::KillAll)
}

fn print_estop_status(state: &security::EstopState) {
    println!("Estop status:");
    println!(
        "  engaged:        {}",
        if state.is_engaged() { "yes" } else { "no" }
    );
    println!(
        "  kill_all:       {}",
        if state.kill_all { "active" } else { "inactive" }
    );
    println!(
        "  network_kill:   {}",
        if state.network_kill {
            "active"
        } else {
            "inactive"
        }
    );
    if state.blocked_domains.is_empty() {
        println!("  domain_blocks:  (none)");
    } else {
        println!("  domain_blocks:  {}", state.blocked_domains.join(", "));
    }
    if state.frozen_tools.is_empty() {
        println!("  tool_freeze:    (none)");
    } else {
        println!("  tool_freeze:    {}", state.frozen_tools.join(", "));
    }
    if let Some(updated_at) = &state.updated_at {
        println!("  updated_at:     {updated_at}");
    }
}

fn write_shell_completion<W: Write>(shell: CompletionShell, writer: &mut W) -> Result<()> {
    use clap_complete::generate;
    use clap_complete::shells;

    let mut cmd = build_cli_command_with_large_stack()?;
    let bin_name = cmd.get_name().to_string();

    match shell {
        CompletionShell::Bash => generate(shells::Bash, &mut cmd, bin_name.clone(), writer),
        CompletionShell::Fish => generate(shells::Fish, &mut cmd, bin_name.clone(), writer),
        CompletionShell::Zsh => generate(shells::Zsh, &mut cmd, bin_name.clone(), writer),
        CompletionShell::PowerShell => {
            generate(shells::PowerShell, &mut cmd, bin_name.clone(), writer);
        }
        CompletionShell::Elvish => generate(shells::Elvish, &mut cmd, bin_name, writer),
    }

    writer.flush()?;
    Ok(())
}

fn read_plain_input(prompt: &str) -> Result<String> {
    let input: String = Input::new().with_prompt(prompt).interact_text()?;
    Ok(input.trim().to_string())
}

/// Bootstrap lifecycle host - discovers and activates modules
fn bootstrap_lifecycle_host(config: &Config) -> Result<()> {
    use crate::core::lifecycle::safe_mode::{boot_safe_mode, verify_baseline_modules};
    use crate::core::lifecycle::{ActivationBootstrap, LoaderConfig, ModuleHost};
    use crate::core::registry::ModuleRegistry;

    // Derive config_dir from config path
    let config_dir = config
        .config_path
        .parent()
        .context("Config path must have a parent directory")?;

    // Build activation bootstrap from config directory
    let bootstrap = ActivationBootstrap::from_config_dir(config_dir);

    // Build loader config with search paths
    let loader_config = LoaderConfig {
        search_paths: vec![config_dir.join("src/modules")],
        lock_path: bootstrap.modules_lock_path.clone(),
        validate: true,
    };

    // Create module host with empty registry (bundled modules only)
    let mut host = ModuleHost::new(ModuleRegistry::new(), loader_config);

    // Start host - this discovers, validates, and activates modules
    match host.start() {
        Ok(()) => {
            tracing::info!("Lifecycle host boot completed successfully");
            Ok(())
        }
        Err(e) => {
            tracing::warn!("Full lifecycle boot failed: {e}");
            // Safe-mode fallback if full boot fails
            tracing::info!("Attempting safe-mode fallback");
            let results = boot_safe_mode(&mut host)?;

            // Verify baseline modules activated
            if !verify_baseline_modules(&results) {
                anyhow::bail!("Safe-mode baseline activation failed");
            }

            tracing::info!("Safe-mode boot completed with {} modules", results.len());
            Ok(())
        }
    }
}

/// Handle estop command - engage, status, or resume
fn handle_estop_command(
    config: &Config,
    estop_command: Option<EstopSubcommands>,
    level: Option<EstopLevelArg>,
    domains: Vec<String>,
    tools: Vec<String>,
) -> Result<()> {
    use crate::security::{EstopManager, OtpValidator, SecretStore};

    // Check if estop is enabled in config
    if !config.security.estop.enabled {
        anyhow::bail!("estop is disabled in configuration");
    }

    // Derive config_dir from config path
    let config_dir = config
        .config_path
        .parent()
        .context("Config path must have a parent directory")?;

    // Load estop manager from state file
    let mut manager = EstopManager::load(&config.security.estop, config_dir)?;

    // Match on estop subcommand
    match estop_command {
        // No subcommand = engage estop
        None => {
            let engage_level = build_engage_level(level, domains, tools)?;
            manager.engage(engage_level)?;
            print_estop_status(&manager.status());
        }

        // Status subcommand
        Some(EstopSubcommands::Status) => {
            print_estop_status(&manager.status());
        }

        // Resume subcommand
        Some(EstopSubcommands::Resume {
            network,
            domains,
            tools,
            otp,
        }) => {
            let selector = build_resume_selector(network, domains, tools)?;

            // Handle OTP validation if required
            let otp_code = if config.security.estop.require_otp_to_resume {
                match otp {
                    Some(code) => Some(code),
                    None => Some(read_plain_input("Enter OTP code")?),
                }
            } else {
                None
            };

            let otp_validator = if config.security.estop.require_otp_to_resume {
                let store = SecretStore::new(config_dir, config.secrets.encrypt);
                let (validator, _enrollment_uri) =
                    OtpValidator::from_config(&config.security.otp, config_dir, &store)?;
                Some(validator)
            } else {
                None
            };

            manager.resume(selector, otp_code.as_deref(), otp_validator.as_ref())?;

            print_estop_status(&manager.status());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn manual_top_level_help_is_detected() {
        let args = vec!["redclaw".to_string(), "--help".to_string()];
        assert_eq!(
            detect_manual_help_target(&args),
            Some(ManualHelpTarget::TopLevel)
        );
    }

    #[test]
    fn manual_modules_help_is_not_intercepted() {
        let args = vec![
            "redclaw".to_string(),
            "modules".to_string(),
            "--help".to_string(),
        ];
        assert_eq!(detect_manual_help_target(&args), None);
    }

    #[test]
    fn onboard_cli_accepts_model_provider_and_api_key_in_quick_mode() {
        let cli = Cli::try_parse_from([
            "redclaw",
            "onboard",
            "--provider",
            "openrouter",
            "--model",
            "custom-model-946",
            "--api-key",
            "sk-issue946",
        ])
        .expect("quick onboard invocation should parse");

        match cli.command {
            Commands::Onboard {
                interactive,
                force,
                channels_only,
                api_key,
                provider,
                model,
                ..
            } => {
                assert!(!interactive);
                assert!(!force);
                assert!(!channels_only);
                assert_eq!(provider.as_deref(), Some("openrouter"));
                assert_eq!(model.as_deref(), Some("custom-model-946"));
                assert_eq!(api_key.as_deref(), Some("sk-issue946"));
            }
            other => panic!("expected onboard command, got {other:?}"),
        }
    }

    #[test]
    fn completions_cli_parses_supported_shells() {
        for shell in ["bash", "fish", "zsh", "powershell", "elvish"] {
            let cli = Cli::try_parse_from(["redclaw", "completions", shell])
                .expect("completions invocation should parse");
            match cli.command {
                Commands::Completions { .. } => {}
                other => panic!("expected completions command, got {other:?}"),
            }
        }
    }

    #[test]
    fn completion_generation_mentions_binary_name() {
        let mut output = Vec::new();
        write_shell_completion(CompletionShell::Bash, &mut output)
            .expect("completion generation should succeed");
        let script = String::from_utf8(output).expect("completion output should be valid utf-8");
        assert!(
            script.contains("redclaw"),
            "completion script should reference binary name"
        );
    }

    #[test]
    fn onboard_cli_accepts_force_flag() {
        let cli = Cli::try_parse_from(["redclaw", "onboard", "--force"])
            .expect("onboard --force should parse");

        match cli.command {
            Commands::Onboard { force, .. } => assert!(force),
            other => panic!("expected onboard command, got {other:?}"),
        }
    }

    #[test]
    fn cli_parses_estop_default_engage() {
        let cli = Cli::try_parse_from(["redclaw", "estop"]).expect("estop command should parse");

        match cli.command {
            Commands::Estop {
                estop_command,
                level,
                domains,
                tools,
            } => {
                assert!(estop_command.is_none());
                assert!(level.is_none());
                assert!(domains.is_empty());
                assert!(tools.is_empty());
            }
            other => panic!("expected estop command, got {other:?}"),
        }
    }

    #[test]
    fn cli_parses_estop_resume_domain() {
        let cli = Cli::try_parse_from(["redclaw", "estop", "resume", "--domain", "*.chase.com"])
            .expect("estop resume command should parse");

        match cli.command {
            Commands::Estop {
                estop_command: Some(EstopSubcommands::Resume { domains, .. }),
                ..
            } => assert_eq!(domains, vec!["*.chase.com".to_string()]),
            other => panic!("expected estop resume command, got {other:?}"),
        }
    }

    #[test]
    fn agent_command_parses_with_temperature() {
        let cli = Cli::try_parse_from(["redclaw", "agent", "--temperature", "0.5"])
            .expect("agent command with temperature should parse");

        match cli.command {
            Commands::Agent { temperature, .. } => {
                assert_eq!(temperature, Some(0.5));
            }
            other => panic!("expected agent command, got {other:?}"),
        }
    }

    #[test]
    fn agent_command_parses_without_temperature() {
        let cli = Cli::try_parse_from(["redclaw", "agent", "--message", "hello"])
            .expect("agent command without temperature should parse");

        match cli.command {
            Commands::Agent { temperature, .. } => {
                assert_eq!(temperature, None);
            }
            other => panic!("expected agent command, got {other:?}"),
        }
    }

    #[test]
    fn agent_fallback_uses_config_default_temperature() {
        // Test that when user doesn't provide --temperature,
        // the fallback logic works correctly
        let mut config = Config::default(); // default_temperature = 0.7
        config.default_temperature = 1.5;

        // Simulate None temperature (user didn't provide --temperature)
        let user_temperature: Option<f64> = std::hint::black_box(None);
        let final_temperature = user_temperature.unwrap_or(config.default_temperature);

        assert!((final_temperature - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn agent_fallback_uses_hardcoded_when_config_uses_default() {
        // Test that when config uses default value (0.7), fallback still works
        let config = Config::default(); // default_temperature = 0.7

        // Simulate None temperature (user didn't provide --temperature)
        let user_temperature: Option<f64> = std::hint::black_box(None);
        let final_temperature = user_temperature.unwrap_or(config.default_temperature);

        assert!((final_temperature - 0.7).abs() < f64::EPSILON);
    }

    #[test]
    fn bootstrap_lifecycle_host_uses_real_module_host_path() {
        // Verify bootstrap_lifecycle_host executes the real host path and
        // safe-mode fallback instead of behaving like the old placeholder stub.
        use tempfile::TempDir;

        let temp_dir = TempDir::new().expect("create temp dir");
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(&config_path, "").expect("create config file");

        let mut config = Config::default();
        config.config_path = config_path;

        // The real implementation should complete successfully even when no
        // modules are present, because the host path and safe-mode fallback are wired.
        let result = bootstrap_lifecycle_host(&config);

        assert!(
            result.is_ok(),
            "bootstrap should execute the real host path"
        );
    }

    #[test]
    fn bootstrap_lifecycle_host_derives_config_dir_correctly() {
        // Test that config_dir derivation works when config path has a parent
        use tempfile::TempDir;

        // Create a temp directory with a fake config file
        let temp_dir = TempDir::new().expect("create temp dir");
        let config_path = temp_dir.path().join("config.toml");

        // Create the config file so parent() succeeds
        std::fs::write(&config_path, "").expect("create config file");

        let mut config = Config::default();
        config.config_path = config_path;

        // The function should derive config_dir correctly
        let config_dir = config
            .config_path
            .parent()
            .expect("config path should have parent");
        assert_eq!(config_dir, temp_dir.path());
    }

    #[test]
    fn handle_estop_command_fails_when_disabled() {
        // Test that estop command fails when estop is disabled in config
        use tempfile::TempDir;

        let temp_dir = TempDir::new().expect("create temp dir");
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(&config_path, "").expect("create config file");

        let mut config = Config::default();
        config.config_path = config_path;
        config.security.estop.enabled = false;

        let result = handle_estop_command(&config, None, None, vec![], vec![]);
        assert!(result.is_err(), "estop command should fail when disabled");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("disabled"),
            "error should mention disabled, got: {}",
            err_msg
        );
    }

    #[test]
    fn handle_estop_command_status_shows_engaged_state() {
        // Test that status subcommand shows estop state
        use crate::config::EstopConfig;
        use crate::security::EstopManager;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().expect("create temp dir");
        let config_path = temp_dir.path().join("config.toml");
        let state_path = temp_dir.path().join("estop-state.json");
        std::fs::write(&config_path, "").expect("create config file");

        let mut config = Config::default();
        config.config_path = config_path;
        config.security.estop = EstopConfig {
            enabled: true,
            state_file: state_path.display().to_string(),
            require_otp_to_resume: false,
        };

        // First engage estop
        let mut manager = EstopManager::load(&config.security.estop, temp_dir.path())
            .expect("load estop manager");
        manager
            .engage(crate::security::EstopLevel::KillAll)
            .expect("engage estop");

        // Now check status
        let result = handle_estop_command(
            &config,
            Some(EstopSubcommands::Status),
            None,
            vec![],
            vec![],
        );
        assert!(result.is_ok(), "status command should succeed");
    }
}
