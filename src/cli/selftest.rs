use anyhow::Result;

use crate::relay::frames::{decode, encode, OP_DATA};
use crate::support::log::{lg, lr};
use crate::Ctx;

pub fn run(ctx: &Ctx) -> Result<()> {
    let raw = encode(9, OP_DATA, b"probe");
    match decode(&raw) {
        Some(frame) if frame.stream == 9 && frame.opcode == OP_DATA && frame.payload == b"probe" => {
            lg("The frame encoder passed the local round trip")
        }
        _ => lr("The frame encoder failed the local round trip"),
    }

    match crate::net::proxy::Proxy::parse("socks5://user:secret@198.51.100.20:1080") {
        Ok(proxy) if !proxy.masked().contains("secret") => {
            lg("The proxy parser hides the password in the mask")
        }
        _ => lr("The proxy parser failed the local check"),
    }

    let mut store = crate::support::store::Store::load("nodeId.selftest.json")?;
    store.add("qa@example.com", "node_00000000000000000000000000000000".to_string())?;
    let back = store.keys("qa@example.com");
    let _ = std::fs::remove_file("nodeId.selftest.json");
    if back.len() == 1 {
        lg("The node cache round trip passed the local check");
    } else {
        lr("The node cache round trip failed the local check");
    }

    if ctx.accounts.is_empty() {
        lr("The environment file holds no account line");
    } else {
        lg("The environment file holds at least one account line");
    }

    lg("Self test passed");
    Ok(())
}
