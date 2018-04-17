#[macro_use]
extern crate serde_derive;

extern crate dotenv;

use std::fs::{File, OpenOptions};
use std::io::{LineWriter, Read, Write};
use std::error::Error;
use std::env;
use std::collections::BTreeSet;
use std::iter::FromIterator;

pub mod syndication;
pub mod instapaper;

use instapaper::URL;

#[derive(Debug)]
pub struct Config {
    instapaper_username: String,
    instapaper_password: String,
    links_log_file: String,
}

impl Config {
    pub fn new() -> Result<Self, Box<Error>> {
        dotenv::dotenv()?;
        let instapaper_username = env::var("INSTAPAPER_USERNAME")?;
        let instapaper_password = env::var("INSTAPAPER_PASSWORD")?;
        let links_log_file = env::var("LINKS_LOG_FILE")?;
        Ok(Config {
            instapaper_username,
            instapaper_password,
            links_log_file,
        })
    }
}

pub fn run(cfg: &Config) -> Result<(), Box<Error>> {
    // println!("{:#?}", cfg);

    let _filename = "samples/junk.xml"; // should fail
    let _filename = "samples/ghc.xml"; //  RSS
    let _filename = "samples/pike.xml"; // Atom

    let mut file = File::open(_filename)?;
    let mut text = String::new();
    file.read_to_string(&mut text)?;

    let _feed = text.parse::<syndication::Feed>()?;
    // println!("{:#?}", _feed);

    let _client = instapaper::Client::new(&cfg.instapaper_username, &cfg.instapaper_password);

    let item = syndication::Item {
        link: Some(String::from("https://www.lipsum.com/feed/html")),
        pub_date: None,
        title: Some(String::from(
            "Neque porro quisquam est qui dolorem ipsum quia dolor sit amet, consectetur, adipisci velit..",
        )),
    };
    let _url = URL::try_from(item)?;

    let mut links = Links::from(&cfg.links_log_file)?;

    if links.saved(&_url.url) {
        println!("link already exists: {}", &_url.url);
    } else {
        let success = _client.add_link(&_url)?;
        println!("added: {}", success);
        if success {
            links.add(&_url.url)?;
        }
    }

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

#[derive(Debug)]
struct Links {
    pub items: BTreeSet<String>,
    file: std::io::LineWriter<File>,
}

impl Links {
    fn from(filename: &str) -> Result<Self, Box<Error>> {
        // reading
        let mut file = File::open(filename)?;
        let mut text = String::new();
        file.read_to_string(&mut text)?;
        let items = BTreeSet::from_iter(text.lines().map(str::to_owned));
        // writing
        let file = OpenOptions::new().append(true).open(filename)?;
        let file = LineWriter::new(file);
        Ok(Links { items, file })
    }

    fn add(&mut self, item: &str) -> Result<(), Box<Error>> {
        let existed = !self.items.insert(item.to_owned());
        if !existed {
            writeln!(self.file, "{}", item)?;
        }
        Ok(())
    }

    fn saved(&self, item: &str) -> bool {
        self.items.contains(item)
    }
}
