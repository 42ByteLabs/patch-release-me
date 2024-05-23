use crate::{config::BumpMode, WorkflowMode};
use anyhow::{anyhow, Result};
use dialoguer::Select;

pub fn select_mode() -> Result<WorkflowMode> {
    let selection = Select::new()
        .with_prompt("Select Mode")
        .default(0)
        .item("Display")
        .item("Bump")
        .interact()?;
    match selection {
        0 => Ok(WorkflowMode::Display),
        1 => {
            let bump_mode = select_bump_mode()?;
            Ok(WorkflowMode::Bump(bump_mode))
        }
        _ => Err(anyhow!("Invalid selection")),
    }
}

pub fn select_bump_mode() -> Result<BumpMode> {
    let selection = Select::new()
        .with_prompt("Select Bump Mode")
        .default(0)
        .item("Patch")
        .item("Minor")
        .item("Major")
        .item("Version (manual)")
        .interact()?;

    match selection {
        0 => Ok(BumpMode::Patch),
        1 => Ok(BumpMode::Minor),
        2 => Ok(BumpMode::Major),
        3 => {
            let version = dialoguer::Input::<String>::new()
                .with_prompt("Enter Version")
                .interact()?;
            Ok(BumpMode::Version(version))
        }
        _ => Err(anyhow!("Invalid selection")),
    }
}
