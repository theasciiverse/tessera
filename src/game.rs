//! One run through one maze: where you are, what you have seen, the clock.

use crate::maze::{Dir, Maze, Pos};
use std::time::{Duration, Instant};

/// A stopwatch that can pause.
#[derive(Default)]
pub struct Clock {
    acc: Duration,
    since: Option<Instant>,
}

impl Clock {
    pub fn run(&mut self) {
        if self.since.is_none() {
            self.since = Some(Instant::now());
        }
    }
    pub fn stop(&mut self) {
        if let Some(s) = self.since.take() {
            self.acc += s.elapsed();
        }
    }
    pub fn add(&mut self, d: Duration) {
        self.acc += d;
    }
    pub fn elapsed(&self) -> Duration {
        self.acc + self.since.map_or(Duration::ZERO, |s| s.elapsed())
    }
}

/// Seconds added to the time for each hint.
pub const HINT_COST: Duration = Duration::from_secs(10);
/// How many steps of the way a hint shows, and for how long.
pub const HINT_STEPS: usize = 4;
pub const HINT_SHOWN: Duration = Duration::from_millis(2500);

pub struct Run {
    pub maze: Maze,
    pub pos: Pos,
    pub fog: bool,
    pub seen: Vec<bool>,
    pub trail: Vec<bool>,
    pub moves: u32,
    pub hints: u32,
    pub clock: Clock,
    pub started: bool,
    pub finished: Option<Duration>,
    /// a move into a wall, for a moment's flash
    pub bump: Option<(Pos, Dir, Instant)>,
    /// cells a hint is showing, until when
    pub hint: Option<(Vec<Pos>, Instant)>,
}

impl Run {
    pub fn new(maze: Maze, fog: bool) -> Run {
        let n = maze.len();
        let mut r = Run {
            pos: maze.start,
            maze,
            fog,
            seen: vec![false; n],
            trail: vec![false; n],
            moves: 0,
            hints: 0,
            clock: Clock::default(),
            started: false,
            finished: None,
            bump: None,
            hint: None,
        };
        r.look();
        r
    }

    /// Marks the cell you stand in, and everything one open step away, as seen.
    fn look(&mut self) {
        let p = self.pos;
        let i = self.maze.index(p);
        self.seen[i] = true;
        self.trail[i] = true;
        for d in Dir::ALL {
            if let Some(n) = self.maze.step(p, d) {
                let j = self.maze.index(n);
                self.seen[j] = true;
            }
        }
    }

    /// Tries a move; true if it went through.
    pub fn go(&mut self, d: Dir) -> bool {
        if self.finished.is_some() {
            return false;
        }
        match self.maze.step(self.pos, d) {
            Some(n) => {
                if !self.started {
                    self.started = true;
                    self.clock.run();
                }
                self.pos = n;
                self.moves += 1;
                self.bump = None;
                self.look();
                if self.pos == self.maze.goal {
                    self.clock.stop();
                    self.finished = Some(self.clock.elapsed());
                }
                true
            }
            None => {
                self.bump = Some((self.pos, d, Instant::now()));
                false
            }
        }
    }

    /// Shows the next few steps toward the goal, at a cost.
    pub fn hint(&mut self) {
        if self.finished.is_some() {
            return;
        }
        let mut cells = Vec::new();
        let mut p = self.pos;
        for d in self.maze.path_to_goal(self.pos).into_iter().take(HINT_STEPS) {
            p = self.maze.step(p, d).expect("path through a wall");
            cells.push(p);
        }
        for c in &cells {
            let i = self.maze.index(*c);
            self.seen[i] = true;
        }
        self.hints += 1;
        self.clock.add(HINT_COST);
        if !self.started {
            self.started = true;
            self.clock.run();
        }
        self.hint = Some((cells, Instant::now() + HINT_SHOWN));
    }

    pub fn elapsed(&self) -> Duration {
        self.finished.unwrap_or_else(|| self.clock.elapsed())
    }

    pub fn steps_left(&self) -> u32 {
        self.maze.distances(self.maze.goal)[self.maze.index(self.pos)]
    }
}

pub fn fmt_time(d: Duration) -> String {
    let ms = d.as_millis() as u64;
    fmt_ms(ms)
}

pub fn fmt_ms(ms: u64) -> String {
    let (m, s, t) = (ms / 60_000, (ms / 1000) % 60, (ms / 100) % 10);
    format!("{m}:{s:02}.{t}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn walking_the_path_finishes() {
        let m = Maze::generate([4, 4, 3, 3], 9);
        let path = m.path_to_goal(m.start);
        let mut r = Run::new(m, false);
        for d in path {
            assert!(r.go(d));
        }
        assert!(r.finished.is_some());
        assert_eq!(r.steps_left(), 0);
    }

    #[test]
    fn walls_bump() {
        let m = Maze::generate([4, 4, 3, 3], 9);
        let shut = Dir::ALL.into_iter().find(|&d| !m.is_open(m.start, d)).expect("start has a wall");
        let mut r = Run::new(m, false);
        assert!(!r.go(shut));
        assert!(r.bump.is_some());
        assert_eq!(r.moves, 0);
        assert!(!r.started, "a bump does not start the clock");
    }

    #[test]
    fn hint_costs_time() {
        let m = Maze::generate([4, 4, 3, 3], 9);
        let mut r = Run::new(m, true);
        r.hint();
        assert_eq!(r.hints, 1);
        assert!(r.elapsed() >= HINT_COST);
        assert!(r.hint.as_ref().is_some_and(|(c, _)| !c.is_empty()));
    }

    #[test]
    fn times() {
        assert_eq!(fmt_ms(83_456), "1:23.4");
        assert_eq!(fmt_ms(5_000), "0:05.0");
    }
}
