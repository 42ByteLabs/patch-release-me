use crate::config::Config;
use crate::update_version;
use crate::{WorkflowMode, config::BumpMode, defaults::Defaults};
use anyhow::{Context, Result, anyhow};
use dialoguer::theme::ColorfulTheme;
use dialoguer::{FuzzySelect, Select};
use log::debug;

pub fn select_mode(config: &Config) -> Result<WorkflowMode> {
    println!("\n🚀 Welcome to Patch Release Me - Interactive Mode\n");
    
    let mut modes = Vec::new();
    let mut descriptions = Vec::new();
    
    if !config.version.is_some() {
        modes.push("Init");
        descriptions.push("Initialize new .release.yml configuration");
    }
    modes.push("Display");
    descriptions.push("Preview version changes (dry-run)");
    modes.push("Bump");
    descriptions.push("Increment version and update files");
    modes.push("Sync");
    descriptions.push("Apply current version to all files");

    // Format items with descriptions
    let items: Vec<String> = modes.iter()
        .zip(descriptions.iter())
        .map(|(mode, desc)| format!("{:<10} - {}", mode, desc))
        .collect();

    let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("What would you like to do?")
        .default(0)
        .items(&items)
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
    println!("\n📝 Configuration Setup\n");
    println!("Let's create your .release.yml configuration file.\n");
    
    let name = dialoguer::Input::<String>::new()
        .with_prompt("Project name")
        .with_initial_text(find_project_name()?)
        .interact_text()?;

    let version = dialoguer::Input::<String>::new()
        .with_prompt("Initial version (semver format)")
        .default("0.1.0".to_string())
        .validate_with(|input: &String| -> Result<(), &str> {
            if semver::Version::parse(input).is_ok() {
                Ok(())
            } else {
                Err("Please enter a valid semver version (e.g., 0.1.0)")
            }
        })
        .interact_text()?;

    let use_defaults = dialoguer::Confirm::new()
        .with_prompt("Enable default patterns for common files? (Recommended)")
        .default(true)
        .interact()?;

    let defaults = dialoguer::Confirm::new()
        .with_prompt("Inline ecosystem patterns into config? (Makes config portable)")
        .default(false)
        .interact()?;

    let language_ecosystems = if defaults {
        println!("\n🔧 Select your project's ecosystems (use Space to select, Enter to confirm):\n");
        
        let defaults = Defaults::load()?;
        let mut lang_list = defaults.get_languages();
        lang_list.sort();

        let lang_index = dialoguer::MultiSelect::new()
            .with_prompt("Language ecosystems")
            .items(&lang_list)
            .interact()?;

        let selected: Vec<String> = lang_list
            .iter()
            .enumerate()
            .filter_map(|(i, &ref lang)| {
                if lang_index.contains(&i) {
                    Some(lang.to_string())
                } else {
                    None
                }
            })
            .collect();
        
        if selected.is_empty() {
            println!("⚠️  No ecosystems selected. You can manually configure patterns later.");
        } else {
            println!("✓ Selected: {}", selected.join(", "));
        }
        
        selected
    } else {
        vec![]
    };
    debug!("Language Ecosystems: {:?}", language_ecosystems);

    // Auto-detect Git repository
    let repository: Option<String> = match std::process::Command::new("git")
        .arg("remote")
        .arg("get-url")
        .arg("origin")
        .output()
    {
        Ok(output) => {
            let mut repo = String::from_utf8(output.stdout)?.trim().to_string();
            if repo.ends_with(".git") {
                repo = repo.trim_end_matches(".git").to_string();
            }
            if repo.starts_with("git@") {
                repo = repo.trim_start_matches("git@").to_string();
                repo = repo.replace("github.com:", "");
            }
            if repo.starts_with("https://github.com/") {
                repo = repo.trim_start_matches("https://github.com/").to_string();
            }
            println!("\n✓ Git repository detected: {}", repo);
            Some(repo)
        }
        Err(_) => {
            println!("\n⚠️  No Git repository detected");
            None
        }
    };
    
    println!("\n✅ Configuration complete! Creating .release.yml...\n");

    Ok(WorkflowMode::Init {
        name: Some(name),
        version: Some(version),
        repository,
        language_ecosystems,
        enable_defaults: Some(use_defaults),
    })
}

pub fn select_bump_mode() -> Result<BumpMode> {
    println!("\n📦 Version Bump Strategy\n");
    
    let items = vec![
        "Patch (x.x.N+1) - Bug fixes, small changes",
        "Minor (x.N+1.0) - New features, backward compatible",
        "Major (N+1.0.0) - Breaking changes",
        "Custom - Manually set version",
    ];
    
    let selection = Select::new()
        .with_prompt("How would you like to bump the version?")
        .default(0)
        .items(&items)
        .interact()?;

    match selection {
        0 => {
            println!("✓ Patch version will be incremented");
            Ok(BumpMode::Patch)
        },
        1 => {
            println!("✓ Minor version will be incremented");
            Ok(BumpMode::Minor)
        },
        2 => {
            println!("✓ Major version will be incremented");
            Ok(BumpMode::Major)
        },
        3 => {
            let version = dialoguer::Input::<String>::new()
                .with_prompt("Enter custom version (semver format)")
                .validate_with(|input: &String| -> Result<(), &str> {
                    if semver::Version::parse(input).is_ok() {
                        Ok(())
                    } else {
                        Err("Please enter a valid semver version (e.g., 1.2.3)")
                    }
                })
                .interact_text()?;
            println!("✓ Version will be set to: {}", version);
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
