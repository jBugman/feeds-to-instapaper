use std::str::FromStr;
use std::error::Error;

use atom_syndication as atom;
use rss;
use url::Url;
use try_from::TryFrom;

#[derive(Debug)]
pub struct Feed {
    pub title: String,
    pub description: Option<String>,
    pub last_update: Option<String>, // TODO: convert to a date?
    pub items: Vec<Item>,
}

#[derive(Debug)]
pub struct Item {
    pub title: Option<String>,
    pub pub_date: Option<String>, // TODO: convert to a date?
    pub link: Option<String>,     // TODO: in process of converting to an url
    url: Option<Url>,             // TODO: ^
}

impl FromStr for Feed {
    type Err = Box<Error>;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        match rss::Channel::from_str(src) {
            Ok(chan) => Feed::try_from(chan),
            Err(rss::Error::InvalidStartTag) => match atom::Feed::from_str(src) {
                Ok(feed) => Feed::try_from(feed),
                Err(err) => Err(err.into()),
            },
            Err(err) => Err(err.into()),
        }
    }
}

impl TryFrom<atom::Feed> for Feed {
    type Err = Box<Error>;

    fn try_from(src: atom::Feed) -> Result<Self, Self::Err> {
        let title = src.title().to_owned();
        let description = src.subtitle().map(str::to_owned);
        let last_update = Some(src.updated().to_owned());
        let items: Result<Vec<Item>, _> = src.entries().into_iter().map(Item::try_from).collect();
        let items = items?;
        Ok(Feed {
            title,
            description,
            last_update,
            items,
        })
    }
}

impl TryFrom<rss::Channel> for Feed {
    type Err = Box<Error>;

    fn try_from(src: rss::Channel) -> Result<Self, Self::Err> {
        let title = src.title().to_owned();
        let description = Some(src.description().to_owned());
        let last_update = src.last_build_date().map(str::to_owned);
        let items: Result<Vec<Item>, _> = src.items().into_iter().map(Item::try_from).collect();
        let items = items?;
        Ok(Feed {
            title,
            description,
            last_update,
            items,
        })
    }
}

impl<'a> TryFrom<&'a rss::Item> for Item {
    type Err = Box<Error>;

    fn try_from(src: &rss::Item) -> Result<Self, Self::Err> {
        let url = parse_url(src.link())?;
        let result = Item {
            title: src.title().map(str::to_owned),
            pub_date: src.pub_date().map(str::to_owned),
            link: src.link().map(str::to_owned),
            url,
        };
        Ok(result)
    }
}

impl<'a> TryFrom<&'a atom::Entry> for Item {
    type Err = Box<Error>;

    fn try_from(src: &atom::Entry) -> Result<Self, Self::Err> {
        let link = src.links()
            .iter()
            .find(|link| link.rel() == "alternate")
            .map(|link| link.href());
        let url = parse_url(link)?;
        let result = Item {
            title: Some(src.title().to_owned()),
            pub_date: src.published().map(str::to_owned),
            link: link.map(str::to_owned),
            url,
        };
        Ok(result)
    }
}

fn parse_url(src: Option<&str>) -> Result<Option<Url>, Box<Error>> {
    match src {
        Some(link) => Url::parse(link).map(Some).map_err(|e| e.into()),
        None => Ok(None),
    }
}
