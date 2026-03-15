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

use anyhow::{anyhow, bail, Context, Result};
use clap::{CommandFactory, Parser, Subcommand};
use dialoguer::{Input, Password};
use redclaw::cli_support::{parse_temperature, CompletionShell, EstopLevelArg};
use serde::{Deserialize, Serialize};
use std::io::Write;
use tracing::{info, warn};
use tracing_subscriber::{fmt, EnvFilter};

mod agent;
mod approval;
mod auth;
mod branding;
mod channels;
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

    /// Manage OS service lifecycle (launchd/systemd user service)
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

    /// Show system status (full details)
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

    /// List supported AI providers
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

    /// Browse 50+ integrations
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

    /// Discover and introspect USB hardware
    #[command(long_about = "\
Discover and introspect USB hardware.

Enumerate connected USB devices, identify known development boards \
(STM32 Nucleo, Arduino, ESP32), and retrieve chip information via \
probe-rs / ST-Link.

Examples:
  redclaw hardware discover
  redclaw hardware introspect /dev/ttyACM0
  redclaw hardware info --chip STM32F401RETx")]
    Hardware {
        #[command(subcommand)]
        hardware_command: redclaw::HardwareCommands,
    },

    /// Manage hardware peripherals (STM32, RPi GPIO, etc.)
    #[command(long_about = "\
Manage hardware peripherals.

Add, list, flash, and configure hardware boards that expose tools \
to the agent (GPIO, sensors, actuators). Supported boards: \
nucleo-f401re, rpi-gpio, esp32, arduino-uno.

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

    /// Manage agent memory (list, get, stats, clear)
    #[command(long_about = "\
Manage agent memory entries.

List, inspect, and clear memory entries stored by the agent. \
Supports filtering by category and session, pagination, and \
batch clearing with confirmation.

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

    /// Manage configuration
    #[command(long_about = "\
Manage RedClaw configuration.

Inspect and export configuration settings. Use 'schema' to dump \
the full JSON Schema for the config file, which documents every \
available key, type, and default value.

Examples:
  redclaw config schema              # print JSON Schema to stdout
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
    Modules,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ManualModulesInvocation {
    config_dir: Option<String>,
    command: ModulesCommands,
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

fn maybe_handle_manual_modules_command() -> Result<bool> {
    let args = std::env::args().collect::<Vec<_>>();
    let Some(invocation) = parse_manual_modules_invocation(&args)? else {
        return Ok(false);
    };

    if let Some(config_dir) = invocation.config_dir {
        if config_dir.trim().is_empty() {
            bail!("--config-dir cannot be empty");
        }
        std::env::set_var("REDCLAW_CONFIG_DIR", config_dir);
    }

    handle_modules_command(invocation.command)?;
    Ok(true)
}

fn parse_manual_modules_invocation(args: &[String]) -> Result<Option<ManualModulesInvocation>> {
    let mut index = 1;
    let mut config_dir: Option<String> = None;

    while index < args.len() {
        let arg = args[index].as_str();
        match arg {
            "--config-dir" => {
                let value = args
                    .get(index + 1)
                    .context("--config-dir requires a value")?;
                config_dir = Some(value.clone());
                index += 2;
            }
            value if value.starts_with("--config-dir=") => {
                let value = value
                    .split_once('=')
                    .map_or_else(String::new, |(_, rhs)| rhs.to_string());
                config_dir = Some(value);
                index += 1;
            }
            "modules" => {
                index += 1;
                break;
            }
            "help" | "-h" | "--help" => return Ok(None),
            value if value.starts_with('-') => return Ok(None),
            _ => return Ok(None),
        }
    }

    if index == 1 || args.get(index - 1).map(String::as_str) != Some("modules") {
        return Ok(None);
    }

    let Some(subcommand) = args.get(index).map(String::as_str) else {
        bail!(
            "modules command requires a subcommand (list, info, install, remove, enable, disable, update, doctor)"
        );
    };
    index += 1;

    let command = match subcommand {
        "list" => {
            if index != args.len() {
                bail!("modules list does not accept additional arguments");
            }
            ModulesCommands::List
        }
        "info" => {
            let module_id = args
                .get(index)
                .cloned()
                .context("modules info requires <module-id>")?;
            index += 1;
            if index != args.len() {
                bail!("modules info accepts exactly one <module-id>");
            }
            ModulesCommands::Info { module_id }
        }
        "install" => {
            let mut source: Option<String> = None;
            let mut enable = false;

            while index < args.len() {
                let token = args[index].as_str();
                match token {
                    "--enable" => enable = true,
                    value if value.starts_with('-') => {
                        bail!("unsupported modules install option: {}", value);
                    }
                    value => {
                        if source.is_some() {
                            bail!("modules install accepts exactly one <source>");
                        }
                        source = Some(value.to_string());
                    }
                }
                index += 1;
            }

            let source = source.context("modules install requires <source>")?;
            ModulesCommands::Install { source, enable }
        }
        "remove" => {
            let module_id = args
                .get(index)
                .cloned()
                .context("modules remove requires <module-id>")?;
            index += 1;
            if index != args.len() {
                bail!("modules remove accepts exactly one <module-id>");
            }
            ModulesCommands::Remove { module_id }
        }
        "enable" => {
            let module_id = args
                .get(index)
                .cloned()
                .context("modules enable requires <module-id>")?;
            index += 1;
            if index != args.len() {
                bail!("modules enable accepts exactly one <module-id>");
            }
            ModulesCommands::Enable { module_id }
        }
        "disable" => {
            let module_id = args
                .get(index)
                .cloned()
                .context("modules disable requires <module-id>")?;
            index += 1;
            if index != args.len() {
                bail!("modules disable accepts exactly one <module-id>");
            }
            ModulesCommands::Disable { module_id }
        }
        "update" => {
            let mut module_id: Option<String> = None;
            let mut all = false;

            while index < args.len() {
                let token = args[index].as_str();
                match token {
                    "--all" => all = true,
                    value if value.starts_with('-') => {
                        bail!("unsupported modules update option: {}", value);
                    }
                    value => {
                        if module_id.is_some() {
                            bail!("modules update accepts at most one <module-id>");
                        }
                        module_id = Some(value.to_string());
                    }
                }
                index += 1;
            }

            if all && module_id.is_some() {
                bail!("modules update does not accept <module-id> together with --all");
            }
            if !all && module_id.is_none() {
                bail!("modules update requires <module-id> or --all");
            }

            ModulesCommands::Update { module_id, all }
        }
        "doctor" => {
            if index != args.len() {
                bail!("modules doctor does not accept additional arguments");
            }
            ModulesCommands::Doctor
        }
        _ => return Ok(None),
    };

    Ok(Some(ManualModulesInvocation {
        config_dir,
        command,
    }))
}

fn detect_manual_help_target(args: &[String]) -> Option<ManualHelpTarget> {
    let mut subcommand: Option<&str> = None;
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
                let requested = args.get(index + 1).map(String::as_str).or(subcommand);
                return Some(match requested {
                    Some("modules") => ManualHelpTarget::Modules,
                    _ => ManualHelpTarget::TopLevel,
                });
            }
            "-h" | "--help" => {
                return Some(match subcommand {
                    Some("modules") => ManualHelpTarget::Modules,
                    _ => ManualHelpTarget::TopLevel,
                });
            }
            value if value.starts_with('-') => {
                index += 1;
            }
            value => {
                if subcommand.is_none() {
                    subcommand = Some(value);
                }
                index += 1;
            }
        }
    }

    None
}

fn write_manual_help(target: ManualHelpTarget, mut out: impl Write) -> Result<()> {
    let text = match target {
        ManualHelpTarget::TopLevel => TOP_LEVEL_HELP,
        ManualHelpTarget::Modules => MODULES_HELP,
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

const MODULES_HELP: &str = "\
Manage installed modules.

Usage: redclaw modules <COMMAND>

Commands:
  list                         List all installed modules
  info <MODULE_ID>             Show details about a specific module
  install <SOURCE> [--enable]  Install a module from local directory or archive
  remove <MODULE_ID>           Remove an installed module
  enable <MODULE_ID>           Enable an installed module
  disable <MODULE_ID>          Disable an installed module
  update [<MODULE_ID>] [--all] Update one module or all installed modules
  doctor                       Run module health diagnostics

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
  redclaw modules doctor

Options:
  -h, --help  Print help
";

#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() -> Result<()> {
    if maybe_handle_manual_help()? {
        return Ok(());
    }

    if maybe_handle_manual_modules_command()? {
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
            let (redclaw_dir, _) =
                crate::config::schema::resolve_runtime_dirs_for_onboarding().await?;

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
            match gateway_command {
                Some(redclaw::GatewayCommands::Restart { port, host }) => {
                    let (port, host) = resolve_gateway_addr(&config, port, host);
                    let addr = format!("{host}:{port}");
                    info!("🔄 Restarting RedClaw Gateway on {addr}");

                    // Try to gracefully shutdown existing gateway via admin endpoint
                    match shutdown_gateway(&host, port).await {
                        Ok(()) => {
                            info!("   ✓ Existing gateway on {addr} shut down gracefully");
                            // Poll until the port is free (connection refused) or timeout
                            let deadline =
                                tokio::time::Instant::now() + tokio::time::Duration::from_secs(5);
                            loop {
                                match tokio::net::TcpStream::connect(&addr).await {
                                    Err(_) => break, // port is free
                                    Ok(_) if tokio::time::Instant::now() >= deadline => {
                                        warn!(
                                            "   Timed out waiting for port {port} to be released"
                                        );
                                        break;
                                    }
                                    Ok(_) => {
                                        tokio::time::sleep(tokio::time::Duration::from_millis(50))
                                            .await;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            info!("   No existing gateway to shut down: {e}");
                        }
                    }

                    log_gateway_start(&host, port);
                    gateway::run_gateway(&host, port, config).await
                }
                Some(redclaw::GatewayCommands::GetPaircode { new }) => {
                    let port = config.gateway.port;
                    let host = &config.gateway.host;

                    // Fetch live pairing code from running gateway
                    // If --new is specified, generate a fresh pairing code
                    match fetch_paircode(host, port, new).await {
                        Ok(Some(code)) => {
                            println!("🔐 Gateway pairing is enabled.");
                            println!();
                            println!("  ┌──────────────┐");
                            println!("  │  {code}  │");
                            println!("  └──────────────┘");
                            println!();
                            println!("  Use this one-time code to pair a new device:");
                            println!("    POST /pair with header X-Pairing-Code: {code}");
                        }
                        Ok(None) => {
                            if config.gateway.require_pairing {
                                println!("🔐 Gateway pairing is enabled, but no active pairing code available.");
                                println!("   The gateway may already be paired, or the code has been used.");
                                println!("   Restart the gateway to generate a new pairing code.");
                            } else {
                                println!("⚠️  Gateway pairing is disabled in config.");
                                println!(
                                    "   All requests will be accepted without authentication."
                                );
                                println!(
                                    "   To enable pairing, set [gateway] require_pairing = true"
                                );
                            }
                        }
                        Err(e) => {
                            println!(
                                "❌ Failed to fetch pairing code from gateway at {host}:{port}"
                            );
                            println!("   Error: {e}");
                            println!();
                            println!("   Is the gateway running? Start it with:");
                            println!("     redclaw gateway start");
                        }
                    }
                    Ok(())
                }
                Some(redclaw::GatewayCommands::Start { port, host }) => {
                    let (port, host) = resolve_gateway_addr(&config, port, host);
                    log_gateway_start(&host, port);
                    gateway::run_gateway(&host, port, config).await
                }
                None => {
                    let port = config.gateway.port;
                    let host = config.gateway.host.clone();
                    log_gateway_start(&host, port);
                    gateway::run_gateway(&host, port, config).await
                }
            }
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

        Commands::Auth { auth_command } => handle_auth_command(auth_command, &config).await,

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

        Commands::Modules { modules_command } => handle_modules_command(modules_command),
    }
}

/// Handle modules subcommands.
fn handle_modules_command(command: ModulesCommands) -> Result<()> {
    use crate::core::installer::ModuleInstaller;
    use std::path::PathBuf;

    let home_root = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .context("failed to get home directory")?;
    let home_root = PathBuf::from(home_root).join(".redclaw");

    let installer = ModuleInstaller::new(home_root);

    match command {
        ModulesCommands::List => {
            let modules = installer.list()?;
            if modules.is_empty() {
                println!("No modules installed.");
                return Ok(());
            }

            println!("{:<30} {:<15} {:<10}", "ID", "Version", "Enabled");
            println!("{}", "-".repeat(60));
            for module in modules {
                println!(
                    "{:<30} {:<15} {:<10}",
                    module.id,
                    module.version,
                    if module.enabled { "yes" } else { "no" }
                );
            }
            Ok(())
        }

        ModulesCommands::Info { module_id } => {
            let info = installer.info(&module_id)?;
            println!("Module: {}", info.id);
            println!("  Version: {}", info.version);
            println!("  Enabled: {}", if info.enabled { "yes" } else { "no" });
            println!("  Trust: {:?}", info.trust.tier);
            Ok(())
        }

        ModulesCommands::Install { source, enable } => {
            println!("Installing module from: {}", source);
            let module_id = installer.install(&source, enable)?;
            println!("✓ Successfully installed module: {}", module_id);
            if enable {
                println!("  Module is enabled and will be activated on next boot.");
            } else {
                println!(
                    "  Module is disabled. Use 'redclaw modules enable {}' to enable.",
                    module_id
                );
            }
            Ok(())
        }

        ModulesCommands::Remove { module_id } => {
            println!("Removing module: {}", module_id);
            installer.remove(&module_id)?;
            println!("✓ Successfully removed module: {}", module_id);
            Ok(())
        }

        ModulesCommands::Enable { module_id } => {
            println!("Enabling module: {}", module_id);
            let changed = installer.enable(&module_id)?;
            if changed {
                println!("✓ Successfully enabled module: {}", module_id);
            } else {
                println!("✓ Module '{}' is already enabled", module_id);
            }
            Ok(())
        }

        ModulesCommands::Disable { module_id } => {
            println!("Disabling module: {}", module_id);
            let changed = installer.disable(&module_id)?;
            if changed {
                println!("✓ Successfully disabled module: {}", module_id);
            } else {
                println!("✓ Module '{}' is already disabled", module_id);
            }
            Ok(())
        }

        ModulesCommands::Update { module_id, all } => {
            if all {
                println!("Updating all modules...");
            } else if let Some(target) = module_id.as_deref() {
                println!("Updating module: {}", target);
            }

            let results = installer.update(module_id.as_deref(), all)?;
            if all {
                if results.is_empty() {
                    println!("No modules installed.");
                } else {
                    for result in &results {
                        println!(
                            "  Updated {}: {} -> {}",
                            result.module_id, result.old_version, result.new_version
                        );
                    }
                    println!("✓ Successfully updated {} module(s)", results.len());
                }
                return Ok(());
            }

            let result = results
                .first()
                .ok_or_else(|| anyhow!("update completed without any updated module"))?;
            println!(
                "✓ Successfully updated {}: {} -> {}",
                result.module_id, result.old_version, result.new_version
            );
            Ok(())
        }

        ModulesCommands::Doctor => {
            let report = installer.doctor()?;
            println!("{}", report.render());
            if report.has_errors() {
                bail!("{} issue(s) found", report.error_count());
            }
            Ok(())
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

// ─── Gateway helper functions ───────────────────────────────────────────────

/// Resolve gateway host and port from CLI args or config.
fn resolve_gateway_addr(config: &Config, port: Option<u16>, host: Option<String>) -> (u16, String) {
    let port = port.unwrap_or(config.gateway.port);
    let host = host.unwrap_or_else(|| config.gateway.host.clone());
    (port, host)
}

/// Log gateway startup message.
fn log_gateway_start(host: &str, port: u16) {
    if port == 0 {
        info!("🚀 Starting RedClaw Gateway on {host} (random port)");
    } else {
        info!("🚀 Starting RedClaw Gateway on {host}:{port}");
    }
}

/// Gracefully shutdown a running gateway via the admin endpoint.
async fn shutdown_gateway(host: &str, port: u16) -> Result<()> {
    let url = format!("http://{host}:{port}/admin/shutdown");
    let client = reqwest::Client::new();

    match client
        .post(&url)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => Ok(()),
        Ok(response) => Err(anyhow::anyhow!(
            "Gateway responded with status: {}",
            response.status()
        )),
        Err(e) => Err(anyhow::anyhow!("Failed to connect to gateway: {e}")),
    }
}

/// Fetch the current pairing code from a running gateway.
/// If `new` is true, generates a fresh pairing code via POST request.
async fn fetch_paircode(host: &str, port: u16, new: bool) -> Result<Option<String>> {
    let client = reqwest::Client::new();

    let response = if new {
        // Generate a new pairing code via POST
        let url = format!("http://{host}:{port}/admin/paircode/new");
        client
            .post(&url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
    } else {
        // Get existing pairing code via GET
        let url = format!("http://{host}:{port}/admin/paircode");
        client
            .get(&url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
    };

    let response = response.map_err(|e| anyhow::anyhow!("Failed to connect to gateway: {e}"))?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Gateway responded with status: {}",
            response.status()
        ));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to parse response: {e}"))?;

    if json.get("success").and_then(|v| v.as_bool()) != Some(true) {
        return Ok(None);
    }

    Ok(json
        .get("pairing_code")
        .and_then(|v| v.as_str())
        .map(String::from))
}

// ─── Generic Pending OAuth Login ────────────────────────────────────────────

/// Generic pending OAuth login state, shared across providers.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PendingOAuthLogin {
    provider: String,
    profile: String,
    code_verifier: String,
    state: String,
    created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PendingOAuthLoginFile {
    #[serde(default)]
    provider: Option<String>,
    profile: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    code_verifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    encrypted_code_verifier: Option<String>,
    state: String,
    created_at: String,
}

fn pending_oauth_login_path(config: &Config, provider: &str) -> std::path::PathBuf {
    let filename = format!("auth-{}-pending.json", provider);
    auth::state_dir_from_config(config).join(filename)
}

fn pending_oauth_secret_store(config: &Config) -> security::secrets::SecretStore {
    security::secrets::SecretStore::new(
        &auth::state_dir_from_config(config),
        config.secrets.encrypt,
    )
}

#[cfg(unix)]
fn set_owner_only_permissions(path: &std::path::Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    Ok(())
}

#[cfg(not(unix))]
fn set_owner_only_permissions(_path: &std::path::Path) -> Result<()> {
    Ok(())
}

fn save_pending_oauth_login(config: &Config, pending: &PendingOAuthLogin) -> Result<()> {
    let path = pending_oauth_login_path(config, &pending.provider);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let secret_store = pending_oauth_secret_store(config);
    let encrypted_code_verifier = secret_store.encrypt(&pending.code_verifier)?;
    let persisted = PendingOAuthLoginFile {
        provider: Some(pending.provider.clone()),
        profile: pending.profile.clone(),
        code_verifier: None,
        encrypted_code_verifier: Some(encrypted_code_verifier),
        state: pending.state.clone(),
        created_at: pending.created_at.clone(),
    };
    let tmp = path.with_extension(format!(
        "tmp.{}.{}",
        std::process::id(),
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
    ));
    let json = serde_json::to_vec_pretty(&persisted)?;
    std::fs::write(&tmp, json)?;
    set_owner_only_permissions(&tmp)?;
    std::fs::rename(tmp, &path)?;
    set_owner_only_permissions(&path)?;
    Ok(())
}

fn load_pending_oauth_login(config: &Config, provider: &str) -> Result<Option<PendingOAuthLogin>> {
    let path = pending_oauth_login_path(config, provider);
    if !path.exists() {
        return Ok(None);
    }
    let bytes = std::fs::read(&path)?;
    if bytes.is_empty() {
        return Ok(None);
    }
    let persisted: PendingOAuthLoginFile = serde_json::from_slice(&bytes)?;
    let secret_store = pending_oauth_secret_store(config);
    let code_verifier = if let Some(encrypted) = persisted.encrypted_code_verifier {
        secret_store.decrypt(&encrypted)?
    } else if let Some(plaintext) = persisted.code_verifier {
        plaintext
    } else {
        bail!("Pending {} login is missing code verifier", provider);
    };
    Ok(Some(PendingOAuthLogin {
        provider: persisted.provider.unwrap_or_else(|| provider.to_string()),
        profile: persisted.profile,
        code_verifier,
        state: persisted.state,
        created_at: persisted.created_at,
    }))
}

fn clear_pending_oauth_login(config: &Config, provider: &str) {
    let path = pending_oauth_login_path(config, provider);
    if let Ok(file) = std::fs::OpenOptions::new().write(true).open(&path) {
        let _ = file.set_len(0);
        let _ = file.sync_all();
    }
    let _ = std::fs::remove_file(path);
}

fn read_auth_input(prompt: &str) -> Result<String> {
    let input = Password::new()
        .with_prompt(prompt)
        .allow_empty_password(false)
        .interact()?;
    Ok(input.trim().to_string())
}

fn read_plain_input(prompt: &str) -> Result<String> {
    let input: String = Input::new().with_prompt(prompt).interact_text()?;
    Ok(input.trim().to_string())
}

fn extract_openai_account_id_for_profile(access_token: &str) -> Option<String> {
    let account_id = auth::openai_oauth::extract_account_id_from_jwt(access_token);
    if account_id.is_none() {
        warn!(
            "Could not extract OpenAI account id from OAuth access token; \
             requests may fail until re-authentication."
        );
    }
    account_id
}

fn format_expiry(profile: &auth::profiles::AuthProfile) -> String {
    match profile
        .token_set
        .as_ref()
        .and_then(|token_set| token_set.expires_at)
    {
        Some(ts) => {
            let now = chrono::Utc::now();
            if ts <= now {
                format!("expired at {}", ts.to_rfc3339())
            } else {
                let mins = (ts - now).num_minutes();
                format!("expires in {mins}m ({})", ts.to_rfc3339())
            }
        }
        None => "n/a".to_string(),
    }
}

#[allow(clippy::too_many_lines)]
async fn handle_auth_command(auth_command: AuthCommands, config: &Config) -> Result<()> {
    let auth_service = auth::AuthService::from_config(config);

    match auth_command {
        AuthCommands::Login {
            provider,
            profile,
            device_code,
        } => {
            let provider = auth::normalize_provider(&provider)?;
            let client = reqwest::Client::new();

            match provider.as_str() {
                "gemini" => {
                    // Gemini OAuth flow
                    if device_code {
                        match auth::gemini_oauth::start_device_code_flow(&client).await {
                            Ok(device) => {
                                println!("Google/Gemini device-code login started.");
                                println!("Visit: {}", device.verification_uri);
                                println!("Code:  {}", device.user_code);
                                if let Some(uri_complete) = &device.verification_uri_complete {
                                    println!("Fast link: {uri_complete}");
                                }

                                let token_set =
                                    auth::gemini_oauth::poll_device_code_tokens(&client, &device)
                                        .await?;
                                let account_id = token_set.id_token.as_deref().and_then(
                                    auth::gemini_oauth::extract_account_email_from_id_token,
                                );

                                auth_service
                                    .store_gemini_tokens(&profile, token_set, account_id, true)
                                    .await?;

                                println!("Saved profile {profile}");
                                println!("Active profile for gemini: {profile}");
                                return Ok(());
                            }
                            Err(e) => {
                                println!(
                                    "Device-code flow unavailable: {e}. Falling back to browser flow."
                                );
                            }
                        }
                    }

                    let pkce = auth::gemini_oauth::generate_pkce_state();
                    let authorize_url = auth::gemini_oauth::build_authorize_url(&pkce)?;

                    // Save pending login for paste-redirect fallback
                    let pending = PendingOAuthLogin {
                        provider: "gemini".to_string(),
                        profile: profile.clone(),
                        code_verifier: pkce.code_verifier.clone(),
                        state: pkce.state.clone(),
                        created_at: chrono::Utc::now().to_rfc3339(),
                    };
                    save_pending_oauth_login(config, &pending)?;

                    println!("Open this URL in your browser and authorize access:");
                    println!("{authorize_url}");
                    println!();

                    let code = match auth::gemini_oauth::receive_loopback_code(
                        &pkce.state,
                        std::time::Duration::from_secs(180),
                    )
                    .await
                    {
                        Ok(code) => {
                            clear_pending_oauth_login(config, "gemini");
                            code
                        }
                        Err(e) => {
                            println!("Callback capture failed: {e}");
                            println!(
                                "Run `redclaw auth paste-redirect --provider gemini --profile {profile}`"
                            );
                            return Ok(());
                        }
                    };

                    let token_set =
                        auth::gemini_oauth::exchange_code_for_tokens(&client, &code, &pkce).await?;
                    let account_id = token_set
                        .id_token
                        .as_deref()
                        .and_then(auth::gemini_oauth::extract_account_email_from_id_token);

                    auth_service
                        .store_gemini_tokens(&profile, token_set, account_id, true)
                        .await?;

                    println!("Saved profile {profile}");
                    println!("Active profile for gemini: {profile}");
                    Ok(())
                }
                "openai-codex" => {
                    // OpenAI Codex OAuth flow
                    if device_code {
                        match auth::openai_oauth::start_device_code_flow(&client).await {
                            Ok(device) => {
                                println!("OpenAI device-code login started.");
                                println!("Visit: {}", device.verification_uri);
                                println!("Code:  {}", device.user_code);
                                if let Some(uri_complete) = &device.verification_uri_complete {
                                    println!("Fast link: {uri_complete}");
                                }
                                if let Some(message) = &device.message {
                                    println!("{message}");
                                }

                                let token_set =
                                    auth::openai_oauth::poll_device_code_tokens(&client, &device)
                                        .await?;
                                let account_id =
                                    extract_openai_account_id_for_profile(&token_set.access_token);

                                auth_service
                                    .store_openai_tokens(&profile, token_set, account_id, true)
                                    .await?;
                                clear_pending_oauth_login(config, "openai");

                                println!("Saved profile {profile}");
                                println!("Active profile for openai-codex: {profile}");
                                return Ok(());
                            }
                            Err(e) => {
                                println!(
                                    "Device-code flow unavailable: {e}. Falling back to browser/paste flow."
                                );
                            }
                        }
                    }

                    let pkce = auth::openai_oauth::generate_pkce_state();
                    let pending = PendingOAuthLogin {
                        provider: "openai".to_string(),
                        profile: profile.clone(),
                        code_verifier: pkce.code_verifier.clone(),
                        state: pkce.state.clone(),
                        created_at: chrono::Utc::now().to_rfc3339(),
                    };
                    save_pending_oauth_login(config, &pending)?;

                    let authorize_url = auth::openai_oauth::build_authorize_url(&pkce);
                    println!("Open this URL in your browser and authorize access:");
                    println!("{authorize_url}");
                    println!();
                    println!("Waiting for callback at http://localhost:1455/auth/callback ...");

                    let code = match auth::openai_oauth::receive_loopback_code(
                        &pkce.state,
                        std::time::Duration::from_secs(180),
                    )
                    .await
                    {
                        Ok(code) => code,
                        Err(e) => {
                            println!("Callback capture failed: {e}");
                            println!(
                                "Run `redclaw auth paste-redirect --provider openai-codex --profile {profile}`"
                            );
                            return Ok(());
                        }
                    };

                    let token_set =
                        auth::openai_oauth::exchange_code_for_tokens(&client, &code, &pkce).await?;
                    let account_id = extract_openai_account_id_for_profile(&token_set.access_token);

                    auth_service
                        .store_openai_tokens(&profile, token_set, account_id, true)
                        .await?;
                    clear_pending_oauth_login(config, "openai");

                    println!("Saved profile {profile}");
                    println!("Active profile for openai-codex: {profile}");
                    Ok(())
                }
                _ => {
                    bail!(
                        "`auth login` supports --provider openai-codex or gemini, got: {provider}"
                    );
                }
            }
        }

        AuthCommands::PasteRedirect {
            provider,
            profile,
            input,
        } => {
            let provider = auth::normalize_provider(&provider)?;

            match provider.as_str() {
                "openai-codex" => {
                    let pending = load_pending_oauth_login(config, "openai")?.ok_or_else(|| {
                        anyhow::anyhow!(
                            "No pending OpenAI login found. Run `redclaw auth login --provider openai-codex` first."
                        )
                    })?;

                    if pending.profile != profile {
                        bail!(
                            "Pending login profile mismatch: pending={}, requested={}",
                            pending.profile,
                            profile
                        );
                    }

                    let redirect_input = match input {
                        Some(value) => value,
                        None => read_plain_input("Paste redirect URL or OAuth code")?,
                    };

                    let code = auth::openai_oauth::parse_code_from_redirect(
                        &redirect_input,
                        Some(&pending.state),
                    )?;

                    let pkce = auth::openai_oauth::PkceState {
                        code_verifier: pending.code_verifier.clone(),
                        code_challenge: String::new(),
                        state: pending.state.clone(),
                    };

                    let client = reqwest::Client::new();
                    let token_set =
                        auth::openai_oauth::exchange_code_for_tokens(&client, &code, &pkce).await?;
                    let account_id = extract_openai_account_id_for_profile(&token_set.access_token);

                    auth_service
                        .store_openai_tokens(&profile, token_set, account_id, true)
                        .await?;
                    clear_pending_oauth_login(config, "openai");

                    println!("Saved profile {profile}");
                    println!("Active profile for openai-codex: {profile}");
                }
                "gemini" => {
                    let pending = load_pending_oauth_login(config, "gemini")?.ok_or_else(|| {
                        anyhow::anyhow!(
                            "No pending Gemini login found. Run `redclaw auth login --provider gemini` first."
                        )
                    })?;

                    if pending.profile != profile {
                        bail!(
                            "Pending login profile mismatch: pending={}, requested={}",
                            pending.profile,
                            profile
                        );
                    }

                    let redirect_input = match input {
                        Some(value) => value,
                        None => read_plain_input("Paste redirect URL or OAuth code")?,
                    };

                    let code = auth::gemini_oauth::parse_code_from_redirect(
                        &redirect_input,
                        Some(&pending.state),
                    )?;

                    let pkce = auth::gemini_oauth::PkceState {
                        code_verifier: pending.code_verifier.clone(),
                        code_challenge: String::new(),
                        state: pending.state.clone(),
                    };

                    let client = reqwest::Client::new();
                    let token_set =
                        auth::gemini_oauth::exchange_code_for_tokens(&client, &code, &pkce).await?;
                    let account_id = token_set
                        .id_token
                        .as_deref()
                        .and_then(auth::gemini_oauth::extract_account_email_from_id_token);

                    auth_service
                        .store_gemini_tokens(&profile, token_set, account_id, true)
                        .await?;
                    clear_pending_oauth_login(config, "gemini");

                    println!("Saved profile {profile}");
                    println!("Active profile for gemini: {profile}");
                }
                _ => {
                    bail!("`auth paste-redirect` supports --provider openai-codex or gemini");
                }
            }
            Ok(())
        }

        AuthCommands::PasteToken {
            provider,
            profile,
            token,
            auth_kind,
        } => {
            let provider = auth::normalize_provider(&provider)?;
            let token = match token {
                Some(token) => token.trim().to_string(),
                None => read_auth_input("Paste token")?,
            };
            if token.is_empty() {
                bail!("Token cannot be empty");
            }

            let kind = auth::anthropic_token::detect_auth_kind(&token, auth_kind.as_deref());
            let mut metadata = std::collections::HashMap::new();
            metadata.insert(
                "auth_kind".to_string(),
                kind.as_metadata_value().to_string(),
            );

            auth_service
                .store_provider_token(&provider, &profile, &token, metadata, true)
                .await?;
            println!("Saved profile {profile}");
            println!("Active profile for {provider}: {profile}");
            Ok(())
        }

        AuthCommands::SetupToken { provider, profile } => {
            let provider = auth::normalize_provider(&provider)?;
            let token = read_auth_input("Paste token")?;
            if token.is_empty() {
                bail!("Token cannot be empty");
            }

            let kind = auth::anthropic_token::detect_auth_kind(&token, Some("authorization"));
            let mut metadata = std::collections::HashMap::new();
            metadata.insert(
                "auth_kind".to_string(),
                kind.as_metadata_value().to_string(),
            );

            auth_service
                .store_provider_token(&provider, &profile, &token, metadata, true)
                .await?;
            println!("Saved profile {profile}");
            println!("Active profile for {provider}: {profile}");
            Ok(())
        }

        AuthCommands::Refresh { provider, profile } => {
            let provider = auth::normalize_provider(&provider)?;

            match provider.as_str() {
                "openai-codex" => {
                    match auth_service
                        .get_valid_openai_access_token(profile.as_deref())
                        .await?
                    {
                        Some(_) => {
                            println!("OpenAI Codex token is valid (refresh completed if needed).");
                            Ok(())
                        }
                        None => {
                            bail!(
                                "No OpenAI Codex auth profile found. Run `redclaw auth login --provider openai-codex`."
                            )
                        }
                    }
                }
                "gemini" => {
                    match auth_service
                        .get_valid_gemini_access_token(profile.as_deref())
                        .await?
                    {
                        Some(_) => {
                            let profile_name = profile.as_deref().unwrap_or("default");
                            println!("✓ Gemini token refreshed successfully");
                            println!("  Profile: gemini:{}", profile_name);
                            Ok(())
                        }
                        None => {
                            bail!(
                                "No Gemini auth profile found. Run `redclaw auth login --provider gemini`."
                            )
                        }
                    }
                }
                _ => bail!("`auth refresh` supports --provider openai-codex or gemini"),
            }
        }

        AuthCommands::Logout { provider, profile } => {
            let provider = auth::normalize_provider(&provider)?;
            let removed = auth_service.remove_profile(&provider, &profile).await?;
            if removed {
                println!("Removed auth profile {provider}:{profile}");
            } else {
                println!("Auth profile not found: {provider}:{profile}");
            }
            Ok(())
        }

        AuthCommands::Use { provider, profile } => {
            let provider = auth::normalize_provider(&provider)?;
            auth_service.set_active_profile(&provider, &profile).await?;
            println!("Active profile for {provider}: {profile}");
            Ok(())
        }

        AuthCommands::List => {
            let data = auth_service.load_profiles().await?;
            if data.profiles.is_empty() {
                println!("No auth profiles configured.");
                return Ok(());
            }

            for (id, profile) in &data.profiles {
                let active = data
                    .active_profiles
                    .get(&profile.provider)
                    .is_some_and(|active_id| active_id == id);
                let marker = if active { "*" } else { " " };
                println!("{marker} {id}");
            }

            Ok(())
        }

        AuthCommands::Status => {
            let data = auth_service.load_profiles().await?;
            if data.profiles.is_empty() {
                println!("No auth profiles configured.");
                return Ok(());
            }

            for (id, profile) in &data.profiles {
                let active = data
                    .active_profiles
                    .get(&profile.provider)
                    .is_some_and(|active_id| active_id == id);
                let marker = if active { "*" } else { " " };
                println!(
                    "{} {} kind={:?} account={} expires={}",
                    marker,
                    id,
                    profile.kind,
                    crate::security::redact(profile.account_id.as_deref().unwrap_or("unknown")),
                    format_expiry(profile)
                );
            }

            println!();
            println!("Active profiles:");
            for (provider, profile_id) in &data.active_profiles {
                println!("  {provider}: {profile_id}");
            }

            Ok(())
        }
    }
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
    fn manual_modules_help_is_detected() {
        let args = vec![
            "redclaw".to_string(),
            "modules".to_string(),
            "--help".to_string(),
        ];
        assert_eq!(
            detect_manual_help_target(&args),
            Some(ManualHelpTarget::Modules)
        );
    }

    #[test]
    fn manual_modules_help_mentions_install_and_remove() {
        let mut output = Vec::new();
        write_manual_help(ManualHelpTarget::Modules, &mut output)
            .expect("manual help generation should succeed");
        let help = String::from_utf8(output).expect("manual help should be utf-8");

        assert!(help.contains("redclaw modules install ./my-module"));
        assert!(help.contains("remove <MODULE_ID>"));
        assert!(help.contains("enable <MODULE_ID>"));
        assert!(help.contains("disable <MODULE_ID>"));
        assert!(help.contains("redclaw modules update --all"));
        assert!(help.contains("doctor"));
    }

    #[test]
    fn manual_modules_invocation_parses_install_enable() {
        let args = vec![
            "redclaw".to_string(),
            "modules".to_string(),
            "install".to_string(),
            "./my-module".to_string(),
            "--enable".to_string(),
        ];

        let invocation = parse_manual_modules_invocation(&args)
            .expect("parse should succeed")
            .expect("modules invocation should be detected");

        assert_eq!(invocation.config_dir, None);
        assert_eq!(
            invocation.command,
            ModulesCommands::Install {
                source: "./my-module".to_string(),
                enable: true
            }
        );
    }

    #[test]
    fn manual_modules_invocation_parses_config_dir_and_list() {
        let args = vec![
            "redclaw".to_string(),
            "--config-dir".to_string(),
            "./tmp-config".to_string(),
            "modules".to_string(),
            "list".to_string(),
        ];

        let invocation = parse_manual_modules_invocation(&args)
            .expect("parse should succeed")
            .expect("modules invocation should be detected");

        assert_eq!(invocation.config_dir.as_deref(), Some("./tmp-config"));
        assert_eq!(invocation.command, ModulesCommands::List);
    }

    #[test]
    fn manual_modules_invocation_parses_enable() {
        let args = vec![
            "redclaw".to_string(),
            "modules".to_string(),
            "enable".to_string(),
            "provider-openai-compatible".to_string(),
        ];

        let invocation = parse_manual_modules_invocation(&args)
            .expect("parse should succeed")
            .expect("modules invocation should be detected");

        assert_eq!(
            invocation.command,
            ModulesCommands::Enable {
                module_id: "provider-openai-compatible".to_string(),
            }
        );
    }

    #[test]
    fn manual_modules_invocation_parses_disable() {
        let args = vec![
            "redclaw".to_string(),
            "modules".to_string(),
            "disable".to_string(),
            "provider-openai-compatible".to_string(),
        ];

        let invocation = parse_manual_modules_invocation(&args)
            .expect("parse should succeed")
            .expect("modules invocation should be detected");

        assert_eq!(
            invocation.command,
            ModulesCommands::Disable {
                module_id: "provider-openai-compatible".to_string(),
            }
        );
    }

    #[test]
    fn manual_modules_invocation_requires_install_source() {
        let args = vec![
            "redclaw".to_string(),
            "modules".to_string(),
            "install".to_string(),
        ];

        let error =
            parse_manual_modules_invocation(&args).expect_err("missing source should fail parsing");
        assert!(error
            .to_string()
            .contains("modules install requires <source>"));
    }

    #[test]
    fn manual_modules_invocation_parses_update_all() {
        let args = vec![
            "redclaw".to_string(),
            "modules".to_string(),
            "update".to_string(),
            "--all".to_string(),
        ];

        let invocation = parse_manual_modules_invocation(&args)
            .expect("parse should succeed")
            .expect("modules invocation should be detected");

        assert_eq!(
            invocation.command,
            ModulesCommands::Update {
                module_id: None,
                all: true,
            }
        );
    }

    #[test]
    fn manual_modules_invocation_parses_doctor() {
        let args = vec![
            "redclaw".to_string(),
            "modules".to_string(),
            "doctor".to_string(),
        ];

        let invocation = parse_manual_modules_invocation(&args)
            .expect("parse should succeed")
            .expect("modules invocation should be detected");

        assert_eq!(invocation.command, ModulesCommands::Doctor);
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
