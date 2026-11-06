use anyhow::{anyhow, Result};
use serde_json::{json, Value};

use crate::net::{client, endpoint};
use crate::support::error::refusal;

pub struct Beat {
    pub today: u64,
    pub total: u64,
}

pub async fn send(shared: &wreq::Client, token: &str) -> Result<Beat> {
    let head = client::bearer(token)?;
    let body = json!({});
    let (status, value) = client::post_json(shared, &endpoint::heartbeat(), head, &body).await?;
    if status != 200 && status != 201 {
        return Err(anyhow!("{}", refusal(status)));
    }
    let today = value
        .get("pointsToday")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let total = value
        .get("pointsTotal")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    Ok(Beat { today, total })
}
