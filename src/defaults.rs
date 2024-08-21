//! This module loads the default locations. This is done at compile time
//! by loading the `default.yml` file from the `src` directory of this crate.
use anyhow::Result;
use std::collections::HashMap;

use crate::LocationPattern;

/// List of default Languages and Ecosystems supported
pub const DEFAULTS: &str = include_str!("defaults.yml");

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Defaults {
    #[serde(rename = "ecosystems")]
    pub ecosystems: HashMap<String, Vec<String>>,
    pub locations: Vec<LocationPattern>,
}

impl Defaults {
    pub fn load() -> Result<Self> {
        Ok(serde_yaml::from_str::<Self>(&DEFAULTS)?)
    }

    pub fn get_locations(&self, ecosystem: impl Into<String>) -> Vec<LocationPattern> {
        let ecosystem = ecosystem.into();
        self.locations
            .iter()
            .filter(|loc| {
                loc.ecosystems.contains(&ecosystem) || loc.excludes.contains(&"All".to_string())
            })
            .cloned()
            .collect()
    }

    pub fn get_languages(&self) -> Vec<String> {
        self.ecosystems.keys().cloned().collect()
    }
}
