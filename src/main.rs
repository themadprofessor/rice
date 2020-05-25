#[cfg(not(unix))]
compile_error!("only unix systems are supported");

use crate::cgroup::Cgroup;
use crate::rule::Rule;
use anyhow::{Context, Result};
use libc::pid_t;
use log::{error, info, trace, warn};
use procfs::process::Process;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

pub(crate) const ANANICY_CONFIG_DIR: &str = "/etc/ananicy.d";

mod cgroup;
mod class;
mod parse;
mod proc_type;
mod rule;

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
    let mut cgroups = cgroup::parse_cgroups();
    let types = proc_type::build_types();
    let rules = rule::parse_rules(&types);

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
        let errors = all_procs()?
            .filter_map(|p| {
                match p.exe() {
                    Ok(path) => path
                        .file_name()
                        .and_then(OsStr::to_str)
                        .and_then(|f| rules.get(f)),
                    Err(e) => {
                        error!("Are you root? {} [{:?}]", e, p.pid);
                        None
                    }
                }
                .map(|r| (r, p))
            })
            .map(|(r, p)| apply_rule(r, p, &mut cgroups))
            .filter_map(Result::err);

        for err in errors {
            error!("{}", err);
        }

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

fn apply_rule(r: &Rule, p: Process, cgroups: &mut HashMap<String, Cgroup>) -> Result<()> {
    r.apply(&p)?;
    if let Some(cgroup_name) = r.cgroup_name() {
        if let Some(cgroup) = cgroups.get_mut(cgroup_name) {
            cgroup.apply(&p)?;
        }
    }
    Ok(())
}

fn all_procs() -> Result<impl Iterator<Item = Process>> {
    let mut iter = ::std::fs::read_dir("/proc/")
        .context("failed to read /proc")?
        .filter_map(|p| p.ok())
        .filter(|p| p.path().is_dir())
        .filter_map(|p| pid_t::from_str(p.file_name().to_str()?).ok())
        .map(Process::new)
        .filter_map(|p| p.ok())
        .filter_map(|p| {
            let pid = p.pid;
            match all_threads(pid) {
                Ok(t) => Some(t.chain(::std::iter::once(p))),
                Err(e) => {
                    warn!("failed to read threads {}", e);
                    None
                }
            }
        })
        .flatten()
        .filter(|p| p.exe().is_ok());

    if iter.by_ref().peekable().peek().is_none() {
        Err(anyhow::anyhow!("no valid processes found"))
    } else {
        Ok(iter)
    }
}

fn all_threads(pid: pid_t) -> Result<impl Iterator<Item = Process>> {
    let path = format!("/proc/{}/task", pid);
    Ok(::std::fs::read_dir(&path)
        .with_context(|| format!("failed to read {}", path))?
        .filter_map(|p| p.ok())
        .filter(|p| p.path().is_dir())
        .filter_map(|p| pid_t::from_str(p.file_name().to_str()?).ok())
        .map(move |p| Process::new_with_root(PathBuf::from(path.as_str()).join(p.to_string())))
        .filter_map(|p| match p {
            Ok(proc) => Some(proc),
            Err(e) => {
                warn!("failed to parse thread {}", e);
                None
            }
        }))
}
