use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use std::io::BufRead;

pub fn parse<F, R, T>(reader: &mut R, mut func: F) -> Result<()>
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
            Err(e) => eprintln!("{:?}", e),
        }
    }
    Ok(())
}
