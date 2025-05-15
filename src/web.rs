use crate::{
    auth::{github_client_id, token_for, username_for},
    cache::CACHE,
    calendar::calendar_from,
    commits::last_commit,
    graphql::GitHubGraphQL,
    repositories::repos_for,
    templates::TEMPLATES,
};
use actix_web::{
    error::{ErrorInternalServerError, ErrorNotFound},
    get,
    http::{header::ContentType, StatusCode},
    web, Error, HttpResponse, Responder,
};
use reqwest::Client;
use serde::Deserialize;
use tokio::sync::mpsc;

#[derive(Deserialize)]
struct CallbackParams {
    code: Option<String>,
}

async fn data_for(token: &str, username: &str) -> anyhow::Result<String> {
    let client = GitHubGraphQL::new(token);
    let repos = repos_for(&client, username).await?;
    let total = repos.len();
    let (sender, mut receiver) = mpsc::channel(16);
    for repo in repos.into_iter() {
        let queue = sender.clone();
        let client = GitHubGraphQL::new(token);
        tokio::spawn(async move {
            let result = last_commit(&client, &repo).await;
            if let Err(e) = queue.send(result).await {
                eprintln!("Error sending first commit for {repo}: {}", e);
            }
        });
    }
    let mut commits = Vec::with_capacity(total);
    for _ in 1..=total {
        if let Some(result) = receiver.recv().await {
            match result {
                Ok(first_commit) => {
                    if let Some(commit) = first_commit {
                        commits.push(commit);
                    }
                }
                Err(e) => return Err(anyhow::anyhow!("Error getting first commit: {e}")),
            }
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
    Ok(liquid::object!({ "username": username, "client_id": github_client_id()?}))
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
        let ctx = context(Some(&username)).map_err(log_and_crash)?;
        CACHE
            .save_token(&username, token.as_ref())
            .await
            .map_err(log_and_crash)?;
        tokio::spawn(async move {
            if let Err(e) = data_for(token.as_ref(), &username).await {
                eprintln!("Error getting data for {}: {}", username, e);
            }
        });
        TEMPLATES.html.home.render(&ctx).map_err(log_and_crash)
    } else {
        Err(log_and_crash("Missing code in GitHub callback"))
    }
}

#[get("/{username}.ical")]
async fn calendar(username: web::Path<String>) -> Result<impl Responder, Error> {
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
