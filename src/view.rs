//! Which two of the four axes are drawn inside each slice and which two
//! lay the slices out. A view is a permutation: screen axis k (0 across a
//! slice, 1 down it, 2 across the slices, 3 down the slices) shows maze
//! axis `axes[k]`. Six views, one per choice of the pair inside a slice;
//! the same maze, turned.

use crate::maze::{Dir, Pos};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct View(pub [usize; 4]);

pub const VIEWS: [View; 6] = [
    View([0, 1, 2, 3]),
    View([0, 2, 1, 3]),
    View([0, 3, 1, 2]),
    View([1, 2, 0, 3]),
    View([1, 3, 0, 2]),
    View([2, 3, 0, 1]),
];

pub const AXIS_NAMES: [&str; 4] = ["x", "y", "z", "w"];

impl View {
    /// The maze's sizes as the screen lays them out.
    pub fn dims(self, d: [usize; 4]) -> [usize; 4] {
        self.0.map(|a| d[a])
    }

    pub fn to_screen(self, p: Pos) -> Pos {
        self.0.map(|a| p[a])
    }

    pub fn to_maze(self, q: Pos) -> Pos {
        let mut p = [0; 4];
        for k in 0..4 {
            p[self.0[k]] = q[k];
        }
        p
    }

    /// A direction on screen as a direction in the maze.
    pub fn dir(self, screen: Dir) -> Dir {
        Dir((self.0[screen.axis()] * 2) as u8 + screen.0 % 2)
    }

    /// A direction in the maze as a direction on screen.
    pub fn screen_dir(self, d: Dir) -> Dir {
        let k = self.0.iter().position(|&a| a == d.axis()).expect("a view covers every axis");
        Dir((k * 2) as u8 + d.0 % 2)
    }

    /// "x y · z w": inside a slice, then across and down the slices.
    pub fn name(self) -> String {
        let n = |k: usize| AXIS_NAMES[self.0[k]];
        format!("{} {} · {} {}", n(0), n(1), n(2), n(3))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips() {
        for v in VIEWS {
            let p = [1, 2, 3, 4];
            assert_eq!(v.to_maze(v.to_screen(p)), p);
            for d in Dir::ALL {
                assert_eq!(v.screen_dir(v.dir(d)), d);
            }
        }
    }

    #[test]
    fn every_pair_inside_a_slice_once() {
        let mut pairs: Vec<[usize; 2]> = VIEWS.iter().map(|v| {
            let mut p = [v.0[0], v.0[1]];
            p.sort();
            p
        }).collect();
        pairs.sort();
        pairs.dedup();
        assert_eq!(pairs.len(), 6);
    }

    #[test]
    fn moves_follow_the_screen() {
        let v = VIEWS[5]; // z and w inside a slice
        assert_eq!(v.dir(Dir::RIGHT), Dir::ZPLUS);
        assert_eq!(v.dir(Dir::WPLUS), Dir::DOWN);
        assert_eq!(v.dims([5, 4, 3, 2]), [3, 2, 5, 4]);
    }
}
