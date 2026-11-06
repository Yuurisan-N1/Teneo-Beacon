use anyhow::{anyhow, Result};
use serde_json::{json, Value};

use crate::net::{client, endpoint};
use crate::support::error::refusal;
use crate::Ctx;

pub struct Claim {
    pub fragments: u64,
    pub power: u64,
}

pub async fn claim(ctx: &Ctx, key: &str) -> Result<Claim> {
    let token = crate::beacon::challenge::token(ctx, key).await?;
    let head = client::bearer(key)?;
    let body = json!({ "token": token });
    let (status, value) = client::post_json(&ctx.client, &endpoint::boost(), head, &body).await?;
    if status != 200 {
        return Err(anyhow!("{}", refusal(status)));
    }
    if value.get("success").and_then(Value::as_bool) == Some(false) {
        let text = value.get("error").and_then(Value::as_str).unwrap_or_default();
        if text.to_ascii_lowercase().contains("cooldown") {
            return Err(anyhow!("the claim is still on cooldown"));
        }
        return Err(anyhow!("the gateway refused the claim"));
    }
    let fragments = value
        .get("fragmentsEarned")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let power = value
        .get("beaconPower")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    Ok(Claim { fragments, power })
}
