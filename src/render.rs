//! Drawing. A slice is a box-drawing maze, two rows per cell (cell rows and
//! wall rows) and four columns (three for the cell, one for the wall);
//! slices sit side by side along z and stacked along w. Inside a cell the
//! left character shows passages to the slices left and right (cyan
//! ◂ ▸ ↔), the right one passages to the slices above and below (amber
//! ▴ ▾ ↕), and the middle you, the goal, your trail or a hint.

use crate::app::{App, Kind, Screen, TITLE_ITEMS};
use crate::game::{fmt_ms, fmt_time, Run, HINT_COST};
use crate::levels::{CAMPAIGN, ENDLESS};
use crate::maze::{Dir, Pos};
use crate::view::View;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use std::time::Duration;

const BG: Color = Color::Rgb(8, 12, 10);
const WALL: Color = Color::Rgb(90, 220, 130);
const WALL_DIM: Color = Color::Rgb(38, 96, 58);
const ZC: Color = Color::Rgb(90, 200, 235);
const WC: Color = Color::Rgb(240, 180, 60);
const ME: Color = Color::Rgb(255, 255, 255);
const GOAL: Color = Color::Rgb(255, 96, 160);
const TRAIL: Color = Color::Rgb(70, 150, 95);
const GHOST: Color = Color::Rgb(105, 115, 110);
const HINT: Color = Color::Rgb(255, 240, 120);
const FOG: Color = Color::Rgb(30, 42, 36);
const BUMP: Color = Color::Rgb(255, 70, 70);
const TEXT: Color = Color::Rgb(200, 215, 205);
const FAINT: Color = Color::Rgb(110, 130, 118);
const AMBER: Color = Color::Rgb(255, 176, 0);

const CELL_W: u16 = 4;
const GAP_X: u16 = 3;
const GAP_Y: u16 = 1;
const BUMP_FOR: Duration = Duration::from_millis(300);

fn put(buf: &mut Buffer, x: u16, y: u16, s: &str, st: Style) {
    let a = buf.area;
    if y < a.y || y >= a.bottom() {
        return;
    }
    let mut cx = x;
    for ch in s.chars() {
        if cx >= a.x && cx < a.right() {
            buf[(cx, y)].set_char(ch).set_style(st);
        }
        cx += 1;
    }
}

fn center(buf: &mut Buffer, area: Rect, y: u16, s: &str, st: Style) {
    let w = s.chars().count() as u16;
    let x = area.x + area.width.saturating_sub(w) / 2;
    put(buf, x, y, s, st);
}

fn fg(c: Color) -> Style {
    Style::default().fg(c).bg(BG)
}

pub fn maze_size(dims: [usize; 4]) -> (u16, u16) {
    let [nx, ny, nz, nw] = dims.map(|d| d as u16);
    let sw = CELL_W * nx + 1;
    let sh = 2 * ny + 1;
    (nz * sw + (nz - 1) * GAP_X, nw * sh + (nw - 1) * GAP_Y)
}

/// Room around the maze: the status line above and the key line below.
pub fn needed(dims: [usize; 4]) -> (u16, u16) {
    let (w, h) = maze_size(dims);
    (w.max(60) + 2, h + 4)
}

pub fn draw(app: &App, area: Rect, buf: &mut Buffer) {
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            buf[(x, y)].reset();
            buf[(x, y)].set_style(Style::default().bg(BG));
        }
    }
    match app.screen {
        Screen::Title => title(app, area, buf),
        Screen::Campaign => campaign(app, area, buf),
        Screen::Endless => endless(app, area, buf),
        Screen::Help => help(app, area, buf),
        Screen::Play => play(app, area, buf),
    }
}

/// The wordmark's letters, three pixels wide and five high; each pixel is
/// drawn two columns wide.
fn letter(c: char) -> [&'static str; 5] {
    match c {
        'T' => ["###", " # ", " # ", " # ", " # "],
        'E' => ["###", "#  ", "## ", "#  ", "###"],
        'S' => ["###", "#  ", "###", "  #", "###"],
        'R' => ["## ", "# #", "## ", "# #", "# #"],
        'A' => [" # ", "# #", "###", "# #", "# #"],
        _ => ["   "; 5],
    }
}

const WORD: &str = "TESSERA";

fn logo(buf: &mut Buffer, area: Rect, y: u16) {
    let n = WORD.chars().count() as u16;
    let w = n * 8 - 2;
    let x0 = area.x + area.width.saturating_sub(w) / 2;
    for (i, c) in WORD.chars().enumerate() {
        for (r, line) in letter(c).iter().enumerate() {
            for (k, px) in line.chars().enumerate() {
                if px != '#' {
                    continue;
                }
                let x = i as u16 * 8 + k as u16 * 2;
                // green at the left to amber at the right
                let t = x as f32 / w as f32;
                let col = Color::Rgb((90.0 + 165.0 * t) as u8, (220.0 - 44.0 * t) as u8, (130.0 * (1.0 - t)) as u8);
                put(buf, x0 + x, y + r as u16, "██", fg(col));
            }
        }
    }
}

fn title(app: &App, area: Rect, buf: &mut Buffer) {
    let top = area.y + area.height.saturating_sub(20) / 2;
    logo(buf, area, top);
    center(buf, area, top + 6, "mazes in four dimensions", fg(FAINT));
    let solved = app.store.solved.min(CAMPAIGN.len());
    let daily_best = app.store.best.get(&format!("daily-{}", app.today)).map(|&ms| format!("best {}", fmt_ms(ms)));
    for (i, item) in TITLE_ITEMS.iter().enumerate() {
        let extra = match i {
            0 => format!("{solved}/{}", CAMPAIGN.len()),
            1 => format!("{}  {}", app.today, daily_best.clone().unwrap_or_default()),
            _ => String::new(),
        };
        let line = format!("{item:<12} {extra:<24}");
        let sel = i == app.cursor;
        let st = if sel { Style::default().fg(BG).bg(AMBER).add_modifier(Modifier::BOLD) } else { fg(TEXT) };
        let x = area.x + area.width.saturating_sub(40) / 2;
        put(buf, x, top + 9 + i as u16, if sel { "▶ " } else { "  " }, fg(AMBER));
        put(buf, x + 2, top + 9 + i as u16, &line, st);
    }
    center(buf, area, top + 17, "↑↓ choose · Enter play · Esc quit", fg(FAINT));
    if let Some((m, _)) = &app.message {
        center(buf, area, top + 19, m, fg(AMBER));
    }
}

fn dims_str(d: [usize; 4]) -> String {
    format!("{}×{}×{}×{}", d[0], d[1], d[2], d[3])
}

fn campaign(app: &App, area: Rect, buf: &mut Buffer) {
    let top = area.y + area.height.saturating_sub(CAMPAIGN.len() as u16 + 8) / 2;
    center(buf, area, top, "CAMPAIGN", fg(AMBER).add_modifier(Modifier::BOLD));
    center(buf, area, top + 1, "the same twelve mazes for everyone; each one opens the next", fg(FAINT));
    let x = area.x + area.width.saturating_sub(56) / 2;
    for (i, l) in CAMPAIGN.iter().enumerate() {
        let open = i <= app.store.solved;
        let best = app.store.best.get(&format!("campaign-{}", i + 1)).map(|&ms| fmt_ms(ms));
        let status = match (open, best) {
            (_, Some(b)) => format!("best {b}"),
            (true, None) => "new".into(),
            (false, None) => "locked".into(),
        };
        let line = format!("{:>2}  {:<16} {:<10} {:<4} {:<10}", i + 1, l.name, dims_str(l.dims), if l.fog { "fog" } else { "" }, status);
        let sel = i == app.campaign_cursor;
        let st = if sel {
            Style::default().fg(BG).bg(AMBER).add_modifier(Modifier::BOLD)
        } else if open {
            fg(TEXT)
        } else {
            fg(GHOST)
        };
        let y = top + 3 + i as u16;
        put(buf, x, y, if sel { "▶ " } else { "  " }, fg(AMBER));
        put(buf, x + 2, y, &line, st);
    }
    let y = top + 4 + CAMPAIGN.len() as u16;
    center(buf, area, y, "↑↓ choose · Enter play · Esc back", fg(FAINT));
    if let Some((m, _)) = &app.message {
        center(buf, area, y + 2, m, fg(AMBER));
    }
}

fn endless(app: &App, area: Rect, buf: &mut Buffer) {
    let top = area.y + area.height.saturating_sub(14) / 2;
    center(buf, area, top, "ENDLESS", fg(AMBER).add_modifier(Modifier::BOLD));
    center(buf, area, top + 1, "a new maze every time", fg(FAINT));
    let x = area.x + area.width.saturating_sub(44) / 2;
    let fog = app.store.fog;
    for (i, s) in ENDLESS.iter().enumerate() {
        let key = format!("endless-{}{}", s.name, if fog { "-fog" } else { "" });
        let best = app.store.best.get(&key).map(|&ms| format!("best {}", fmt_ms(ms))).unwrap_or_default();
        let line = format!("{:<8} {:<10} {:<12}", s.name, dims_str(s.dims), best);
        let sel = i == app.endless_cursor;
        let st = if sel { Style::default().fg(BG).bg(AMBER).add_modifier(Modifier::BOLD) } else { fg(TEXT) };
        let y = top + 3 + i as u16;
        put(buf, x, y, if sel { "▶ " } else { "  " }, fg(AMBER));
        put(buf, x + 2, y, &line, st);
    }
    let y = top + 4 + ENDLESS.len() as u16;
    put(buf, x + 2, y, &format!("Fog: {}", if fog { "on, you see only where you have been" } else { "off" }), fg(if fog { AMBER } else { TEXT }));
    center(buf, area, y + 2, "↑↓ size · ←→ or F fog · Enter play · Esc back", fg(FAINT));
}

const HELP: [&[&str]; 2] = [
    &[
        "HOW TO PLAY",
        "",
        "Each maze has four dimensions. You see it as slices: every box is one",
        "flat maze, and the boxes side by side and above and below each other",
        "are its neighbours in the third and fourth dimensions.",
        "",
        "Walk from the top left corner of the top left slice to the pink ◆ in",
        "the bottom right corner of the bottom right slice.",
        "",
        "Arrows or WASD walk inside a slice. J and L step to the slice on the",
        "left or right, I and K to the slice above or below, staying in the",
        "same place inside it (Shift with the arrows works too).",
        "",
        "A cell's marks say where you can step through:",
        "  ◂ ▸ ↔  to the slice on the left, right, or both  (J, L)",
        "  ▴ ▾ ↕  to the slice above, below, or both        (I, K)",
        "Your place is shown faintly in every slice; where you can step",
        "through to, it lights up.",
    ],
    &[
        "TURNING, HINTS AND THE GAMEPAD",
        "",
        "V turns the maze: any two of the four directions can be the slices,",
        "so the same maze can be seen six ways. A way that winds through many",
        "slices in one view may run straight in another. Walking always",
        "follows the screen.",
        "",
        "H shows the next steps of the way, but adds ten seconds. P pauses,",
        "R starts the maze over, Esc goes back.",
        "",
        "Fog shows only what you have walked past: two campaign mazes are in",
        "fog, and endless mazes can be.",
        "",
        "Gamepad: left stick or d-pad walk, right stick or the face buttons",
        "step between slices, the triggers turn the maze, LB hints, Start",
        "pauses, Back goes back.",
    ],
];

fn help(app: &App, area: Rect, buf: &mut Buffer) {
    let page = HELP[app.help_page.min(HELP.len() - 1)];
    let top = area.y + area.height.saturating_sub(20) / 2;
    let x = area.x + area.width.saturating_sub(72) / 2;
    for (i, l) in page.iter().enumerate() {
        let st = if i == 0 { fg(AMBER).add_modifier(Modifier::BOLD) } else { fg(TEXT) };
        put(buf, x, top + i as u16, l, st);
    }
    let foot = format!("page {} of {} · ←→ turn the page · Esc back", app.help_page + 1, HELP.len());
    center(buf, area, top + 19, &foot, fg(FAINT));
}

fn play(app: &App, area: Rect, buf: &mut Buffer) {
    let Some(r) = &app.run else { return };
    let view = app.view();
    let sh = Shown { r, v: view };
    let dims = sh.dims();
    let (nw, nh) = needed(dims);
    if area.width < nw || area.height < nh {
        let mid = area.y + area.height / 2;
        center(buf, area, mid.saturating_sub(1), &format!("This maze needs a {nw}×{nh} terminal"), fg(AMBER));
        center(buf, area, mid, &format!("(now {}×{}); make the window bigger", area.width, area.height), fg(TEXT));
        center(buf, area, mid + 1, "Esc goes back", fg(FAINT));
        return;
    }
    // status line
    let status_y = area.y;
    put(buf, area.x + 1, status_y, "TESSERA", fg(AMBER).add_modifier(Modifier::BOLD));
    let mut title = app.title();
    if app.view != 0 {
        title.push_str(&format!(" · view {}: {}", app.view + 1, view.name()));
    }
    put(buf, area.x + 10, status_y, &title, fg(TEXT));
    let mut right = format!("{}   {} move{}", fmt_time(r.elapsed()), r.moves, if r.moves == 1 { "" } else { "s" });
    if r.hints > 0 {
        right.push_str(&format!("   {} hint{}", r.hints, if r.hints == 1 { "" } else { "s" }));
    }
    let rw = right.chars().count() as u16;
    put(buf, area.right().saturating_sub(rw + 1), status_y, &right, fg(if r.started { TEXT } else { FAINT }));

    let (mw, mh) = maze_size(dims);
    let ox = area.x + area.width.saturating_sub(mw) / 2;
    let oy = area.y + 2 + (area.height - 4).saturating_sub(mh) / 2;
    let sw = CELL_W * dims[0] as u16 + 1;
    let slice_h = 2 * dims[1] as u16 + 1;
    for w in 0..dims[3] {
        for z in 0..dims[2] {
            let sx = ox + z as u16 * (sw + GAP_X);
            let sy = oy + w as u16 * (slice_h + GAP_Y);
            slice(&sh, z, w, sx, sy, buf);
        }
    }

    // key line, or a message
    let foot = area.bottom() - 1;
    if let Some((m, _)) = &app.message {
        center(buf, area, foot, m, fg(AMBER));
    } else if app.kind == Kind::Watch {
        center(buf, area, foot, "Watching the way through · N another maze · Esc back", fg(FAINT));
    } else {
        center(buf, area, foot, "←↑↓→ walk · J L ◂▸ · I K ▴▾ slices · V turn · H hint · P pause · R restart · Esc back", fg(FAINT));
    }

    if app.paused {
        boxed(buf, area, &["PAUSED", "", "P or Enter to go on · Esc back"]);
    } else if let (Some(t), Some((best, before))) = (r.finished, app.result) {
        let mut lines = vec!["SOLVED".to_string(), String::new()];
        let mut l = format!("{} · {} moves", fmt_time(t), r.moves);
        if r.hints > 0 {
            l.push_str(&format!(" · {} hint{} (+{} s each)", r.hints, if r.hints == 1 { "" } else { "s" }, HINT_COST.as_secs()));
        }
        lines.push(l);
        lines.push(match (best, before) {
            (true, Some(b)) => format!("New best! (was {})", fmt_ms(b)),
            (true, None) => "First time through".into(),
            (false, Some(b)) => format!("Best {}", fmt_ms(b)),
            _ => String::new(),
        });
        lines.push(String::new());
        lines.push(match &app.kind {
            Kind::Campaign(i) if i + 1 < CAMPAIGN.len() => format!("Enter: {} · R again · Esc back", CAMPAIGN[i + 1].name),
            Kind::Campaign(_) => "The campaign is done! Enter · R again · Esc back".into(),
            Kind::Daily(_) => "Come back tomorrow for the next one · R again · Esc back".into(),
            _ => "Enter: another maze · R again · Esc back".into(),
        });
        let refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
        boxed(buf, area, &refs);
    } else if !r.started && app.kind != Kind::Watch {
        center(buf, area, foot.saturating_sub(1), "The clock starts with your first step", fg(FAINT));
    }
}

fn boxed(buf: &mut Buffer, area: Rect, lines: &[&str]) {
    let w = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0) as u16 + 6;
    let h = lines.len() as u16 + 2;
    let x0 = area.x + area.width.saturating_sub(w) / 2;
    let y0 = area.y + area.height.saturating_sub(h) / 2;
    let st = Style::default().fg(AMBER).bg(Color::Rgb(20, 26, 22));
    for y in y0..y0 + h {
        for x in x0..x0 + w {
            put(buf, x, y, " ", st);
        }
    }
    put(buf, x0, y0, &format!("╭{}╮", "─".repeat(w as usize - 2)), st);
    put(buf, x0, y0 + h - 1, &format!("╰{}╯", "─".repeat(w as usize - 2)), st);
    for y in y0 + 1..y0 + h - 1 {
        put(buf, x0, y, "│", st);
        put(buf, x0 + w - 1, y, "│", st);
    }
    for (i, l) in lines.iter().enumerate() {
        let lw = l.chars().count() as u16;
        let s = if i == 0 { st.add_modifier(Modifier::BOLD) } else { st.fg(TEXT) };
        put(buf, x0 + (w - lw) / 2, y0 + 1 + i as u16, l, s);
    }
}

fn corner(u: bool, d: bool, l: bool, r: bool) -> &'static str {
    match (u, d, l, r) {
        (true, true, true, true) => "┼",
        (true, true, true, false) => "┤",
        (true, true, false, true) => "├",
        (true, false, true, true) => "┴",
        (false, true, true, true) => "┬",
        (true, true, false, false) => "│",
        (false, false, true, true) => "─",
        (true, false, true, false) => "╯",
        (true, false, false, true) => "╰",
        (false, true, true, false) => "╮",
        (false, true, false, true) => "╭",
        (true, false, false, false) => "╵",
        (false, true, false, false) => "╷",
        (false, false, true, false) => "╴",
        (false, false, false, true) => "╶",
        (false, false, false, false) => " ",
    }
}

/// The run as the screen shows it: positions and directions turned by the
/// view, so the drawing below only ever thinks in screen axes.
struct Shown<'a> {
    r: &'a Run,
    v: View,
}

impl Shown<'_> {
    fn dims(&self) -> [usize; 4] {
        self.v.dims(self.r.maze.dims)
    }
    fn open(&self, q: Pos, d: Dir) -> bool {
        self.r.maze.is_open(self.v.to_maze(q), self.v.dir(d))
    }
    fn step(&self, q: Pos, d: Dir) -> Option<Pos> {
        self.r.maze.step(self.v.to_maze(q), self.v.dir(d)).map(|p| self.v.to_screen(p))
    }
    fn pos(&self) -> Pos {
        self.v.to_screen(self.r.pos)
    }
    fn goal(&self) -> Pos {
        self.v.to_screen(self.r.maze.goal)
    }
    fn index(&self, q: Pos) -> usize {
        self.r.maze.index(self.v.to_maze(q))
    }
    fn seen(&self, q: Pos) -> bool {
        !self.r.fog || self.r.seen[self.index(q)]
    }
    fn trail(&self, q: Pos) -> bool {
        self.r.trail[self.index(q)]
    }
    fn hinted(&self, q: Pos) -> bool {
        let p = self.v.to_maze(q);
        self.r.hint.as_ref().is_some_and(|(cells, _)| cells.contains(&p))
    }
    /// a recent move into a wall, in screen terms
    fn bump(&self) -> Option<(Pos, Dir)> {
        self.r.bump.filter(|(_, _, t)| t.elapsed() < BUMP_FOR).map(|(p, d, _)| (self.v.to_screen(p), self.v.screen_dir(d)))
    }
}

fn slice(sh: &Shown, z: usize, w: usize, sx: u16, sy: u16, buf: &mut Buffer) {
    let [nx, ny, _, _] = sh.dims();
    let me = sh.pos();
    let here = me[2] == z && me[3] == w;
    let at = |x: usize, y: usize| -> Pos { [x, y, z, w] };
    let bump = sh.bump();

    // walls: horizontal between rows yb-1 and yb at column x, vertical
    // between columns xb-1 and xb at row y; (is there, is it the bump)
    let hwall = |x: usize, yb: usize| -> (bool, bool) {
        let wall = yb == 0 || yb == ny || !sh.open(at(x, yb - 1), Dir::DOWN);
        let visible = (yb > 0 && sh.seen(at(x, yb - 1))) || (yb < ny && sh.seen(at(x, yb)));
        let hit = bump.is_some_and(|(p, d)| p[2] == z && p[3] == w && p[0] == x && ((d == Dir::UP && p[1] == yb) || (d == Dir::DOWN && p[1] + 1 == yb)));
        (wall && visible, hit)
    };
    let vwall = |xb: usize, y: usize| -> (bool, bool) {
        let wall = xb == 0 || xb == nx || !sh.open(at(xb - 1, y), Dir::RIGHT);
        let visible = (xb > 0 && sh.seen(at(xb - 1, y))) || (xb < nx && sh.seen(at(xb, y)));
        let hit = bump.is_some_and(|(p, d)| p[2] == z && p[3] == w && p[1] == y && ((d == Dir::LEFT && p[0] == xb) || (d == Dir::RIGHT && p[0] + 1 == xb)));
        (wall && visible, hit)
    };
    let wall_st = fg(if here { WALL } else { WALL_DIM });

    for yb in 0..=ny {
        let row = sy + 2 * yb as u16;
        for xb in 0..=nx {
            let u = yb > 0 && vwall(xb, yb - 1).0;
            let d = yb < ny && vwall(xb, yb).0;
            let l = xb > 0 && hwall(xb - 1, yb).0;
            let rr = xb < nx && hwall(xb, yb).0;
            put(buf, sx + CELL_W * xb as u16, row, corner(u, d, l, rr), wall_st);
            if xb < nx {
                let (on, hit) = hwall(xb, yb);
                let st = if hit { fg(BUMP) } else { wall_st };
                put(buf, sx + CELL_W * xb as u16 + 1, row, if on { "───" } else { "   " }, st);
            }
        }
    }
    for y in 0..ny {
        let row = sy + 2 * y as u16 + 1;
        for xb in 0..=nx {
            let (on, hit) = vwall(xb, y);
            put(buf, sx + CELL_W * xb as u16, row, if on { "│" } else { " " }, if hit { fg(BUMP) } else { wall_st });
        }
        for x in 0..nx {
            let cx = sx + CELL_W * x as u16 + 1;
            cell(sh, at(x, y), cx, row, buf, here);
        }
    }
}

fn cell(sh: &Shown, q: Pos, cx: u16, row: u16, buf: &mut Buffer, here: bool) {
    if !sh.seen(q) {
        put(buf, cx, row, "░░░", fg(FOG));
        return;
    }
    let dim = |c: Color| if here { c } else { blend(c, BG, 0.45) };
    let open = |d: Dir| sh.open(q, d);
    let zm = match (open(Dir::ZMINUS), open(Dir::ZPLUS)) {
        (true, true) => "↔",
        (true, false) => "◂",
        (false, true) => "▸",
        _ => " ",
    };
    let wm = match (open(Dir::WMINUS), open(Dir::WPLUS)) {
        (true, true) => "↕",
        (true, false) => "▴",
        (false, true) => "▾",
        _ => " ",
    };
    put(buf, cx, row, zm, fg(dim(ZC)));
    put(buf, cx + 2, row, wm, fg(dim(WC)));

    let me = sh.pos();
    // a step across the slices that hit nothing: mark it on you
    if let Some((bp, d)) = sh.bump() {
        if bp == q && d.axis() >= 2 {
            put(buf, if d.axis() == 2 { cx } else { cx + 2 }, row, "×", fg(BUMP));
        }
    }

    let ghost = !here && q[0] == me[0] && q[1] == me[1];
    let (ch, st) = if q == me {
        ("@", fg(ME).add_modifier(Modifier::BOLD))
    } else if sh.hinted(q) {
        ("•", fg(HINT).add_modifier(Modifier::BOLD))
    } else if q == sh.goal() {
        ("◆", fg(if here { GOAL } else { blend(GOAL, BG, 0.25) }).add_modifier(Modifier::BOLD))
    } else if ghost {
        // where a step across the slices would land
        let reach = Dir::ALL[4..].iter().find(|&&d| sh.step(me, d) == Some(q));
        match reach {
            Some(d) if d.axis() == 2 => ("○", fg(ZC).add_modifier(Modifier::BOLD)),
            Some(_) => ("○", fg(WC).add_modifier(Modifier::BOLD)),
            None => ("∙", fg(GHOST)),
        }
    } else if sh.trail(q) {
        ("·", fg(dim(TRAIL)))
    } else {
        (" ", fg(BG))
    };
    put(buf, cx + 1, row, ch, st);
}

fn blend(a: Color, b: Color, t: f32) -> Color {
    match (a, b) {
        (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) => {
            let m = |x: u8, y: u8| (x as f32 * t + y as f32 * (1.0 - t)) as u8;
            Color::Rgb(m(r1, r2), m(g1, g2), m(b1, b2))
        }
        _ => a,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::levels::DAILY_DIMS;

    #[test]
    fn campaign_fits_a_hundred_by_forty() {
        for l in CAMPAIGN.iter() {
            let (w, h) = needed(l.dims);
            assert!(w <= 100 && h <= 40, "{} needs {w}x{h}", l.name);
        }
        let (w, h) = needed(DAILY_DIMS);
        assert!(w <= 100 && h <= 40);
    }

    #[test]
    fn draws_a_maze() {
        let mut app = App::new(crate::store::Store::default(), 3);
        app.start(Kind::Daily("2026-11-02".into()));
        let area = Rect::new(0, 0, 100, 40);
        let mut buf = Buffer::empty(area);
        draw(&app, area, &mut buf);
        let text: String = (0..40).map(|y| (0..100).map(|x| buf[(x, y)].symbol().to_string()).collect::<String>() + "\n").collect();
        assert!(text.contains('@'));
        assert!(text.contains('◆'));
        assert!(text.contains("TESSERA"));
    }

    fn text(app: &App, w: u16, h: u16) -> String {
        let area = Rect::new(0, 0, w, h);
        let mut buf = Buffer::empty(area);
        draw(app, area, &mut buf);
        (0..h).map(|y| (0..w).map(|x| buf[(x, y)].symbol().to_string()).collect::<String>().trim_end().to_string() + "\n").collect()
    }

    /// `cargo test dump -- --ignored --nocapture` prints the screens.
    #[test]
    #[ignore]
    fn dump() {
        let mut app = App::new(crate::store::Store::default(), 3);
        app.start(Kind::Campaign(8));
        for _ in 0..6 {
            let r = app.run.as_ref().unwrap();
            let d = r.maze.path_to_goal(r.pos)[0];
            app.apply(crate::app::Cmd::Move(d));
        }
        println!("{}", text(&app, 100, 40));
        let path = {
            let r = app.run.as_ref().unwrap();
            r.maze.path_to_goal(r.pos)
        };
        app.apply(crate::app::Cmd::Hint);
        for d in path {
            app.apply(crate::app::Cmd::Move(d));
        }
        println!("{}", text(&app, 100, 40));
        app.screen = Screen::Campaign;
        println!("{}", text(&app, 100, 30));
        app.screen = Screen::Endless;
        println!("{}", text(&app, 100, 30));
        app.screen = Screen::Help;
        println!("{}", text(&app, 80, 24));
        app.help_page = 1;
        println!("{}", text(&app, 80, 24));
    }

    #[test]
    fn help_pages_fit_eighty_by_twenty_four() {
        for page in HELP {
            assert!(page.len() <= 19, "a help page has {} lines", page.len());
            assert!(page.iter().all(|l| l.chars().count() <= 72));
        }
    }
}
