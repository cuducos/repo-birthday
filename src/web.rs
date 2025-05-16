use crate::{
    auth::{token_for, username_for},
    cache::CACHE,
    calendar::calendar_from,
    commits::last_commit,
    envvar,
    graphql::GitHubGraphQL,
    repositories::repos_for,
    templates::TEMPLATES,
};
use actix_web::{
    error::{ErrorInternalServerError, ErrorNotFound},
    get,
    http::{header::ContentType, StatusCode},
    web::{self, Redirect},
    Error, HttpResponse, Responder,
};
use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Semaphore;

#[derive(Deserialize)]
struct CallbackParams {
    code: Option<String>,
}

pub const DEFAULT_PORT: u16 = 8000;
pub const DEFAULT_IP: &str = "0.0.0.0";

async fn data_for(token: &str, username: &str) -> anyhow::Result<String> {
    let client = GitHubGraphQL::new(token);
    let repos = repos_for(&client, username).await?;
    let total = repos.len();
    let semaphore = Arc::new(Semaphore::new(16));
    let mut results = Vec::with_capacity(total);
    for repo in repos.into_iter() {
        let tkn = token.to_string();
        let sem = semaphore.clone();
        let result = tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            let client = GitHubGraphQL::new(tkn.as_ref());
            last_commit(&client, &repo).await
        });
        results.push(result);
    }
    let mut commits = Vec::with_capacity(total);
    for result in results {
        if let Some(commit) = result.await?? {
            commits.push(commit);
        }
    }
    commits.sort_by_key(|commit| commit.days_to_next_anniversary());
    let contents = format!("{}", calendar_from(username, &commits));
    CACHE.save_calendar(username, contents.as_ref()).await?;
    Ok(contents)
}

fn log_and_crash(error: impl std::fmt::Display) -> Error {
    eprintln!("{}", error);
    ErrorInternalServerError("Internal server error")
}

fn context(username: Option<&String>) -> anyhow::Result<liquid::Object> {
    let domain = envvar::get("DOMAIN").unwrap_or(format!(
        "{}:{}",
        DEFAULT_IP,
        envvar::get("PORT").unwrap_or(DEFAULT_PORT.to_string())
    ));
    let protocol = if domain.starts_with(DEFAULT_IP) {
        "http"
    } else {
        "https"
    };
    Ok(liquid::object!({
        "url": format!("{}://{}",protocol, domain),
        "username": username,
        "client_id": envvar::get("GITHUB_APP_CLIENT_ID")?,
    }))
}

#[get("/")]
async fn index() -> Result<impl Responder, Error> {
    TEMPLATES
        .html
        .home
        .render(&context(None).map_err(log_and_crash)?)
        .map(|html| {
            HttpResponse::build(StatusCode::OK)
                .content_type(ContentType::html())
                .body(html)
        })
        .map_err(log_and_crash)
}

#[get("/github/auth/callback")]
async fn callback(info: web::Query<CallbackParams>) -> Result<impl Responder, Error> {
    if let Some(code) = &info.code {
        let client = Client::new();
        let token = token_for(&client, code).await.map_err(log_and_crash)?;
        let username = username_for(&client, &token).await.map_err(log_and_crash)?;
        CACHE
            .save_token(&username, token.as_ref())
            .await
            .map_err(log_and_crash)?;
        Ok(Redirect::to(format!("/{}", username)))
    } else {
        Err(log_and_crash("Missing code in GitHub callback"))
    }
}

async fn _calendar(username: web::Path<String>) -> Result<impl Responder, Error> {
    if let Ok(contents) = CACHE.calendar(username.as_ref()).await {
        return Ok(HttpResponse::build(StatusCode::OK)
            .content_type("text/calendar")
            .body(contents));
    }
    if let Ok(token) = CACHE.token(username.as_ref()).await {
        return data_for(token.as_ref(), username.as_ref())
            .await
            .map(|contents| {
                HttpResponse::build(StatusCode::OK)
                    .content_type("text/calendar")
                    .body(contents)
            })
            .map_err(log_and_crash);
    }
    Err(ErrorNotFound("Not found"))
}

#[get("/{username}.ical")]
async fn calendar(username: web::Path<String>) -> Result<impl Responder, Error> {
    _calendar(username).await
}

#[get("/{username}.ics")]
async fn calendar_alt(username: web::Path<String>) -> Result<impl Responder, Error> {
    _calendar(username).await
}

#[get("/{username}")]
async fn view(username: web::Path<String>) -> Result<impl Responder, Error> {
    if let Ok(token) = CACHE.token(username.as_ref()).await {
        if CACHE.calendar(username.as_ref()).await.is_err() {
            let user = username.clone();
            tokio::spawn(async move {
                if let Err(e) = data_for(token.as_ref(), user.as_ref()).await {
                    eprintln!("Error creating calendar for {}: {}", user, e);
                }
            });
        }
        TEMPLATES
            .html
            .home
            .render(&context(Some(&username)).map_err(log_and_crash)?)
            .map(|html| {
                HttpResponse::build(StatusCode::OK)
                    .content_type(ContentType::html())
                    .body(html)
            })
            .map_err(log_and_crash)
    } else {
        Err(ErrorNotFound("Not found"))
    }
}
