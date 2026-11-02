//! What carries over between runs: how far the campaign goes, best times,
//! and the fog setting for endless mazes. Kept in
//! $XDG_DATA_HOME/tessera/state (~/.local/share/tessera/state, or
//! %APPDATA%\tessera\state on Windows) as plain `key=value` lines.

use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Default, Debug)]
pub struct Store {
    /// campaign levels solved in a row from the first
    pub solved: usize,
    pub fog: bool,
    pub muted: bool,
    /// best times in milliseconds, by run key (`campaign-3`, `daily-2026-11-02`, `endless-large`)
    pub best: BTreeMap<String, u64>,
}

fn path() -> Option<PathBuf> {
    let env = |k: &str| std::env::var_os(k).filter(|v| !v.is_empty()).map(PathBuf::from);
    let base = env("XDG_DATA_HOME")
        .or_else(|| if cfg!(windows) { env("APPDATA") } else { None })
        .or_else(|| env("HOME").map(|h| h.join(".local/share")))?;
    Some(base.join("tessera").join("state"))
}

impl Store {
    pub fn load() -> Store {
        let mut s = Store::default();
        let Some(text) = path().and_then(|p| std::fs::read_to_string(p).ok()) else { return s };
        for line in text.lines() {
            let Some((k, v)) = line.split_once('=') else { continue };
            let (k, v) = (k.trim(), v.trim());
            match k {
                "solved" => s.solved = v.parse().unwrap_or(0),
                "fog" => s.fog = v == "true",
                "muted" => s.muted = v == "true",
                _ => {
                    if let (Some(key), Ok(ms)) = (k.strip_prefix("best."), v.parse()) {
                        s.best.insert(key.to_string(), ms);
                    }
                }
            }
        }
        s
    }

    pub fn save(&self) {
        let Some(p) = path() else { return };
        let mut text = format!("solved={}\nfog={}\nmuted={}\n", self.solved, self.fog, self.muted);
        for (k, ms) in &self.best {
            text.push_str(&format!("best.{k}={ms}\n"));
        }
        let result = p.parent().map_or(Ok(()), std::fs::create_dir_all).and_then(|_| std::fs::write(&p, text));
        if let Err(e) = result {
            log::warn!("can't save to {}: {e}", p.display());
        }
    }

    /// Records a time; true if it is a new best.
    pub fn record(&mut self, key: &str, ms: u64) -> bool {
        let better = self.best.get(key).is_none_or(|&b| ms < b);
        if better {
            self.best.insert(key.to_string(), ms);
        }
        better
    }
}
