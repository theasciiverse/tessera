# Tessera

Mazes in four dimensions, for the Asciiverse catalog. Rust + Ratatui.
Started 2026-11-02 (David: "4D Maze for the terminal", pointing at
bleuje.com/4dmaze). Ours, written from scratch: the 4D maze shown as a
grid of 2D slices is the genre's way of drawing it; name, controls,
marks, structure and code are our own, credited "after bleuje's 4D maze".

**Status:** 0.5.0, playable. Own git repo, folder `games/tessera`, private at
github.com/theasciiverse/tessera. What comes next is in `ROADMAP.md`.

## Model (src/maze.rs)

- Axes x, y (inside a slice), z (slices side by side), w (slices stacked).
  `Dir(0..8)`: -x +x -y +y -z +z -w +w; a cell's passages are one bit each
- Growing tree: newest cell 3 times in 4, else a random one; directions
  weighted 3 to 1 for inside a slice (`IN_SLICE_WEIGHT`)
- Start (0,0,0,0), goal the far corner. `generate_long` grows 48 variants
  of the seed and keeps the longest way through (median 38 to 51 steps
  for the campaign sizes, against 16 to 25 for a single tree)
- Seeds: campaign `hash("tessera-campaign-N")`, daily
  `hash("tessera-daily-YYYY-MM-DD")` (UTC), endless and watch from the
  clock or `--seed`

## Campaign curve (2026-11-02, `cargo test campaign_curve -- --ignored --nocapture`)

Path length, forks on the way (cells with a side passage) and steps
through to other slices, level by level: 19/4/3, 28/6/8, 26/7/6, 20/5/6,
30/7/8, 35/9/11, 46/16/14, 45/9/13, 40/10/12 (fog), 48/16/8, 47/14/13,
58/14/22 (fog). "The square" (4) is easier than 3 on purpose: the first
maze with both extra axes. Changing the generator changes every maze:
rerun this and re-check the curve.

## Drawing (src/render.rs)

Two rows and four columns per cell (box-drawing lattice; corners worked
out from the four segments meeting there). Underlined south walls were
tried on paper and dropped: avt has no underline colour (SGR 58), so in
casts the underline takes each character's colour. Cell = z mark, core,
w mark. The campaign fits 100×40 (a test checks); the game says when a
maze needs a bigger terminal.

## Views (src/view.rs)

Six views, one per pair of axes inside the slices: a permutation from
screen axes (across a slice, down it, across the slices, down them) to
maze axes. The renderer draws through `Shown` (render.rs), which turns
positions and directions, so slice and cell drawing think only in screen
axes; `Cmd::Move` carries a screen direction that the app turns with
`View::dir`. Turning skips views that need more than the terminal
(`App::term`, set on every draw). A new maze starts in view 1.

## Notes

- `cargo test dump -- --ignored --nocapture` prints the play, solved,
  campaign, endless and help screens as text
- `cargo test path_lengths -- --ignored --nocapture` prints path length
  statistics per size
- Sound (src/audio.rs) is synthesised like 1D-Pong's: square, triangle
  and noise notes into a rodio mixer; `samples()` is what the tests check.
  Audio opens before the alternate screen so ALSA chatter lands above it
- Best runs (src/ghost.rs): moves with clock times, kept per campaign and
  daily key in `<data>/ghosts/<key>`, checked against the maze when
  loaded. `store::data_dir()` is a temp folder under `cargo test`: tests
  must never write the player's files (they did before 0.4.0)
- gilrs reports 3D mice as gamepads; only SDL-mapped pads count,
  `TESSERA_ANY_PAD=1` allows all
- State in $XDG_DATA_HOME/tessera/state (`%APPDATA%\tessera\state`)
- Catalog preview: `asciiverse/tessera.cast`, recorded with
  `hub/tools/castrec.py` (see its entry in games/ROSTER.md)
