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
    Init {
        #[clap(short, long, env)]
        name: Option<String>,

        #[clap(short, long, env)]
        version: Option<String>,

        #[clap(short, long, env)]
        language_ecosystems: Vec<String>,

        #[clap(short, long, env, default_value = "false")]
        defaults: Option<bool>,
    },
    Display,
    Sync,
    Bump {
        /// Set Version
        #[clap(short, long, env, default_value = "")]
        set_version: String,

        #[clap(short, long, env)]
        mode: Option<String>,

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
