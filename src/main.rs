#[macro_use]
extern crate clap;
extern crate failure;
extern crate feeds_to_instapaper;

use failure::Error;
use clap::{App, Arg, SubCommand};

mod fs_ext;

use feeds_to_instapaper::{Config, failure_ext::FmtResultExt};
use feeds_to_instapaper::TryFrom;
use fs_ext::read_to_string; // TODO: (Rust 1.26) replace with fs::read_to_string

fn main() {
    // Arguments
    let app = App::new("Feeds to Instapaper")
        .version(crate_version!())
        .about(crate_description!())
        .arg(
            Arg::with_name("config")
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file"),
        )
        .subcommand(
            SubCommand::with_name("import")
                .about("Import exported Instapaper CSV to pre-fill link log")
                .arg(Arg::with_name("INPUT").required(true).index(1)),
        )
        .get_matches();
    // Config
    let config_path = app.value_of("config").unwrap_or("config.yaml");
    let config = parse_config(config_path).unwrap_or_exit();

    feeds_to_instapaper::run(config, app.subcommand()).unwrap_or_exit();
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
                eprintln!("error: {}", causes.next().unwrap());
                for c in causes {
                    eprintln!(" caused by: {}", c);
                }
                std::process::exit(1);
            }
            Ok(v) => v,
        }
    }
}
