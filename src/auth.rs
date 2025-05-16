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

// #[derive(Deserialize, Debug)]
// struct AccessToken {
//     access_token: String,
// }

pub fn github_client_id() -> Result<String> {
    envvar::get("GITHUB_APP_CLIENT_ID")
}

pub async fn token_for(client: &Client, code: &str) -> anyhow::Result<String> {
    let params = [
        ("code", code.to_string()),
        ("client_id", github_client_id()?),
        ("client_secret", GITHUB_APP_SECRET.to_string()),
    ];
    let res = client
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .header("User-Agent", USER_AGENT)
        .form(&params)
        .send()
        .await?;
    let body = res.text().await?;
    println!("{}", &body);
    Ok("test".to_string())
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
