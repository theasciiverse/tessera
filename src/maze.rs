//! A maze in four dimensions. Cells sit on an x by y by z by w grid; each
//! has up to eight passages, two along each axis. On screen x and y are the
//! rows and columns of one slice, z picks the slice left or right, w the
//! slice up or down.
//!
//! Generation is a growing tree: start at one cell, keep a list of cells
//! that still have unvisited neighbours, and extend from one of them, chosen
//! newest-first most of the time (long corridors) and at random otherwise
//! (branches), and in a direction inside the slice three times as often
//! as between slices. Every cell ends up reachable by exactly one path.

use crate::rng::Rng;
use std::collections::VecDeque;

/// The eight directions: -x, +x, -y, +y, -z, +z, -w, +w.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Dir(pub u8);

impl Dir {
    pub const ALL: [Dir; 8] = [Dir(0), Dir(1), Dir(2), Dir(3), Dir(4), Dir(5), Dir(6), Dir(7)];
    pub const LEFT: Dir = Dir(0);
    pub const RIGHT: Dir = Dir(1);
    pub const UP: Dir = Dir(2);
    pub const DOWN: Dir = Dir(3);
    pub const ZMINUS: Dir = Dir(4);
    pub const ZPLUS: Dir = Dir(5);
    pub const WMINUS: Dir = Dir(6);
    pub const WPLUS: Dir = Dir(7);

    pub fn axis(self) -> usize {
        (self.0 / 2) as usize
    }

    pub fn sign(self) -> isize {
        if self.0.is_multiple_of(2) { -1 } else { 1 }
    }

    pub fn opposite(self) -> Dir {
        Dir(self.0 ^ 1)
    }

    pub fn bit(self) -> u8 {
        1 << self.0
    }

    pub fn name(self) -> &'static str {
        ["left", "right", "up", "down", "z-", "z+", "w-", "w+"][self.0 as usize]
    }
}

pub type Pos = [usize; 4];

/// How much likelier a passage inside a slice is than one between slices.
const IN_SLICE_WEIGHT: usize = 3;

/// How many mazes `generate_long` grows to pick from.
const VARIANTS: u64 = 48;

pub struct Maze {
    pub dims: [usize; 4],
    /// open passages per cell, one bit per Dir
    open: Vec<u8>,
    pub start: Pos,
    pub goal: Pos,
}

impl Maze {
    pub fn generate(dims: [usize; 4], seed: u64) -> Maze {
        assert!(dims.iter().all(|&d| d >= 1));
        let n = dims.iter().product();
        let mut m = Maze { dims, open: vec![0; n], start: [0; 4], goal: [dims[0] - 1, dims[1] - 1, dims[2] - 1, dims[3] - 1] };
        let mut rng = Rng::new(seed);
        let mut visited = vec![false; n];
        let first = m.start;
        visited[m.index(first)] = true;
        let mut active = vec![first];
        while !active.is_empty() {
            // newest three times in four, else any
            let i = if rng.below(4) == 0 { rng.below(active.len()) } else { active.len() - 1 };
            let here = active[i];
            let choices: Vec<(Dir, Pos)> = Dir::ALL
                .iter()
                .filter_map(|&d| m.step_raw(here, d).map(|p| (d, p)))
                .filter(|&(_, p)| !visited[m.index(p)])
                .collect();
            if choices.is_empty() {
                active.swap_remove(i);
                continue;
            }
            // passages inside a slice weigh three times a step between
            // slices, so slices get corridors and the steps through count
            let total: usize = choices.iter().map(|(d, _)| if d.axis() < 2 { IN_SLICE_WEIGHT } else { 1 }).sum();
            let mut pick = rng.below(total);
            let mut chosen = choices[0];
            for &(d, p) in &choices {
                let wgt = if d.axis() < 2 { IN_SLICE_WEIGHT } else { 1 };
                if pick < wgt {
                    chosen = (d, p);
                    break;
                }
                pick -= wgt;
            }
            let (d, next) = chosen;
            m.carve(here, d);
            visited[m.index(next)] = true;
            active.push(next);
        }
        m
    }

    /// Of `VARIANTS` mazes grown from the seed, the one whose way from the
    /// start to the goal is longest: corner to corner in four dimensions
    /// is often short, and a maze should be a walk.
    pub fn generate_long(dims: [usize; 4], seed: u64) -> Maze {
        let mut best: Option<(usize, Maze)> = None;
        for k in 0..VARIANTS {
            let m = Maze::generate(dims, seed.wrapping_add(k.wrapping_mul(0x9E37_79B9)));
            let len = m.distances(m.goal)[m.index(m.start)] as usize;
            if best.as_ref().is_none_or(|(b, _)| len > *b) {
                best = Some((len, m));
            }
        }
        best.expect("at least one variant").1
    }

    pub fn len(&self) -> usize {
        self.open.len()
    }

    pub fn index(&self, p: Pos) -> usize {
        let [a, b, c, _] = self.dims;
        p[0] + a * (p[1] + b * (p[2] + c * p[3]))
    }

    #[cfg(test)]
    pub fn pos(&self, mut i: usize) -> Pos {
        let mut p = [0; 4];
        for (k, d) in self.dims.iter().enumerate() {
            p[k] = i % d;
            i /= d;
        }
        p
    }

    /// The neighbouring cell in a direction, walls or not; None off the grid.
    pub fn step_raw(&self, p: Pos, d: Dir) -> Option<Pos> {
        let a = d.axis();
        let v = p[a] as isize + d.sign();
        if v < 0 || v as usize >= self.dims[a] {
            return None;
        }
        let mut q = p;
        q[a] = v as usize;
        Some(q)
    }

    pub fn is_open(&self, p: Pos, d: Dir) -> bool {
        self.open[self.index(p)] & d.bit() != 0
    }

    /// Where a move goes, if the passage is open.
    pub fn step(&self, p: Pos, d: Dir) -> Option<Pos> {
        if self.is_open(p, d) { self.step_raw(p, d) } else { None }
    }

    fn carve(&mut self, p: Pos, d: Dir) {
        let q = self.step_raw(p, d).expect("carving off the grid");
        let (i, j) = (self.index(p), self.index(q));
        self.open[i] |= d.bit();
        self.open[j] |= d.opposite().bit();
    }

    /// Steps from every cell to `to` (breadth first).
    pub fn distances(&self, to: Pos) -> Vec<u32> {
        let mut dist = vec![u32::MAX; self.len()];
        let mut q = VecDeque::new();
        dist[self.index(to)] = 0;
        q.push_back(to);
        while let Some(p) = q.pop_front() {
            let dp = dist[self.index(p)];
            for d in Dir::ALL {
                if let Some(n) = self.step(p, d) {
                    let i = self.index(n);
                    if dist[i] == u32::MAX {
                        dist[i] = dp + 1;
                        q.push_back(n);
                    }
                }
            }
        }
        dist
    }

    /// The shortest way from `from` to the goal, as directions.
    pub fn path_to_goal(&self, from: Pos) -> Vec<Dir> {
        let dist = self.distances(self.goal);
        let mut out = Vec::new();
        let mut p = from;
        while p != self.goal {
            let here = dist[self.index(p)];
            let Some((d, n)) = Dir::ALL.iter().filter_map(|&d| self.step(p, d).map(|n| (d, n))).find(|&(_, n)| dist[self.index(n)] + 1 == here)
            else {
                break;
            };
            out.push(d);
            p = n;
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn passages(m: &Maze) -> usize {
        (0..m.len()).map(|i| m.open[i].count_ones() as usize).sum::<usize>() / 2
    }

    #[test]
    fn every_cell_reachable_by_one_path() {
        for (dims, seed) in [([3, 3, 2, 2], 1), ([5, 5, 3, 3], 7), ([4, 6, 1, 3], 99), ([1, 1, 1, 1], 3)] {
            let m = Maze::generate(dims, seed);
            let dist = m.distances(m.start);
            assert!(dist.iter().all(|&d| d != u32::MAX), "{dims:?} unreachable cell");
            // a spanning tree has exactly cells - 1 passages: no loops
            assert_eq!(passages(&m), m.len() - 1, "{dims:?}");
        }
    }

    #[test]
    fn passages_are_symmetric_and_on_the_grid() {
        let m = Maze::generate([4, 4, 3, 2], 5);
        for i in 0..m.len() {
            let p = m.pos(i);
            assert_eq!(m.index(p), i);
            for d in Dir::ALL {
                if m.is_open(p, d) {
                    let q = m.step_raw(p, d).expect("open passage off the grid");
                    assert!(m.is_open(q, d.opposite()));
                }
            }
        }
    }

    #[test]
    fn same_seed_same_maze() {
        let a = Maze::generate([5, 5, 3, 3], 42);
        let b = Maze::generate([5, 5, 3, 3], 42);
        let c = Maze::generate([5, 5, 3, 3], 43);
        assert_eq!(a.open, b.open);
        assert_ne!(a.open, c.open);
    }

    #[test]
    fn path_reaches_goal() {
        let m = Maze::generate([5, 5, 3, 3], 11);
        let path = m.path_to_goal(m.start);
        let mut p = m.start;
        for d in &path {
            p = m.step(p, *d).expect("path walks through a wall");
        }
        assert_eq!(p, m.goal);
        assert_eq!(path.len() as u32, m.distances(m.goal)[m.index(m.start)]);
    }

    #[test]
    fn uses_the_hidden_axes() {
        // the way through should not stay in one slice
        let m = Maze::generate([5, 5, 3, 3], 3);
        let path = m.path_to_goal(m.start);
        assert!(path.iter().any(|d| d.axis() >= 2));
    }
}

#[cfg(test)]
mod stats {
    use super::*;

    #[test]
    #[ignore]
    fn path_lengths() {
        for dims in [[4, 4, 2, 1], [4, 4, 3, 3], [5, 5, 3, 3], [5, 5, 4, 3]] {
            let mut ls: Vec<usize> = (0..200).map(|s| {
                let m = Maze::generate_long(dims, s);
                m.path_to_goal(m.start).len()
            }).collect();
            ls.sort();
            let cells: usize = dims.iter().product();
            println!("{dims:?} cells {cells} min {} p25 {} median {} p75 {} max {}", ls[0], ls[50], ls[100], ls[150], ls[199]);
        }
    }
}
