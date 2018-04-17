extern crate atom_syndication as atom;
extern crate rss;

use std::str::FromStr;
use std::error::Error;

#[derive(Debug)]
pub struct Feed {
    pub title: String,
    pub description: Option<String>,
    pub last_update: Option<String>,
    pub items: Vec<Item>,
}

#[derive(Debug)]
pub struct Item {
    pub title: Option<String>,
    pub pub_date: Option<String>,
    pub link: Option<String>,
}

impl FromStr for Feed {
    type Err = Box<Error>;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        rss::Channel::from_str(src)
            .map(Feed::from)
            .or_else(|err| match err {
                rss::Error::InvalidStartTag => atom::Feed::from_str(src)
                    .map(Feed::from)
                    .map_err(Self::Err::from),
                e => Err(e).map_err(Self::Err::from),
            })
    }
}

impl From<atom::Feed> for Feed {
    fn from(src: atom::Feed) -> Self {
        Feed {
            title: src.title().to_owned(),
            description: src.subtitle().map(str::to_owned),
            last_update: Some(src.updated().to_owned()),
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
            items: src.items().iter().map(Item::from).collect(),
        }
    }
}

impl<'a> From<&'a rss::Item> for Item {
    fn from(src: &rss::Item) -> Self {
        Item {
            title: src.title().map(str::to_owned),
            pub_date: src.pub_date().map(str::to_owned),
            link: src.link().map(str::to_owned),
        }
    }
}

impl<'a> From<&'a atom::Entry> for Item {
    fn from(src: &atom::Entry) -> Self {
        Item {
            title: Some(src.title().to_owned()),
            pub_date: src.published().map(str::to_owned),
            link: src.links()
                .iter()
                .find(|link| link.rel() == "alternate")
                .map(|link| link.href().to_owned()),
        }
    }
}
