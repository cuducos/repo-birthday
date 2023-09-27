use anyhow::{anyhow, Result};
use chrono::NaiveDateTime;
use serde::Deserialize;

use crate::date_time_serializer;
use crate::graphql::GitHubGraphQL;
use crate::models::FirstCommit;
use crate::repositories::Repository as Repo;

#[derive(Debug, Deserialize)]
pub struct PageInfo {
    #[serde(rename = "endCursor")]
    pub end_cursor: String,
}

impl PageInfo {
    pub fn cursor(&self) -> Result<&str> {
        match &self.end_cursor.split(' ').next() {
            Some(cursor) => Ok(cursor),
            None => Err(anyhow!("Cursor hash not found for {}", self.end_cursor)),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Commit {
    pub message: String,

    #[serde(with = "date_time_serializer", rename = "committedDate")]
    pub committed_date: NaiveDateTime,
}

impl Commit {
    fn as_first_commit(&self, repo: &Repo) -> FirstCommit {
        FirstCommit {
            message: self.message.clone(),
            date: self.committed_date,
            name: repo.name.clone(),
            owner: repo.owner.login.clone(),
        }
    }
}

async fn last_commit_from_cursor(
    client: &GitHubGraphQL,
    repo: &Repo,
    cursor: String,
) -> Result<Option<Commit>> {
    let name = repo.name.as_str();
    let owner = repo.owner.login.as_str();
    let resp = client.last_commit(name, owner, cursor.as_str()).await?;
    let contents: Response = serde_json::from_str(&resp)
        .map_err(|e| anyhow::anyhow!("Error parsing last commit: {}\n{}", e, resp))?;

    Ok(contents
        .data
        .repository
        .default_branch_ref
        .and_then(|branch| branch.last_commit()))
}

pub async fn last_commit(client: &GitHubGraphQL, repo: &Repo) -> Result<Option<FirstCommit>> {
    let resp = client
        .cursor_or_last_commit(repo.name.as_str(), repo.owner.login.as_str())
        .await?;
    let contents: Response = serde_json::from_str(&resp).map_err(|e| {
        anyhow::anyhow!(
            "Error parsing last commit or cursor for {repo}: {}\n{}",
            e,
            resp
        )
    })?;

    match contents.data.repository.default_branch_ref {
        None => Ok(None),
        Some(branch) => {
            if branch.target.history.total_count == 0 {
                return Ok(None);
            }
            if branch.target.history.total_count == 1 {
                if let Some(commit) = branch.target.history.nodes.first() {
                    return Ok(Some(commit.as_first_commit(repo)));
                }
            }

            let cursor = branch.last_commit_cursor()?;
            match last_commit_from_cursor(client, repo, cursor).await? {
                Some(commit) => Ok(Some(commit.as_first_commit(repo))),
                None => Ok(None),
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct History {
    pub nodes: Vec<Commit>,

    #[serde(rename = "totalCount")]
    pub total_count: i32,

    #[serde(rename = "pageInfo")]
    pub page_info: PageInfo,
}

#[derive(Debug, Deserialize)]
pub struct Target {
    pub history: History,
}

#[derive(Debug, Deserialize)]
pub struct Branch {
    pub target: Target,
}

impl Branch {
    pub fn last_commit_cursor(&self) -> Result<String> {
        let mut pos = self.target.history.total_count - 2;
        if pos < 0 {
            pos = 0;
        }

        Ok(format!(
            "{} {}",
            self.target.history.page_info.cursor()?,
            pos
        ))
    }

    pub fn last_commit(&self) -> Option<Commit> {
        self.target.history.nodes.last().map(|commit| Commit {
            message: commit.message.clone(),
            committed_date: commit.committed_date,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct Repository {
    #[serde(rename = "defaultBranchRef")]
    pub default_branch_ref: Option<Branch>,
}

#[derive(Debug, Deserialize)]
pub struct Data {
    pub repository: Repository,
}

#[derive(Debug, Deserialize)]
pub struct Response {
    pub data: Data,
}
