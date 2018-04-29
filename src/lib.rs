extern crate atom_syndication;
#[macro_use]
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

use std::collections::BTreeSet;
use std::fs::{File, OpenOptions};
use std::io::{LineWriter, Read, Write};
use std::iter::FromIterator;
use std::path::Path;

use clap::{App, Arg, SubCommand};
use dialoguer::Confirmation;
use failure::{Error, ResultExt};
use try_from::TryFrom; // TODO: convert to std(?) TryFrom when stabilized
use url::Url;

pub mod syndication;
pub mod instapaper;
mod failure_ext;

use failure_ext::*;
use instapaper::{Client, Credentials, Link};
use syndication::{Feed, Item};

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Deserialize)]
struct Config {
    instapaper: Credentials,
    log_file: String,
    urls: Vec<String>,
}

pub fn run() -> Result<()> {
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
    let config_file = app.value_of("config").unwrap_or("config.yaml");
    let config =
        read_to_string(&config_file).context_fmt("failed to read config file", &config_file)?;
    let config: Config = serde_yaml::from_str(&config).context("failed to parse config")?;
    // Loading already added links
    let mut links = Links::from(&config.log_file)?;
    // Dispatching subcommands
    match app.subcommand() {
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
    println!("> imported: {}, duplicates: {}", total - existed, existed);
    Ok(())
}

// TODO: replace when stabilized
fn read_to_string<P: AsRef<Path>>(path: P) -> std::result::Result<String, std::io::Error> {
    let mut file = File::open(path)?;
    let buf_size = file.metadata().map(|m| m.len() as usize + 1).unwrap_or(0);
    let mut string = String::with_capacity(buf_size);
    file.read_to_string(&mut string)?;
    Ok(string)
}

fn process_feed(client: &Client, links: &mut Links, url: &str) -> Result<()> {
    // Downloading feed
    println!("Downloading {}…", url);
    let xml = reqwest::get(url)
        .context_fmt("failed to download feed", url)?
        .text()?;
    // Parsing
    let feed = xml.parse::<Feed>().context("failed to parse feed")?;
    println!("Processing \"{}\"", &feed.title);

    let mut skip_count = 0;
    let print_skips = |count: &mut u16| {
        if *count > 0 {
            println!("> skipped {} already existing links", count);
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
        if Confirmation::new(&format!("Add \"{}\"?", name)).interact()? {
            println!("Adding to Instapaper…");
            client.add_link(&link)?;
            println!("> done");
            links.add(&link.url)?;
        } else {
            links.add(&link.url)?;
            println!("> marked {} as skipped", &link.url);
        }
    }
    print_skips(&mut skip_count);

    Ok(())
}

impl TryFrom<Item> for Link {
    type Err = Error;

    fn try_from(src: Item) -> Result<Self> {
        let link = src.link.or_fail("url is missing in post")?;
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
