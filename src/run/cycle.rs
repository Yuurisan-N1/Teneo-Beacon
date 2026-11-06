use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use tokio::task::JoinHandle;

use crate::support::banner::print_banner;
use crate::support::log::{countdown, describe, gap, lg, lr, ly, unit_word};
use crate::Ctx;

pub async fn run(ctx: &Arc<Ctx>) -> Result<()> {
    let mut handles: Vec<JoinHandle<()>> = Vec::new();
    let mut open_counts: Vec<Arc<AtomicU64>> = Vec::new();
    let mut cycle: u64 = 0;

    loop {
        cycle += 1;
        ly(&format!("Starting automation cycle number {}", cycle));

        for (index, account) in ctx.accounts.iter().enumerate() {
            if index > 0 {
                gap();
            }
            let keys = match crate::node::pool::ensure(ctx, account).await {
                Ok(keys) => keys,
                Err(err) => {
                    lr(&format!(
                        "The node cache could not be prepared because {}",
                        describe(&err)
                    ));
                    continue;
                }
            };

            let open = open_counts
                .get(index)
                .cloned()
                .unwrap_or_else(|| Arc::new(AtomicU64::new(0)));
            if open_counts.len() <= index {
                open_counts.push(open.clone());
            }

            if cycle == 1 {
                for (position, key) in keys.iter().enumerate() {
                    let label = position + 1;
                    let key = key.clone();
                    let token = account.token.clone();
                    let email = account.email.clone();
                    let open = open.clone();
                    handles.push(tokio::spawn(async move {
                        relay_loop(key, label, token, email, open).await;
                    }));
                }
                let beat_ctx = ctx.clone();
                let token = account.token.clone();
                let seconds = ctx.cfg.settings.heartbeat_minutes.max(1) * 60;
                handles.push(tokio::spawn(async move {
                    heartbeat_loop(beat_ctx, token, seconds).await;
                }));
            }

            let Some(first) = keys.first() else {
                lr("The account holds no node key so the beacon state is skipped");
                continue;
            };

            claim_all(ctx, &keys).await;
            report(ctx, first).await;
            lg(&format!(
                "The relay layer holds {} of {} channels open to the gateway",
                open.load(Ordering::Relaxed),
                keys.len()
            ));
            if crate::beacon::state::read(ctx, first)
                .await
                .map(|state| state.connected == 0)
                .unwrap_or(false)
            {
                lr("The gateway reports no active node for this account");
            }
        }

        countdown(ctx.cfg.settings.sleep_seconds, "Next cycle starts in");
        print_banner();
    }

}

async fn relay_loop(
    key: String,
    label: usize,
    token: String,
    email: String,
    open: Arc<AtomicU64>,
) {
    let mut attempt: u32 = 0;
    loop {
        match crate::relay::stream::open(&key, &token, &email).await {
            Ok(ws) => {
                attempt = 0;
                open.fetch_add(1, Ordering::Relaxed);
                lg(&format!(
                    "The relay channel number {} was opened to the gateway",
                    label
                ));
                let code = crate::relay::stream::serve(ws).await;
                open.fetch_sub(1, Ordering::Relaxed);
                if matches!(code, Some(4001) | Some(4005)) {
                    tokio::time::sleep(Duration::from_secs(1800)).await;
                    continue;
                }
            }
            Err(err) => {
                attempt += 1;
                if attempt == 1 {
                    lr(&format!(
                        "The relay channel number {} could not be opened because {}",
                        label,
                        describe(&err)
                    ));
                }
            }
        }
        let wait = (5u64 << attempt.min(6)).min(300);
        tokio::time::sleep(Duration::from_secs(wait)).await;
    }
}

async fn heartbeat_loop(ctx: Arc<Ctx>, token: String, seconds: u64) {
    loop {
        match crate::points::heartbeat::send(&ctx, &token).await {
            Ok(beat) => lg(&format!(
                "The heartbeat was credited with {} points and the total is {}",
                beat.today, beat.total
            )),
            Err(err) => lr(&format!("The heartbeat was refused because {}", describe(&err))),
        }
        tokio::time::sleep(Duration::from_secs(seconds)).await;
    }
}

pub async fn claim_all(ctx: &Ctx, keys: &[String]) {
    let Some(first) = keys.first() else {
        return;
    };
    let mut state = match crate::beacon::state::read(ctx, first).await {
        Ok(state) => state,
        Err(err) => {
            lr(&format!("The beacon state was not read because {}", describe(&err)));
            return;
        }
    };
    if let Some(next) = state.next_boost {
        if next > chrono::Utc::now() {
            ly("The claim is not due yet so the fragments stay unclaimed");
            return;
        }
    }
    if state.fragments == 0 {
        lg("The beacon state holds no unclaimed fragments right now");
        return;
    }

    let mut claimed: u64 = 0;
    let mut used: u64 = 0;
    for key in keys {
        if state.fragments == 0 {
            break;
        }
        match crate::beacon::boost::claim(ctx, key).await {
            Ok(done) => {
                used += 1;
                claimed += done.fragments;
                lg(&format!(
                    "The claim was credited with {} fragments so the power is {}",
                    done.fragments, done.power
                ));
                match crate::beacon::state::read(ctx, key).await {
                    Ok(fresh) => state.fragments = fresh.fragments,
                    Err(_) => state.fragments = state.fragments.saturating_sub(done.fragments),
                }
            }
            Err(err) => {
                lr(&format!("The claim was refused because {}", describe(&err)));
            }
        }
    }
    lg(&format!(
        "The account claimed {} fragments with {} {}",
        claimed,
        used,
        unit_word(used, "node", "nodes")
    ));
}

pub async fn report(ctx: &Ctx, key: &str) {
    if let Ok(state) = crate::beacon::state::read(ctx, key).await {
        lg(&format!(
            "The beacon state holds {} unclaimed fragments at power {} with {} {} online",
            state.fragments,
            state.power,
            state.connected,
            unit_word(state.connected, "node", "nodes")
        ));
    }
    if let Ok(fleet) = crate::beacon::fleet::read(ctx, key).await {
        lg(&format!(
            "The fleet shows {} of {} nodes online with {} fragments earned",
            fleet.online, fleet.total, fleet.fragments
        ));
    }
}
