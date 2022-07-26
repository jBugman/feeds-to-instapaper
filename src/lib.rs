#[macro_use]
extern crate serde_derive;

use std::collections::BTreeSet;
use std::convert::TryFrom;
use std::fs::{read_to_string, File, OpenOptions};
use std::io::{LineWriter, Read, Write};
use std::iter::FromIterator;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use yansi::Paint;

pub mod instapaper;
pub mod syndication;

use crate::instapaper::{Credentials, Link};

#[derive(Debug, Deserialize)]
pub struct Config {
    instapaper: Credentials,
    pub log_file: String,
    urls: Vec<String>,
    #[serde(skip)]
    pub auto_add: bool,
    #[serde(skip)]
    pub skip_download_errors: bool,
}

impl Config {
    pub fn new(path: impl AsRef<Path> + std::fmt::Display) -> Result<Self> {
        let yaml =
            read_to_string(&path).context(format!("failed to read config file: {}", path))?;
        serde_yaml::from_str(&yaml).context("failed to parse config")
    }
}

pub fn run_link_processing(config: Config, links: &mut Links) -> Result<()> {
    let client = instapaper::Client::new(config.instapaper);

    for url in config.urls {
        if let Err(err) = process_feed(&client, links, config.auto_add, &url) {
            if config.skip_download_errors {
                eprintln!("{} {}", Paint::yellow("error:"), err);
            } else {
                return Err(err);
            }
        }
    }
    Ok(())
}

pub fn run_import(links: &mut Links, csv_path: &str) -> Result<()> {
    let mut csv_reader = csv::Reader::from_path(csv_path)
        .context(format!("failed to read csv file: {}", csv_path))?;
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

fn process_feed(
    client: &instapaper::Client,
    links: &mut Links,
    auto_add: bool,
    url: &str,
) -> Result<()> {
    // Downloading feed
    println!("Downloading {}", Paint::white(url));

    let xml = reqwest::blocking::get(url)
        .context(format!("failed to download feed: {}", url))?
        .text()?;
    // Parsing
    let feed = xml
        .parse::<syndication::Feed>()
        .context("failed to parse feed")?;
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
    let feed_url =
        url::Url::parse(&feed_url).context(format!("failed to parse url: {}", feed_url))?;
    for item in feed.items.into_iter().rev() {
        let link = Link::try_from(item)?.fix_url_schema(&feed_url)?;
        // skipping if already added
        if links.saved(&link.url) {
            skip_count += 1;
            continue;
        }
        print_skips(&mut skip_count);

        let name = link.title.as_ref().unwrap_or(&link.url);
        let mut add = auto_add;
        if !auto_add {
            add = dialoguer::Confirm::new()
                .with_prompt(&format!(
                    "{}Add \"{}\"?",
                    Paint::masked("📎  "),
                    Paint::white(name)
                ))
                .interact()?
        }
        if add {
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

impl TryFrom<syndication::Item> for Link {
    type Error = anyhow::Error;

    fn try_from(src: syndication::Item) -> Result<Self> {
        let link = src.link.ok_or_else(|| anyhow!("url is missing in post"))?;
        let title = src.title.filter(|s| !s.is_empty());
        Ok(Link { url: link, title })
    }
}

#[derive(Debug)]
pub struct Links {
    pub items: BTreeSet<String>,
    file: LineWriter<File>,
}

impl Links {
    pub fn from(path: &str) -> Result<Self> {
        // expanding home directory in path
        let path: &str = &shellexpand::tilde(path);
        let path = Path::new(path);
        // ensuring log file directory
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).context(format!(
                "failed to create parent dir for a log file: {:?}",
                parent
            ))?;
        }
        // open or create file
        let mut file = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(path)
            .context(format!("failed to open log file: {:?}", path))?;
        // read
        let mut text = String::new();
        file.read_to_string(&mut text)
            .context(format!("failed to read log file: {:?}", path))?;
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
