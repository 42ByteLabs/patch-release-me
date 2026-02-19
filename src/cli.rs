use clap::{Parser, Subcommand};
use console::style;
use std::io::Write;
use std::path::PathBuf;

pub const VERSION_NUMBER: &str = env!("CARGO_PKG_VERSION");
pub const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");

pub const BANNER: &str = r#"______     _       _    ______     _                    ___  ___     
| ___ \   | |     | |   | ___ \   | |                   |  \/  |     
| |_/ /_ _| |_ ___| |__ | |_/ /___| | ___  __ _ ___  ___| .  . | ___ 
|  __/ _` | __/ __| '_ \|    // _ \ |/ _ \/ _` / __|/ _ \ |\/| |/ _ \
| | | (_| | || (__| | | | |\ \  __/ |  __/ (_| \__ \  __/ |  | |  __/
\_|  \__,_|\__\___|_| |_\_| \_\___|_|\___|\__,_|___/\___\_|  |_/\___|"#;

#[derive(Parser, Debug)]
#[command(
    author, 
    version, 
    about = "A tool to automate version bumping across multiple file types in your project",
    long_about = "Patch Release Me automatically updates version numbers in your project files.\n\
                  Run without a subcommand to enter interactive mode."
)]
pub struct Arguments {
    /// Enable debug logging for troubleshooting
    #[clap(long, env, default_value_t = false)]
    pub debug: bool,

    /// Hide the ASCII banner on startup
    #[clap(long, default_value_t = false)]
    pub disable_banner: bool,

    /// Path to your project root directory
    #[clap(short, long, env, default_value = ".")]
    pub root: PathBuf,

    /// Path to the release configuration file
    #[clap(short, long, env, default_value = ".release.yml")]
    pub config: PathBuf,

    /// Subcommands
    #[clap(subcommand)]
    pub commands: Option<ArgumentCommands>,
}

#[derive(Subcommand, Debug)]
pub enum ArgumentCommands {
    /// Initialize a new .release.yml configuration file
    #[command(about = "Create a new configuration file for version management")]
    Init {
        /// Project name (defaults to current directory name)
        #[clap(short, long, env, help = "Set the project name")]
        name: Option<String>,

        /// Initial version number (e.g., 0.1.0)
        #[clap(short, long, env, help = "Set the starting version")]
        version: Option<String>,

        /// Programming languages/ecosystems to include (e.g., Rust, Python, Docker)
        #[clap(short, long, env, help = "Specify ecosystems: Rust, Python, Node, Docker, etc.")]
        language_ecosystems: Vec<String>,

        /// Include default patterns for common files
        #[clap(short, long, env, default_value = "false", help = "Enable default file patterns")]
        defaults: Option<bool>,
    },
    
    /// Show what files and versions would be updated (dry-run)
    #[command(about = "Preview changes without modifying files")]
    Display,
    
    /// Sync all files to the current version in .release.yml
    #[command(about = "Apply current version to all tracked files")]
    Sync,
    
    /// Bump version and update all tracked files
    #[command(about = "Increment version and update files")]
    Bump {
        /// Manually set a specific version (e.g., 1.2.3)
        #[clap(short, long, env, default_value = "", help = "Specify exact version to set")]
        set_version: String,

        /// Bump mode: major, minor, or patch
        #[clap(short, long, env, help = "Choose: major, minor, or patch")]
        mode: Option<String>,

        /// Increment patch version (x.x.N+1) - default if no flags set
        #[clap(long, default_value = "true", help = "Bump patch version (default)")]
        patch: bool,

        /// Increment minor version (x.N+1.0)
        #[clap(long, default_value = "false", help = "Bump minor version")]
        minor: bool,

        /// Increment major version (N+1.0.0)
        #[clap(long, default_value = "false", help = "Bump major version")]
        major: bool,
    },
}

pub fn init() -> Arguments {
    let arguments = Arguments::parse();

    let log_level = match &arguments.debug {
        false => log::LevelFilter::Info,
        true => log::LevelFilter::Debug,
    };

    env_logger::builder()
        .parse_default_env()
        .filter_level(log_level)
        .format(|buf, record| {
            let level = match record.level() {
                log::Level::Error => style(record.level()).red(),
                log::Level::Warn => style(record.level()).yellow(),
                log::Level::Info => style(record.level()).blue(),
                log::Level::Debug => style(record.level()).cyan(),
                log::Level::Trace => style(record.level()).white(),
            };
            writeln!(buf, "[{:^5}] {}", level, record.args())
        })
        .format_module_path(false)
        .init();

    if !arguments.disable_banner {
        println!(
            "{}    by {} - v{}\n",
            style(BANNER).green(),
            style(AUTHOR).red(),
            style(VERSION_NUMBER).blue()
        );
    }

    arguments
}
