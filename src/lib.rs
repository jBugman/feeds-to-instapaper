#[macro_use]
extern crate serde_derive;
extern crate atom_syndication;
extern crate dialoguer;
extern crate dotenv;
extern crate reqwest;
extern crate rss;
extern crate try_from;
extern crate url;

use std::fs::{File, OpenOptions};
use std::io::{LineWriter, Read, Write};
use std::error::Error;
use std::env;
use std::collections::BTreeSet;
use std::iter::FromIterator;

use dialoguer::Confirmation;
use try_from::TryFrom; // TODO: convert to std(?) TryFrom when stabilized
use url::Url;

pub mod syndication;
pub mod instapaper;

use instapaper::{Client, Link};
use syndication::{Feed, Item};

pub fn run() -> Result<(), Box<Error>> {
    // Config
    dotenv::dotenv()?;
    let instapaper_username = env::var("INSTAPAPER_USERNAME")?;
    let instapaper_password = env::var("INSTAPAPER_PASSWORD")?;
    let links_log_file = env::var("LINKS_LOG_FILE")?;
    let links_list_file = env::var("LINKS_LIST_FILE")?;

    let mut links = Links::from(&links_log_file)?;
    let client = Client::new(&instapaper_username, &instapaper_password);

    let urls = load_link_list(&links_list_file)?;
    for url in urls {
        process_feed(&client, &mut links, &url)?;
    }
    Ok(())
}

fn load_link_list(path: &str) -> Result<Vec<String>, Box<Error>> {
    let mut file = File::open(path)?;
    let mut text = String::new();
    file.read_to_string(&mut text)?;
    Ok(text.lines().map(str::to_owned).collect())
}

fn process_feed(client: &Client, links: &mut Links, url: &str) -> Result<(), Box<Error>> {
    // Downloading feed
    println!("Downloading {}…", url);
    let xml = reqwest::get(url)?.text()?;
    // Parsing
    let feed = xml.parse::<Feed>()?;
    println!("Processing \"{}\"", &feed.title);

    let mut skip_count = 0;
    let print_skips = |count: &mut u16| {
        if *count > 0 {
            println!("> skipped {} already existing links", count);
            *count = 0;
        }
    };

    // get base url for a feed
    let feed_url = feed.link.unwrap_or(url.to_owned());
    let feed_url = Url::parse(&feed_url)?;
    for item in feed.items.into_iter().rev() {
        let link = Link::try_from(item)?.fix_url_schema(&feed_url)?;
        // skipping if already added
        if links.saved(&link.url) {
            skip_count += 1;
            continue;
        }
        print_skips(&mut skip_count);

        let name = link.title.as_ref().unwrap_or(&link.url);
        if Confirmation::new(&format!("Add \"{}\"?", name)).interact()? {
            println!("Adding to Instapaper…");
            let success = client.add_link(&link)?;
            if success {
                println!("> done");
                links.add(&link.url)?;
            }
        } else {
            links.add(&link.url)?;
            println!("> marked {} as skipped", &link.url);
        }
    }
    print_skips(&mut skip_count);

    Ok(())
}

impl TryFrom<Item> for Link {
    type Err = Box<Error>;

    fn try_from(src: Item) -> Result<Self, Self::Err> {
        let link = src.link.ok_or("link was not present")?;
        // TODO: replace with .filter when stabilized
        let title = src.title.into_iter().find(|s| !s.is_empty()); // dropping empty titles
        Ok(Link { url: link, title })
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
