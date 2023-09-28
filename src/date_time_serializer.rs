use chrono::NaiveDateTime;
use serde::{self, Deserialize, Deserializer};

const DATE_FORMAT: &str = "%Y-%m-%dT%H:%M:%SZ";

pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    NaiveDateTime::parse_from_str(&s, DATE_FORMAT).map_err(serde::de::Error::custom)
}
