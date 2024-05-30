use std::path::PathBuf;

use regex::Regex;

use crate::LocationPattern;

pub fn default_locations() -> Vec<LocationPattern> {
    vec![
        // Cargo.toml Version
        LocationPattern {
            name: Some(String::from("Default - Rust Cargo")),
            paths: vec![PathBuf::from("**/Cargo.toml")],
            excludes: vec![
                // Crates and target directories
                String::from("/crates/"),
                String::from("/target/"),
            ],
            regexes: vec![Regex::new(r#"\nversion\s*=\s*"([0-9]+\.[0-9]+\.[0-9])""#).unwrap()],
            ..Default::default()
        },
        // Python
        LocationPattern {
            name: Some(String::from("Default - Python Pyproject")),
            paths: vec![
                // Pyproject.toml
                PathBuf::from("pyproject.toml"),
            ],
            regexes: vec![Regex::new(r#"\nversion\s*=\s*"([0-9]+\.[0-9]+\.[0-9])""#).unwrap()],
            ..Default::default()
        },
        LocationPattern {
            name: Some(String::from("Default - Python Init / Version")),
            paths: vec![
                PathBuf::from("**/__init__.py"),
                PathBuf::from("**/__version__.py"),
            ],
            excludes: vec![String::from("/__pycache__/"), String::from("/vendor/")],
            regexes: vec![
                Regex::new(r#"__version__\s*=\s*["|']([0-9]+\.[0-9]+\.[0-9])["|']"#).unwrap(),
            ],
            ..Default::default()
        },
    ]
}
