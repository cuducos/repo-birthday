use crate::envvar;
use anyhow::Result;
use lazy_static::lazy_static;
use reqwest::Client;
use serde::Deserialize;

const USER_AGENT: &str = "github.com/cuducos/repo-birthday";

lazy_static! {
    static ref GITHUB_APP_ID: String = envvar::get("GITHUB_APP_ID").unwrap();
    static ref GITHUB_APP_SECRET: String = envvar::get("GITHUB_APP_SECRET").unwrap();
}

#[derive(Deserialize, Debug)]
struct UserInfo {
    login: String,
}

#[derive(Deserialize, Debug)]
struct AccessToken {
    access_token: String,
}

pub async fn token_for(client: &Client, code: &str) -> anyhow::Result<String> {
    let params = [
        ("code", code),
        ("client_id", GITHUB_APP_ID.as_ref()),
        ("client_secret", GITHUB_APP_SECRET.as_ref()),
    ];
    let res = client
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .header("User-Agent", USER_AGENT)
        .form(&params)
        .send()
        .await?
        .json::<AccessToken>()
        .await?;
    Ok(res.access_token)
}

pub async fn username_for(client: &Client, token: &str) -> anyhow::Result<String> {
    let res = client
        .get("https://api.github.com/user")
        .bearer_auth(token)
        .header("Accept", "application/json")
        .header("User-Agent", USER_AGENT)
        .send()
        .await?
        .json::<UserInfo>()
        .await?;
    Ok(res.login)
}

pub fn github_client_id() -> Result<String> {
    envvar::get("GITHUB_APP_CLIENT_ID")
}
