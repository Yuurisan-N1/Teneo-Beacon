use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::net::{client, endpoint};
use crate::support::error::refusal;
use crate::Ctx;

pub struct Beacon {
    pub fragments: u64,
    pub power: u64,
    pub connected: u64,
    pub next_boost: Option<DateTime<Utc>>,
}

pub async fn read(ctx: &Ctx, key: &str) -> Result<Beacon> {
    let head = client::bearer(key)?;
    let (status, body) = client::get_json(&ctx.client, &endpoint::state(), head).await?;
    if status != 200 {
        return Err(anyhow!("{}", refusal(status)));
    }
    let fragments = body
        .get("unclaimedFragments")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let power = body
        .get("beaconPower")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let connected = body
        .get("connectedNodes")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let next_boost = body
        .get("nextBoostAt")
        .and_then(Value::as_str)
        .and_then(|text| DateTime::parse_from_rfc3339(text).ok())
        .map(|stamp| stamp.with_timezone(&Utc));
    Ok(Beacon {
        fragments,
        power,
        connected,
        next_boost,
    })
}
