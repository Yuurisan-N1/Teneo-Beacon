use anyhow::{anyhow, Result};
use serde_json::Value;

use crate::net::{client, endpoint};
use crate::support::error::refusal;
use crate::Ctx;

pub struct Points {
    pub total: u64,
    pub daily: u64,
}

pub async fn read(ctx: &Ctx, token: &str) -> Result<Points> {
    let head = client::bearer(token)?;
    let (status, value) = client::get_json(&ctx.client, &endpoint::points(), head).await?;
    if status != 200 {
        return Err(anyhow!("{}", refusal(status)));
    }
    Ok(Points {
        total: value.get("total").and_then(Value::as_u64).unwrap_or(0),
        daily: value.get("daily").and_then(Value::as_u64).unwrap_or(0),
    })
}
