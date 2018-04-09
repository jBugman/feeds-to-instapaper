extern crate feeds_to_instapaper;

use std::fs::File;
use std::io::Read;
use std::error::Error;

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {}", err);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<Error>> {
    let _filename = "samples/junk.xml"; // should fail
    let _filename = "samples/ghc.xml"; //  RSS
    let _filename = "samples/pike.xml"; // Atom

    let mut file = File::open(_filename)?;
    let mut text = String::new();
    file.read_to_string(&mut text)?;

    let feed = text.parse::<feeds_to_instapaper::Feed>()?;
    println!("{:#?}", feed);
    Ok(())
}
