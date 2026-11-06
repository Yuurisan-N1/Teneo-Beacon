mod beacon;
mod cli;
mod net;
mod node;
mod points;
mod relay;
mod run;
mod support;

use std::process::ExitCode;
use std::sync::Mutex;

use support::log::{install, ly, lr};

pub struct Ctx {
    pub cfg: support::cfg::schema::Config,
    pub client: wreq::Client,
    pub store: Mutex<support::store::Store>,
    pub accounts: Vec<support::env::Account>,
}

fn main() -> ExitCode {
    let _ = rustls::crypto::ring::default_provider().install_default();
    install();

    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(err) => {
            lr(&format!("The program stopped: {}", support::error::sanitize(&err.to_string())));
            return ExitCode::FAILURE;
        }
    };

    match runtime.block_on(async_main()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            lr(&format!("The program stopped: {}", support::log::describe(&err)));
            ExitCode::FAILURE
        }
    }
}

async fn async_main() -> anyhow::Result<()> {
    support::signal::watch_interrupt();
    let mode = cli::dispatch::parse();
    support::banner::print_banner();
    let cfg = support::cfg::loader::load(support::consts::net::CONFIG_PATH)?;
    let proxy = net::proxy::load(support::consts::net::PROXY_PATH)?;
    if let Some(found) = proxy.as_ref() {
        ly(&format!("Using proxy {}", found.masked()));
    }
    let client = net::client::build(proxy.as_ref())?;
    let store = support::store::Store::load(support::consts::net::STORE_PATH)?;
    let accounts = support::env::load(support::consts::net::ENV_PATH)?;

    if accounts.is_empty() {
        lr("No account was found in the environment file");
        return Ok(());
    }

    let ctx = std::sync::Arc::new(Ctx {
        cfg,
        client,
        store: Mutex::new(store),
        accounts,
    });

    match mode {
        cli::dispatch::Mode::Run => run::cycle::run(&ctx).await,
        cli::dispatch::Mode::Probe => cli::probe::run(&ctx).await,
        cli::dispatch::Mode::SelfTest => cli::selftest::run(&ctx),
    }
}
