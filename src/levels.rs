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
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
    civil((secs / 86_400) as i64)
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
