use log::warn;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct RawCgroup {
    cgroup: String,
    #[serde(alias = "CPUQuota")]
    cpu_quota: u8,
}

pub struct Cgroup {
    pub cpu_quota: u8,
}

pub fn parse_cgroups() -> HashMap<String, Cgroup> {
    let mut map = HashMap::new();

    crate::parse::walk("/etc/ananicy.d/", "cgroups", |r: RawCgroup| {
        if r.cpu_quota > 100 {
            warn!("invalid CPUQuota {} for rule {}", r.cpu_quota, r.cgroup);
        } else {
            map.insert(
                r.cgroup,
                Cgroup {
                    cpu_quota: r.cpu_quota,
                },
            );
        }
    });

    map
}
