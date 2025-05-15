use anyhow::{anyhow, Result};

pub fn get(name: &str) -> Result<String> {
    std::env::var(name).map_err(|e| anyhow!("{} environment variable not found: {}", name, e))
}
