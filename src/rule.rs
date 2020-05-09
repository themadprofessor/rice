use crate::class::IoClass;
use crate::proc_type::Type;
use anyhow::{anyhow, bail, Context, Result};
use libc::c_int;
use nix::errno::Errno;
use procfs::process::Process;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

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
        self.apply_nice(proc)
    }

    pub fn apply_nice(&self, proc: &Process) -> Result<()> {
        if let Some(nice) = self
            .nice
            .or_else(|| self.proc_type.as_ref().and_then(|t| t.nice))
        {
            let ret;
            unsafe {
                // nix hasn't implemented setpriority yet
                Errno::clear();
                ret = libc::setpriority(libc::PRIO_PROCESS as u32, proc.pid as u32, nice);
            }
            if ret == -1 {
                let errno = nix::errno::errno();
                eprintln!("FAILURE");
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
            // changing ionice either needs a syscall or run the ionice program
            let mut cmd = std::process::Command::new("ionice");

            match c {
                IoClass::RealTime | IoClass::BestEffort => {
                    let nice = self
                        .ionice
                        .or_else(|| self.proc_type.as_ref().and_then(|t| t.ionice));
                    if let Some(n) = nice {
                        cmd.args(&["-n".to_string(), n.to_string()]);
                    }
                }
                _ => {}
            }

            cmd.args(&["-c".to_string(), c.to_string()]);

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

pub fn parse_rules(types: &HashMap<String, Type>) -> Result<HashMap<String, Rule>> {
    let mut map = HashMap::new();
    walk_dir("/etc/ananicy.d/", &mut map, types).context("failed to access config dir")?;
    Ok(map)
}

fn walk_dir<T>(
    path: T,
    map: &mut HashMap<String, Rule>,
    types: &HashMap<String, Type>,
) -> Result<()>
where
    T: AsRef<Path>,
{
    for f in std::fs::read_dir(path).context("failed to access rule dir")? {
        let path = f?.path();
        if path.is_dir() {
            walk_dir(path, map, types)?;
        } else if let Some(ex) = path.extension() {
            if ex == "rules" {
                parse_file(path, map, types)?;
            }
        }
    }

    Ok(())
}

fn parse_file<T>(
    path: T,
    map: &mut HashMap<String, Rule>,
    types: &HashMap<String, Type>,
) -> Result<()>
where
    T: AsRef<Path>,
{
    let mut f = BufReader::new(File::open(path).context("failed to open rule")?);
    crate::parse::parse(&mut f, |r: RawRule| {
        if let Some(nice) = r.ionice {
            if nice > 7 {
                eprintln!("invalid ionice value {}", nice);
                return;
            }
        }

        if let Some(nice) = r.nice {
            if nice > 20 || nice < -19 {
                eprintln!("invalid nice value {}", nice);
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
    })
}
