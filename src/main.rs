#[macro_use]
extern crate clap;

extern crate feeds_to_instapaper as app;

use crate::app::Config;
use anyhow::Result;
use clap::{App, Arg, SubCommand};

fn main() -> Result<()> {
    // Arguments
    let args = App::new("Feeds to Instapaper")
        .version(crate_version!())
        .about(crate_description!())
        .arg(
            Arg::with_name("config")
                .long("config")
                .default_value("config.yaml")
                .value_name("FILE")
                .help("Sets a custom config file"),
        )
        .arg(
            Arg::with_name("no-color")
                .long("no-color")
                .help("Disable colors in output (disabled automatically on non-TTY)"),
        )
        .arg(
            Arg::with_name("auto-add")
                .long("auto-add")
                .short("y")
                .help("Add posts to Instapaper without asking"),
        )
        .arg(
            Arg::with_name("skip-download-errors")
                .long("skip-download-errors")
                .short("s")
                .help("Proceed after failed feed download"),
        )
        .subcommand(
            SubCommand::with_name("import")
                .about("Import exported Instapaper CSV to pre-fill link log")
                .arg(
                    Arg::with_name("INPUT")
                        .takes_value(true)
                        .required(true)
                        .index(1),
                ),
        )
        .get_matches();
    // Colors
    let enabled = !args.is_present("no-color");
    let enabled = enabled && yansi::Paint::enable_windows_ascii(); // always true on non-windows machine
    let enabled = enabled && atty::is(atty::Stream::Stdout) && atty::is(atty::Stream::Stderr);
    if !enabled {
        yansi::Paint::disable();
    }
    // Config
    let config_path = args.value_of("config").unwrap();
    let mut config = Config::new(config_path)?;
    config.auto_add = args.is_present("auto-add");
    config.skip_download_errors = args.is_present("skip-download-errors");

    app::run(config, args.subcommand())
}
