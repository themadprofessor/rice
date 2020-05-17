use anyhow::{Context, Result};
use log::warn;
use serde::de::DeserializeOwned;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

pub fn parse<F, R, T>(reader: &mut R, func: &mut F) -> Result<()>
where
    F: FnMut(T),
    R: BufRead,
    T: DeserializeOwned,
{
    let mut line = String::new();

    loop {
        line.clear();
        let n = reader.read_line(&mut line).context("failed to read file")?;
        if n == 0 {
            break;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        match serde_json::from_str::<T>(&line).context::<&str>("failed to parse") {
            Ok(t) => func(t),
            Err(e) => warn!("{}", e),
        }
    }
    Ok(())
}

pub fn walk<F, P, T>(root: P, ext: &str, mut func: F)
where
    F: FnMut(T),
    T: DeserializeOwned,
    P: AsRef<Path>,
{
    let walker =
        WalkDir::new(root)
            .into_iter()
            .filter_entry(|e: &DirEntry| match e.path().extension() {
                Some(p) => p == ext,
                None => false,
            });

    for entry in walker {
        if let Ok(entry) = entry {
            let f = match File::open(entry.path()) {
                Ok(x) => x,
                Err(e) => {
                    warn!("failed to open {} {}", ext, e);
                    continue;
                }
            };

            let mut read = BufReader::new(f);
            if let Err(e) = parse(&mut read, &mut func) {
                warn!("failed to parse {} {}", entry.path().display(), e)
            }
        }
    }
}
