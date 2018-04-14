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
        // println!("{:?}", res);
        Ok(res.status() == StatusCode::Ok)
    }
}
