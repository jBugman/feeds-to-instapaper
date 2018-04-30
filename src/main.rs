extern crate atty;
#[macro_use]
extern crate clap;
extern crate failure;
extern crate yansi;

extern crate feeds_to_instapaper as app;

use failure::Error;
use clap::{App, Arg, SubCommand};
use yansi::Paint;

mod fs_ext;

use app::Config;
use app::TryFrom;
use app::failure_ext::FmtResultExt;
use fs_ext::read_to_string; // TODO: (Rust 1.26) replace with fs::read_to_string

fn main() {
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
    let enabled = enabled && Paint::enable_windows_ascii(); // always true on non-windows machine
    let enabled = enabled && atty::is(atty::Stream::Stdout) && atty::is(atty::Stream::Stderr);
    if !enabled {
        Paint::disable();
    }
    // Config
    let config_path = args.value_of("config").unwrap();
    let config = parse_config(config_path).unwrap_or_exit();

    app::run(config, args.subcommand()).unwrap_or_exit();
}

fn parse_config(path: &str) -> Result<Config, Error> {
    let config = read_to_string(&path).context_fmt("failed to read config file", &path)?;
    let config = Config::try_from(&config)?;
    Ok(config)
}

trait FailureHandler<T> {
    fn unwrap_or_exit(self) -> T;
}

impl<T> FailureHandler<T> for Result<T, Error> {
    fn unwrap_or_exit(self) -> T {
        match self {
            Err(err) => {
                let mut causes = err.causes();
                eprintln!("{} {}", Paint::red("error:"), causes.next().unwrap());
                for c in causes {
                    eprintln!(" caused by: {}", c);
                }
                std::process::exit(1);
            }
            Ok(v) => v,
        }
    }
}
