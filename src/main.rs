use reqwest::Client;
use md5::{Md5, Digest};
use std::collections::HashMap;
use scraper::{Html, Selector};
use regex::Regex;
use std::str::FromStr;

const USER: &str = "admin";
const PASSWORD: &str = "sccrules47";
const IP: &str = "140.221.146.68";

#[tokio::main]
async fn main() {
    let client = login(USER, PASSWORD, IP).await.unwrap();
    let outlets = status(&client).await.unwrap(); 
    println!("{:?}", outlets);
    update(&client, 1, Status::OFF).await.unwrap();
    let outlets = status(&client).await.unwrap(); 
    println!("{:?}", outlets);
    update(&client, 1, Status::ON).await.unwrap();
    let outlets = status(&client).await.unwrap(); 
    println!("{:?}", outlets);
}

#[derive(Debug)]
pub enum Status {
    ON,
    OFF,
}

impl FromStr for Status {

    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "ON"  => Ok(Self::ON),
            "OFF"  => Ok(Self::OFF),
            _      => Err(()),
        }
    }
}


pub async fn update_all(client: &Client, status: Status) -> Result<(), ()> {
    let _resp = client.get(format!("http://{}/outlet?a={:?}", IP, status)).send().await.unwrap();
    Ok(())
}
pub async fn update(client: &Client, outlet: u8, status: Status) -> Result<(), ()> {
    let _resp = client.get(format!("http://{}/outlet?{}={:?}", IP, outlet, status)).send().await.unwrap();
    Ok(())
}

pub async fn status(client: &Client) -> Result<Vec<Status>,()> {
    let resp = client.get(format!("http://{}/index.htm", IP)).send().await.unwrap();

    let status = resp.text().await.unwrap();
    let doc = Html::parse_document(&status);
    let elems = Selector::parse("td").unwrap();
    let outlet_regex = Regex::new(r"^Outlet (\d+)$").unwrap();
    let status_regex = Regex::new(r"^\n<b><font color=.*>([A-Z]+)</font></b>$").unwrap();
    let mut elems_iter = doc.select(&elems);
    let mut outlets = Vec::new();
    while let Some(elem) = elems_iter.next() {
        if let Some(_outlet) = outlet_regex.captures(&elem.inner_html()) {
            if let Some(status) = status_regex.captures(&elems_iter.next().unwrap().inner_html()) {
                outlets.push(status[1].parse().unwrap());
            }
        }
    }
    Ok(outlets)
}

pub async fn login(user: &str, password: &str, ip: &str) -> Result<Client,()> {
    let client = Client::builder().cookie_store(true).build().unwrap();
    let resp = client.get(format!("http://{}", ip)).send().await.unwrap().text().await.unwrap();
    let c = challenge(resp);
    let form_response = format!("{}{}{}{}", c, user, password, c);
    let mut hasher = Md5::new();
    hasher.update(form_response);
    let login = hasher.finalize();
    //Content-Type': 'application/x-www-form-urlencoded
    let mut login_data = HashMap::new();
    login_data.insert("Username", user.to_string());
    login_data.insert("Password", format!("{:x}", login));
    let _resp = client.post(format!("http://{}/login.tgi", IP)).form(&login_data).send().await.unwrap();
    //let cookie = &resp.headers().get("Set-Cookie").unwrap();
    Ok(client)
}

fn challenge(resp: String) -> String {
    for l in resp.lines() {
        if let Some(challenge_line) = l.strip_prefix("<input type=\"hidden\" name=\"Challenge\" value=\"") {
            let (c, _) = challenge_line.split_once('"').unwrap();
            return c.to_string();
        }
    }
    panic!("No challenge found");
}
