use chrono::{Duration, NaiveDate, Utc};
use clap::{value_parser, Arg, ArgAction, Command};
use rpassword;
use serde::{Deserialize, Serialize};
use std::env;
use std::process;
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
struct User {
    id: String,
    username: String,
    acct: String,
    url: String,
    bot: bool,
    last_status_at: Option<String>,
}

impl User {
    fn is_active(&self, days: i32) -> bool {
        if self.last_status_at.is_none() {
            return false;
        }
        let today = Utc::now().date_naive();
        let limit = today - Duration::days(days as i64);

        match NaiveDate::parse_from_str(self.last_status_at.as_ref().unwrap().as_str(), "%Y-%m-%d")
        {
            Ok(last_status_date) => {
                if last_status_date < limit {
                    false
                } else {
                    true
                }
            }
            _ => false,
        }
    }
}

fn main() {
    let matches = Command::new("jacanaoesta")
        .version("0.1.0")
        .author("Gonçalo Valério <gon@ovalerio.net>")
        .about("Find people that no longer are active in your Mastodon follow list.")
        .arg(Arg::new("instance").required(true))
        .arg(
            Arg::new("api-key")
                .short('k')
                .long("api-key")
                .help("Ask for API key")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("days")
                .short('d')
                .long("days")
                .help("Days since last status to consider inactive")
                .value_parser(value_parser!(i32))
                .default_value("180"),
        )
        .get_matches();

    let ask_for_key = matches.get_flag("api-key");
    let api_key = match get_api_key(ask_for_key) {
        Ok(key) => key,
        _ => {
            println!("Could not find a valid API Key");
            process::exit(1);
        }
    };
    let instance = matches
        .get_one::<String>("instance")
        .expect("Instance is missing");

    let days = matches
        .get_one::<i32>("days")
        .expect("Days must be a integer");

    if *days < 1 {
        println!("Days must be a positive integer");
        process::exit(1);
    }

    if !validate_url(&instance) {
        println!("Invalid instance URL");
        process::exit(1);
    }

    let user_id: String;
    match check_api_availability(&instance, &api_key) {
        Ok(id) => user_id = id,
        Err(reason) => {
            println!("{}", reason);
            process::exit(1);
        }
    }

    let following;
    match fetch_followed_users(&instance, &api_key, &user_id) {
        Ok(users) => following = users,
        Err(reason) => {
            println!("{}", reason);
            process::exit(1);
        }
    }
    println!("Found {} users. Checking...", following.len());
    let inactive_users: i32 = following
        .iter()
        .map(|user| {
            if user.is_active(*days) {
                return 0;
            } else {
                println!("{} ({}) seems to be inactive", user.username, user.url);
                return 1;
            }
        })
        .sum();
    println!(
        "{} of them seem to be inactive for at least {} days",
        inactive_users, days
    );
}

fn get_api_key(ask: bool) -> Result<String, ()> {
    if ask {
        match rpassword::prompt_password("Paste API Key here:") {
            Ok(key) => return Ok(key),
            _ => Err(()),
        }
    } else {
        match env::var("JCNE_MAST_API_KEY") {
            Ok(key) => return Ok(key),
            _ => Err(()),
        }
    }
}

fn validate_url(instance: &String) -> bool {
    let parsed_url;
    match Url::parse(instance) {
        Ok(url) => parsed_url = url,
        _ => return false,
    }

    let scheme = parsed_url.scheme();
    if scheme != "https" && scheme != "http" {
        return false;
    }

    let path = parsed_url.path();
    if path != "/" {
        return false;
    }
    true
}

fn check_api_availability(instance: &String, api_key: &String) -> Result<String, String> {
    let client = reqwest::blocking::Client::new();
    let response;

    match client
        .get(format!("{}/api/v1/accounts/verify_credentials", instance))
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
    {
        Ok(resp) => response = resp,
        _ => return Err("Failed to connect ".to_string()),
    }

    if response.status().is_success() {
        let data = response.json::<User>().unwrap();
        Ok(data.id)
    } else {
        Err("Failed to connect ".to_string())
    }
}

fn fetch_followed_users(
    instance: &String,
    api_key: &String,
    user_id: &String,
) -> Result<Vec<User>, String> {
    let mut response;
    let mut users = Vec::new();
    let mut url = format!("{}/api/v1/accounts/{}/following", instance, user_id);
    let mut still_has_followers = true;
    let client = reqwest::blocking::Client::new();

    while still_has_followers {
        match client
            .get(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
        {
            Ok(resp) => response = resp,
            _ => return Err("Unable to fetch all followers".to_string()),
        }

        if response.status().is_success() {
            let headers = response.headers().clone();
            let mut data = response.json::<Vec<User>>().unwrap();
            users.append(&mut data);

            let link_header;
            match headers.get("link") {
                Some(link) => link_header = link.to_str().unwrap().to_string(),
                _ => {
                    still_has_followers = false;
                    continue;
                }
            }

            match get_next_page(link_header) {
                Ok(next_page) => url = next_page,
                _ => still_has_followers = false,
            }
        } else {
            return Err("Invalid server response".to_string());
        }
    }

    Ok(users)
}

fn get_next_page(link_header: String) -> Result<String, ()> {
    let adj_pages: Vec<&str> = link_header.split(",").collect();

    // It is too late in the night and I don't have the patience for this.
    // Will fix later, for now lets assume the nonsense below works.
    let next_page = adj_pages[0];
    if next_page.find("next").unwrap_or(0) == 0 {
        return Err(());
    }

    let url_start = next_page.find("<").unwrap();
    let url_end = next_page.find(">").unwrap();

    Ok(next_page[url_start + 1..url_end].to_string())
}
