use anyhow::Result;
use log::{debug, info};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, path::PathBuf};

use crate::default_locations::default_locations;

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
    /// Update versions in these locations
    pub locations: Vec<LocationPattern>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            name: None,
            repository: None,
            version: None,
            default: Some(true),
            locations: default_locations(),
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
    #[serde(default)]
    pub r#type: LocationType,
    /// If this is a default location
    #[serde(skip, default)]
    pub default: bool,

    /// Paths to match
    pub paths: Vec<PathBuf>,
    /// Patterns to match
    pub patterns: Vec<String>,
    /// Excludes to ignore
    #[serde(default = "Vec::new")]
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

        if config.use_default() {
            // TODO: Can this be done better?
            let user_locations = config.locations.clone();

            config.locations = default_locations();
            debug!(
                "Using default locations ({} locations)",
                config.locations.len()
            );
            config.locations.extend(user_locations);
        }

        info!("Configuration loaded successfully");

        Ok(config)
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
