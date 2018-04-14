extern crate dotenv;

use std::fs::File;
use std::io::Read;
use std::error::Error;
use std::env;

pub mod syndication;

#[derive(Debug)]
pub struct Config {
    instapaper_username: String,
    instapaper_password: String,
}

impl Config {
    pub fn new() -> Result<Self, Box<Error>> {
        dotenv::dotenv()?;
        let instapaper_username = env::var("INSTAPAPER_USERNAME")?;
        let instapaper_password = env::var("INSTAPAPER_PASSWORD")?;
        Ok(Config {
            instapaper_username,
            instapaper_password,
        })
    }
}

pub fn run(cfg: &Config) -> Result<(), Box<Error>> {
    println!("{:#?}", cfg);

    let _filename = "samples/junk.xml"; // should fail
    let _filename = "samples/ghc.xml"; //  RSS
    let _filename = "samples/pike.xml"; // Atom

    let mut file = File::open(_filename)?;
    let mut text = String::new();
    file.read_to_string(&mut text)?;

    let feed = text.parse::<syndication::Feed>()?;
    println!("{:#?}", feed);
    Ok(())
}
