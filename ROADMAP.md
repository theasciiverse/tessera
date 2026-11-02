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

## Next

- Sound: soft steps, a chime for stepping through, a bump at walls
- A compact view (one row per cell) so the largest mazes fit 80×24
- Turn the view: choose which two axes are the slices and which two are
  laid out, so the same maze can be seen four ways
- Replay your best run as a ghost beside you
- Five dimensions as the last campaign chapter
