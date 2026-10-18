# game

Bevy 0.15.3 host for the explore-to-combat loop. `rules` and `content` stay engine-free; this crate maps fixture data onto Bevy entities and `GameMode`.

`GameMode` is the only mode switch: `Explore`, `TransitioningToCombat`, `Combat`. Explore movement runs only in `Explore`. There is no real-time-with-pause schedule.

## What you see

- Orthographic diamond room. Walkable squares use the room floor (stone, grass, or dirt). Walls are darker tiles. Tokens are Y-sorted by grid depth.
- Four portrait slots on the left, always, including a dying or dead party member. A portrait at 0 HP reads Dying or Dead. The fight ends when one side is dead.
- Combat bar: Stride, Step, Strike, Raise Shield, End Turn, and, only when that action is legal, Drop Prone, Stand, Crawl, Leap, Take Cover, Seek, Point Out, Interact, Release, Delay, and Ready. Feedback and the initiative line sit in the upper right. When the Hall Guard leaves a fighter's reach, the log adds a reactive strike line. When a party member would take damage and Shield Block is legal, the fight asks; B uses it and N does not. A shield block line contains `shield block`. A modal covers the end of the fight and illegal clicks. The Hall Guard has no shield and does not Raise a Shield. An enemy who is Undetected by the active combatant is not drawn. A Hidden enemy can still be clicked for Strike.
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

Explore: D east, A west, W north, S south, on the grid. 1–4 or a portrait selects the formation leader. Combat: M then a lit square to Stride, T then a lit square to Step, F then the guard to Strike, R to Raise a Shield, P to Drop Prone, G to Stand, C then a lit square to Crawl, L then a lit square to Leap, V to Take Cover, K to Seek, O then a click on a creature you observe to Point Out, I to Interact, X to Release the shield if it is held and otherwise the weapon, D to Delay, Y to Ready, Space to end the turn. Ready opens a choice of trigger (E enters reach, W leaves reach, Q manipulate) and then the readied action. Strike, Stride, and Step still need a target or square. Drop Prone, Stand, Crawl, Leap, Take Cover, Seek, Point Out, Interact, Release, Delay, and Ready appear only when legal. B uses Shield Block and N declines it. When a delaying party member may resume, U takes the turn and H keeps delaying. Once a fighter is adjacent, wait on the Hall Guard's turn. The guard strides out of that reach, and the log shows `reactive strike`. The guard has no Reactive Strike and no shield. The enemy spends its own actions. `cargo run -p game -- --headless` uses Shield Block whenever it is legal. On the first party turn it drops prone and then stands, and the log contains `drops prone` and `stands`. It does not drop prone again. On one later party turn it Seeks once, and the log contains `seeks`. On one party turn, while that fighter is outside the Hall Guard's reach, it releases the shield and picks it up, and the log contains `releases` and `picks up`. On one later party turn it Delays and resumes at the next end of turn, and the log contains `delays` and `resumes`. It does not Ready. The window driver `control-genesis` is not in this tree, so there is no scripted window pass.

## Wiring

Bevy entities never enter `rules`. `CombatantId` is the only combatant key. Input becomes `Action::Stride`, `Action::Step`, `Action::Strike`, `Action::RaiseShield`, `Action::DropProne`, `Action::Stand`, `Action::Crawl`, `Action::Leap`, `Action::Seek`, `Action::PointOut`, `Action::TakeCover`, `Action::Interact`, `Action::Release`, `Action::Delay`, or `Action::Ready`, or an end-turn request. A Shield Block offer is answered with accept or decline. A Delay resume is answered the same way: accept resumes, decline keeps delaying. Headless always resumes. Seeded dice use the fixture `rng_seed`. The Hall Guard and the fighters have no Stealth DC.
