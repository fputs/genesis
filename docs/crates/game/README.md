# game

Bevy 0.15.3 host for the explore-to-combat loop. `rules` and `content` stay engine-free; this crate maps fixture data onto Bevy entities and `GameMode`.

`GameMode` is the only mode switch: `Explore`, `TransitioningToCombat`, `Combat`. Explore movement runs only in `Explore`. There is no real-time-with-pause schedule.

## What you see

- Orthographic diamond room. Walkable squares use the room floor (stone, grass, or dirt). Walls are darker tiles. Tokens are Y-sorted by grid depth.
- Four portrait slots on the left, always, including a dying or dead party member. A portrait at 0 HP reads Dying or Dead. The fight ends when one side is dead.
- Combat bar: Stride, Step, Strike, End Turn. Feedback and the initiative line sit in the upper right. When the Hall Guard leaves a fighter's reach, the log adds a reactive strike line. A modal covers the end of the fight and illegal clicks.
- Player chrome is Bevy UI.

The floor is not `bevy_ecs_tilemap`. Each square is a sprite, which keeps the combat grid explicit without a plugin pin.

## Run

`rust-toolchain.toml` selects Rust 1.99.1. Bevy's current crates.io graph does not build on 1.83. A window also needs `libasound2-dev`, `libudev-dev`, `libwayland-dev`, `libxkbcommon-dev`, and `libxkbcommon-x11-0` (see `TESTER.md`).

```
cargo run -p game
cargo run -p game -- --headless
cargo run -p game -- --help
```

The binary loads `fixtures/encounters/room_one.json` through `content::load_encounter`. A missing `DISPLAY` and `WAYLAND_DISPLAY` runs the headless smoke instead of opening a window.

Explore: D east, A west, W north, S south, on the grid. 1–4 or a portrait selects the formation leader. Combat: M then a lit square to Stride, T then a lit square to Step, F then the guard to Strike, Space to end the turn. Once a fighter is adjacent, wait on the Hall Guard's turn. The guard strides out of that reach, and the log shows `reactive strike`. The guard has no Reactive Strike. The enemy spends its own actions.

## Wiring

Bevy entities never enter `rules`. `CombatantId` is the only combatant key. Input becomes `Action::Stride`, `Action::Step`, or `Action::Strike`, or an end-turn request. Seeded dice use the fixture `rng_seed`.
