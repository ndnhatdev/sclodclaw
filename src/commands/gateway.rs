use anyhow::Result;
use redclaw::GatewayCommands;
use tracing::{info, warn};

use crate::config::Config;

pub async fn handle_gateway_command(
    gateway_command: Option<GatewayCommands>,
    config: Config,
) -> Result<()> {
    match gateway_command {
        Some(GatewayCommands::Restart { port, host }) => {
            let (port, host) = resolve_gateway_addr(&config, port, host);
            let addr = format!("{host}:{port}");
            info!("🔄 Restarting RedClaw Gateway on {addr}");

            match shutdown_gateway(&host, port).await {
                Ok(()) => {
                    info!("   ✓ Existing gateway on {addr} shut down gracefully");
                    let deadline =
                        tokio::time::Instant::now() + tokio::time::Duration::from_secs(5);
                    loop {
                        match tokio::net::TcpStream::connect(&addr).await {
                            Err(_) => break,
                            Ok(_) if tokio::time::Instant::now() >= deadline => {
                                warn!("   Timed out waiting for port {port} to be released");
                                break;
                            }
                            Ok(_) => {
                                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                            }
                        }
                    }
                }
                Err(e) => {
                    info!("   No existing gateway to shut down: {e}");
                }
            }

            log_gateway_start(&host, port);
            crate::gateway::run_gateway(&host, port, config).await
        }
        Some(GatewayCommands::GetPaircode { new }) => {
            let port = config.gateway.port;
            let host = &config.gateway.host;

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
                        println!(
                            "🔐 Gateway pairing is enabled, but no active pairing code available."
                        );
                        println!(
                            "   The gateway may already be paired, or the code has been used."
                        );
                        println!("   Restart the gateway to generate a new pairing code.");
                    } else {
                        println!("⚠️  Gateway pairing is disabled in config.");
                        println!("   All requests will be accepted without authentication.");
                        println!("   To enable pairing, set [gateway] require_pairing = true");
                    }
                }
                Err(e) => {
                    println!("❌ Failed to fetch pairing code from gateway at {host}:{port}");
                    println!("   Error: {e}");
                    println!();
                    println!("   Is the gateway running? Start it with:");
                    println!("     redclaw gateway start");
                }
            }
            Ok(())
        }
        Some(GatewayCommands::Start { port, host }) => {
            let (port, host) = resolve_gateway_addr(&config, port, host);
            log_gateway_start(&host, port);
            crate::gateway::run_gateway(&host, port, config).await
        }
        None => {
            let port = config.gateway.port;
            let host = config.gateway.host.clone();
            log_gateway_start(&host, port);
            crate::gateway::run_gateway(&host, port, config).await
        }
    }
}

fn resolve_gateway_addr(config: &Config, port: Option<u16>, host: Option<String>) -> (u16, String) {
    let port = port.unwrap_or(config.gateway.port);
    let host = host.unwrap_or_else(|| config.gateway.host.clone());
    (port, host)
}

fn log_gateway_start(host: &str, port: u16) {
    if port == 0 {
        info!("🚀 Starting RedClaw Gateway on {host} (random port)");
    } else {
        info!("🚀 Starting RedClaw Gateway on {host}:{port}");
    }
}

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

async fn fetch_paircode(host: &str, port: u16, new: bool) -> Result<Option<String>> {
    let client = reqwest::Client::new();

    let response = if new {
        let url = format!("http://{host}:{port}/admin/paircode/new");
        client
            .post(&url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
    } else {
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
