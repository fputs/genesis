# Genesis verification map

This directory is the maintained source for verifying the user-facing behavior of the Genesis window. Read the index before driving the app, then use the matching feature file as the recipe.

## Baseline preconditions

- `DISPLAY` is set to an X server with a window manager. A missing display makes `cargo run -p game` run the headless smoke and exit.
- Launch with `.cursor/skills/verify-genesis/scripts/control-genesis launch` from the repo root.
- The process loads `fixtures/encounters/room_one.json` (seed 115): Sera Voss, Bram Holt, Quill Ames, Doss Kade, and the Hall Guard.
- Run `control-genesis doctor` and require `ok`, the launched pid, one Genesis window, and `loaded encounter room_one`.
- Never drive a Genesis window that this verification run did not start. Two instances cannot share the display.

## Driving conventions

- Start every recipe from a fresh launch unless its preconditions say the fight is already underway.
- Drive keys with `control-genesis key`. Drive lit squares with `control-genesis click --gold`. Drive a portrait with `control-genesis click --portrait`.
- Treat every command as literal. Keep key names, hold times, and HUD substrings unchanged.
- Read the upper-right panel with `control-genesis read-hud`. The terminal log does not repeat those lines.
- Compare HUD text by substring. OCR may wrap a line or change a leading capital.
- Restore nothing in the fixture files. The process holds the only mutable state, and cleanup ends that process.
- Do not remove `/tmp/genesis-verify-evidence` during cleanup.

## Proof and skip reporting

- Capture the user action and the resulting state, not only the final screen.
- Window proof includes a client screenshot and the HUD OCR from that moment.
- Explore movement proof includes `control-genesis tokens` on the before and after screenshots.
- Combat proof includes the HUD line the action wrote (`strides to`, `steps to`, or `d20` with `vs AC` and `HP left`) and the actions-remaining count.
- Record the feature id and the entry point with every artifact.
- Report an unreachable path with the attempted command and the unmet precondition.
- Do not report a skipped entry point as verified through a different path.

## Feature entry contract

Each feature file starts with an H1 title and one paragraph describing the user-visible behavior. It then uses exactly four H2 sections in this order.

1. `Sub-features` lists short IDs with one line for each behavior.
2. `How to get to it (user POV)` lists every user entry point.
3. `Driving it with control-genesis` starts with `Preconditions:` and uses labeled bullets that pair each user action with an exact command and observable result.
4. `Gotchas` lists traps that can waste or invalidate a verification run.

Keep implementation details out of the map. Name only user paths, stable handles, required state, commands, and observable proof.

## Features

- [Explore the hall](./explore.md) covers the opening room, a short east step, and choosing a formation leader.
- [Start the encounter](./encounter-start.md) covers walking onto the trigger until the initiative log appears.
- [Stride](./move.md) covers aiming a Stride and clicking a lit square.
- [Step](./step.md) covers aiming a Step and clicking a lit square 5 feet away.
- [Strike](./strike.md) covers aiming a Strike and clicking the guard.
- [End the fight](./combat-end.md) covers the victory modal, the return to Explore, and a wiped party.
