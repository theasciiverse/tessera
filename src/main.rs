mod app;
mod audio;
mod game;
mod ghost;
mod levels;
mod maze;
mod render;
mod rng;
mod store;
mod view;

use app::{App, Cmd, Kind, Screen};
use maze::Dir;
use ratatui::crossterm::event::{self, DisableFocusChange, EnableFocusChange, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::crossterm::execute;
use std::io;
use std::time::{Duration, Instant, SystemTime};

const FRAME: Duration = Duration::from_millis(50);
const SNAPSHOT_EVERY: Duration = Duration::from_millis(500);

const HELP: &str = "tessera [--daily] [--level N] [--endless small|medium|large|huge] [--fog] [--watch] [--mute] [--seed N]

Mazes in four dimensions. Every box on screen is one flat slice of the
maze; the boxes beside and above each other are its neighbours in the
third and fourth dimensions. Walk from the top left of the first slice
to the pink diamond in the bottom right of the last.

Arrows or WASD walk inside a slice. J and L step to the slice on the
left or right, I and K to the slice above or below (or Shift with the
arrows). In a cell, cyan ◂ ▸ ↔ mark a way through to the slices left and
right, amber ▴ ▾ ↕ to the slices above and below.
V (or Tab) turns the maze: any two of the four directions can be the
slices, six views of the same maze; walking follows the screen.
H shows the next steps (+10 s), P pauses, R restarts, M turns the
sound off and on, G hides or shows your best run, Esc goes back.
Gamepad: left stick or d-pad walk, right stick or face buttons step
between slices, the triggers turn the maze, LB hints, Start pauses,
Back goes back.

  --daily         today's daily maze, the same for everyone
  --level N       campaign maze N (1 to 12)
  --endless SIZE  a new maze: small, medium, large or huge
  --fog           endless mazes in fog: you see only where you have been
  --watch         watch the computer walk the way through
  --mute          no sound (M in the game; remembered)
  --seed N        fixed random seed for endless and watch mazes

Best times and campaign progress live in ~/.local/share/tessera/state
(%APPDATA%\\tessera\\state on Windows).
Logging: RUST_LOG=debug tessera 2>tessera.log
";

fn main() -> io::Result<()> {
    env_logger::init();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let has = |f: &str| args.iter().any(|a| a == f);
    let after = |f: &str| args.iter().position(|a| a == f).and_then(|i| args.get(i + 1)).cloned();
    if has("--help") || has("-h") {
        print!("{HELP}");
        return Ok(());
    }
    if has("--version") || has("-V") {
        println!("tessera {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    let fail = |msg: &str| -> ! {
        eprintln!("{msg}");
        std::process::exit(2);
    };
    let seed = after("--seed")
        .map(|v| v.parse().unwrap_or_else(|_| fail("--seed takes a number")))
        .unwrap_or_else(|| SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).map_or(1, |d| d.as_nanos() as u64));
    let store = store::Store::load();
    // open audio before the alternate screen, so any ALSA chatter lands above it
    let audio = audio::Audio::new(store.muted || has("--mute"));
    log::info!("start version {} seed {seed} audio {} muted {} store {store:?}", env!("CARGO_PKG_VERSION"), audio.available(), audio.muted);
    let mut app = App::new(store, seed, audio);
    // the terminal's size before any maze starts, so it starts in a view that fits
    if let Ok((w, h)) = ratatui::crossterm::terminal::size() {
        app.term = (w, h);
    }
    if has("--fog") {
        app.store.fog = true;
    }
    if has("--daily") {
        let d = app.today.clone();
        app.start(Kind::Daily(d));
    } else if let Some(v) = after("--level") {
        match v.parse::<usize>() {
            Ok(n) if (1..=levels::CAMPAIGN.len()).contains(&n) => app.start(Kind::Campaign(n - 1)),
            _ => fail(&format!("--level takes 1 to {}", levels::CAMPAIGN.len())),
        }
    } else if let Some(v) = after("--endless") {
        match levels::ENDLESS.iter().position(|s| s.name == v) {
            Some(i) => app.start(Kind::Endless(i)),
            None => fail("--endless takes small, medium, large or huge"),
        }
    } else if has("--watch") {
        app.start(Kind::Watch);
    }

    let mut terminal = ratatui::init();
    execute!(io::stdout(), EnableFocusChange)?;
    let size = terminal.size()?;
    log::info!("terminal {}x{}", size.width, size.height);
    let mut pads = Pads::new();
    let result = run(&mut terminal, &mut app, &mut pads);
    let _ = execute!(io::stdout(), DisableFocusChange);
    ratatui::restore();
    result
}

fn run(terminal: &mut ratatui::DefaultTerminal, app: &mut App, pads: &mut Pads) -> io::Result<()> {
    let mut snapshot = Instant::now();
    let mut next_frame = Instant::now();
    loop {
        let wait = next_frame.saturating_duration_since(Instant::now()).min(Duration::from_millis(10));
        if event::poll(wait)? {
            match event::read()? {
                Event::Key(k) => {
                    log::debug!("key {:?} {:?} {:?}", k.code, k.kind, k.modifiers);
                    if let Some(cmd) = key_cmd(app, k) {
                        app.apply(cmd);
                    }
                }
                Event::Resize(w, h) => log::info!("resize {w}x{h}"),
                Event::FocusLost => {
                    let playing = app.screen == Screen::Play && app.kind != Kind::Watch;
                    if playing && !app.paused && app.run.as_ref().is_some_and(|r| r.started && r.finished.is_none()) {
                        app.apply(Cmd::Pause);
                        log::info!("focus lost, paused");
                    }
                }
                Event::FocusGained => log::info!("focus gained"),
                _ => {}
            }
        }
        for cmd in pads.poll(app) {
            app.apply(cmd);
        }
        if app.quit {
            return Ok(());
        }
        app.tick();
        if Instant::now() >= next_frame {
            terminal.draw(|f| {
                app.term = (f.area().width, f.area().height);
                render::draw(app, f.area(), f.buffer_mut())
            })?;
            next_frame = Instant::now() + FRAME;
        }
        if snapshot.elapsed() >= SNAPSHOT_EVERY {
            snapshot = Instant::now();
            log::debug!("snapshot {}", app.snapshot());
        }
    }
}

fn key_cmd(app: &App, k: KeyEvent) -> Option<Cmd> {
    // presses and held-key repeats; releases (kitty protocol) are not moves
    if k.kind == KeyEventKind::Release {
        return None;
    }
    if k.modifiers.contains(KeyModifiers::CONTROL) && matches!(k.code, KeyCode::Char('c')) {
        return Some(Cmd::Quit);
    }
    let shift = k.modifiers.contains(KeyModifiers::SHIFT);
    if matches!(k.code, KeyCode::Char('m' | 'M')) {
        return Some(Cmd::Mute);
    }
    let ch = match k.code {
        KeyCode::Char(c) => Some(c.to_ascii_lowercase()),
        _ => None,
    };
    if app.screen != Screen::Play {
        return match (k.code, ch) {
            (KeyCode::Up, _) | (_, Some('w' | 'k')) => Some(Cmd::Up),
            (KeyCode::Down, _) | (_, Some('s' | 'j')) => Some(Cmd::Down),
            (KeyCode::Left, _) | (_, Some('a' | 'h')) => Some(Cmd::Left),
            (KeyCode::Right, _) | (_, Some('d' | 'l')) => Some(Cmd::Right),
            (KeyCode::Enter, _) | (_, Some(' ')) => Some(Cmd::Confirm),
            (_, Some('f')) => Some(Cmd::Fog),
            (KeyCode::Esc | KeyCode::Backspace, _) => Some(Cmd::Back),
            (_, Some('q')) if app.screen == Screen::Title => Some(Cmd::Quit),
            (_, Some('q')) => Some(Cmd::Back),
            _ => None,
        };
    }
    match (k.code, ch) {
        (KeyCode::Left, _) if shift => Some(Cmd::Move(Dir::ZMINUS)),
        (KeyCode::Right, _) if shift => Some(Cmd::Move(Dir::ZPLUS)),
        (KeyCode::Up, _) if shift => Some(Cmd::Move(Dir::WMINUS)),
        (KeyCode::Down, _) if shift => Some(Cmd::Move(Dir::WPLUS)),
        (KeyCode::Left, _) | (_, Some('a')) => Some(Cmd::Move(Dir::LEFT)),
        (KeyCode::Right, _) | (_, Some('d')) => Some(Cmd::Move(Dir::RIGHT)),
        (KeyCode::Up, _) | (_, Some('w')) => Some(Cmd::Move(Dir::UP)),
        (KeyCode::Down, _) | (_, Some('s')) => Some(Cmd::Move(Dir::DOWN)),
        (_, Some('j')) => Some(Cmd::Move(Dir::ZMINUS)),
        (_, Some('l')) => Some(Cmd::Move(Dir::ZPLUS)),
        (_, Some('i')) => Some(Cmd::Move(Dir::WMINUS)),
        (_, Some('k')) => Some(Cmd::Move(Dir::WPLUS)),
        (_, Some('h')) => Some(Cmd::Hint),
        (_, Some('p')) => Some(Cmd::Pause),
        (_, Some('r')) => Some(Cmd::Restart),
        (_, Some('n')) => Some(Cmd::NewMaze),
        (_, Some('v')) | (KeyCode::Tab, _) => Some(Cmd::Turn),
        (_, Some('g')) => Some(Cmd::Ghost),
        (KeyCode::Enter, _) | (_, Some(' ')) => Some(Cmd::Confirm),
        (KeyCode::Esc | KeyCode::Backspace, _) | (_, Some('q')) => Some(Cmd::Back),
        _ => None,
    }
}

/// Gamepads, any number, all driving the one player. Held directions on
/// the d-pad and sticks repeat.
struct Pads {
    #[cfg(feature = "gamepad")]
    gilrs: Option<gilrs::Gilrs>,
    #[cfg(feature = "gamepad")]
    held: Vec<(Held, Instant)>,
}

#[cfg(feature = "gamepad")]
#[derive(Clone, Copy, Debug, PartialEq)]
enum Held {
    Button(gilrs::Button),
    /// a stick pushed past the threshold: axis, sign
    Stick(gilrs::Axis, i8),
}

#[cfg(feature = "gamepad")]
const REPEAT_FIRST: Duration = Duration::from_millis(280);
#[cfg(feature = "gamepad")]
const REPEAT: Duration = Duration::from_millis(120);

impl Pads {
    fn new() -> Pads {
        #[cfg(feature = "gamepad")]
        {
            let gilrs = match gilrs::Gilrs::new() {
                Ok(g) => Some(g),
                Err(e) => {
                    log::warn!("no gamepads: {e}");
                    None
                }
            };
            Pads { gilrs, held: Vec::new() }
        }
        #[cfg(not(feature = "gamepad"))]
        Pads {}
    }

    #[cfg(not(feature = "gamepad"))]
    fn poll(&mut self, _app: &App) -> Vec<Cmd> {
        Vec::new()
    }

    #[cfg(feature = "gamepad")]
    fn poll(&mut self, app: &App) -> Vec<Cmd> {
        use gilrs::{Axis, EventType};
        let mut out = Vec::new();
        let Some(g) = &mut self.gilrs else { return out };
        let now = Instant::now();
        while let Some(ev) = g.next_event() {
            // gilrs also reports 3D mice and tablet key pads as gamepads;
            // only trust the ones SDL knows
            let any = std::env::var("TESSERA_ANY_PAD").is_ok_and(|v| v == "1");
            if !any && g.gamepad(ev.id).mapping_source() != gilrs::MappingSource::SdlMappings {
                continue;
            }
            match ev.event {
                EventType::Connected => log::info!("gamepad connected: {}", g.gamepad(ev.id).name()),
                EventType::Disconnected => log::info!("gamepad disconnected"),
                EventType::ButtonPressed(b, _) => {
                    log::debug!("pad button {b:?}");
                    if let Some(c) = pad_cmd(app, Held::Button(b)) {
                        out.push(c);
                        if repeats(Held::Button(b)) {
                            self.held.push((Held::Button(b), now + REPEAT_FIRST));
                        }
                    }
                }
                EventType::ButtonReleased(b, _) => self.held.retain(|(h, _)| *h != Held::Button(b)),
                EventType::AxisChanged(a @ (Axis::LeftStickX | Axis::LeftStickY | Axis::RightStickX | Axis::RightStickY), v, _) => {
                    // y points up on gilrs sticks; screen y points down
                    let s = if v > 0.6 { 1 } else if v < -0.6 { -1 } else { 0 };
                    let was = self.held.iter().position(|(h, _)| matches!(h, Held::Stick(x, _) if *x == a));
                    if v.abs() < 0.4 {
                        if let Some(i) = was {
                            self.held.remove(i);
                        }
                    } else if s != 0 && was.is_none_or(|i| self.held[i].0 != Held::Stick(a, s)) {
                        if let Some(i) = was {
                            self.held.remove(i);
                        }
                        log::debug!("pad stick {a:?} {s}");
                        if let Some(c) = pad_cmd(app, Held::Stick(a, s)) {
                            out.push(c);
                        }
                        self.held.push((Held::Stick(a, s), now + REPEAT_FIRST));
                    }
                }
                _ => {}
            }
        }
        for (h, next) in self.held.iter_mut() {
            if now >= *next {
                *next = now + REPEAT;
                out.extend(pad_cmd(app, *h));
            }
        }
        out
    }
}

#[cfg(feature = "gamepad")]
fn repeats(h: Held) -> bool {
    use gilrs::Button::*;
    matches!(h, Held::Button(DPadUp | DPadDown | DPadLeft | DPadRight))
}

#[cfg(feature = "gamepad")]
fn pad_cmd(app: &App, h: Held) -> Option<Cmd> {
    use gilrs::{Axis, Button::*};
    let overlay = app.paused || app.run.as_ref().is_some_and(|r| r.finished.is_some());
    if app.screen != Screen::Play || overlay {
        return match h {
            Held::Button(DPadUp) | Held::Stick(Axis::LeftStickY, 1) => Some(Cmd::Up),
            Held::Button(DPadDown) | Held::Stick(Axis::LeftStickY, -1) => Some(Cmd::Down),
            Held::Button(DPadLeft) | Held::Stick(Axis::LeftStickX, -1) => Some(Cmd::Left),
            Held::Button(DPadRight) | Held::Stick(Axis::LeftStickX, 1) => Some(Cmd::Right),
            Held::Button(South) => Some(Cmd::Confirm),
            Held::Button(Start) if overlay => Some(Cmd::Pause),
            Held::Button(Start) => Some(Cmd::Confirm),
            Held::Button(West) => Some(Cmd::Fog),
            Held::Button(North) if overlay => Some(Cmd::Restart),
            Held::Button(East | Select) => Some(Cmd::Back),
            _ => None,
        };
    }
    match h {
        Held::Button(DPadLeft) | Held::Stick(Axis::LeftStickX, -1) => Some(Cmd::Move(Dir::LEFT)),
        Held::Button(DPadRight) | Held::Stick(Axis::LeftStickX, 1) => Some(Cmd::Move(Dir::RIGHT)),
        Held::Button(DPadUp) | Held::Stick(Axis::LeftStickY, 1) => Some(Cmd::Move(Dir::UP)),
        Held::Button(DPadDown) | Held::Stick(Axis::LeftStickY, -1) => Some(Cmd::Move(Dir::DOWN)),
        Held::Button(West) | Held::Stick(Axis::RightStickX, -1) => Some(Cmd::Move(Dir::ZMINUS)),
        Held::Button(East) | Held::Stick(Axis::RightStickX, 1) => Some(Cmd::Move(Dir::ZPLUS)),
        Held::Button(North) | Held::Stick(Axis::RightStickY, 1) => Some(Cmd::Move(Dir::WMINUS)),
        Held::Button(South) | Held::Stick(Axis::RightStickY, -1) => Some(Cmd::Move(Dir::WPLUS)),
        Held::Button(LeftTrigger) => Some(Cmd::Hint),
        Held::Button(LeftTrigger2 | RightTrigger2) => Some(Cmd::Turn),
        Held::Button(RightThumb) => Some(Cmd::Ghost),
        Held::Button(RightTrigger) => Some(Cmd::Restart),
        Held::Button(Start) => Some(Cmd::Pause),
        Held::Button(Select) => Some(Cmd::Back),
        _ => None,
    }
}
