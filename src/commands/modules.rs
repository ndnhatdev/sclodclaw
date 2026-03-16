use anyhow::{anyhow, Context, Result};
use redclaw::ModulesCommands;

pub fn handle_modules_command(command: ModulesCommands) -> Result<()> {
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
                anyhow::bail!("{} issue(s) found", report.error_count());
            }
            Ok(())
        }
    }
}
