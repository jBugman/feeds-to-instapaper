extern crate feeds_to_instapaper;

fn main() {
    if let Err(err) = feeds_to_instapaper::run() {
        eprintln!("error: {}", err);
        std::process::exit(1);
    };
}
