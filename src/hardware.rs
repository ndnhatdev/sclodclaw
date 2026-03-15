pub use crate::config::{HardwareConfig, HardwareTransport};

#[derive(Debug, Clone)]
pub struct DiscoveredDevice {
    pub name: String,
    pub transport: HardwareTransport,
    pub device_path: Option<String>,
    pub detail: Option<String>,
}

pub fn discover_hardware() -> Vec<DiscoveredDevice> {
    Vec::new()
}

pub fn recommended_wizard_default(devices: &[DiscoveredDevice]) -> usize {
    if devices
        .iter()
        .any(|d| d.transport == HardwareTransport::Native)
    {
        return 0;
    }

    if devices
        .iter()
        .any(|d| d.transport == HardwareTransport::Serial)
    {
        return 1;
    }

    if devices
        .iter()
        .any(|d| d.transport == HardwareTransport::Probe)
    {
        return 2;
    }

    3
}

pub fn config_from_wizard_choice(choice: usize, devices: &[DiscoveredDevice]) -> HardwareConfig {
    match choice {
        0 => HardwareConfig {
            enabled: true,
            transport: HardwareTransport::Native,
            ..HardwareConfig::default()
        },
        1 => {
            let serial_port = devices
                .iter()
                .find(|d| d.transport == HardwareTransport::Serial)
                .and_then(|d| d.device_path.clone());

            HardwareConfig {
                enabled: true,
                transport: HardwareTransport::Serial,
                serial_port,
                ..HardwareConfig::default()
            }
        }
        2 => HardwareConfig {
            enabled: true,
            transport: HardwareTransport::Probe,
            ..HardwareConfig::default()
        },
        _ => HardwareConfig::default(),
    }
}

pub fn handle_command(
    _command: crate::HardwareCommands,
    _config: &crate::config::Config,
) -> anyhow::Result<()> {
    println!("Hardware command surface is scaffolded in this cut.");
    Ok(())
}
