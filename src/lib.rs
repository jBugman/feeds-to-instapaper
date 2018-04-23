#[macro_use]
extern crate serde_derive;
extern crate atom_syndication;
extern crate dialoguer;
extern crate dotenv;
extern crate reqwest;
extern crate rss;

use std::fs::{File, OpenOptions};
use std::io::{LineWriter, Read, Write};
use std::error::Error;
use std::env;
use std::collections::BTreeSet;
use std::iter::FromIterator;
use dialoguer::Confirmation;

pub mod syndication;
pub mod instapaper;

use instapaper::{Client, URL};
use syndication::{Feed, Item};

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
    let mut links = Links::from(&cfg.links_log_file)?;
    let client = Client::new(&cfg.instapaper_username, &cfg.instapaper_password);

    let urls = vec![
        // TODO: load list from file
        "https://blog.rust-lang.org/feed.xml",
    ];
    for url in urls {
        println!("Loading {}...", url);
        let xml = reqwest::get(url)?.text()?;
        process_feed(&client, &mut links, &xml)?;
    }
    Ok(())
}

fn process_feed(client: &Client, links: &mut Links, xml: &str) -> Result<(), Box<Error>> {
    let feed: Feed = xml.parse()?;
    println!("Processing \"{}\"", &feed.title);

    let mut skip_count = 0;
    let print_skips = |count: &mut u16| {
        if *count > 0 {
            println!("> skipped {} already existing links", count);
            *count = 0;
        }
    };

    for item in feed.items.into_iter().rev() {
        let u = URL::try_from(item)?;
        // skipping if already added
        if links.saved(&u.url) {
            skip_count += 1;
            continue;
        }
        print_skips(&mut skip_count);

        let name: &str = u.title.as_ref().unwrap_or(&u.url);
        if Confirmation::new(&format!("Add \"{}\"?", name)).interact()? {
            println!("Adding to instapaper...");
            let success = client.add_link(&u)?;
            if success {
                println!("> done");
                links.add(&u.url)?;
            }
        } else {
            links.add(&u.url)?;
            println!("> marked {} as skipped", &u.url);
        }
    }
    print_skips(&mut skip_count);

    Ok(())
}

impl URL {
    // TODO: convert to TryFrom when stabilized.
    pub fn try_from(src: Item) -> Result<URL, Box<Error>> {
        if let Some(url) = src.link {
            let u = URL {
                url,
                // TODO: replace with .filter when stabilized
                title: src.title.into_iter().filter(|s| !s.is_empty()).next(), // dropping empty titles
            };
            return Ok(u);
        }
        Err("link was not present".into())
    }
}

#[derive(Debug)]
struct Links {
    pub items: BTreeSet<String>,
    file: LineWriter<File>,
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
