use anyhow::Result;
use libc::c_int;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

#[derive(Deserialize, Debug)]
struct RawType {
    #[serde(alias = "type")]
    proc_type: String,
    nice: Option<c_int>,
    ioclass: Option<String>,
    ionice: Option<c_int>,
    cgroup: Option<String>,
    oom_scote_adj: Option<c_int>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Type {
    pub nice: Option<c_int>,
    pub ioclass: Option<String>,
    pub ionice: Option<c_int>,
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

pub fn build_types() -> Result<HashMap<String, Type>> {
    let mut map = HashMap::new();
    let mut reader = BufReader::new(File::open("/etc/ananicy.d/00-types.types")?);

    crate::parse::parse(&mut reader, |raw: RawType| {
        let (name, proc_type) = raw.into();
        map.insert(name, proc_type);
    })?;

    Ok(map)
}
