#[cfg(not(unix))]
compile_error!("only unix systems are supported");

use anyhow::{Context, Result};
use log::{error, info, trace};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[cfg(log = "stderr")]
fn init_log() {
    pretty_env_logger::init();
}

#[cfg(log = "syslog")]
fn init_log() {
    syslog::init(syslog::Facility::LOG_DAEMON, log::LevelFilter::Debug, None).unwrap();
}

fn main() {
    init_log();
    if let Err(e) = run() {
        error!("{}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let mut cgroups = rice::parse_cgroups();
    let types = rice::build_types();
    let rules = rice::parse_rules(&types);

    info!("{} cgroups loaded", cgroups.len());
    trace!("{:?}", cgroups);
    info!("{} types loaded", types.len());
    trace!("{:?}", types);
    info!("{} rules loaded", rules.len());
    trace!("{:?}", rules);

    let term = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::SIGINT, Arc::clone(&term))
        .context("failed to register SIGINT handler")?;
    signal_hook::flag::register(signal_hook::SIGTERM, Arc::clone(&term))
        .context("failed to register SIGTERM handler")?;

    while !term.load(Ordering::Relaxed) {
        let errors = rice::apply_all_rules(&rules, &mut cgroups)?
            .filter_map(Result::err);

        for err in errors {
            error!("{}", err);
        }

        //40-50ms per loop

        let mut remaining = Duration::from_secs(5);
        while let Some(remain) = shuteye::sleep(remaining) {
            if term.load(Ordering::Relaxed) {
                break;
            }

            remaining = remain;
        }
    }

    Ok(())
}
