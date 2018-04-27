extern crate atom_syndication;
extern crate dialoguer;
extern crate dotenv;
#[macro_use]
extern crate failure;
extern crate reqwest;
extern crate rss;
#[macro_use]
extern crate serde_derive;
extern crate shellexpand;
extern crate try_from;
extern crate url;

use std::collections::BTreeSet;
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{LineWriter, Read, Write};
use std::iter::FromIterator;
use std::path::Path;

use dialoguer::Confirmation;
use failure::{Error, ResultExt, SyncFailure};
use try_from::TryFrom; // TODO: convert to std(?) TryFrom when stabilized
use url::Url;

pub mod syndication;
pub mod instapaper;

use instapaper::{Client, Credentials, Link};
use syndication::{Feed, Item};

type Result<T> = std::result::Result<T, Error>;

pub fn run() -> Result<()> {
    // Config
    dotenv::dotenv()
        .map_err(SyncFailure::new)
        .context("failed to read .env file")?;
    let instapaper_username =
        env::var("INSTAPAPER_USERNAME").context("instapaper username is not set")?;
    let instapaper_password =
        env::var("INSTAPAPER_PASSWORD").context("instapaper password is not set")?;
    let links_log_file = env::var("LINKS_LOG_FILE").context("log file path is not set")?;
    let links_list_file = env::var("LINKS_LIST_FILE").context("links list file path is not set")?;

    let mut links = Links::from(&links_log_file)?;
    let client = Client::new(Credentials::new(instapaper_username, instapaper_password));

    let urls = load_link_list(&links_list_file)?;
    for url in urls {
        process_feed(&client, &mut links, &url)?;
    }
    Ok(())
}

// TODO: replace when stabilized
fn read_to_string<P: AsRef<Path>>(path: P) -> Result<String> {
    let mut file = File::open(path)?;
    let buf_size = file.metadata().map(|m| m.len() as usize + 1).unwrap_or(0);
    let mut string = String::with_capacity(buf_size);
    file.read_to_string(&mut string)?;
    Ok(string)
}

fn load_link_list(path: &str) -> Result<Vec<String>> {
    let text =
        read_to_string(path).context(format_err!("failed to read link list file ({})", path))?;
    Ok(text.lines().map(str::to_owned).collect())
}

fn process_feed(client: &Client, links: &mut Links, url: &str) -> Result<()> {
    // Downloading feed
    println!("Downloading {}…", url);
    let xml = reqwest::get(url)
        .context(format_err!("failed to download feed ({})", url))?
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
    let feed_url =
        Url::parse(&feed_url).context(format_err!("failed to parse url ({})", feed_url))?;
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
        let link = src.link
            .ok_or_else(|| format_err!("url is missing in post"))?;
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
            std::fs::create_dir_all(parent).context(format_err!(
                "failed to create parent dir for a log file ({})",
                parent.display()
            ))?;
        }
        // open or create file
        let mut file = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(path)
            .context(format_err!("failed to open log file ({})", path.display()))?;
        // read
        let mut text = String::new();
        file.read_to_string(&mut text)
            .context(format_err!("failed to read log file ({})", path.display()))?;
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
