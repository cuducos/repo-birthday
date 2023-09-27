use anyhow::{anyhow, Result};
use async_recursion::async_recursion;
use liquid::{ParserBuilder, Template};

const GITHUB_GRAPHQL_URL: &str = "https://api.github.com/graphql";
const REPOS_QUERY: &str = include_str!("graphql/repos.graphql");
const CURSOR_QUERY: &str = include_str!("graphql/cursor.graphql");
const LAST_COMMIT_QUERY: &str = include_str!("graphql/last_commit.graphql");
const DEFAULT_WAIT: u64 = 3;

struct Templates {
    repos: Template,
    cursor: Template,
    last_commit: Template,
}

pub struct GitHubGraphQL {
    api_key: String,
    templates: Templates,
}

impl GitHubGraphQL {
    pub fn new() -> Result<Self> {
        let parser = ParserBuilder::with_stdlib().build()?;

        Ok(GitHubGraphQL {
            api_key: std::env::var("GITHUB_ACCESS_TOKEN")
                .map_err(|e| anyhow!("GITHUB_ACCESS_TOKEN environment variable not found: {e}"))?,
            templates: Templates {
                repos: parser.parse(REPOS_QUERY)?,
                cursor: parser.parse(CURSOR_QUERY)?,
                last_commit: parser.parse(LAST_COMMIT_QUERY)?,
            },
        })
    }

    #[async_recursion]
    async fn request(&self, query: String) -> Result<reqwest::Response> {
        let mut input = std::collections::HashMap::new();
        input.insert("query", &query);

        let resp = reqwest::Client::new()
            .post(GITHUB_GRAPHQL_URL)
            .header("User-Agent", "github.com/cuducos/repo-birthday")
            .bearer_auth(&self.api_key)
            .json(&input)
            .send()
            .await?;

        if !resp.status().is_success() {
            if resp.status() == reqwest::StatusCode::FORBIDDEN {
                let q = query.clone();
                let w = DEFAULT_WAIT.to_string();
                let wait = resp
                    .headers()
                    .get("retry-after")
                    .map(|value| value.to_str().unwrap_or(w.as_str()))
                    .map(|value| value.parse::<u64>().unwrap_or(DEFAULT_WAIT))
                    .unwrap_or(DEFAULT_WAIT);

                std::thread::sleep(std::time::Duration::from_secs(wait));
                return self.request(q).await;
            }

            return Err(anyhow::anyhow!(
                "Request failed with status code {}: {}",
                resp.status(),
                resp.text().await?
            ));
        }

        Ok(resp)
    }

    pub async fn repos(&self, username: &str, cursor: &str) -> Result<String> {
        let context = liquid::object!({
            "username": username,
            "cursor": cursor,
        });
        let query = self.templates.repos.render(&context)?;
        let resp = self.request(query).await?;

        Ok(resp.text().await?)
    }

    pub async fn cursor_or_last_commit(&self, name: &str, owner: &str) -> Result<String> {
        let context = liquid::object!({
            "name": name,
            "owner": owner,
        });
        let query = self.templates.cursor.render(&context)?;
        let resp = self.request(query).await?;

        Ok(resp.text().await?)
    }

    pub async fn last_commit(&self, name: &str, owner: &str, cursor: &str) -> Result<String> {
        let context = liquid::object!({
            "name": name,
            "owner": owner,
            "cursor": cursor,
        });
        let query = self.templates.last_commit.render(&context)?;
        let resp = self.request(query).await?;

        Ok(resp.text().await?)
    }
}
