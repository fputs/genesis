# editor

Bevy 0.15.3 window for painting a room's floors. It links `content` and reads the same room JSON as the game. `rules` stays engine-free; this crate does not roll checks or turns.

## What you see

- Top bar: Load, Save, the room id, and the cell size in feet.
- Left: Stone, Grass, and Dirt. The selected floor has a light border.
- Center: one square per map cell, row-major, first file row at the top.
- Right: the same cells as smaller diamonds.
- Bottom status: `Paint floor · {id} · {width}×{height} · {cell_feet} ft`. After Save it reads `Saved · ...`. A failed load replaces that line with the error, and the grid says `Could not load this map.`

Floors are only stone, grass, and dirt. Load and Save use `content::load_room` and `content::save_room`. Walls stay in the room's `#` / `.` rows.

## Run

```
cargo run -p editor
cargo run -p editor -- fixtures/rooms/small_room.json
cargo run -p editor -- --help
```

The default path is `fixtures/rooms/small_room.json`, from the repository root. The window is titled **Genesis Editor**. Esc closes it. With neither `DISPLAY` nor `WAYLAND_DISPLAY`, the process exits instead of opening a window.
