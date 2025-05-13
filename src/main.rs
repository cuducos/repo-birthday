mod calendar;
mod commits;
mod date_time_serializer;
mod graphql;
mod models;
mod repositories;

use anyhow::{anyhow, Result};
use tokio::sync::mpsc;

use crate::calendar::{calendar_from, to_ical};
use crate::commits::last_commit;
use crate::graphql::GitHubGraphQL;
use crate::repositories::repos_for;

const HELP: &str = "Usage: repo-birthday <USERNAME> [--ical]";

fn parse_args(values: &[String]) -> (&str, bool) {
    if values.len() != 2 && values.len() != 3 {
        panic!("{}", HELP);
    }

    match (values.get(1), values.get(2)) {
        (Some(value), None) => (value.as_str(), false),
        (Some(value1), Some(value2)) => {
            let (v1, v2) = (value1.as_str(), value2.as_str());
            if v1 == "--ical" {
                (v2, true)
            } else if v2 == "--ical" {
                (v1, true)
            } else {
                panic!("{}", HELP);
            }
        }
        (_, _) => panic!("{}", HELP),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = std::env::args().collect::<Vec<String>>();
    let (username, ical) = parse_args(&args);
    let client = GitHubGraphQL::new()?;
    let repos = repos_for(&client, username).await?;
    let total = repos.len();

    println!("Getting the first commit for {total} repositories {username} has access to…");
    let (sender, mut receiver) = mpsc::channel(16);
    for repo in repos.into_iter() {
        let queue = sender.clone();
        tokio::spawn(async move {
            let result = match GitHubGraphQL::new() {
                Ok(client) => last_commit(&client, &repo).await,
                Err(e) => Err(e),
            };
            if let Err(e) = queue.send(result).await {
                panic!("Error sending first commit for {repo}: {}", e);
            }
        });
    }

    let commits = &mut Vec::with_capacity(total);
    for _ in 1..=total {
        if let Some(result) = receiver.recv().await {
            match result {
                Ok(first_commit) => {
                    if let Some(commit) = first_commit {
                        commits.push(commit);
                    }
                }
                Err(e) => return Err(anyhow!("Error getting first commit: {e}")),
            }
        }
    }
    commits.sort_by_key(|commit| commit.days_to_next_anniversary());

    if ical {
        let calendar = calendar_from(username, commits);
        let filename = to_ical(username, calendar)?;
        println!("{} created!", filename);
    } else {
        for commit in commits {
            println!(
                "In {} days {}/{} is turning {} years old: {commit}",
                commit.days_to_next_anniversary(),
                commit.owner,
                commit.name,
                commit.age(),
            );
        }
    }
    Ok(())
}
