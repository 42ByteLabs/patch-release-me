use std::path::PathBuf;

use regex::Regex;

use crate::LocationPattern;

pub fn default_locations() -> Vec<LocationPattern> {
    vec![
        // Release File
        LocationPattern {
            name: String::from("Release File"),
            default: true,
            paths: vec![PathBuf::from(".release.yml")],
            regexes: vec![Regex::new(r#"version:\s*\"?\'?([0-9]+\.[0-9]+\.[0-9])\"?\'?"#).unwrap()],
            ..Default::default()
        },
        // Cargo.toml Version
        LocationPattern {
            name: String::from("Rust Cargo"),
            default: true,
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
            name: String::from("Python Pyproject"),
            default: true,
            paths: vec![
                // Pyproject.toml
                PathBuf::from("pyproject.toml"),
            ],
            regexes: vec![Regex::new(r#"\nversion\s*=\s*"([0-9]+\.[0-9]+\.[0-9])""#).unwrap()],
            ..Default::default()
        },
        LocationPattern {
            name: String::from("Python Init / Version"),
            default: true,
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
        // Node / NPM
        LocationPattern {
            name: String::from("Node Package"),
            default: true,
            paths: vec![PathBuf::from("**/package.json")],
            regexes: vec![Regex::new(r#"\n"version"\s*:\s*"([0-9]+\.[0-9]+\.[0-9])""#).unwrap()],
            ..Default::default()
        },
    ]
}
