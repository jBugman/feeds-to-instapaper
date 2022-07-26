extern crate feeds_to_instapaper as app;

use crate::app::Config;
use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(name = "Feeds to Instapaper",version, about, long_about = None)]
struct Cli {
    #[clap(
        long,
        value_parser,
        value_name = "FILE",
        default_value = "config.yaml",
        help = "Sets a custom config file"
    )]
    config: String,

    #[clap(
        long,
        value_parser,
        action,
        help = "Disable colors in output (disabled automatically on non-TTY)"
    )]
    no_color: bool,

    #[clap(
        short = 'y',
        long,
        value_parser,
        action,
        help = "Add posts to Instapaper without asking"
    )]
    auto_add: bool,

    #[clap(
        short = 's',
        long = "skip-download-errors",
        action,
        value_parser,
        help = "Proceed after failed feed download"
    )]
    skip_download_errors: bool,

    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Import exported Instapaper CSV to pre-fill link log
    Import {
        #[clap(value_parser, value_name = "INPUT")]
        source: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Colors
    let enabled = !cli.no_color &&
    yansi::Paint::enable_windows_ascii() && // always true on non-windows machine
    atty::is(atty::Stream::Stdout) &&
    atty::is(atty::Stream::Stderr);
    if !enabled {
        yansi::Paint::disable();
    }

    // Config
    let config_path = cli.config;
    let mut config = Config::new(config_path)?;
    config.auto_add = cli.auto_add;
    config.skip_download_errors = cli.skip_download_errors;

    let mut links = app::Links::from(&config.log_file)?;
    match &cli.command {
        Some(Commands::Import { source }) => app::run_import(&mut links, source),
        None => app::run_link_processing(config, &mut links),
    }
}
