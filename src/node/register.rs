use anyhow::{anyhow, Result};
use serde_json::{json, Value};

use crate::net::proxy::Proxy;
use crate::net::{client, endpoint};
use crate::support::error::refusal;
use crate::Ctx;

pub async fn create(ctx: &Ctx, email: &str, proxy: Option<&Proxy>) -> Result<String> {
    let shared = match proxy {
        Some(found) => client::build(Some(found))?,
        None => ctx.client.clone(),
    };
    let head = client::plain()?;
    let body = json!({ "accountId": email });
    let (status, value) = client::post_json(&shared, &endpoint::register(), head, &body).await?;
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
