mod commits;
mod date_time_serializer;
mod graphql;
mod models;
mod repositories;

use anyhow::{anyhow, Result};
use tokio::sync::mpsc;

use crate::commits::last_commit;
use crate::graphql::GitHubGraphQL;
use crate::repositories::repos_for;


#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        panic!("Usage: repo-birthday <USERNAME>");
    }

    let username = args[1].as_str();
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

    commits.sort_by_key(|commit| commit.days_to_next_aniversary());
    for commit in commits {
        println!(
            "In {} days {}/{} is turning {} years old: {commit}",
            commit.days_to_next_aniversary(),
            commit.owner,
            commit.name,
            commit.age(),
        );
    }

    Ok(())
}
