//! # Patch Release Me Tool
#![deny(missing_docs)]
#![deny(unsafe_code)]

use anyhow::Result;
use console::style;
use defaults::Defaults;
use log::debug;
use log::info;
use log::warn;

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
            let version = if let Some(version) = &config.version {
                semver::Version::parse(version.as_str())?
            } else {
                semver::Version::parse("0.0.0")?
            };
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

            let version = if let Some(version) = &config.version {
                let mut version = semver::Version::parse(version.as_str())?;
                update_version(&mut version, &bump_mode);
                version
            } else {
                panic!("Version not set in config");
            };

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

            info!("Configuration saved");
        }
        WorkflowMode::Display => {
            info!(
                "Current Version - {}",
                style(config.version.unwrap_or_default()).green()
            );

            workflow.display()?;
        }
        WorkflowMode::Bump { mode, .. } => {
            info!("Bumping version - {:?}", mode);
            workflow.patch().await?;
        }
    }

    Ok(())
}
