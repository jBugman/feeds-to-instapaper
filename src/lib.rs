extern crate atom_syndication;
extern crate clap;
extern crate csv;
extern crate dialoguer;
#[macro_use]
extern crate failure;
extern crate reqwest;
extern crate rss;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
extern crate shellexpand;
extern crate try_from;
extern crate url;
extern crate yansi;

use std::collections::BTreeSet;
use std::fs::{File, OpenOptions};
use std::io::{LineWriter, Read, Write};
use std::iter::FromIterator;
use std::path::Path;

pub use try_from::TryFrom; // TODO: (Rust 1.27+) replace with std (https://github.com/rust-lang/rust/issues/33417)
use dialoguer::Confirmation;
use failure::{Error, ResultExt};
use url::Url;
use yansi::Paint;

pub mod syndication;
pub mod instapaper;
pub mod failure_ext;

use failure_ext::*;
use instapaper::{Client, Credentials, Link};
use syndication::{Feed, Item};

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Deserialize)]
pub struct Config {
    instapaper: Credentials,
    log_file: String,
    urls: Vec<String>,
}

impl<'a> TryFrom<&'a str> for Config {
    type Err = Error;

    fn try_from(src: &str) -> Result<Self> {
        serde_yaml::from_str(src)
            .context("failed to parse config")
            .map_err(Error::from)
    }
}

type Subcommand<'a> = (&'a str, Option<&'a clap::ArgMatches<'a>>);

pub fn run(config: Config, subcommand: Subcommand) -> Result<()> {
    // Loading already added links
    let mut links = Links::from(&config.log_file)?;
    // Dispatching subcommands
    match subcommand {
        ("import", Some(args)) => {
            let csv_path = args.value_of("INPUT").unwrap();
            run_import(&mut links, csv_path)
        }
        _ => run_link_processing(config, &mut links), // Processing links by default
    }
}

fn run_link_processing(config: Config, links: &mut Links) -> Result<()> {
    let client = Client::new(config.instapaper);

    for url in config.urls {
        process_feed(&client, links, &url)?;
    }
    Ok(())
}

fn run_import(links: &mut Links, csv_path: &str) -> Result<()> {
    let mut csv_reader =
        csv::Reader::from_path(csv_path).context_fmt("failed to read csv file", csv_path)?;
    let mut existed = 0u16;
    let mut total = 0u16;
    for r in csv_reader.records() {
        let line = r.context("failed to parse csv record")?;
        if let Some(url) = line.get(0) {
            if !links.saved(url) {
                // println!("Importing {}", url);
                links.add(url)?;
            } else {
                existed += 1;
            }
        }
        total += 1;
    }
    println!(
        "{} imported: {}, duplicates: {}",
        Paint::green("Successfully"),
        total - existed,
        existed
    );
    Ok(())
}

fn process_feed(client: &Client, links: &mut Links, url: &str) -> Result<()> {
    // Downloading feed
    println!("Downloading {}{}", Paint::white(url), Paint::masked(" 🕓"));
    let xml = reqwest::get(url)
        .context_fmt("failed to download feed", url)?
        .text()?;
    // Parsing
    let feed = xml.parse::<Feed>().context("failed to parse feed")?;
    println!("Processing \"{}\"", Paint::white(&feed.title));

    let mut skip_count = 0;
    let print_skips = |count: &mut u16| {
        if *count > 0 {
            println!("Skipped pre-existing links ({})", count);
            *count = 0;
        }
    };

    // get base url for a feed
    let feed_url = feed.link.unwrap_or_else(|| url.to_owned());
    let feed_url = Url::parse(&feed_url).context_fmt("failed to parse url", feed_url)?;
    for item in feed.items.into_iter().rev() {
        let link = Link::try_from(item)?.fix_url_schema(&feed_url)?;
        // skipping if already added
        if links.saved(&link.url) {
            skip_count += 1;
            continue;
        }
        print_skips(&mut skip_count);

        let name = link.title.as_ref().unwrap_or(&link.url);
        if Confirmation::new(&format!(
            "{}Add \"{}\"?",
            Paint::masked("📎  "),
            Paint::white(name)
        )).interact()?
        {
            println!("Adding {} to Instapaper", Paint::white(&link.url));
            client.add_link(&link)?;
            println!("{} added", Paint::green("Successfully"));
            links.add(&link.url)?;
        } else {
            links.add(&link.url)?;
            println!("Marked {} as skipped", Paint::white(&link.url));
        }
    }
    print_skips(&mut skip_count);

    Ok(())
}

impl TryFrom<Item> for Link {
    type Err = Error;

    fn try_from(src: Item) -> Result<Self> {
        let link = src.link.or_fail("url is missing in post")?;
        // TODO: (Rust 1.26) replace with .filter
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
    fn from(path: &str) -> Result<Self> {
        // expanding home directory in path
        let path: &str = &shellexpand::tilde(path);
        let path = Path::new(path);
        // ensuring log file directory
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .context_path("failed to create parent dir for a log file", parent)?;
        }
        // open or create file
        let mut file = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(path)
            .context_path("failed to open log file", path)?;
        // read
        let mut text = String::new();
        file.read_to_string(&mut text)
            .context_path("failed to read log file", path)?;
        let items = BTreeSet::from_iter(text.lines().map(str::to_owned));
        // set up writing
        let file = LineWriter::new(file);
        Ok(Links { items, file })
    }

    fn add(&mut self, item: &str) -> Result<()> {
        let existed = !self.items.insert(item.to_owned());
        if !existed {
            writeln!(self.file, "{}", item).context("failed to write an url to a log file")?;
        }
        Ok(())
    }

    fn saved(&self, item: &str) -> bool {
        self.items.contains(item)
    }
}
