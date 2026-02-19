//! # Patch Release Me Tool
#![deny(missing_docs)]
#![deny(unsafe_code)]

use anyhow::Result;
use console::style;
use defaults::Defaults;
use log::{debug, warn};

mod cli;
mod config;
mod defaults;
mod error;
mod interactive;
mod workflows;

use crate::cli::*;
use crate::config::*;
use crate::interactive::*;
use crate::workflows::*;

/// Detect current version from Cargo.toml
fn detect_current_version(root: &std::path::Path) -> Result<semver::Version> {
    let cargo_toml = root.join("Cargo.toml");
    if cargo_toml.exists() {
        let content = std::fs::read_to_string(&cargo_toml)?;
        // Match version only in [package] section, before any other section
        let version_regex =
            regex::Regex::new(r#"(?m)^\[package\][^\[]*?^version\s*=\s*"([^"]+)""#)?;
        if let Some(cap) = version_regex.captures(&content) {
            if let Some(version_str) = cap.get(1) {
                return semver::Version::parse(version_str.as_str()).map_err(|e| {
                    anyhow::anyhow!("Failed to parse version from Cargo.toml: {}", e)
                });
            }
        }
    }
    anyhow::bail!(
        "Could not detect version from Cargo.toml. Please ensure it exists and has a valid version field."
    )
}

#[tokio::main]
async fn main() -> Result<()> {
    let arguments = init();
    debug!("Finished initialising, starting main workflow...");

    // Load Configuration
    let defaults = Defaults::load()?;
    debug!("Defaults Count - {}", defaults.locations.len());

    let mut config = match Config::load(&arguments.root, &arguments.config) {
        Ok(config) => config,
        Err(e) => {
            warn!("Failed to load configuration");
            debug!("Error: {}", e);
            Config::default()
        }
    };

    // Subcommands
    let mode = match &arguments.commands {
        Some(ArgumentCommands::Init {
            name,
            version,
            language_ecosystems,
            defaults,
        }) => {
            debug!("Init Mode");

            WorkflowMode::Init {
                name: name.clone(),
                version: version.clone(),
                repository: None,
                language_ecosystems: language_ecosystems.clone(),
                enable_defaults: *defaults,
            }
        }
        Some(ArgumentCommands::Sync) => {
            debug!("Sync Mode");
            // For sync, detect the actual version from Cargo.toml at runtime
            let version = detect_current_version(&arguments.root)?;
            WorkflowMode::Bump {
                mode: BumpMode::Version(version.to_string()),
                version,
            }
        }
        Some(ArgumentCommands::Bump {
            set_version,
            mode,
            patch: _,
            minor,
            major,
        }) => {
            debug!("Bump Mode");

            let bump_mode = if !set_version.is_empty() {
                debug!("Manually setting version: {}", set_version);
                BumpMode::Version(set_version.clone())
            } else if let Some(mode) = mode {
                debug!("Setting mode: {} (dynamic)", mode);
                BumpMode::from(mode)
            } else if let Some(ref version) = config.version {
                debug!("Setting mode: Version (from config)");
                BumpMode::Version(version.clone())
            } else {
                if *minor {
                    BumpMode::Minor
                } else if *major {
                    BumpMode::Major
                } else {
                    BumpMode::Patch
                }
            };
            debug!("CLI Mode: {:?}", bump_mode);

            let version = new_version(&config, &bump_mode)?;

            WorkflowMode::Bump {
                mode: bump_mode,
                version,
            }
        }
        Some(ArgumentCommands::Display) => WorkflowMode::Display,
        None => select_mode(&config)?,
    };

    let workflow = Workflow::init()
        .root(arguments.root.clone())?
        .mode(mode.clone())
        .locations(config.locations.clone())?
        .build();

    match mode {
        WorkflowMode::Init {
            name,
            version,
            repository,
            language_ecosystems,
            enable_defaults,
        } => {
            config.name = name.clone();
            config.version = version.clone();
            config.repository = repository.clone();
            config.default = enable_defaults;

            // Inline defaults
            if !language_ecosystems.is_empty() {
                for ecosystem in language_ecosystems {
                    config.locations.extend(defaults.get_locations(&ecosystem));
                }

                config.default = Some(false);
            }

            // Save configuration
            config.write(&arguments.root.join(&arguments.config))?;

            println!("\n{}", style("━".repeat(60)).dim());
            println!(
                "{} Configuration saved successfully!",
                style("✓").green().bold()
            );
            println!(
                "{} File: {}",
                style("→").dim(),
                style(&arguments.config.display()).cyan()
            );
            println!("{}", style("━".repeat(60)).dim());
            println!("\nNext steps:");
            println!(
                "  {} Run 'patch-release-me display' to preview changes",
                style("1.").bold()
            );
            println!(
                "  {} Run 'patch-release-me bump' to update versions",
                style("2.").bold()
            );
            println!();
        }
        WorkflowMode::Display => {
            println!();
            let version_text = config
                .version
                .as_ref()
                .map(|v| format!("{}", style(v).green().bold()))
                .unwrap_or_else(|| style("Not set").yellow().to_string());

            println!("{} Current version: {}", style("ℹ").blue(), version_text);
            println!("{}", style("─".repeat(60)).dim());
            println!();

            workflow.display()?;

            println!();
            println!("{}", style("Note:").bold());
            println!("  This is a dry-run. No files were modified.");
            println!("  Run 'patch-release-me bump' to apply changes.");
            println!();
        }
        WorkflowMode::Bump { mode, .. } => {
            println!("\n{} Bumping version: {:?}", style("→").cyan(), mode);
            println!("{}", style("─".repeat(60)).dim());

            workflow.patch().await?;

            println!();
            println!("{}", style("━".repeat(60)).dim());
            println!(
                "{} Version bump completed successfully!",
                style("✓").green().bold()
            );
            println!("{}", style("━".repeat(60)).dim());
            println!();
        }
    }

    Ok(())
}
