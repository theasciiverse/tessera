//! Your best run on a fixed maze (campaign or daily), kept as its moves
//! and the clock time of each, replayed beside you next time. Kept in
//! $XDG_DATA_HOME/tessera/ghosts/<key>, one `ms direction` line per move.
//! A ghost that does not walk its maze (a different maze under the same
//! name) is dropped.

use crate::maze::{Dir, Maze, Pos};
use std::path::PathBuf;

pub struct Ghost {
    /// clock milliseconds and maze direction of each move
    moves: Vec<(u64, Dir)>,
    next: usize,
    pub pos: Pos,
    /// the best time it ran
    pub total: u64,
}

fn dir() -> Option<PathBuf> {
    Some(crate::store::data_dir()?.join("ghosts"))
}

/// Ghosts are only kept for mazes that are the same every time.
pub fn keeps(key: &str) -> bool {
    key.starts_with("campaign-") || key.starts_with("daily-")
}

pub fn save(key: &str, moves: &[(u64, Dir)], total: u64) {
    let Some(d) = dir() else { return };
    let mut text = format!("total {total}\n");
    for (ms, dir) in moves {
        text.push_str(&format!("{ms} {}\n", dir.0));
    }
    let p = d.join(key);
    if let Err(e) = std::fs::create_dir_all(&d).and_then(|_| std::fs::write(&p, text)) {
        log::warn!("can't save ghost {}: {e}", p.display());
    }
}

/// Today's daily ghosts are the only daily ones worth keeping; older
/// days' files are cleared out when a daily ghost is saved.
pub fn prune_dailies(keep: &str) {
    let Some(d) = dir() else { return };
    let Ok(entries) = std::fs::read_dir(&d) else { return };
    for e in entries.flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        if name.starts_with("daily-") && name != keep {
            let _ = std::fs::remove_file(e.path());
        }
    }
}

pub fn load(key: &str, maze: &Maze) -> Option<Ghost> {
    let text = std::fs::read_to_string(dir()?.join(key)).ok()?;
    parse(&text, maze)
}

pub fn parse(text: &str, maze: &Maze) -> Option<Ghost> {
    let mut lines = text.lines();
    let total = lines.next()?.strip_prefix("total ")?.trim().parse().ok()?;
    let mut moves = Vec::new();
    let mut p = maze.start;
    for l in lines {
        let (ms, d) = l.split_once(' ')?;
        let d: u8 = d.trim().parse().ok()?;
        if d > 7 {
            return None;
        }
        let d = Dir(d);
        // it must walk this maze, or it belongs to another
        p = maze.step(p, d)?;
        moves.push((ms.parse().ok()?, d));
    }
    if p != maze.goal {
        return None;
    }
    Some(Ghost { moves, next: 0, pos: maze.start, total })
}

impl Ghost {
    /// Brings the ghost up to the clock.
    pub fn advance(&mut self, maze: &Maze, now_ms: u64) {
        while let Some(&(ms, d)) = self.moves.get(self.next) {
            if ms > now_ms {
                break;
            }
            if let Some(n) = maze.step(self.pos, d) {
                self.pos = n;
            }
            self.next += 1;
        }
    }

    #[cfg(test)]
    pub fn done(&self) -> bool {
        self.next >= self.moves.len()
    }

    /// How far the ghost has gone, out of its moves.
    pub fn progress(&self) -> (usize, usize) {
        (self.next, self.moves.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn walk(maze: &Maze) -> Vec<(u64, Dir)> {
        maze.path_to_goal(maze.start).into_iter().enumerate().map(|(i, d)| (i as u64 * 250, d)).collect()
    }

    fn text(moves: &[(u64, Dir)], total: u64) -> String {
        let mut t = format!("total {total}\n");
        for (ms, d) in moves {
            t.push_str(&format!("{ms} {}\n", d.0));
        }
        t
    }

    #[test]
    fn replays_in_time() {
        let m = Maze::generate_long([4, 4, 2, 2], 5);
        let moves = walk(&m);
        let mut g = parse(&text(&moves, 9000), &m).expect("a good ghost");
        g.advance(&m, 0);
        assert_eq!(g.progress().0, 1);
        g.advance(&m, 1000);
        assert_eq!(g.progress().0, 5);
        g.advance(&m, u64::MAX);
        assert!(g.done());
        assert_eq!(g.pos, m.goal);
    }

    #[test]
    fn a_ghost_from_another_maze_is_dropped() {
        let a = Maze::generate_long([4, 4, 2, 2], 5);
        let b = Maze::generate_long([4, 4, 2, 2], 6);
        let t = text(&walk(&a), 9000);
        assert!(parse(&t, &a).is_some());
        assert!(parse(&t, &b).is_none());
        assert!(parse("nonsense", &a).is_none());
    }
}
