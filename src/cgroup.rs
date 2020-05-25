use anyhow::{Context, Result};
use controlgroup::v1::{Builder, UnifiedRepr};
use log::warn;
use procfs::process::Process;
use serde::Deserialize;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::path::PathBuf;

const PERIOD_US: u64 = 100_000;

#[derive(Debug)]
pub struct Cgroup(UnifiedRepr);

#[derive(Debug, Deserialize)]
struct RawCgroup {
    cgroup: String,
    #[serde(alias = "CPUQuota")]
    cpu_quota: u8,
}

pub fn parse_cgroups() -> HashMap<String, Cgroup> {
    let mut map = HashMap::new();

    crate::parse::walk("/etc/ananicy.d/", "cgroups", |r: RawCgroup| {
        if r.cpu_quota > 100 {
            warn!("invalid CPUQuota {} for rule {}", r.cpu_quota, r.cgroup);
        } else {
            let quota =
                match (PERIOD_US * num_cpus::get() as u64 * r.cpu_quota as u64 / 100).try_into() {
                    Ok(x) => x,
                    Err(e) => {
                        warn!("failed to convert quota into i64: {}", e);
                        return;
                    }
                };

            let builder = Builder::new(PathBuf::from(&r.cgroup))
                .cpu()
                .shares(1025 * r.cpu_quota as u64 / 100)
                .cfs_period_us(PERIOD_US)
                .cfs_quota_us(quota)
                .done();

            let cgroup = match builder.build() {
                Ok(c) => c,
                Err(e) => {
                    warn!("failed to build cgroup {}: {}", &r.cgroup, e);
                    return;
                }
            };

            map.insert(r.cgroup, Cgroup(cgroup));
        }
    });

    map
}

impl Cgroup {
    pub fn apply(&mut self, proc: &Process) -> Result<()> {
        self.0
            .add_task(
                u32::try_from(proc.pid)
                    .context("failed to convert pid into unsigned int")?
                    .into(),
            )
            .context("failed to add process to cgroup")
    }
}
