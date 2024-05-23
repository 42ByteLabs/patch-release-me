use anyhow::Result;
use log::{debug, info};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, path::PathBuf};

use crate::default_locations::default_locations;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BumpMode {
    Patch,
    Minor,
    Major,
    /// Set the version to the specified version
    Version(String),
}

/// Configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,

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
            default: Some(true),
            locations: default_locations(),
        }
    }
}

/// Location Pattern to match a file path and a regex pattern
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LocationPattern {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    pub paths: Vec<PathBuf>,

    pub patterns: Vec<String>,

    #[serde(default = "Vec::new")]
    pub excludes: Vec<String>,

    #[serde(skip)]
    pub regexes: Vec<Regex>,
}

impl Config {
    /// Load YAML the configuration from a file path
    pub fn load(root: &PathBuf, path: &PathBuf) -> Result<Self> {
        let resroot = root.canonicalize()?;
        debug!("Project Root: {:?}", resroot);

        let respath = resroot.join(path);
        debug!("Loading configuration from: {:?}", respath);

        let config_data = std::fs::read_to_string(respath)
            .map_err(|e| anyhow::anyhow!("Failed to read configuration file: {:?}", e))?;
        let mut config: Self = serde_yaml::from_str(&config_data)?;

        if let Some(def) = config.default {
            if def {
                debug!("Using default locations");
                config.locations.extend(default_locations());
            }
        } else {
            debug!("Using default locations");
            config.locations.extend(default_locations());
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
        if let Some(name) = &self.name {
            write!(f, "{}", name)
        } else {
            write!(f, "{:?}", self.paths)
        }
    }
}
