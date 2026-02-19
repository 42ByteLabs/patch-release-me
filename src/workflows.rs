use anyhow::Result;
use console::style;
use log::{debug, info, warn};
use std::path::PathBuf;

use crate::config::{BumpMode, LocationPattern};

#[derive(Debug, Clone)]
pub enum WorkflowMode {
    Init {
        name: Option<String>,
        version: Option<String>,
        repository: Option<String>,
        language_ecosystems: Vec<String>,
        enable_defaults: Option<bool>,
    },
    Bump {
        /// Bump Mode
        mode: BumpMode,
        /// Version to set
        version: semver::Version,
    },
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
        use std::sync::{Arc, Mutex};
        let file_count = Arc::new(Mutex::new(0));
        let match_count = Arc::new(Mutex::new(0));
        
        let fc = file_count.clone();
        let mc = match_count.clone();
        
        self.process(move |path, captures| {
            if !captures.is_empty() {
                *fc.lock().unwrap() += 1;
                
                // Print file header
                println!("  {} {}", style("📄").dim(), style(path.display()).cyan());
                
                for capture in captures {
                    *mc.lock().unwrap() += 1;
                    let data = capture.get(1).unwrap();
                    let start = data.start();

                    match &self.mode {
                        WorkflowMode::Display => {
                            println!(
                                "     {} {} (line position: {})",
                                style("→").dim(),
                                style(data.as_str()).red().bold(),
                                style(start).dim()
                            );
                        }
                        WorkflowMode::Bump { version, .. } => {
                            println!(
                                "     {} {} {} {}",
                                style("→").dim(),
                                style(data.as_str()).red(),
                                style("→").green(),
                                style(version).green().bold()
                            );
                        }
                        _ => {}
                    };
                }
                println!();
            }
            Ok(())
        })?;
        
        let files = *file_count.lock().unwrap();
        let matches = *match_count.lock().unwrap();
        
        println!("{}", style("─".repeat(60)).dim());
        println!("  {} files with {} version references", 
                 style(files).cyan().bold(), 
                 style(matches).cyan().bold());
        
        Ok(())
    }

    /// Patch Mode - Update the versions
    pub async fn patch(&self) -> Result<()> {
        use std::sync::{Arc, Mutex};
        let file_count = Arc::new(Mutex::new(0));
        let update_count = Arc::new(Mutex::new(0));
        
        let fc = file_count.clone();
        let uc = update_count.clone();
        
        self.process(move |path, captures| {
            let mut content = std::fs::read_to_string(&path)?;
            let mut file_updated = false;

            for capture in captures {
                let data = capture.get(1).unwrap();
                let start = data.start();
                let end = data.end();

                if let WorkflowMode::Bump { version, .. } = &self.mode {
                    if !file_updated {
                        println!("  {} {}", style("📝").cyan(), style(path.display()).bold());
                        file_updated = true;
                        *fc.lock().unwrap() += 1;
                    }
                    
                    println!(
                        "     {} {} {} {}",
                        style("✓").green(),
                        style(data.as_str()).red(),
                        style("→").dim(),
                        style(version.clone()).green().bold()
                    );

                    content.replace_range(start..end, version.to_string().as_str());
                    *uc.lock().unwrap() += 1;
                };
            }

            // Write content back to file
            if file_updated {
                std::fs::write(&path, content)?;
                println!();
            }

            Ok(())
        })?;
        
        let files = *file_count.lock().unwrap();
        let updates = *update_count.lock().unwrap();
        
        println!("{}", style("─".repeat(60)).dim());
        println!("  {} files updated with {} changes", 
                 style(files).cyan().bold(), 
                 style(updates).cyan().bold());
        
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

    /// Add locations to the workflow
    pub fn locations(mut self, locations: Vec<LocationPattern>) -> Result<Self> {
        // Compile regexes
        for location in &locations {
            let mut new_location = location.clone();
            if new_location.regexes.is_empty() {
                // TODO: Support replacement ${...} syntax

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
        BumpMode::Version(version_str) => {
            let new_version = semver::Version::parse(version_str).expect("Invalid version");
            *version = new_version;
        }
    }
}
