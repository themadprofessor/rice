#[cfg(not(unix))]
compile_error!("only unix systems are supported");

use anyhow::{Context, Result};
use libc::pid_t;
use log::{error, warn};
use procfs::process::Process;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::str::FromStr;

mod cgroup;
mod class;
mod parse;
mod proc_type;
mod rule;

fn main() {
    pretty_env_logger::init();
    if let Err(e) = run() {
        error!("{}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let mut cgroups = cgroup::parse_cgroups();
    let types = proc_type::build_types();
    let rules = rule::parse_rules(&types);

    all_procs()?
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
        .for_each(|(r, p)| {
            if let Err(e) = r.apply(&p) {
                error!("{}", e);
            }
            if let Some(cgroup) = r.cgroup_name() {
                if let Some(proc) = cgroups.get_mut(cgroup) {
                    if let Err(e) = proc.apply(&p) {
                        error!("{}", e);
                    }
                }
            }
        });

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
