use anyhow::Result;

use crate::support::log::{describe, lg, lr, ly};
use crate::Ctx;

pub async fn run(ctx: &Ctx) -> Result<()> {
    ly("Probe mode runs read only so nothing is claimed");
    for account in &ctx.accounts {
        let keys = {
            let guard = ctx.store.lock().unwrap();
            guard.keys(&account.email)
        };
        let Some(key) = keys.first() else {
            lr("The account holds no node key so the probe is skipped");
            continue;
        };
        match crate::beacon::state::read(ctx, key).await {
            Ok(state) => lg(&format!(
                "The beacon state holds {} unclaimed fragments at power {} with {} nodes online",
                state.fragments, state.power, state.connected
            )),
            Err(err) => lr(&format!("The beacon state was not read {}", describe(&err))),
        }
        match crate::beacon::fleet::read(ctx, key).await {
            Ok(fleet) => lg(&format!(
                "The fleet shows {} of {} nodes online with {} fragments earned",
                fleet.online, fleet.total, fleet.fragments
            )),
            Err(err) => lr(&format!("The fleet was not read {}", describe(&err))),
        }
        match crate::points::streak::read(ctx, &account.token).await {
            Ok(points) => lg(&format!(
                "The account holds {} points and {} were earned today",
                points.total, points.daily
            )),
            Err(err) => lr(&format!("The points were not read {}", describe(&err))),
        }
    }
    Ok(())
}
