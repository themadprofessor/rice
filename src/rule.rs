use crate::class::IoClass;
use crate::proc_type::Type;
use anyhow::{anyhow, bail, Context, Result};
use libc::c_int;
use log::{debug, warn};
use nix::errno::Errno;
use procfs::process::Process;
use serde::Deserialize;
use std::collections::HashMap;
use crate::cgroup::Cgroup;

#[derive(Deserialize, Debug, PartialEq)]
struct RawRule {
    name: String,
    #[serde(alias = "type")]
    proc_type: Option<String>,
    nice: Option<c_int>,
    #[serde(alias = "io-class")]
    io_class: Option<IoClass>,
    ionice: Option<u8>,
    cgroup: Option<String>,
}

#[derive(Debug)]
pub struct Rule {
    pub proc_type: Option<Type>,
    pub nice: Option<c_int>,
    pub io_class: Option<IoClass>,
    pub ionice: Option<u8>,
    pub cgroup: Option<String>,
}

impl Rule {
    pub fn apply(&self, proc: &Process) -> Result<()> {
        self.apply_nice(proc)?;
        self.apply_io(proc)
    }

    pub fn apply_nice(&self, proc: &Process) -> Result<()> {
        if let Some(nice) = self
            .nice
            .or_else(|| self.proc_type.as_ref().and_then(|t| t.nice))
        {
            debug!("applying nice value {} to {}", nice, proc.pid);
            let ret;
            unsafe {
                // nix hasn't implemented setpriority yet
                Errno::clear();
                ret = libc::setpriority(libc::PRIO_PROCESS as u32, proc.pid as u32, nice);
            }
            if ret == -1 {
                let errno = nix::errno::errno();
                if errno != 0 {
                    let errno = Errno::from_i32(errno);
                    return Err(match errno {
                        Errno::EINVAL => panic!("invalid which value"), // can't happen
                        Errno::ESRCH => anyhow!("process [{}] not found", proc.pid),
                        Errno::EACCES => {
                            anyhow!("permission denied or nice value larger than rlimit")
                        }
                        Errno::EPERM => anyhow!("permission denied"),
                        _ => panic!("unexpected errno [{}]", errno),
                    });
                }
            }
        }

        Ok(())
    }

    pub fn apply_io(&self, proc: &Process) -> Result<()> {
        let class = self
            .io_class
            .or_else(|| self.proc_type.as_ref().and_then(|t| t.ioclass));

        if let Some(c) = class {
            debug!("applying ioclass {} to {}", c, proc.pid);

            // changing ionice either needs a syscall or run the ionice program
            let mut cmd = std::process::Command::new("ionice");

            match c {
                IoClass::RealTime | IoClass::BestEffort => {
                    let nice = self
                        .ionice
                        .or_else(|| self.proc_type.as_ref().and_then(|t| t.ionice));
                    if let Some(n) = nice {
                        debug!("applying ionice {} to {}", n, proc.pid);
                        cmd.args(&["-n".to_string(), n.to_string()]);
                    }
                }
                _ => {}
            }

            cmd.args(&["-c", &(c as u8).to_string(), "-p", &proc.pid.to_string()]);

            if !cmd
                .spawn()
                .context("failed to set ionice")?
                .wait()
                .context("failed to wait for ionice")?
                .success()
            {
                bail!("failed to find process [{}]", proc.pid);
            }
        }

        Ok(())
    }
}

pub fn parse_rules(
    types: &HashMap<String, Type>,
    _cgroups: &HashMap<String, Cgroup>,
) -> HashMap<String, Rule> {
    let mut map = HashMap::new();
    crate::parse::walk("/etc/ananicy.d/", "rules", |r: RawRule| {
        if let Some(nice) = r.ionice {
            if nice > 7 {
                warn!("invalid ionice value {} for rule {}", nice, r.name);
                return;
            }
        }

        if let Some(nice) = r.nice {
            if nice > 20 || nice < -19 {
                warn!("invalid nice value {} for rule {}", nice, r.name);
                return;
            }
        }

        let (name, rule) = (
            r.name,
            Rule {
                proc_type: r.proc_type.and_then(|t| types.get(&t)).map(Clone::clone),
                nice: r.nice,
                io_class: r.io_class,
                ionice: r.ionice,
                cgroup: r.cgroup,
            },
        );

        map.insert(name, rule);
    });

    map
}
