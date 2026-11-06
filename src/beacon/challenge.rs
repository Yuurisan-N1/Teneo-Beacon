use anyhow::{anyhow, Result};
use serde_json::Value;

use crate::net::{client, endpoint};
use crate::support::error::refusal;
use crate::Ctx;

pub async fn token(ctx: &Ctx, key: &str) -> Result<String> {
    let head = client::bearer(key)?;
    let (status, body) = client::get_json(&ctx.client, &endpoint::challenge(), head).await?;
    if status != 200 {
        return Err(anyhow!("{}", refusal(status)));
    }
    let token = body
        .get("token")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    if token.is_empty() {
        return Err(anyhow!("the challenge answer carries no token"));
    }
    Ok(token)
}
