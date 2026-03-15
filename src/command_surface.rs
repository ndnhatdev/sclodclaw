use clap::Subcommand;
use serde::{Deserialize, Serialize};

/// Gateway management subcommands
#[derive(Subcommand, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GatewayCommands {
    /// Start the gateway server (default if no subcommand specified)
    #[command(long_about = "\
Start the gateway server (webhooks, websockets).

Runs the HTTP/WebSocket gateway that accepts incoming webhook events \
and WebSocket connections. Bind address defaults to the values in \
your config file (gateway.host / gateway.port).

Examples:
  redclaw gateway start              # use config defaults
  redclaw gateway start -p 8080      # listen on port 8080
  redclaw gateway start --host 0.0.0.0   # requires [gateway].allow_public_bind=true or a tunnel
  redclaw gateway start -p 0         # random available port")]
    Start {
        /// Port to listen on (use 0 for random available port); defaults to config gateway.port
        #[arg(short, long)]
        port: Option<u16>,

        /// Host to bind to; defaults to config gateway.host
        /// Note: Binding to 0.0.0.0 requires `gateway.allow_public_bind = true` in config
        #[arg(long)]
        host: Option<String>,
    },
    /// Restart the gateway server
    #[command(long_about = "\
Restart the gateway server.

Stops the running gateway if present, then starts a new instance \
with the current configuration.

Examples:
  redclaw gateway restart            # restart with config defaults
  redclaw gateway restart -p 8080    # restart on port 8080")]
    Restart {
        /// Port to listen on (use 0 for random available port); defaults to config gateway.port
        #[arg(short, long)]
        port: Option<u16>,

        /// Host to bind to; defaults to config gateway.host
        /// Note: Binding to 0.0.0.0 requires `gateway.allow_public_bind = true` in config
        #[arg(long)]
        host: Option<String>,
    },
    /// Show or generate the pairing code without restarting
    #[command(long_about = "\
Show or generate the gateway pairing code.

Displays the pairing code for connecting new clients without \
restarting the gateway. Requires the gateway to be running.

With --new, generates a fresh pairing code even if the gateway \
was previously paired (useful for adding additional clients).

Examples:
  redclaw gateway get-paircode       # show current pairing code
  redclaw gateway get-paircode --new # generate a new pairing code")]
    GetPaircode {
        /// Generate a new pairing code (even if already paired)
        #[arg(long)]
        new: bool,
    },
}

/// Service management subcommands
#[derive(Subcommand, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServiceCommands {
    /// Install daemon service unit for auto-start and restart
    Install,
    /// Start daemon service
    Start,
    /// Stop daemon service
    Stop,
    /// Restart daemon service to apply latest config
    Restart,
    /// Check daemon service status
    Status,
    /// Uninstall daemon service unit
    Uninstall,
}

/// Channel management subcommands
#[derive(Subcommand, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChannelCommands {
    /// List all configured channels
    List,
    /// Start all configured channels (handled in main.rs for async)
    Start,
    /// Run health checks for configured channels (handled in main.rs for async)
    Doctor,
    /// Add a new channel configuration
    #[command(long_about = "\
Add a new channel configuration.

Provide the channel type and a JSON object with the required \
configuration keys for that channel type.

Supported types: telegram, discord, slack, whatsapp, matrix, imessage, email.

Examples:
  redclaw channel add telegram '{\"bot_token\":\"...\",\"name\":\"my-bot\"}'
  redclaw channel add discord '{\"bot_token\":\"...\",\"name\":\"my-discord\"}'")]
    Add {
        /// Channel type (telegram, discord, slack, whatsapp, matrix, imessage, email)
        channel_type: String,
        /// Optional configuration as JSON
        config: String,
    },
    /// Remove a channel configuration
    Remove {
        /// Channel name to remove
        name: String,
    },
    /// Bind a Telegram identity (username or numeric user ID) into allowlist
    #[command(long_about = "\
Bind a Telegram identity into the allowlist.

Adds a Telegram username (without the '@' prefix) or numeric user \
ID to the channel allowlist so the agent will respond to messages \
from that identity.

Examples:
redclaw channel bind-telegram redclaw_user
  redclaw channel bind-telegram 123456789")]
    BindTelegram {
        /// Telegram identity to allow (username without '@' or numeric user ID)
        identity: String,
    },
    /// Send a message to a configured channel
    #[command(long_about = "\
Send a one-off message to a configured channel.

Sends a text message through the specified channel without starting \
the full agent loop. Useful for scripted notifications, hardware \
sensor alerts, and automation pipelines.

The --channel-id selects the channel by its config section name \
(e.g. 'telegram', 'discord', 'slack'). The --recipient is the \
platform-specific destination (e.g. a Telegram chat ID).

Examples:
  redclaw channel send 'Someone is near your device.' --channel-id telegram --recipient 123456789
  redclaw channel send 'Build succeeded!' --channel-id discord --recipient 987654321")]
    Send {
        /// Message text to send
        message: String,
        /// Channel config name (e.g. telegram, discord, slack)
        #[arg(long)]
        channel_id: String,
        /// Recipient identifier (platform-specific, e.g. Telegram chat ID)
        #[arg(long)]
        recipient: String,
    },
}

/// Skills management subcommands
#[derive(Subcommand, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SkillCommands {
    /// List all installed skills
    List,
    /// Audit a skill source directory or installed skill name
    Audit {
        /// Skill path or installed skill name
        source: String,
    },
    /// Install a new skill from a URL or local path
    Install {
        /// Source URL or local path
        source: String,
    },
    /// Remove an installed skill
    Remove {
        /// Skill name to remove
        name: String,
    },
}

/// Migration subcommands
#[derive(Subcommand, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MigrateCommands {
    /// Import memory from an `OpenClaw` workspace into this `RedClaw` workspace
    Openclaw {
        /// Optional path to `OpenClaw` workspace (defaults to ~/.openclaw/workspace)
        #[arg(long)]
        source: Option<std::path::PathBuf>,

        /// Validate and preview migration without writing any data
        #[arg(long)]
        dry_run: bool,
    },
}

/// Cron subcommands
#[derive(Subcommand, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CronCommands {
    /// List all scheduled tasks
    List,
    /// Add a new scheduled task
    #[command(long_about = "\
Add a new recurring scheduled task.

Uses standard 5-field cron syntax: 'min hour day month weekday'. \
Times are evaluated in UTC by default; use --tz with an IANA \
timezone name to override.

Examples:
  redclaw cron add '0 9 * * 1-5' 'Good morning' --tz America/New_York
  redclaw cron add '*/30 * * * *' 'Check system health'")]
    Add {
        /// Cron expression
        expression: String,
        /// Optional IANA timezone (e.g. America/Los_Angeles)
        #[arg(long)]
        tz: Option<String>,
        /// Command to run
        command: String,
    },
    /// Add a one-shot scheduled task at an RFC3339 timestamp
    #[command(long_about = "\
Add a one-shot task that fires at a specific UTC timestamp.

The timestamp must be in RFC 3339 format (e.g. 2025-01-15T14:00:00Z).

Examples:
  redclaw cron add-at 2025-01-15T14:00:00Z 'Send reminder'
  redclaw cron add-at 2025-12-31T23:59:00Z 'Happy New Year!'")]
    AddAt {
        /// One-shot timestamp in RFC3339 format
        at: String,
        /// Command to run
        command: String,
    },
    /// Add a fixed-interval scheduled task
    #[command(long_about = "\
Add a task that repeats at a fixed interval.

Interval is specified in milliseconds. For example, 60000 = 1 minute.

Examples:
  redclaw cron add-every 60000 'Ping heartbeat'     # every minute
  redclaw cron add-every 3600000 'Hourly report'    # every hour")]
    AddEvery {
        /// Interval in milliseconds
        every_ms: u64,
        /// Command to run
        command: String,
    },
    /// Add a one-shot delayed task (e.g. "30m", "2h", "1d")
    #[command(long_about = "\
Add a one-shot task that fires after a delay from now.

Accepts human-readable durations: s (seconds), m (minutes), \
h (hours), d (days).

Examples:
  redclaw cron once 30m 'Run backup in 30 minutes'
  redclaw cron once 2h 'Follow up on deployment'
  redclaw cron once 1d 'Daily check'")]
    Once {
        /// Delay duration
        delay: String,
        /// Command to run
        command: String,
    },
    /// Remove a scheduled task
    Remove {
        /// Task ID
        id: String,
    },
    /// Update a scheduled task
    #[command(long_about = "\
Update one or more fields of an existing scheduled task.

Only the fields you specify are changed; others remain unchanged.

Examples:
  redclaw cron update <task-id> --expression '0 8 * * *'
  redclaw cron update <task-id> --tz Europe/London --name 'Morning check'
  redclaw cron update <task-id> --command 'Updated message'")]
    Update {
        /// Task ID
        id: String,
        /// New cron expression
        #[arg(long)]
        expression: Option<String>,
        /// New IANA timezone
        #[arg(long)]
        tz: Option<String>,
        /// New command to run
        #[arg(long)]
        command: Option<String>,
        /// New job name
        #[arg(long)]
        name: Option<String>,
    },
    /// Pause a scheduled task
    Pause {
        /// Task ID
        id: String,
    },
    /// Resume a paused task
    Resume {
        /// Task ID
        id: String,
    },
}

/// Memory management subcommands
#[derive(Subcommand, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MemoryCommands {
    /// List memory entries with optional filters
    List {
        /// Filter by category (core, daily, conversation, or custom name)
        #[arg(long)]
        category: Option<String>,
        /// Filter by session ID
        #[arg(long)]
        session: Option<String>,
        /// Maximum number of entries to display
        #[arg(long, default_value = "50")]
        limit: usize,
        /// Number of entries to skip (for pagination)
        #[arg(long, default_value = "0")]
        offset: usize,
    },
    /// Get a specific memory entry by key
    Get {
        /// Memory key to look up
        key: String,
    },
    /// Show memory backend statistics and health
    Stats,
    /// Clear memories by category, by key, or clear all
    Clear {
        /// Delete a single entry by key (supports prefix match)
        #[arg(long)]
        key: Option<String>,
        /// Only clear entries in this category
        #[arg(long)]
        category: Option<String>,
        /// Skip confirmation prompt
        #[arg(long)]
        yes: bool,
    },
}

/// Integration subcommands
#[derive(Subcommand, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IntegrationCommands {
    /// Show details about a specific integration
    Info {
        /// Integration name
        name: String,
    },
}

/// Hardware discovery subcommands
#[derive(Subcommand, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HardwareCommands {
    /// Enumerate USB devices (VID/PID) and show known boards
    #[command(long_about = "\
Enumerate USB devices and show known boards.

Scans connected USB devices by VID/PID and matches them against \
known development boards (STM32 Nucleo, Arduino, ESP32).

Examples:
  redclaw hardware discover")]
    Discover,
    /// Introspect a device by path (e.g. /dev/ttyACM0)
    #[command(long_about = "\
Introspect a device by its serial or device path.

Opens the specified device path and queries for board information, \
firmware version, and supported capabilities.

Examples:
  redclaw hardware introspect /dev/ttyACM0
  redclaw hardware introspect COM3")]
    Introspect {
        /// Serial or device path
        path: String,
    },
    /// Get chip info via USB (probe-rs over ST-Link). No firmware needed on target.
    #[command(long_about = "\
Get chip info via USB using probe-rs over ST-Link.

Queries the target MCU directly through the debug probe without \
requiring any firmware on the target board.

Examples:
  redclaw hardware info
  redclaw hardware info --chip STM32F401RETx")]
    Info {
        /// Chip name (e.g. STM32F401RETx). Default: STM32F401RETx for Nucleo-F401RE
        #[arg(long, default_value = "STM32F401RETx")]
        chip: String,
    },
}

/// Peripheral (hardware) management subcommands
#[derive(Subcommand, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PeripheralCommands {
    /// List configured peripherals
    List,
    /// Add a peripheral (board path, e.g. nucleo-f401re /dev/ttyACM0)
    #[command(long_about = "\
Add a peripheral by board type and transport path.

Registers a hardware board so the agent can use its tools (GPIO, \
sensors, actuators). Use 'native' as path for local GPIO on \
single-board computers like Raspberry Pi.

Supported boards: nucleo-f401re, rpi-gpio, esp32, arduino-uno.

Examples:
  redclaw peripheral add nucleo-f401re /dev/ttyACM0
  redclaw peripheral add rpi-gpio native
  redclaw peripheral add esp32 /dev/ttyUSB0")]
    Add {
        /// Board type (nucleo-f401re, rpi-gpio, esp32)
        board: String,
        /// Path for serial transport (/dev/ttyACM0) or "native" for local GPIO
        path: String,
    },
    /// Flash RedClaw firmware to Arduino (creates .ino, installs arduino-cli if needed, uploads)
    #[command(long_about = "\
Flash RedClaw firmware to an Arduino board.

Generates the .ino sketch, installs arduino-cli if it is not \
already available, compiles, and uploads the firmware.

Examples:
  redclaw peripheral flash
  redclaw peripheral flash --port /dev/cu.usbmodem12345
  redclaw peripheral flash -p COM3")]
    Flash {
        /// Serial port (e.g. /dev/cu.usbmodem12345). If omitted, uses first arduino-uno from config.
        #[arg(short, long)]
        port: Option<String>,
    },
    /// Setup Arduino Uno Q Bridge app (deploy GPIO bridge for agent control)
    SetupUnoQ {
        /// Uno Q IP (e.g. 192.168.0.48). If omitted, assumes running ON the Uno Q.
        #[arg(long)]
        host: Option<String>,
    },
    /// Flash RedClaw firmware to Nucleo-F401RE (builds + probe-rs run)
    FlashNucleo,
}

/// Module management subcommands
#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum ModulesCommands {
    /// List all installed modules
    List,
    /// Show details about a specific module
    Info {
        /// Module ID
        module_id: String,
    },
    /// Install a module from local directory or archive
    Install {
        /// Module source (local directory or archive path)
        source: String,
        /// Enable the module after installation
        #[arg(long)]
        enable: bool,
    },
    /// Remove an installed module
    Remove {
        /// Module ID to remove
        module_id: String,
    },
    /// Enable an installed module
    Enable {
        /// Module ID to enable
        module_id: String,
    },
    /// Disable an installed module
    Disable {
        /// Module ID to disable
        module_id: String,
    },
    /// Update an installed module
    Update {
        /// Module ID to update (omit when using --all)
        module_id: Option<String>,
        /// Update all installed modules
        #[arg(long)]
        all: bool,
    },
    /// Run diagnostics on installed modules
    Doctor,
}
