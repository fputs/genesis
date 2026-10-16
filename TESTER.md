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
```

`cargo test` includes headless rules, fixture validation, and a Bevy `MinimalPlugins` transition that snaps into combat and resolves one Move or Strike. It does not need a GPU.

## Headless smoke

```
cargo run -p game -- --help
cargo run -p game -- --headless
```

`--headless` loads `fixtures/encounters/room_one.json`, snaps a party that has walked onto the trigger, rolls initiative with seed 115, and resolves actions until a Strike. The last line is `headless ok`.

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

The bar at the bottom shows **Move (M)**, **Strike (F)**, and **End Turn (Space)**. The heading names the active creature, the round, and actions remaining.

- On the guard's turn, wait. The guard Strides or Strikes on its own, about half a second per action.
- On a player turn, press **M**. Gold squares are legal Strides (one action, up to Speed 25). Click one. The log records the square and the feet spent.
- Press **F** and click the guard when a gold square is on them. The log records the d20, the total against Armor Class, and HP left. A second and third Strike in the same turn show the multiple attack penalty in the total.
- **Space** or **End Turn** passes the rest of the turn. **Esc** cancels a target or dismisses a modal.
- An illegal click opens a modal. **Continue** or **Esc** closes it.

Repeat until the guard is at 0 HP. The modal reads `The guard falls. The hall is quiet.` **Continue** returns to Explore. The red token is gone. **D** moves the party again. The trigger does not start a second fight.

A wiped party shows `The party is down.` Portraits stay at four slots and read **Down**.

## Fixture the loop uses

| Piece | File |
| --- | --- |
| Encounter | `fixtures/encounters/room_one.json` |
| Room | `fixtures/rooms/small_room.json` |
| Fighters | `fixtures/creatures/fighter_l1.json` |
| Guard | `fixtures/creatures/enemy_mvp.json` |
