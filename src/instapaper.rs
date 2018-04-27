use failure::{Fail, ResultExt};
use reqwest;
use reqwest::StatusCode;
use url::{ParseError, Url};

use ::Result;

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
    pub fn fix_url_schema(mut self, feed_url: &Url) -> Result<Self> {
        match Url::parse(&self.url) {
            Ok(_) => Ok(self),
            Err(ParseError::RelativeUrlWithoutBase) => {
                let url = feed_url.join(&self.url).context("failed to fix post url")?;
                self.url = url.into_string();
                Ok(self)
            }
            Err(err) => Err(err.context("failed to parse post url"))?,
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

    pub fn validate_credentials(&self) -> Result<bool> {
        let url = self.base_url.join("authenticate")?;
        let res = self.client
            .post(url)
            .basic_auth(self.username.to_owned(), Some(self.password.to_owned()))
            .send()
            .context("could not validate instapaper credentials")?;
        Ok(res.status() == StatusCode::Ok)
    }

    pub fn add_link(&self, link: &Link) -> Result<bool> {
        let url = self.base_url.join("add")?;
        let res = self.client
            .post(url)
            .basic_auth(self.username.to_owned(), Some(self.password.to_owned()))
            .form(link)
            .send()
            .context(format_err!(
                "could not post new link to instapaper ({})",
                link.url
            ))?;
        Ok(res.status() == StatusCode::Created)
        // TODO: use returned saved link somehow?
    }
}
