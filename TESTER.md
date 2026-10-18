# Tester guide

Walkthrough for the explore-to-combat slice. Run these from the repository root.

## Build

Rust comes from `rust-toolchain.toml` (1.99.1). `rustup` installs it on first `cargo` invocation.

Linux packages the Bevy window needs:

```
sudo apt-get install -y pkg-config libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev libxkbcommon-x11-0 mesa-vulkan-drivers
```

```
cargo test
cargo build -p game
cargo build -p editor
```

`cargo test` includes headless rules, fixture validation, and a Bevy `MinimalPlugins` transition that snaps into combat and resolves one Stride or Strike. It does not need a GPU.

## Headless smoke

```
cargo run -p game -- --help
cargo run -p game -- --headless
```

`--headless` loads `fixtures/encounters/room_one.json`, snaps a party that has walked onto the trigger, rolls initiative with seed 115, and resolves the fight. On the first party turn the log contains `drops prone` and `stands`. On one later party turn the log contains `seeks`. The last line is `headless ok`. There is no scripted window pass: `control-genesis` is not in this tree.

If `DISPLAY` and `WAYLAND_DISPLAY` are both unset, `cargo run -p game` runs that smoke and exits.

## Window

```
cargo run -p game
```

A window titled **Genesis** opens. The log line `loaded encounter room_one` appears in the terminal. If startup panics with `Unable to find a GPU`, install `mesa-vulkan-drivers` and run again. llvmpipe is enough; the renderer warns that software rendering is slow. No sound device is expected.

What you should see:

- Diamond stone room, walls around the edge.
- Four tokens (blue, green, burgundy, ochre) grouped on the west side.
- A red token, the Hall Guard, toward the east.
- Left column, four portraits: Sera Voss, Bram Holt, Quill Ames, Doss Kade, each with HP.
- Upper right: **Explore** and the line `D walks east, toward the guard.`

### Explore

- **D** moves the party grid-east (down-right on screen), toward the guard. **A** west, **W** north, **S** south. Arrows match those grid directions.
- **1–4** or a portrait click makes that character the formation leader.
- Hold **D** until the upper-right heading changes to **Combat**. The trigger is the squares around (6, 4). The fight does not start on the opening tiles.
- The party tokens snap onto squares. Initiative totals print in the log. Explore movement stops.

### Combat

The bar at the bottom shows **Stride (M)**, **Step (T)**, **Strike (F)**, **Raise Shield (R)**, and **End Turn (Space)**. When the action is legal it also shows **Drop Prone (P)**, **Stand (G)**, **Crawl (C)**, **Leap (L)**, **Take Cover (V)**, **Seek (K)**, and **Point Out (O)**. Point Out then takes a click on a creature you observe. The heading names the active creature, the round, actions remaining, and `Reaction ready`. An enemy who is Undetected by the active combatant is not drawn. A Hidden enemy can still be clicked for Strike.

- Once a fighter is adjacent to the Hall Guard, wait on the guard's turn. The guard opens with a Stride that leaves that square. The log then shows `reactive strike`, `vs AC`, and `HP left` for the fighter who reacted. The guard has no Reactive Strike of their own. A Step away does not write that line. The guard still does not Step, and still Strikes on a later action. About half a second passes per action.
- On a player turn, press **M**. Gold squares are legal Strides (one action, up to Speed 25). Click one. The log records `strides to`, the square, and the feet spent.
- Press **T**. Gold squares are legal Steps: one action, 5 feet, and only when Speed is at least 10. Click one. The log records `steps to` and `5 ft`, and the action count drops by one.
- Press **F** and click the guard when a gold square is on them. The log records the d20, the total against Armor Class, and HP left. A second and third Strike in the same turn show the multiple attack penalty in the total.
- **R** or **Raise Shield (R)** spends one action when the active fighter is wielding a shield that is not broken. The log records `raises a shield`.
- When a party member would take damage and Shield Block is legal, the fight asks. **B** uses it. **N** takes the hit. A block writes `shield block` in the log. The Hall Guard has no shield and does not raise one.
- **Space** or **End Turn** passes the rest of the turn. **Esc** cancels a target or dismisses a modal. It does not answer Shield Block.
- An illegal click opens a modal. **Continue** or **Esc** closes it.

Repeat until the log says the guard `is dead`. A drop to 0 HP writes `is dying` and the fight continues. Keep Striking the dying guard. The red token stays until that death. The modal then reads `The guard falls. The hall is quiet.` **Continue** returns to Explore. The red token is gone. **D** moves the party again. The trigger does not start a second fight.

A wiped party shows `The party is down.` Portraits stay at four slots. A dying character reads **Dying** and a dead character reads **Dead**. The window driver `control-genesis` is not in this tree, so this walkthrough has no scripted window pass. Use the headless log until that script is restored.

## Map editor

Separate window from the game. No portraits and no combat bar.

```
cargo run -p editor -- fixtures/rooms/small_room.json
```

A window titled **Genesis Editor** opens. The terminal logs `loaded room small_room`. You should see:

- Top bar: **Load**, **Save**, map name `small_room`, cell size `5 ft`.
- Left column, three floors: **Stone** (selected, light border), **Grass**, **Dirt**.
- Center: a 12 by 8 grid of stone squares. The first row of the file is the top row.
- Right: the same 96 squares as smaller diamonds.
- Bottom: `Paint floor · small_room · 12×8 · 5 ft`.

Click **Grass**. Its border turns light. Click one square in the grid (an interior square is easy to see). That square and the matching diamond turn green. The status stays `Paint floor · small_room · 12×8 · 5 ft`.

Click **Save**. The status becomes `Saved · small_room · 12×8 · 5 ft`. `fixtures/rooms/small_room.json` now has `"grass"` in that cell's place in `floors`. The `#` and `.` rows are unchanged.

Press **Esc**. Run the same command again. The painted square is still green in the grid and in the diamonds. **Load** reads the file again and shows those same cells.

**Dirt** paints the same way. The only floors are stone, grass, and dirt. A bad path leaves the grid reading `Could not load this map.` and puts the load error in the status line.

## Fixture the loop uses

| Piece | File |
| --- | --- |
| Encounter | `fixtures/encounters/room_one.json` |
| Room | `fixtures/rooms/small_room.json` |
| Fighters | `fixtures/creatures/fighter_l1.json` |
| Guard | `fixtures/creatures/enemy_mvp.json` |
