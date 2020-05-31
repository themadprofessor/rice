use serde::Deserialize;
use std::fmt;

#[derive(Debug, Copy, Clone, Deserialize, Eq, PartialEq)]
pub enum IoClass {
    #[serde(alias = "idle")]
    Idle = 0,
    #[serde(alias = "best-effort")]
    BestEffort,
    #[serde(alias = "realtime")]
    RealTime,
}

impl fmt::Display for IoClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IoClass::Idle => f.write_str("idle"),
            IoClass::BestEffort => f.write_str("best effort"),
            IoClass::RealTime => f.write_str("real time"),
        }
    }
}
