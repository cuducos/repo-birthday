use crate::envvar;
use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use reqwest::Client;
use serde::{Deserialize, Serialize};

const USER_AGENT: &str = "github.com/cuducos/repo-birthday";

lazy_static! {
    static ref GITHUB_APP_SECRET: String = envvar::get("GITHUB_APP_SECRET").unwrap();
}

#[derive(Deserialize)]
struct UserInfo {
    login: String,
}

#[derive(Serialize)]
struct ExchangeToken {
    code: String,
    client_id: String,
    client_secret: String,
}

#[derive(Deserialize)]
struct AccessToken {
    access_token: String,
}

pub fn github_client_id() -> Result<String> {
    envvar::get("GITHUB_APP_CLIENT_ID")
}

pub async fn token_for(client: &Client, code: &str) -> anyhow::Result<String> {
    let params = ExchangeToken {
        code: code.to_string(),
        client_id: github_client_id()?,
        client_secret: envvar::get("GITHUB_APP_CLIENT_ID")?,
    };
    let res = client
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .header("User-Agent", USER_AGENT)
        .json(&params)
        .send()
        .await?;
    if !res.status().is_success() {
        println!(
            "Error getting access token ({:?}): {:?}",
            res.status(),
            res.text().await?
        );
        return Err(anyhow!("Error getting access token"));
    }
    let body = res.text().await?;
    match serde_json::from_str::<AccessToken>(&body) {
        Ok(data) => Ok(data.access_token),
        Err(e) => {
            eprintln!("{}: {}", e, body);
            Err(anyhow!("{}", e))
        }
    }
}

pub async fn username_for(client: &Client, token: &str) -> anyhow::Result<String> {
    let res = client
        .get("https://api.github.com/user")
        .bearer_auth(token)
        .header("Accept", "application/json")
        .header("User-Agent", USER_AGENT)
        .send()
        .await?;
    if !res.status().is_success() {
        println!(
            "Error getting username ({:?}): {:?}",
            res.status(),
            res.text().await?
        );
        return Err(anyhow!("Error getting username"));
    }
    let body = res.text().await?;
    match serde_json::from_str::<UserInfo>(&body) {
        Ok(data) => Ok(data.login),
        Err(e) => {
            eprintln!("{}: {}", e, body);
            Err(anyhow!("{}", e))
        }
    }
}
