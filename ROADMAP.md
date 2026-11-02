# Roadmap

Tessera, mazes in four dimensions (David, 2026-11-02: "4D Maze for the
terminal", after bleuje's browser 4D maze). Versions follow the house
rule: 1.0.0 when everything planned below is in.

## 0.1.0: built

- Four-dimensional mazes grown as a weighted growing tree (passages inside
  a slice three times as likely as between slices), the longest way of 48
  variants of the seed, so every maze is a walk
- Slices drawn as box-drawing mazes, z side by side and w stacked; marks in
  every cell for the ways through to other slices; your place shown in
  every slice, lit where a step through would land
- Campaign of twelve fixed mazes, each opening the next, two in fog;
  a daily maze from the date; endless in four sizes, fog optional
- Hints (the next four steps, +10 s), pause, restart, best times saved
- Watch mode: the way through walked by the computer
- Keyboard and gamepads; session logging

## 0.2.0: built

- Turn the maze (V or Tab, the gamepad's triggers): any two of the four
  axes can be the slices and the other two lay them out, six views of the
  same maze; walking follows the screen, the status line names the view,
  and only views that fit the terminal come up
- Watch mode walks 4×4×3×2 mazes and turns them a third and two thirds
  of the way through

## 0.3.0: built

- Sound, all synthesised: soft steps, a two-note chime for stepping
  through (higher across, lower up and down), a thud at walls, a sweep
  when the maze turns, a run of notes for a hint, a fanfare when solved
  and a longer one for a new best
- M turns the sound off and on anywhere, remembered; `--mute`

## Next

- A map view for small terminals: one character per cell in half blocks,
  walls and passages as colour; it cannot carry the marks for the ways
  through, which is why the full view needs two rows per cell (underlined
  walls would, but casts lose underline colour)
- Replay your best run as a ghost beside you
- Five dimensions as the last campaign chapter
