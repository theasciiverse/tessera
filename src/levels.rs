//! What there is to play: the campaign (fixed mazes, the same for
//! everyone, each unlocking the next), the daily maze (one a day, from the
//! date), and endless mazes in four sizes.

use crate::rng::hash;

pub struct Level {
    pub name: &'static str,
    pub dims: [usize; 4],
    pub fog: bool,
}

/// x, y, z, w: x and y inside a slice, z slices side by side, w stacked.
pub const CAMPAIGN: [Level; 12] = [
    Level { name: "Side by side", dims: [4, 4, 2, 1], fog: false },
    Level { name: "Three rooms", dims: [4, 4, 3, 1], fog: false },
    Level { name: "Upstairs", dims: [4, 4, 1, 3], fog: false },
    Level { name: "The square", dims: [3, 3, 2, 2], fog: false },
    Level { name: "Four corners", dims: [4, 4, 2, 2], fog: false },
    Level { name: "Six rooms", dims: [4, 4, 3, 2], fog: false },
    Level { name: "Nine rooms", dims: [4, 4, 3, 3], fog: false },
    Level { name: "Wide halls", dims: [5, 5, 3, 2], fog: false },
    Level { name: "Lights out", dims: [4, 4, 3, 3], fog: true },
    Level { name: "The long way", dims: [5, 5, 3, 3], fog: false },
    Level { name: "Twelve rooms", dims: [5, 5, 4, 3], fog: false },
    Level { name: "Dark tesseract", dims: [5, 5, 3, 3], fog: true },
];

pub fn campaign_seed(i: usize) -> u64 {
    hash(&format!("tessera-campaign-{}", i + 1))
}

pub const DAILY_DIMS: [usize; 4] = [5, 5, 3, 3];

pub fn daily_seed(date: &str) -> u64 {
    hash(&format!("tessera-daily-{date}"))
}

pub struct Size {
    pub name: &'static str,
    pub dims: [usize; 4],
}

pub const ENDLESS: [Size; 4] = [
    Size { name: "small", dims: [4, 4, 2, 2] },
    Size { name: "medium", dims: [4, 4, 3, 3] },
    Size { name: "large", dims: [5, 5, 3, 3] },
    Size { name: "huge", dims: [5, 5, 4, 3] },
];

/// Today's date in UTC as YYYY-MM-DD, without a date library: days since
/// 1970 to a civil date (Howard Hinnant's days_from_civil, inverted).
pub fn today() -> String {
    civil(today_days())
}

/// Days since 1970-01-01, UTC.
pub fn today_days() -> i64 {
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
    (secs / 86_400) as i64
}

/// The day before a YYYY-MM-DD date, or None if it is not one.
pub fn day_before(date: &str) -> Option<String> {
    // walk back from a day near it; dates here are always recent
    let target = date.to_string();
    let mut d = today_days() + 2;
    for _ in 0..4000 {
        if civil(d) == target {
            return Some(civil(d - 1));
        }
        d -= 1;
    }
    None
}

pub fn civil(days: i64) -> String {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    format!("{y:04}-{m:02}-{d:02}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil_dates() {
        assert_eq!(civil(0), "1970-01-01");
        assert_eq!(civil(11_016), "2000-02-29");
        assert_eq!(civil(20_759), "2026-11-02");
    }

    #[test]
    fn campaign_mazes_need_the_hidden_axes() {
        for (i, l) in CAMPAIGN.iter().enumerate() {
            let m = crate::maze::Maze::generate(l.dims, campaign_seed(i));
            let path = m.path_to_goal(m.start);
            assert!(path.iter().any(|d| d.axis() >= 2), "level {} never leaves its slice", i + 1);
        }
    }
}

#[cfg(test)]
mod curve {
    use super::*;
    use crate::maze::{Dir, Maze};

    /// `cargo test campaign_curve -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn campaign_curve() {
        for (i, l) in CAMPAIGN.iter().enumerate() {
            let m = Maze::generate_long(l.dims, campaign_seed(i));
            let path = m.path_to_goal(m.start);
            // cells on the way with a side passage: places to go wrong
            let mut p = m.start;
            let mut forks = 0;
            let mut through = 0;
            for d in &path {
                let exits = Dir::ALL.iter().filter(|&&e| m.is_open(p, e)).count();
                if exits > 2 || (p == m.start && exits > 1) {
                    forks += 1;
                }
                if d.axis() >= 2 {
                    through += 1;
                }
                p = m.step(p, *d).unwrap();
            }
            println!("{:>2} {:<16} {:?} cells {:>3} path {:>3} forks {:>2} steps-through {:>2} fog {}", i + 1, l.name, l.dims, m.len(), path.len(), forks, through, l.fog);
        }
    }

    /// Which campaign levels can be played at 80x24 in some view.
    #[test]
    #[ignore]
    fn fits_eighty_by_twenty_four() {
        for (i, l) in CAMPAIGN.iter().enumerate() {
            let views: Vec<usize> = crate::view::VIEWS
                .iter()
                .enumerate()
                .filter(|(_, v)| {
                    let (w, h) = crate::render::needed(v.dims(l.dims));
                    w <= 80 && h <= 24
                })
                .map(|(k, _)| k + 1)
                .collect();
            println!("{:>2} {:<16} {:?} views that fit 80x24: {views:?}", i + 1, l.name, l.dims);
        }
    }
}
