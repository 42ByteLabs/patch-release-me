use anyhow::Result;
use log::{debug, info};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, path::PathBuf};

use crate::defaults::Defaults;

/// Bump mode for the version
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BumpMode {
    /// Patch version (x.y.Z -> x.y.Z+1)
    Patch,
    /// Minor version (x.Y.z -> x.Y+1.0)
    Minor,
    /// Major version (X.y.z -> X+1.0.0)
    Major,
    /// Set the version to the specified version
    Version(String),
}

/// Configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Name of the project
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Repository of the project
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    /// Version to set
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// If Default locations should be used or not
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<bool>,

    /// Ecosystem to use for the project
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ecosystem: Option<String>,
    /// Ecosystem(s) used in the project
    #[serde(default = "Vec::new", skip_serializing_if = "Vec::is_empty")]
    pub ecosystems: Vec<String>,

    /// Global excludes patterns
    #[serde(default = "Vec::new", skip_serializing_if = "Vec::is_empty")]
    pub excludes: Vec<String>,

    /// Update versions in these locations
    #[serde(default = "Vec::new", skip_serializing_if = "Vec::is_empty")]
    pub locations: Vec<LocationPattern>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            name: None,
            repository: None,
            version: None,
            default: Some(true),
            ecosystem: None,
            ecosystems: Vec::new(),
            excludes: Vec::new(),
            locations: Vec::new(),
        }
    }
}

impl From<&String> for BumpMode {
    fn from(s: &String) -> Self {
        match s.as_str() {
            "patch" => BumpMode::Patch,
            "minor" => BumpMode::Minor,
            "major" => BumpMode::Major,
            _ => BumpMode::Patch,
        }
    }
}

/// Location Pattern to match a file path and a regex pattern
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LocationPattern {
    /// Nameo of the LocationPattern
    #[serde(default = "String::new")]
    pub name: String,
    /// Type of the location
    #[serde(default, skip_serializing)]
    #[allow(dead_code)]
    pub r#type: LocationType,
    /// If this is a default location
    #[serde(skip, default)]
    pub default: bool,

    #[serde(default = "Vec::new", skip_serializing)]
    pub ecosystems: Vec<String>,

    /// Paths to match
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub paths: Vec<PathBuf>,
    /// Patterns to match
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub patterns: Vec<String>,
    /// Excludes to ignore
    #[serde(default = "Vec::new", skip_serializing_if = "Vec::is_empty")]
    pub excludes: Vec<String>,
    /// Regexes to match (this is not serialized)
    #[serde(skip)]
    pub regexes: Vec<Regex>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum LocationType {
    #[default]
    #[serde(rename = "version")]
    Version,
}

impl Config {
    /// If default locations should be used
    pub fn use_default(&self) -> bool {
        self.default.unwrap_or(true)
    }

    /// Load YAML the configuration from a file path
    pub fn load(root: &PathBuf, path: &PathBuf) -> Result<Self> {
        let resroot = root.canonicalize()?;
        debug!("Project Root: {:?}", resroot);

        let respath = resroot.join(path);
        debug!("Loading configuration from: {:?}", respath);

        let config_data = std::fs::read_to_string(respath)
            .map_err(|e| anyhow::anyhow!("Failed to read configuration file: {:?}", e))?;
        let mut config: Self = serde_yaml::from_str(&config_data)?;

        if let Some(eco) = &config.ecosystem {
            debug!("Using ecosystem: {}", eco);
            config.ecosystems.push(eco.clone());
        }

        // Defaults
        if config.use_default() {
            let defaults = Defaults::load()?;
            debug!(
                "Using default locations ({} locations)",
                config.locations.len()
            );

            if config.ecosystems.is_empty() {
                debug!("No ecosystems specified, using all locations");
                config.locations.extend(defaults.locations);
            } else {
                debug!("Filtering locations by ecosystems");
                config.ecosystems.iter().for_each(|eco| {
                    defaults.locations.iter().for_each(|loc| {
                        if loc.ecosystems.contains(eco)
                            || loc.ecosystems.contains(&"All".to_string())
                        {
                            if config.locations.iter().any(|l| l.name == loc.name) {
                                debug!("Location already exists, skipping: {}", loc.name);
                            } else {
                                debug!("Adding location: {}", loc.name);
                                config.locations.push(loc.clone());
                            }
                        }
                    });
                });
            }
        }

        // Update excludes paths
        if !config.excludes.is_empty() {
            debug!("Adding global excludes to default locations");
            for loc in config.locations.iter_mut() {
                loc.excludes.extend(config.excludes.clone());
            }
        }

        // Update any placeholders in the configuration
        config.update_placeholders();

        info!("Configuration loaded successfully");

        Ok(config)
    }

    // Update placeholders with semantic version regexes
    #[allow(unused_assignments)]
    fn update_placeholders(&mut self) {
        // TODO: Add pre-release and build metadata
        let semver = "([0-9]+\\.[0-9]+\\.[0-9]+)";
        let mut placeholders = vec![
            ("{major}", "([0-9]+)"),
            ("{minor}", "([0-9]+\\.[0-9]+)"),
            ("{patch}", semver),
            ("{version}", semver),
            ("{semver}", semver),
        ];

        // TODO: we should probably do something else here
        let mut owner_case = "".to_string();
        let mut name_case = "".to_string();
        let mut repo_case = "".to_string();

        if let Some(repo) = &self.repository {
            // repo could be `owner/name` or `name`
            // repo isn't case sensitive so we
            repo_case = format!("(?i){}(?-i)", repo);

            if let Some((owner, name)) = repo.split_once('/') {
                debug!("Full repository name: {}/{}", owner, name);
                owner_case = format!("(?i){}(?-i)", owner);
                name_case = format!("(?i){}(?-i)", name);

                placeholders.push(("{owner}", &owner_case));
                placeholders.push(("{name}", &name_case));
            } else {
                debug!("Repository name: {}", repo);
                placeholders.push(("{name}", &repo_case));
            };
            placeholders.push(("{repository}", &repo_case));
            placeholders.push(("{repo}", &repo_case));
        }

        self.locations.iter_mut().for_each(|loc| {
            loc.patterns.iter_mut().for_each(|pattern| {
                placeholders.iter().for_each(|(ph, repl)| {
                    *pattern = pattern.replace(ph, repl);
                });
            });
        });
    }

    /// Write the configuration to a file path
    pub fn write(&self, path: &PathBuf) -> Result<()> {
        let config_data = serde_yaml::to_string(&self)?;
        std::fs::write(path, config_data)?;

        Ok(())
    }
}

impl LocationPattern {
    /// Create a new LocationPattern
    pub fn regexes(patterns: &[String]) -> Result<Vec<regex::Regex>> {
        Ok(patterns
            .iter()
            .map(|pattern_str| match regex::Regex::new(pattern_str) {
                Ok(pattern) => Ok(pattern),
                Err(e) => {
                    debug!("Error: {:?}", e);
                    Err(e)
                }
            })
            .filter_map(Result::ok)
            .collect::<Vec<regex::Regex>>())
    }
}

impl Display for LocationPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.default {
            write!(f, "Default - {}", self.name)
        } else {
            write!(f, "{}", self.name)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder() {
        let mut config = Config {
            version: Some("1.2.3".to_string()),
            locations: vec![LocationPattern {
                name: "Cargo.toml".to_string(),
                paths: vec![PathBuf::from("Cargo.toml")],
                patterns: vec![
                    "version = \"{version}\"".to_string(),
                    "semver = \"{semver}\"".to_string(),
                    "major = \"{major}\"".to_string(),
                    "minor = \"{minor}\"".to_string(),
                    "patch = \"{patch}\"".to_string(),
                ],
                ..Default::default()
            }],
            ..Default::default()
        };
        config.update_placeholders();

        let loc = &config.locations[0];
        assert_eq!(loc.patterns[0], "version = \"([0-9]+\\.[0-9]+\\.[0-9]+)\"");
        assert_eq!(loc.patterns[1], "semver = \"([0-9]+\\.[0-9]+\\.[0-9]+)\"");
        assert_eq!(loc.patterns[2], "major = \"([0-9]+)\"");
        assert_eq!(loc.patterns[3], "minor = \"([0-9]+\\.[0-9]+)\"");
        assert_eq!(loc.patterns[4], "patch = \"([0-9]+\\.[0-9]+\\.[0-9]+)\"");
    }

    #[test]
    fn test_placeholder_repo() {
        let mut config = Config {
            version: Some("1.2.3".to_string()),
            repository: Some("42ByteLabs/patch-release-me".to_string()),
            locations: vec![LocationPattern {
                name: "Cargo.toml".to_string(),
                paths: vec![PathBuf::from("Cargo.toml")],
                patterns: vec![
                    "repository = \"{repository}\"".to_string(),
                    "owner = \"{owner}\"".to_string(),
                    "name = \"{name}\"".to_string(),
                ],
                ..Default::default()
            }],
            ..Default::default()
        };
        config.update_placeholders();

        let loc = &config.locations[0];
        // All repo related placeholders should be replaced with the repo + case sensitive regex
        assert_eq!(
            loc.patterns[0],
            "repository = \"(?i)42ByteLabs/patch-release-me(?-i)\""
        );
        assert_eq!(loc.patterns[1], "owner = \"(?i)42ByteLabs(?-i)\"");
        assert_eq!(loc.patterns[2], "name = \"(?i)patch-release-me(?-i)\"");
    }
}
