use anyhow::{anyhow, Result};
use serde_json::{json, Value};

use crate::net::{client, endpoint};
use crate::support::error::refusal;
use crate::Ctx;

pub async fn create(ctx: &Ctx, email: &str) -> Result<String> {
    let head = client::plain()?;
    let body = json!({ "accountId": email });
    let (status, value) = client::post_json(&ctx.client, &endpoint::register(), head, &body).await?;
    if status != 200 && status != 201 {
        return Err(anyhow!("{}", refusal(status)));
    }
    let key = value
        .get("apiKey")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    if key.is_empty() {
        return Err(anyhow!("the beacon gateway answered without a node key"));
    }
    Ok(key)
}
