use anyhow::Result;
use console::style;
use log::{debug, info, warn};
use std::path::PathBuf;

use crate::config::{BumpMode, LocationPattern};

#[derive(Debug, Clone)]
pub enum WorkflowMode {
    Bump(BumpMode),
    Display,
}

#[derive(Debug, Clone)]
pub struct Workflow {
    /// Project Root
    root: PathBuf,
    /// Workflow Mode
    mode: WorkflowMode,
    /// Locations to update
    locations: Vec<LocationPattern>,
}

impl Workflow {
    pub fn init() -> WorkflowBuilder {
        WorkflowBuilder::default()
    }

    pub fn display(&self) -> Result<()> {
        self.process(|path, captures| {
            for capture in captures {
                let data = capture.get(1).unwrap();
                let version = semver::Version::parse(data.as_str())?;
                let start = data.start();
                let end = data.end();

                match self.mode {
                    WorkflowMode::Display => {
                        info!(
                            "{:>8} :: {}#{}",
                            style(data.as_str()).red(),
                            path.display(),
                            start
                        );
                    }
                    WorkflowMode::Bump(ref mode) => {
                        let mut new_version = version.clone();
                        update_version(&mut new_version, mode);

                        info!(
                            "{:>8} -> {:<8} :: {}#{}-{}",
                            style(data.as_str()).red(),
                            style(new_version).green(),
                            path.display(),
                            start,
                            end
                        );
                    }
                };
            }
            Ok(())
        })?;
        Ok(())
    }

    /// Patch Mode - Update the versions
    pub async fn patch(&self) -> Result<()> {
        self.process(|path, captures| {
            let mut content = std::fs::read_to_string(&path)?;

            for capture in captures {
                let data = capture.get(1).unwrap();
                let version = semver::Version::parse(data.as_str())?;
                let start = data.start();
                let end = data.end();

                let location = format!("{}#{}-{}", path.display(), start, end);

                if let WorkflowMode::Bump(ref mode) = self.mode {
                    let mut new_version = version.clone();
                    update_version(&mut new_version, mode);

                    info!(
                        "{:>8} -> {:<8} :: {}",
                        style(data.as_str()).red(),
                        style(new_version.clone()).green(),
                        location
                    );

                    content.replace_range(start..end, new_version.to_string().as_str());
                };
            }

            // Write content back to file
            std::fs::write(&path, content)?;

            Ok(())
        })?;
        Ok(())
    }

    pub fn process<F>(&self, action: F) -> Result<()>
    where
        F: Fn(PathBuf, Vec<regex::Captures>) -> Result<()>,
    {
        for location in &self.locations {
            info!("Processing Location :: {}", location);

            if location.regexes.is_empty() {
                warn!("No regexes found for location, skipping...");
                continue;
            }

            for path in &location.paths {
                let full_location = self.root.join(path);

                for entry in glob::glob(full_location.to_str().unwrap())? {
                    let entry = match entry {
                        Ok(entry) => entry,
                        Err(e) => {
                            debug!("Error: {:?}", e);
                            continue;
                        }
                    };

                    // Check if entry matches exclude patterns
                    if location
                        .excludes
                        .iter()
                        .any(|pattern| String::from(entry.to_str().unwrap()).contains(pattern))
                    {
                        debug!("Excluded: {:?}", entry);
                        continue;
                    }

                    // Load file
                    let file_contents = std::fs::read_to_string(&entry)?;

                    let mut captures: Vec<regex::Captures> = Vec::new();
                    location.regexes.iter().for_each(|regex| {
                        regex.captures_iter(&file_contents).for_each(|capture| {
                            captures.push(capture);
                        });
                    });

                    if captures.is_empty() {
                        debug!("No captures found in file, skipping...");
                        continue;
                    }

                    action(entry, captures)?;
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct WorkflowBuilder {
    root: Option<PathBuf>,
    mode: Option<WorkflowMode>,
    locations: Vec<LocationPattern>,
}

impl Default for WorkflowBuilder {
    fn default() -> Self {
        Self {
            root: Some(PathBuf::from("./")),
            mode: Some(WorkflowMode::Display),
            locations: Vec::new(),
        }
    }
}

impl WorkflowBuilder {
    pub fn root(mut self, root: PathBuf) -> Result<Self> {
        let path: String = root
            .canonicalize()?
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Failed to convert path to string"))?
            .to_string();
        self.root = Some(PathBuf::from(path));
        Ok(self)
    }
    pub fn mode(mut self, mode: WorkflowMode) -> Self {
        self.mode = Some(mode);
        self
    }
    pub fn locations(mut self, locations: Vec<LocationPattern>) -> Result<Self> {
        // Compile regexes
        for location in &locations {
            let mut new_location = location.clone();
            if new_location.regexes.is_empty() {
                new_location.regexes = LocationPattern::regexes(&location.patterns)?;
            }

            self.locations.push(new_location);
        }
        Ok(self)
    }

    pub fn build(self) -> Workflow {
        Workflow {
            root: self.root.expect("Root is required"),
            mode: self.mode.expect("Mode is required"),
            locations: self.locations,
        }
    }
}

pub(crate) fn update_version(version: &mut semver::Version, bump_mode: &BumpMode) {
    match bump_mode {
        BumpMode::Patch => {
            version.patch += 1;
        }
        BumpMode::Minor => {
            version.minor += 1;
            version.patch = 0;
        }
        BumpMode::Major => {
            version.major += 1;
            version.minor = 0;
            version.patch = 0;
        }
        BumpMode::Version(ref version_str) => {
            let new_version = semver::Version::parse(version_str).expect("Invalid version");
            *version = new_version;
        }
    }
}
