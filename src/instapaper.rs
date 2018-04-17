extern crate reqwest;

use std::error::Error;
use self::reqwest::StatusCode;

const BASE_URL: &str = "https://www.instapaper.com/api/";

pub struct Client {
    client: reqwest::Client,
    base_url: reqwest::Url,
    username: String,
    password: String,
}

#[derive(Debug, Serialize)]
pub struct URL {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
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

    pub fn validate_credentials(self) -> Result<bool, Box<Error>> {
        let url = self.base_url.join("authenticate")?;
        let res = self.client
            .post(url)
            .basic_auth(self.username, Some(self.password))
            .send()?;
        Ok(res.status() == StatusCode::Ok)
    }

    pub fn add_link(self, u: &URL) -> Result<bool, Box<Error>> {
        println!("{:?}", u);
        let url = self.base_url.join("add")?;
        let res = self.client
            .post(url)
            .basic_auth(self.username, Some(self.password))
            .form(u)
            .send()?;
        Ok(res.status() == StatusCode::Created)
        // TODO: use returned saved link somehow?
    }
}
