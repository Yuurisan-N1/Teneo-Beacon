use anyhow::{anyhow, Result};
use serde_json::Value;

use crate::net::{client, endpoint};
use crate::support::error::refusal;
use crate::Ctx;

pub struct Fleet {
    pub online: u64,
    pub total: u64,
    pub fragments: u64,
    pub uptime: u64,
}

pub async fn read(ctx: &Ctx, key: &str) -> Result<Fleet> {
    let head = client::bearer(key)?;
    let (status, body) = client::get_json(&ctx.client, &endpoint::fleet(), head).await?;
    if status != 200 {
        return Err(anyhow!("{}", refusal(status)));
    }
    let list = match body.as_array() {
        Some(items) => items,
        None => return Err(anyhow!("the fleet answer carries no node list")),
    };
    let mut fleet = Fleet {
        online: 0,
        total: 0,
        fragments: 0,
        uptime: 0,
    };
    for node in list {
        fleet.total += 1;
        let state = node.get("status").and_then(Value::as_str).unwrap_or("");
        if state.eq_ignore_ascii_case("online") || state.eq_ignore_ascii_case("connected") {
            fleet.online += 1;
        }
        fleet.fragments += node
            .get("fragmentsEarned")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        fleet.uptime += node
            .get("uptimeSeconds")
            .and_then(Value::as_u64)
            .unwrap_or(0);
    }
    Ok(fleet)
}
