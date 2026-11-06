use anyhow::Result;

use crate::support::env::Account;
use crate::support::log::{lg, lr, unit_word};
use crate::Ctx;

pub async fn ensure(ctx: &Ctx, account: &Account) -> Result<Vec<String>> {
    let mut keys = {
        let guard = ctx.store.lock().unwrap();
        guard.keys(&account.email)
    };
    if keys.len() >= account.nodes {
        lg(&format!(
            "The node cache holds {} {} for this account",
            keys.len(),
            unit_word(keys.len() as u64, "key", "keys")
        ));
        return Ok(keys);
    }
    let missing = account.nodes - keys.len();
    for _ in 0..missing {
        match crate::node::register::create(ctx, &account.email).await {
            Ok(key) => {
                {
                    let mut guard = ctx.store.lock().unwrap();
                    guard.add(&account.email, key.clone())?;
                }
                keys.push(key);
                lg("A new node key was added to the local cache");
            }
            Err(err) => {
                lr(&format!(
                    "The node key was not issued so the cache stays as it is because {}",
                    crate::support::log::describe(&err)
                ));
                break;
            }
        }
    }
    Ok(keys)
}
