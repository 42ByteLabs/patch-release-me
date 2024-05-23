use clap::{Parser, Subcommand};
use console::style;
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
#[command(author, version, about, long_about = None)]
pub struct Arguments {
    /// Enable Debugging
    #[clap(long, env, default_value_t = false)]
    pub debug: bool,

    /// Disable Banner
    #[clap(long, default_value_t = false)]
    pub disable_banner: bool,

    /// Project Root
    #[clap(short, long, env, default_value = ".")]
    pub root: PathBuf,

    /// Configuration file path
    #[clap(short, long, env, default_value = ".release.yml")]
    pub config: PathBuf,

    /// Subcommands
    #[clap(subcommand)]
    pub commands: Option<ArgumentCommands>,
}

#[derive(Subcommand, Debug)]
pub enum ArgumentCommands {
    Display,
    Bump {
        /// Set Version
        #[clap(short, long, env, default_value = "")]
        set_version: String,
        /// Update Patch version
        #[clap(long, default_value = "true")]
        patch: bool,

        /// Update Minor version
        #[clap(long, default_value = "false")]
        minor: bool,

        /// Update Major version
        #[clap(long, default_value = "false")]
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
