# Tessera

Mazes in four dimensions, in the terminal.

Every box on screen is one flat slice of the maze. The boxes side by
side are its neighbours in the third dimension, the boxes above and below
its neighbours in the fourth. Walk inside a slice with the arrow keys,
and step through to the slice beside, above or below where a cell says
there is a way: cyan `◂ ▸ ↔` for the slices left and right, amber
`▴ ▾ ↕` for the slices above and below. Your place shows faintly in every
slice, and lights up where a step through would land.

Get from the top left corner of the first slice to the pink `◆` in the
bottom right corner of the last.

Press V to turn the maze. Any two of its four directions can be the
slices and the other two lay them out, so every maze can be seen six
ways; a way that winds through many slices in one view may run straight
in another. Walking always follows the screen.

```
╭───────┬───┬───╮   ╭───────────────╮
│▸@  ·  │▸  │ ·▾│   │↔○    ▸     ▾  │
│   ╷   ╵   ╰───┤   │   ╭───────╮   │
│  ▴│       ·   │   │   │ ∙     │ ▴ │
╰───┴───────────╯   ╰───┴───────┴───╯
```

## Play

```bash
cargo run --release                     # title and menu
cargo run --release -- --daily          # today's maze, the same for everyone
cargo run --release -- --endless large  # a new maze: small, medium, large or huge
cargo run --release -- --watch          # watch the way through being walked
cargo run --release -- --help           # everything else
```

- **Campaign**: twelve mazes, the same for everyone, from two slices side
  by side to twelve slices of 5×5; each one opens the next, and two of
  them are in fog
- **Daily maze**: one 5×5×3×3 maze a day, from the date
- **Endless**: a new maze every time in four sizes, with fog if you like
- Best times are kept for every maze

Arrows or WASD walk inside a slice. J and L step to the slice on the left
or right, I and K to the slice above or below (Shift with the arrows does
the same). V or Tab turns the maze. H shows the next steps of the way
and adds ten seconds. P pauses, R starts over, M turns the sound off and
on, G hides or shows your best run, Esc goes back.

Beat a campaign maze or the daily one and your run is kept: next time a
grey `@` walks it beside you, on your clock, so you can see whether you
are ahead.

All its sounds are made as it plays, from simple waves: a tick for each
step, a chime for stepping through to another slice, a thud at walls.
On Linux it needs ALSA (libasound2) for sound and libudev for gamepads.

Gamepad: left stick or d-pad walk, right stick or the face buttons step
between slices, the triggers turn the maze, LB hints, Start pauses, Back
goes back.

The bigger mazes need a bigger terminal (the largest about 93×37); the
game says so when a maze does not fit.

## Where it comes from

Tessera is an Asciiverse game, after bleuje's browser 4D maze
(bleuje.com/4dmaze), which shows a four-dimensional maze as a grid of
flat ones. Written from scratch in Rust with Ratatui.

## License

MIT
