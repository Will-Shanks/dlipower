use md5::{Digest, Md5};
use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum Status {
    ON,
    OFF,
}

impl FromStr for Status {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "ON" => Ok(Self::ON),
            "OFF" => Ok(Self::OFF),
            _ => Err(()),
        }
    }
}

pub struct PowerStrip {
    base_url: String,
    client: Client,
}
impl PowerStrip {
    pub async fn new(user: String, password: String, ip: String) -> Result<Self, ()> {
        let base_url = format!("http://{}", ip);
        let client = login(&user, &password, &base_url).await.unwrap();
        Ok(PowerStrip { base_url, client })
    }
    pub async fn update_all(&self, status: Status) -> Result<(), ()> {
        let _resp = self
            .client
            .get(format!("{}/outlet?a={:?}", self.base_url, status))
            .send()
            .await
            .unwrap();
        Ok(())
    }
    pub async fn update(&self, outlet: u8, status: Status) -> Result<(), ()> {
        let _resp = self
            .client
            .get(format!("{}/outlet?{}={:?}", self.base_url, outlet, status))
            .send()
            .await
            .unwrap();
        Ok(())
    }

    pub async fn status(&self) -> Result<Vec<Status>, ()> {
        let resp = self
            .client
            .get(format!("{}/index.htm", self.base_url))
            .send()
            .await
            .unwrap();
        let status = resp.text().await.unwrap();
        let doc = Html::parse_document(&status);
        let elems = Selector::parse("td").unwrap();
        let outlet_regex = Regex::new(r"^Outlet (\d+)$").unwrap();
        let status_regex = Regex::new(r"^\n<b><font color=.*>([A-Z]+)</font></b>$").unwrap();
        let mut elems_iter = doc.select(&elems);
        let mut outlets = Vec::new();
        while let Some(elem) = elems_iter.next() {
            if let Some(_outlet) = outlet_regex.captures(&elem.inner_html()) {
                if let Some(status) =
                    status_regex.captures(&elems_iter.next().unwrap().inner_html())
                {
                    outlets.push(status[1].parse().unwrap());
                }
            }
        }
        Ok(outlets)
    }
}
async fn login(user: &str, password: &str, base_url: &str) -> Result<Client, ()> {
    let client = Client::builder().cookie_store(true).build().unwrap();
    let resp = client
        .get(base_url)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let c = challenge(resp);
    let form_response = format!("{}{}{}{}", c, user, password, c);
    let mut hasher = Md5::new();
    hasher.update(form_response);
    let login = hasher.finalize();
    //Content-Type': 'application/x-www-form-urlencoded
    let mut login_data = HashMap::new();
    login_data.insert("Username", user.to_string());
    login_data.insert("Password", format!("{:x}", login));
    let _resp = client
        .post(format!("{}/login.tgi", base_url))
        .form(&login_data)
        .send()
        .await
        .unwrap();
    //let cookie = &resp.headers().get("Set-Cookie").unwrap();
    Ok(client)
}

fn challenge(resp: String) -> String {
    for l in resp.lines() {
        if let Some(challenge_line) =
            l.strip_prefix("<input type=\"hidden\" name=\"Challenge\" value=\"")
        {
            let (c, _) = challenge_line.split_once('"').unwrap();
            return c.to_string();
        }
    }
    panic!("No challenge found");
}
