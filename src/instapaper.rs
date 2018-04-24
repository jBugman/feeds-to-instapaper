use std::error::Error;

use reqwest;
use reqwest::StatusCode;
use url::{ParseError, Url};

const BASE_URL: &str = "https://www.instapaper.com/api/";

pub struct Client {
    client: reqwest::Client,
    base_url: Url,
    username: String,
    password: String,
}

#[derive(Debug, Serialize)]
pub struct Link {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

impl Link {
    // fixes url schema using feed url as a template
    pub fn fix_url_schema(mut self, feed_url: &Url) -> Result<Self, Box<Error>> {
        match Url::parse(&self.url) {
            Ok(_) => Ok(self),
            Err(ParseError::RelativeUrlWithoutBase) => {
                let url = feed_url.join(&self.url)?;
                self.url = url.into_string();
                Ok(self)
            }
            Err(err) => Err(err.into()),
        }
    }
}

impl Client {
    pub fn new(username: &str, password: &str) -> Client {
        let base_url = reqwest::Url::parse(BASE_URL).expect("typo in constant");
        Client {
            client: reqwest::Client::new(),
            base_url,
            username: username.to_owned(),
            password: password.to_owned(),
        }
    }

    pub fn validate_credentials(&self) -> Result<bool, Box<Error>> {
        let url = self.base_url.join("authenticate")?;
        let res = self.client
            .post(url)
            .basic_auth(self.username.to_owned(), Some(self.password.to_owned()))
            .send()?;
        Ok(res.status() == StatusCode::Ok)
    }

    pub fn add_link(&self, link: &Link) -> Result<bool, Box<Error>> {
        let url = self.base_url.join("add")?;
        let res = self.client
            .post(url)
            .basic_auth(self.username.to_owned(), Some(self.password.to_owned()))
            .form(link)
            .send()?;
        Ok(res.status() == StatusCode::Created)
        // TODO: use returned saved link somehow?
    }
}
