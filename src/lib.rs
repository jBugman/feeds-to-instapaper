extern crate atom_syndication as atom;
extern crate rss;

use std::str::FromStr;
use std::error::Error;

#[derive(Debug)]
pub struct Feed {
    title: String,
    description: Option<String>,
    last_update: Option<String>,
    items: Vec<Item>,
}

#[derive(Debug)]
pub struct Item {
    title: Option<String>,
    pub_date: Option<String>,
    link: Option<String>,
}

impl FromStr for Feed {
    type Err = Box<Error>;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        let s = Syndication::from(source)?;
        let f = Feed::from(s);
        Ok(f)
    }
}

impl From<atom::Feed> for Feed {
    fn from(src: atom::Feed) -> Self {
        Feed {
            title: src.title().to_owned(),
            description: src.subtitle().map(|x| x.to_owned()),
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
            last_update: src.last_build_date().map(|x| x.to_owned()),
            items: src.items().iter().map(Item::from).collect(),
        }
    }
}

impl<'a> From<&'a rss::Item> for Item {
    fn from(src: &rss::Item) -> Self {
        Item {
            title: src.title().map(|x| x.to_owned()),
            pub_date: src.pub_date().map(|x| x.to_owned()),
            link: src.link().map(|x| x.to_owned()),
        }
    }
}

impl<'a> From<&'a atom::Entry> for Item {
    fn from(src: &atom::Entry) -> Self {
        Item {
            title: Some(src.title().to_owned()),
            pub_date: src.published().map(|x| x.to_owned()),
            link: link_from_alternate(src.links()),
        }
    }
}

fn link_from_alternate(links: &[atom::Link]) -> Option<String> {
    for link in links.iter() {
        if link.rel() == "alternate" {
            return Some(link.href().to_owned())
        }
    }
    None
}

// // TODO: replace with functional-style conversion
// fn link_from_content(src: Option<&atom::Content>) -> Option<String> {
//     match src {
//         None => None,
//         Some(c) => c.src().map(|x| x.to_owned()),
//     }
// }

// impl ToOwned for Option<&str> {  // TODO: lifetime
//     fn to_owned(&self) -> Self::Owned {
//         &self.map(|x| x.to_owned())
//     }
// }

#[derive(Debug)]
pub enum Syndication {
    Atom(atom::Feed),
    RSS(rss::Channel),
}

impl Syndication {
    pub fn from(s: &str) -> Result<Syndication, Box<Error>> {
        match s.parse::<atom::Feed>() {
            Ok(feed) => Ok(Syndication::Atom(feed)),
            Err(atom::Error::InvalidStartTag) => match s.parse::<rss::Channel>() {
                Ok(channel) => Ok(Syndication::RSS(channel)),
                Err(err) => Err(err.into()),
            },
            Err(err) => Err(err.into()),
        }
    }
}

impl From<Syndication> for Feed {
    fn from(s: Syndication) -> Self {
        match s {
            Syndication::Atom(x) => Feed::from(x),
            Syndication::RSS(x) => Feed::from(x),
        }
    }
}
