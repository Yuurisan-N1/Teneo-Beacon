use anyhow::{anyhow, Context, Result};

pub struct Account {
    pub email: String,
    pub token: String,
    pub nodes: usize,
}

pub fn load(path: &str) -> Result<Vec<Account>> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("the environment file {} could not be read", path))?;
    let mut out = Vec::new();
    for line in raw.lines() {
        let text = line.trim();
        if text.is_empty() || text.starts_with('#') {
            continue;
        }
        let (key, value) = match text.split_once('=') {
            Some(pair) => pair,
            None => continue,
        };
        if !key.trim().starts_with("ACCOUNTS_") {
            continue;
        }
        let parts: Vec<&str> = value.split('|').collect();
        if parts.len() < 3 {
            return Err(anyhow!(
                "the environment line {} is missing a field",
                key.trim()
            ));
        }
        let email = parts[0].trim().to_string();
        let token = parts[1].trim().to_string();
        let nodes = parts[2].trim().parse::<usize>().unwrap_or(1).max(1);
        if email.is_empty() || token.is_empty() {
            return Err(anyhow!("the environment line {} is empty", key.trim()));
        }
        out.push(Account {
            email,
            token,
            nodes,
        });
    }
    out.sort_by(|a, b| a.email.cmp(&b.email));
    Ok(out)
}
