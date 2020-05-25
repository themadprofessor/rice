use crate::class::IoClass;
use libc::c_int;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
struct RawType {
    #[serde(alias = "type")]
    proc_type: String,
    nice: Option<c_int>,
    ioclass: Option<IoClass>,
    ionice: Option<u8>,
    cgroup: Option<String>,
    oom_scote_adj: Option<c_int>,
}

#[derive(Debug)]
pub struct Type {
    pub nice: Option<c_int>,
    pub ioclass: Option<IoClass>,
    pub ionice: Option<u8>,
    pub cgroup: Option<String>,
    pub oom_scote_adj: Option<c_int>,
}

pub fn build_types() -> HashMap<String, Type> {
    let mut map = HashMap::new();
    crate::parse::walk("/etc/ananicy.d/", "types", |raw: RawType| {
        map.insert(
            raw.proc_type,
            Type {
                nice: raw.nice,
                ionice: raw.ionice,
                ioclass: raw.ioclass,
                cgroup: raw.cgroup,
                oom_scote_adj: raw.oom_scote_adj,
            },
        );
    });

    map
}
