use crate::config::Config;
use crate::update_version;
use crate::{WorkflowMode, config::BumpMode, defaults::Defaults};
use anyhow::{Context, Result, anyhow};
use dialoguer::theme::ColorfulTheme;
use dialoguer::{FuzzySelect, Select};
use log::debug;

pub fn select_mode(config: &Config) -> Result<WorkflowMode> {
    let mut modes = Vec::new();
    if !config.version.is_some() {
        modes.push("Init");
    }
    modes.push("Display");
    modes.push("Bump");
    modes.push("Sync");

    let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select Mode")
        .default(0)
        .items(modes.as_slice())
        .interact()?;

    let text = modes.get(selection).ok_or(anyhow!("Invalid selection"))?;

    match *text {
        "Init" => Ok(interactive_init()?),
        "Display" => Ok(WorkflowMode::Display),
        "Sync" => {
            let version = new_version(config, &BumpMode::Version("0.0.0".to_string()))?;
            Ok(WorkflowMode::Bump {
                mode: BumpMode::Version(version.to_string()),
                version,
            })
        }
        "Bump" => {
            let bump_mode = select_bump_mode()?;
            let version = new_version(config, &bump_mode)?;
            Ok(WorkflowMode::Bump {
                mode: bump_mode,
                version,
            })
        }
        _ => Err(anyhow!("Invalid selection")),
    }
}

pub fn interactive_init() -> Result<WorkflowMode> {
    let name = dialoguer::Input::<String>::new()
        .with_prompt("Enter Project Name")
        .default(find_project_name()?)
        .interact()?;

    let version = dialoguer::Input::<String>::new()
        .with_prompt("Enter Version")
        .default("0.1.0".to_string())
        .interact()?;

    let use_defaults = dialoguer::Confirm::new()
        .with_prompt("Use Default Locations?")
        .default(true)
        .interact()?;

    let defaults = dialoguer::Confirm::new()
        .with_prompt("Inline Defaults?")
        .default(false)
        .interact()?;

    let language_ecosystems = if defaults {
        let defaults = Defaults::load()?;
        let mut lang_list = defaults.get_languages();
        lang_list.sort();

        let lang_index = dialoguer::MultiSelect::new()
            .with_prompt("Select Language Ecosystems")
            .items(&lang_list)
            .interact()?;

        lang_list
            .iter()
            .enumerate()
            .filter_map(|(i, &ref lang)| {
                if lang_index.contains(&i) {
                    Some(lang.to_string())
                } else {
                    None
                }
            })
            .collect()
    } else {
        vec![]
    };
    debug!("Language Ecosystems: {:?}", language_ecosystems);

    // Use git to get the repository using command
    let repository: Option<String> = match std::process::Command::new("git")
        .arg("remote")
        .arg("get-url")
        .arg("origin")
        .output()
    {
        Ok(output) => {
            let mut repo = String::from_utf8(output.stdout)?.trim().to_string();
            // Remove the .git extension
            if repo.ends_with(".git") {
                repo = repo.trim_end_matches(".git").to_string();
            }
            // Remove the git@
            if repo.starts_with("git@") {
                repo = repo.trim_start_matches("git@").to_string();
                repo = repo.replace("github.com:", "");
            }
            Some(repo)
        }
        Err(_) => None,
    };

    Ok(WorkflowMode::Init {
        name: Some(name),
        version: Some(version),
        repository,
        language_ecosystems,
        enable_defaults: Some(use_defaults),
    })
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

fn prompt_version() -> Result<semver::Version> {
    let version = dialoguer::Input::<String>::new()
        .with_prompt("Enter Version")
        .default("0.1.0".to_string())
        .interact()
        .context("Failed to read version")?;
    semver::Version::parse(&version).map_err(|e| anyhow!("Invalid version: {}", e))
}

/// Prompt for a new version based on the current version and bump mode
pub fn new_version(config: &Config, bump_mode: &BumpMode) -> Result<semver::Version> {
    let mut version = if let Some(version) = &config.version {
        semver::Version::parse(version).context(format!("Failed to parse version: {version}"))?
    } else {
        prompt_version()?
    };
    update_version(&mut version, bump_mode);
    Ok(version)
}

fn find_project_name() -> Result<String> {
    let current_dir = std::env::current_dir()?;
    let project_name = current_dir
        .file_name()
        .ok_or_else(|| anyhow!("Failed to get current directory"))?
        .to_str()
        .ok_or_else(|| anyhow!("Failed to convert path to string"))?;
    Ok(project_name.to_string())
}
