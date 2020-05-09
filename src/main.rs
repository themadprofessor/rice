use anyhow::{Context, Result};

use libc::pid_t;
use procfs::process::Process;

use std::ffi::OsStr;
use std::str::FromStr;

mod parse;
mod proc_type;
mod rule;

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let rules = {
        let types = proc_type::build_types()?;
        rule::parse_rules(&types)?
    };

    all_procs()?
        .filter_map(|p| {
            match p.exe() {
                Ok(path) => path
                    .file_name()
                    .and_then(OsStr::to_str)
                    .and_then(|f| rules.get(f)),
                Err(e) => {
                    eprintln!("Are you root? {} [{:?}]", e, p.pid);
                    None
                }
            }
            .map(|r| (r, p))
        })
        .for_each(|(r, p)| {
            if let Err(e) = r.apply(&p) {
                eprintln!("{}", e);
            }
        });

    Ok(())
}

fn all_procs() -> Result<impl Iterator<Item = Process>> {
    let mut iter = ::std::fs::read_dir("/proc/")
        .context("failed to read /proc")?
        .filter_map(|p| p.ok())
        .filter(|p| p.path().is_dir() && std::fs::read_link(p.path().join("exe")).is_ok())
        .filter_map(|p| pid_t::from_str(p.file_name().to_str()?).ok())
        .map(Process::new)
        .filter_map(|p| p.ok());

    if iter.by_ref().peekable().peek().is_none() {
        Err(anyhow::anyhow!("no valid processes found"))
    } else {
        Ok(iter)
    }
}
