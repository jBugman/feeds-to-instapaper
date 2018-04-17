#[macro_use]
extern crate serde_derive;

extern crate dotenv;

use std::fs::File;
use std::io::Read;
use std::error::Error;
use std::env;

pub mod syndication;
pub mod instapaper;

use instapaper::URL;

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

    let _feed = text.parse::<syndication::Feed>()?;
    // println!("{:#?}", _feed);

    let client = instapaper::Client::new(&cfg.instapaper_username, &cfg.instapaper_password);

    let item = syndication::Item {
        link: Some(String::from("https://www.lipsum.com/feed/html")),
        pub_date: None,
        title: Some(String::from(
            "Neque porro quisquam est qui dolorem ipsum quia dolor sit amet, consectetur, adipisci velit..",
        )),
    };
    let url = URL::try_from(item)?;
    let status = client.add(&url)?;
    println!("add: {}", status);

    Ok(())
}

impl URL {
    // convert to TryFrom when stabilized.
    pub fn try_from(src: syndication::Item) -> Result<URL, Box<Error>> {
        if let Some(url) = src.link {
            let u = URL {
                url,
                title: src.title,
            };
            return Ok(u);
        }
        Err("link was not present".into())
    }
}
