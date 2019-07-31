use failure::{Fail, ResultExt};
use reqwest;
use url::{ParseError, Url};

use failure_ext::{FmtResultExt, Result};
const BASE_URL: &str = "https://www.instapaper.com/api/";

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

#[derive(Debug, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

trait UsingCredentials {
    fn using(&mut self, c: &Credentials) -> &mut reqwest::RequestBuilder;
}

impl UsingCredentials for reqwest::RequestBuilder {
    fn using(&mut self, c: &Credentials) -> &mut reqwest::RequestBuilder {
        self.basic_auth(c.username.clone(), Some(c.password.clone()))
    }
}

pub struct Client {
    client: reqwest::Client,
    base_url: Url,
    credentials: Credentials,
}

impl Client {
    pub fn new(credentials: Credentials) -> Client {
        let base_url = Url::parse(BASE_URL).expect("typo in constant");
        Client {
            client: reqwest::Client::new(),
            base_url,
            credentials,
        }
    }

    pub fn validate_credentials(&self) -> Result<()> {
        let url = self.base_url.join("authenticate")?;
        self.client
            .post(url)
            .using(&self.credentials)
            .send()
            .context("error accessing instapaper api")?
            .error_for_status()
            .context("could not validate instapaper credentials")?;
        Ok(())
    }

    pub fn add_link(&self, link: &Link) -> Result<()> {
        let url = self.base_url.join("add")?;
        self.client
            .post(url)
            .using(&self.credentials)
            .form(link)
            .send()
            .context("error accessing instapaper api")?
            .error_for_status()
            .context_fmt("could not post new link to instapaper", &link.url)?;
        Ok(())
    }
}
