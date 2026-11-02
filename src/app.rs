//! Screens and what each command does on them.

use crate::game::{fmt_time, Run};
use crate::levels::{self, CAMPAIGN, ENDLESS};
use crate::maze::{Dir, Maze};
use crate::store::Store;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Screen {
    Title,
    Campaign,
    Endless,
    Help,
    Play,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
    Campaign(usize),
    Daily(String),
    Endless(usize),
    /// the computer walks the way through
    Watch,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cmd {
    Move(Dir),
    Up,
    Down,
    Left,
    Right,
    Confirm,
    Back,
    Pause,
    Hint,
    Restart,
    NewMaze,
    Fog,
    Quit,
}

pub const TITLE_ITEMS: [&str; 6] = ["Campaign", "Daily maze", "Endless", "Watch", "How to play", "Quit"];

/// The watch mode's pace, steps per second.
const WATCH_STEP: Duration = Duration::from_millis(160);

pub struct App {
    pub screen: Screen,
    pub store: Store,
    pub cursor: usize,
    pub campaign_cursor: usize,
    pub endless_cursor: usize,
    pub run: Option<Run>,
    pub kind: Kind,
    pub paused: bool,
    pub quit: bool,
    pub seed: u64,
    pub today: String,
    /// shown under the maze for a moment
    pub message: Option<(String, Instant)>,
    /// set when a run ends: a new best, and the earlier best
    pub result: Option<(bool, Option<u64>)>,
    watch_next: Instant,
    watch_path: Vec<Dir>,
}

impl App {
    pub fn new(store: Store, seed: u64) -> App {
        let campaign_cursor = store.solved.min(CAMPAIGN.len() - 1);
        App {
            screen: Screen::Title,
            store,
            cursor: 0,
            campaign_cursor,
            endless_cursor: 1,
            run: None,
            kind: Kind::Watch,
            paused: false,
            quit: false,
            seed,
            today: levels::today(),
            message: None,
            result: None,
            watch_next: Instant::now(),
            watch_path: Vec::new(),
        }
    }

    pub fn key(&self) -> String {
        match &self.kind {
            Kind::Campaign(i) => format!("campaign-{}", i + 1),
            Kind::Daily(d) => format!("daily-{d}"),
            Kind::Endless(s) => format!("endless-{}{}", ENDLESS[*s].name, if self.store.fog { "-fog" } else { "" }),
            Kind::Watch => "watch".into(),
        }
    }

    pub fn title(&self) -> String {
        match &self.kind {
            Kind::Campaign(i) => format!("{} · {}", i + 1, CAMPAIGN[*i].name),
            Kind::Daily(d) => format!("Daily maze · {d}"),
            Kind::Endless(s) => format!("Endless · {}{}", ENDLESS[*s].name, if self.store.fog { " · fog" } else { "" }),
            Kind::Watch => "Watching".into(),
        }
    }

    pub fn start(&mut self, kind: Kind) {
        let (dims, seed, fog) = match &kind {
            Kind::Campaign(i) => (CAMPAIGN[*i].dims, levels::campaign_seed(*i), CAMPAIGN[*i].fog),
            Kind::Daily(d) => (levels::DAILY_DIMS, levels::daily_seed(d), false),
            Kind::Endless(s) => {
                self.seed = crate::rng::Rng::new(self.seed).next();
                (ENDLESS[*s].dims, self.seed, self.store.fog)
            }
            Kind::Watch => {
                self.seed = crate::rng::Rng::new(self.seed).next();
                ([5, 5, 3, 2], self.seed, false)
            }
        };
        log::info!("start {kind:?} dims {dims:?} seed {seed} fog {fog}");
        let maze = Maze::generate_long(dims, seed);
        if kind == Kind::Watch {
            self.watch_path = maze.path_to_goal(maze.start);
            self.watch_path.reverse();
            self.watch_next = Instant::now() + Duration::from_millis(1200);
        }
        self.run = Some(Run::new(maze, fog));
        self.kind = kind;
        self.paused = false;
        self.result = None;
        self.message = None;
        self.screen = Screen::Play;
    }

    /// The same maze again, from the start.
    pub fn restart(&mut self) {
        let Some(r) = self.run.take() else { return };
        log::info!("restart");
        let fog = r.fog;
        let maze = r.maze;
        if self.kind == Kind::Watch {
            self.watch_path = maze.path_to_goal(maze.start);
            self.watch_path.reverse();
        }
        self.run = Some(Run::new(maze, fog));
        self.paused = false;
        self.result = None;
    }

    pub fn tick(&mut self) {
        if self.message.as_ref().is_some_and(|(_, until)| Instant::now() >= *until) {
            self.message = None;
        }
        if let Some(r) = &mut self.run {
            if r.hint.as_ref().is_some_and(|(_, until)| Instant::now() >= *until) {
                r.hint = None;
            }
        }
        if self.screen == Screen::Play && self.kind == Kind::Watch && !self.paused && Instant::now() >= self.watch_next {
            if let Some(r) = &mut self.run {
                if r.finished.is_some() {
                    // a moment on the solved maze, then another
                    self.start(Kind::Watch);
                    return;
                }
                if let Some(d) = self.watch_path.pop() {
                    r.go(d);
                    self.watch_next = Instant::now() + WATCH_STEP;
                    if r.finished.is_some() {
                        self.watch_next = Instant::now() + Duration::from_millis(3000);
                    }
                }
            }
        }
    }

    pub fn apply(&mut self, cmd: Cmd) {
        log::debug!("cmd {cmd:?} on {:?}", self.screen);
        if cmd == Cmd::Quit {
            self.quit = true;
            return;
        }
        match self.screen {
            Screen::Title => match cmd {
                Cmd::Up | Cmd::Move(Dir::UP) => self.cursor = (self.cursor + TITLE_ITEMS.len() - 1) % TITLE_ITEMS.len(),
                Cmd::Down | Cmd::Move(Dir::DOWN) => self.cursor = (self.cursor + 1) % TITLE_ITEMS.len(),
                Cmd::Confirm => match self.cursor {
                    0 => self.screen = Screen::Campaign,
                    1 => {
                        let d = self.today.clone();
                        self.start(Kind::Daily(d));
                    }
                    2 => self.screen = Screen::Endless,
                    3 => self.start(Kind::Watch),
                    4 => self.screen = Screen::Help,
                    _ => self.quit = true,
                },
                Cmd::Back => self.quit = true,
                _ => {}
            },
            Screen::Campaign => {
                let open = self.store.solved.min(CAMPAIGN.len() - 1);
                match cmd {
                    Cmd::Up | Cmd::Move(Dir::UP) => self.campaign_cursor = self.campaign_cursor.saturating_sub(1),
                    Cmd::Down | Cmd::Move(Dir::DOWN) => self.campaign_cursor = (self.campaign_cursor + 1).min(CAMPAIGN.len() - 1),
                    Cmd::Confirm => {
                        if self.campaign_cursor <= open {
                            self.start(Kind::Campaign(self.campaign_cursor));
                        } else {
                            self.say(format!("Solve {} first", CAMPAIGN[open].name));
                        }
                    }
                    Cmd::Back => self.screen = Screen::Title,
                    _ => {}
                }
            }
            Screen::Endless => match cmd {
                Cmd::Up | Cmd::Move(Dir::UP) => self.endless_cursor = self.endless_cursor.saturating_sub(1),
                Cmd::Down | Cmd::Move(Dir::DOWN) => self.endless_cursor = (self.endless_cursor + 1).min(ENDLESS.len() - 1),
                Cmd::Fog | Cmd::Left | Cmd::Right | Cmd::Move(Dir::LEFT) | Cmd::Move(Dir::RIGHT) => {
                    self.store.fog = !self.store.fog;
                    self.store.save();
                }
                Cmd::Confirm => self.start(Kind::Endless(self.endless_cursor)),
                Cmd::Back => self.screen = Screen::Title,
                _ => {}
            },
            Screen::Help => {
                if matches!(cmd, Cmd::Back | Cmd::Confirm) {
                    self.screen = Screen::Title;
                }
            }
            Screen::Play => self.play_cmd(cmd),
        }
    }

    fn play_cmd(&mut self, cmd: Cmd) {
        let finished = self.run.as_ref().is_some_and(|r| r.finished.is_some());
        match cmd {
            Cmd::Back => {
                if let Some(r) = &mut self.run {
                    r.clock.stop();
                }
                self.message = None;
                self.screen = match self.kind {
                    Kind::Campaign(_) => Screen::Campaign,
                    Kind::Endless(_) => Screen::Endless,
                    _ => Screen::Title,
                };
                return;
            }
            Cmd::Pause if !finished && self.kind != Kind::Watch => {
                self.paused = !self.paused;
                if let Some(r) = &mut self.run {
                    if self.paused {
                        r.clock.stop();
                    } else if r.started {
                        r.clock.run();
                    }
                }
                return;
            }
            Cmd::Restart => return self.restart(),
            Cmd::NewMaze => {
                if let Kind::Endless(_) | Kind::Watch = self.kind {
                    let k = self.kind.clone();
                    self.start(k);
                }
                return;
            }
            _ => {}
        }
        if finished {
            if cmd == Cmd::Confirm {
                match self.kind.clone() {
                    Kind::Campaign(i) if i + 1 < CAMPAIGN.len() => self.start(Kind::Campaign(i + 1)),
                    Kind::Campaign(_) | Kind::Daily(_) => self.screen = Screen::Title,
                    k => self.start(k),
                }
            }
            return;
        }
        if self.paused {
            if cmd == Cmd::Confirm {
                self.apply(Cmd::Pause);
            }
            return;
        }
        if self.kind == Kind::Watch {
            return;
        }
        let Some(r) = &mut self.run else { return };
        match cmd {
            Cmd::Move(d) => {
                let went = r.go(d);
                log::debug!("move {} {} pos {:?} moves {}", d.name(), if went { "ok" } else { "wall" }, r.pos, r.moves);
                if !went && d.axis() >= 2 {
                    let msg = match d {
                        Dir::ZMINUS => "No way through to the slice on the left",
                        Dir::ZPLUS => "No way through to the slice on the right",
                        Dir::WMINUS => "No way through to the slice above",
                        _ => "No way through to the slice below",
                    };
                    self.say(msg.into());
                }
                if self.run.as_ref().is_some_and(|r| r.finished.is_some()) {
                    self.finish();
                }
            }
            Cmd::Hint => {
                r.hint();
                log::info!("hint {} at {:?}", r.hints, r.pos);
                self.say(format!("Hint: +{} s", crate::game::HINT_COST.as_secs()));
            }
            _ => {}
        }
    }

    fn finish(&mut self) {
        let Some(r) = &self.run else { return };
        let t = r.elapsed();
        let ms = t.as_millis() as u64;
        let key = self.key();
        let before = self.store.best.get(&key).copied();
        let best = self.store.record(&key, ms);
        if let Kind::Campaign(i) = self.kind {
            if i + 1 > self.store.solved {
                self.store.solved = i + 1;
                self.campaign_cursor = (i + 1).min(CAMPAIGN.len() - 1);
            }
        }
        self.store.save();
        log::info!("solved {key} in {} moves {} hints {} best {best}", fmt_time(t), r.moves, r.hints);
        self.result = Some((best, before));
    }

    pub fn say(&mut self, s: String) {
        self.message = Some((s, Instant::now() + Duration::from_millis(1800)));
    }

    pub fn snapshot(&self) -> String {
        match &self.run {
            Some(r) if self.screen == Screen::Play => format!(
                "{:?} {} pos {:?} moves {} hints {} time {} paused {} left {}",
                self.kind,
                self.key(),
                r.pos,
                r.moves,
                r.hints,
                fmt_time(r.elapsed()),
                self.paused,
                r.steps_left()
            ),
            _ => format!("{:?} cursor {}", self.screen, self.cursor),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> App {
        App::new(Store::default(), 1)
    }

    #[test]
    fn solving_a_campaign_level_unlocks_the_next() {
        let mut a = app();
        a.start(Kind::Campaign(0));
        let path = {
            let r = a.run.as_ref().unwrap();
            r.maze.path_to_goal(r.pos)
        };
        for d in path {
            a.apply(Cmd::Move(d));
        }
        assert!(a.result.is_some());
        assert_eq!(a.store.solved, 1);
        a.apply(Cmd::Confirm);
        assert_eq!(a.kind, Kind::Campaign(1));
    }

    #[test]
    fn locked_levels_stay_locked() {
        let mut a = app();
        a.screen = Screen::Campaign;
        a.campaign_cursor = 3;
        a.apply(Cmd::Confirm);
        assert_eq!(a.screen, Screen::Campaign);
        assert!(a.message.is_some());
    }

    #[test]
    fn pause_stops_moves() {
        let mut a = app();
        a.start(Kind::Daily("2026-11-02".into()));
        a.apply(Cmd::Pause);
        let before = a.run.as_ref().unwrap().pos;
        for d in Dir::ALL {
            a.apply(Cmd::Move(d));
        }
        assert_eq!(a.run.as_ref().unwrap().pos, before);
    }

    #[test]
    fn daily_is_the_same_maze_for_the_same_date() {
        let mut a = app();
        a.start(Kind::Daily("2026-11-02".into()));
        let p1 = a.run.as_ref().unwrap().maze.path_to_goal([0; 4]);
        let mut b = App::new(Store::default(), 999);
        b.start(Kind::Daily("2026-11-02".into()));
        assert_eq!(p1, b.run.as_ref().unwrap().maze.path_to_goal([0; 4]));
    }
}
