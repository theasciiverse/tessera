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
    /// the last day the daily maze was solved, and how many days in a row
    pub daily_last: String,
    pub daily_streak: u32,
    /// best times in milliseconds, by run key (`campaign-3`, `daily-2026-11-02`, `endless-large`)
    pub best: BTreeMap<String, u64>,
}

/// Where Tessera keeps its files. Tests get a folder of their own under
/// the system's temp folder, never the player's.
pub fn data_dir() -> Option<PathBuf> {
    if cfg!(test) {
        return Some(std::env::temp_dir().join(format!("tessera-test-{}", std::process::id())));
    }
    let env = |k: &str| std::env::var_os(k).filter(|v| !v.is_empty()).map(PathBuf::from);
    let base = env("XDG_DATA_HOME")
        .or_else(|| if cfg!(windows) { env("APPDATA") } else { None })
        .or_else(|| env("HOME").map(|h| h.join(".local/share")))?;
    Some(base.join("tessera"))
}

fn path() -> Option<PathBuf> {
    Some(data_dir()?.join("state"))
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
                "daily_last" => s.daily_last = v.to_string(),
                "daily_streak" => s.daily_streak = v.parse().unwrap_or(0),
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
        let mut text = format!(
            "solved={}\nfog={}\nmuted={}\ndaily_last={}\ndaily_streak={}\n",
            self.solved, self.fog, self.muted, self.daily_last, self.daily_streak
        );
        for (k, ms) in &self.best {
            text.push_str(&format!("best.{k}={ms}\n"));
        }
        let result = p.parent().map_or(Ok(()), std::fs::create_dir_all).and_then(|_| std::fs::write(&p, text));
        if let Err(e) = result {
            log::warn!("can't save to {}: {e}", p.display());
        }
    }

    /// Counts a solved daily maze toward the streak of days in a row.
    pub fn daily_solved(&mut self, date: &str) {
        if self.daily_last == date {
            return;
        }
        let yesterday = crate::levels::day_before(date);
        self.daily_streak = if yesterday.as_deref() == Some(self.daily_last.as_str()) { self.daily_streak + 1 } else { 1 };
        self.daily_last = date.to_string();
    }

    /// The streak as it stands today: it lapses when yesterday was missed.
    pub fn streak_now(&self, today: &str) -> u32 {
        if self.daily_last == today || crate::levels::day_before(today).as_deref() == Some(self.daily_last.as_str()) {
            self.daily_streak
        } else {
            0
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::levels::{civil, today_days};

    #[test]
    fn streaks_count_days_in_a_row() {
        let mut s = Store::default();
        let d0 = civil(today_days() - 2);
        let d1 = civil(today_days() - 1);
        let d2 = civil(today_days());
        s.daily_solved(&d0);
        assert_eq!(s.daily_streak, 1);
        s.daily_solved(&d1);
        s.daily_solved(&d1);
        assert_eq!(s.daily_streak, 2);
        s.daily_solved(&d2);
        assert_eq!(s.daily_streak, 3);
        assert_eq!(s.streak_now(&d2), 3);
    }

    #[test]
    fn a_missed_day_breaks_the_streak() {
        let mut s = Store::default();
        s.daily_solved(&civil(today_days() - 3));
        s.daily_solved(&civil(today_days() - 2));
        assert_eq!(s.daily_streak, 2);
        assert_eq!(s.streak_now(&civil(today_days())), 0);
        s.daily_solved(&civil(today_days()));
        assert_eq!(s.daily_streak, 1);
    }
}
