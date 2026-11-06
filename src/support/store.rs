use std::collections::BTreeMap;
use std::io::Write;

use anyhow::{Context, Result};

pub struct Store {
    path: String,
    map: BTreeMap<String, Vec<String>>,
}

impl Store {
    pub fn load(path: &str) -> Result<Store> {
        let raw = std::fs::read_to_string(path).unwrap_or_default();
        let map: BTreeMap<String, Vec<String>> = serde_json::from_str(&raw).unwrap_or_default();
        Ok(Store {
            path: path.to_string(),
            map,
        })
    }

    pub fn keys(&self, email: &str) -> Vec<String> {
        self.map.get(email).cloned().unwrap_or_default()
    }

    pub fn add(&mut self, email: &str, key: String) -> Result<()> {
        self.map.entry(email.to_string()).or_default().push(key);
        self.save()
    }

    fn save(&self) -> Result<()> {
        let body = serde_json::to_string_pretty(&self.map)?;
        let tmp = format!("{}.tmp", self.path);
        let mut handle =
            std::fs::File::create(&tmp).with_context(|| "the cache file could not be created")?;
        handle.write_all(body.as_bytes())?;
        handle.sync_all()?;
        std::fs::rename(&tmp, &self.path)?;
        Ok(())
    }
}
