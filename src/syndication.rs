use std::str::FromStr;

use anyhow::{Context, Error, Result};
use atom_syndication as atom;
use rss;

#[derive(Debug)]
pub struct Feed {
    pub title: String,
    pub description: Option<String>,
    pub last_update: Option<String>,
    pub link: Option<String>,
    pub items: Vec<Item>,
}

#[derive(Debug)]
pub struct Item {
    pub title: Option<String>,
    pub pub_date: Option<String>,
    pub link: Option<String>,
}

impl FromStr for Feed {
    type Err = anyhow::Error;

    fn from_str(src: &str) -> Result<Self> {
        match rss::Channel::from_str(src) {
            Ok(chan) => Ok(Feed::from(chan)),
            Err(rss::Error::InvalidStartTag) => atom::Feed::from_str(src)
                .map(Feed::from)
                .map_err(Error::from)
                .map_err(|e| e.context("failed to parse atom xml")),
            Err(err) => Err(err).context("failed to parse rss xml"),
        }
    }
}

impl From<atom::Feed> for Feed {
    fn from(src: atom::Feed) -> Self {
        Feed {
            title: src.title().to_owned(),
            description: src.subtitle().map(str::to_owned),
            last_update: Some(src.updated().to_rfc3339()),
            link: find_alternate(src.links()),
            items: src.entries().iter().map(Item::from).collect(),
        }
    }
}

impl From<rss::Channel> for Feed {
    fn from(src: rss::Channel) -> Self {
        Feed {
            title: src.title().to_owned(),
            description: Some(src.description().to_owned()),
            last_update: src.last_build_date().map(str::to_owned),
            link: Some(src.link().to_owned()),
            items: src.items().iter().map(Item::from).collect(),
        }
    }
}

impl From<&rss::Item> for Item {
    fn from(src: &rss::Item) -> Self {
        Item {
            title: src.title().map(str::to_owned),
            pub_date: src.pub_date().map(str::to_owned),
            link: src.link().map(str::to_owned),
        }
    }
}

impl From<&atom::Entry> for Item {
    fn from(src: &atom::Entry) -> Self {
        Item {
            title: Some(src.title().to_owned()),
            pub_date: src.published().map(atom::FixedDateTime::to_rfc3339),
            link: find_alternate(src.links()),
        }
    }
}

fn find_alternate(links: &[atom::Link]) -> Option<String> {
    links
        .iter()
        .find(|link| link.rel() == "alternate" || link.rel().is_empty())
        .map(|link| link.href().to_owned())
}
