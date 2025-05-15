use crate::envvar;
use anyhow::Result;
use chrono::Duration;
use lazy_static::lazy_static;
use magic_crypt::{new_magic_crypt, MagicCrypt256, MagicCryptTrait};

const TOKEN_SUFFIX: &str = "token";
const CALENDAR_SUFFIX: &str = "calendar";
const CALENDAR_TTL: Duration = Duration::days(1);

lazy_static! {
    pub static ref CACHE: Cache = Cache::new().expect("Error building cache");
}

struct KVStorage {
    account_id: String,
    namespace_id: String,
    api_key: String,
}

impl KVStorage {
    pub fn new() -> Result<Self> {
        Ok(Self {
            account_id: envvar::get("CLOUDFLARE_ACCOUNT_ID")?,
            namespace_id: envvar::get("CLOUDFLARE_KV_NAMESPACE")?,
            api_key: envvar::get("CLOUDFLARE_API_TOKEN")?,
        })
    }

    fn url(&self, key: &str, duration: Option<Duration>) -> String {
        let base = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/storage/kv/namespaces/{}/values/{}",
            self.account_id, self.namespace_id, key
        );
        match duration {
            Some(ttl) => format!("{}?expiration_ttl={}", base, ttl.num_seconds()),
            None => base,
        }
    }

    async fn save(&self, key: &str, value: &str, ttl: Option<Duration>) -> Result<()> {
        let resp = reqwest::Client::new()
            .put(self.url(key, ttl))
            .header("User-Agent", "github.com/cuducos/repo-birthday")
            .bearer_auth(&self.api_key)
            .body(value.to_string())
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(anyhow::anyhow!(
                "Request failed with status code {}: {}",
                resp.status(),
                resp.text().await?
            ));
        }
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<String> {
        let resp = reqwest::Client::new()
            .get(self.url(key, None))
            .header("User-Agent", "github.com/cuducos/repo-birthday")
            .bearer_auth(&self.api_key)
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(anyhow::anyhow!(
                "Request failed with status code {}: {}",
                resp.status(),
                resp.text().await?
            ));
        }
        Ok(resp.text().await?)
    }
}

pub struct Cache {
    storage: KVStorage,
    secret: MagicCrypt256,
    separator: String,
}

impl Cache {
    pub fn new() -> Result<Self> {
        Ok(Self {
            storage: KVStorage::new()?,
            secret: new_magic_crypt!(envvar::get("SECRET_KEY")?, 256),
            separator: "%3A".to_string(), // this is the character ":" url-encoded
        })
    }

    fn to_key(&self, parts: &[&str]) -> String {
        parts.join(self.separator.as_str())
    }

    pub async fn save_token(&self, user: &str, token: &str) -> Result<()> {
        let key = self.to_key(&[user, TOKEN_SUFFIX]);
        let value = self.secret.encrypt_str_to_base64(token);
        self.storage
            .save(key.as_str(), value.as_str(), None)
            .await?;
        Ok(())
    }

    pub async fn token(&self, user: &str) -> Result<String> {
        let key = self.to_key(&[user, TOKEN_SUFFIX]);
        let value = self.storage.get(key.as_str()).await?;
        Ok(self.secret.decrypt_base64_to_string(value)?)
    }

    pub async fn save_calendar(&self, user: &str, calendar: &str) -> Result<()> {
        let key = self.to_key(&[user, CALENDAR_SUFFIX]);
        self.storage
            .save(key.as_str(), calendar, Some(CALENDAR_TTL))
            .await?;
        Ok(())
    }

    pub async fn calendar(&self, user: &str) -> Result<String> {
        let key = self.to_key(&[user, CALENDAR_SUFFIX]);
        self.storage.get(key.as_str()).await
    }
}
