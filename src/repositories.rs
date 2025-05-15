use anyhow::Result;
use serde::Deserialize;

use crate::graphql::GitHubGraphQL;

#[derive(Debug, Deserialize)]
pub struct Owner {
    pub login: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Repository {
    pub name: String,
    pub is_fork: bool,
    pub owner: Owner,
}

impl Clone for Repository {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            is_fork: self.is_fork,
            owner: Owner {
                login: self.owner.login.clone(),
            },
        }
    }
}

impl std::fmt::Display for Repository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.owner.login, self.name)
    }
}

pub async fn repos_for(client: &GitHubGraphQL, username: &str) -> Result<Vec<Repository>> {
    let mut repos: Vec<Repository> = vec![];
    let mut has_next_page = true;
    let mut cursor = "".to_string();

    while has_next_page {
        let response = client.repos(username, cursor.as_str()).await?;
        let body: Response = serde_json::from_str(&response)?;
        repos.extend(
            body.data
                .user
                .repositories
                .nodes
                .into_iter()
                .filter(|r| !r.is_fork),
        );
        has_next_page = body.data.user.repositories.page_info.has_next_page;
        cursor = body.data.user.repositories.page_info.end_cursor.clone();
    }

    Ok(repos)
}

#[derive(Debug, Deserialize)]
pub struct PageInfo {
    #[serde(rename = "endCursor")]
    pub end_cursor: String,

    #[serde(rename = "hasNextPage")]
    pub has_next_page: bool,
}

#[derive(Debug, Deserialize)]
pub struct Repositories {
    pub nodes: Vec<Repository>,

    #[serde(rename = "pageInfo")]
    page_info: PageInfo,
}

#[derive(Debug, Deserialize)]
pub struct User {
    pub repositories: Repositories,
}

#[derive(Debug, Deserialize)]
pub struct Data {
    pub user: User,
}

#[derive(Debug, Deserialize)]
pub struct Response {
    pub data: Data,
}
