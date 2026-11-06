use anyhow::{Context, Result};

use super::schema::Config;

pub fn load(path: &str) -> Result<Config> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("the config file {} could not be read", path))?;
    let cfg: Config =
        serde_json::from_str(&raw).with_context(|| "the config file is not valid json")?;
    Ok(cfg)
}
