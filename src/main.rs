//! # Patch Release Me Tool
#![deny(missing_docs)]
use anyhow::Result;
use log::debug;
use log::info;
use log::warn;

mod cli;
mod config;
mod default_locations;
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
    let config = match Config::load(&arguments.root, &arguments.config) {
        Ok(config) => config,
        Err(e) => {
            warn!("Failed to load configuration");
            debug!("Error: {}", e);
            Config::default()
        }
    };

    // Subcommands
    let mode = match &arguments.commands {
        Some(ArgumentCommands::Bump {
            set_version,
            mode,
            patch: _,
            minor,
            major,
        }) => {
            debug!("Bump Mode");

            let cli_mode = if *minor {
                BumpMode::Minor
            } else if *major {
                BumpMode::Major
            } else {
                BumpMode::Patch
            };
            debug!("CLI Mode: {:?}", cli_mode);

            let bump_mode = if !set_version.is_empty() {
                debug!("Manually setting version: {}", set_version);
                BumpMode::Version(set_version.clone())
            } else if let Some(mode) = mode {
                debug!("Setting mode: {} (dynamic)", mode);
                BumpMode::from(mode)
            } else if let Some(ref version) = config.version {
                debug!("Setting mode: Version (from config)");
                // Update version from config file
                let mut new_version = semver::Version::parse(version)?;
                update_version(&mut new_version, &cli_mode);

                BumpMode::Version(new_version.to_string())
            } else {
                cli_mode
            };

            WorkflowMode::Bump(bump_mode)
        }
        Some(ArgumentCommands::Display) => WorkflowMode::Display,
        None => select_mode()?,
    };

    let workflow = Workflow::init()
        .root(arguments.root.clone())?
        .mode(mode.clone())
        .locations(config.locations.clone())?
        .build();

    match mode {
        WorkflowMode::Display => {
            workflow.display()?;
        }
        WorkflowMode::Bump(mode) => {
            info!("Bumping version - {:?}", mode);
            workflow.patch().await?;
        }
    }

    Ok(())
}
