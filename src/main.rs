extern crate feeds_to_instapaper as feeds;

fn main() {
    let config = feeds::Config::new().unwrap_or_else(|err| {
        eprintln!("config error: {}", err);
        std::process::exit(1);
    });
    if let Err(err) = feeds::run(&config) {
        eprintln!("error: {}", err);
        std::process::exit(1);
    };
}
