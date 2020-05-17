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

#[derive(Debug, PartialEq, Clone)]
pub struct Type {
    pub nice: Option<c_int>,
    pub ioclass: Option<IoClass>,
    pub ionice: Option<u8>,
    pub cgroup: Option<String>,
    pub oom_scote_adj: Option<c_int>,
}

impl From<RawType> for (String, Type) {
    fn from(x: RawType) -> Self {
        (
            x.proc_type,
            Type {
                nice: x.nice,
                ioclass: x.ioclass,
                ionice: x.ionice,
                cgroup: x.cgroup,
                oom_scote_adj: x.oom_scote_adj,
            },
        )
    }
}

pub fn build_types() -> HashMap<String, Type> {
    let mut map = HashMap::new();
    crate::parse::walk("/etc/ananicy.d/", "types", |raw: RawType| {
        let (name, proc_type) = raw.into();
        map.insert(name, proc_type);
    });

    map
}
