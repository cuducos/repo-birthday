use anyhow::Result;
use lazy_static::lazy_static;
use liquid::{ParserBuilder, Template};

const REPOS_QUERY: &str = include_str!("graphql/repos.graphql");
const CURSOR_QUERY: &str = include_str!("graphql/cursor.graphql");
const LAST_COMMIT_QUERY: &str = include_str!("graphql/last_commit.graphql");
const INDEX_HTML: &str = include_str!("html/index.html");

lazy_static! {
    pub static ref TEMPLATES: Templates = Templates::new().expect("Error building templates");
}

pub struct HTMLTemplates {
    pub home: Template,
}

pub struct GraphQLTemplates {
    pub repos: Template,
    pub cursor: Template,
    pub last_commit: Template,
}

pub struct Templates {
    pub html: HTMLTemplates,
    pub graphql: GraphQLTemplates,
}

impl Templates {
    fn new() -> Result<Self> {
        let parser = ParserBuilder::with_stdlib().build()?;
        Ok(Self {
            html: HTMLTemplates {
                home: parser.parse(INDEX_HTML)?,
            },
            graphql: GraphQLTemplates {
                repos: parser.parse(REPOS_QUERY)?,
                cursor: parser.parse(CURSOR_QUERY)?,
                last_commit: parser.parse(LAST_COMMIT_QUERY)?,
            },
        })
    }
}
