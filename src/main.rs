//! # Patch Release Me Tool
#![deny(missing_docs)]
use anyhow::Result;
use log::debug;
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

fn main() -> Result<()> {
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
            patch,
            minor,
            major,
        }) => {
            debug!("Bump Mode");

            let bump_mode = match mode {
                Some(mode) => BumpMode::from(mode),
                None => {
                    if !set_version.is_empty() {
                        BumpMode::Version(set_version.clone())
                    } else if *patch {
                        BumpMode::Patch
                    } else if *minor {
                        BumpMode::Minor
                    } else if *major {
                        BumpMode::Major
                    } else {
                        BumpMode::Patch
                    }
                }
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
        WorkflowMode::Bump(_) => {
            workflow.patch()?;
        }
    }

    Ok(())
}
