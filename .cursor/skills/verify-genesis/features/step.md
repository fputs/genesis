# Step

Step spends one action and lands on a lit square 5 feet away. The log names the creature, the destination square, `5 ft`, and the actions still left.

## Sub-features

- `step-aim` lights only the legal Step squares.
- `step-square` moves the active creature onto the clicked square and spends one action.
- `step-illegal` rejects a click that is not a lit square.

## How to get to it (user POV)

- Be in the fight, on a player character's turn.
- Press `T`, or click the `Step (T)` button on the bottom bar.
- Click a gold square.
- Press `Escape` to put the aim away without moving.

## Driving it with control-genesis

Preconditions:

- The encounter has started. `control-genesis read-hud` contains `Initiative:`.
- The HUD names a player character as the active creature, not Hall Guard. On a fresh fight with seed 115 that creature is Sera Voss, with three actions.
- No modal is open. If one is, run `.cursor/skills/verify-genesis/scripts/control-genesis key Escape`.

- **Aim.** Press `T`. Run `.cursor/skills/verify-genesis/scripts/control-genesis key T`. Gold squares appear on the squares 5 feet away. The bottom bar shows `Step (T)` beside `Stride (M)`, then `Strike (F)` and `End Turn (Space)`. The heading reads `Combat` after this rebuild.
- **Step.** Click a lit square. Run `.cursor/skills/verify-genesis/scripts/control-genesis click --gold`. The printed HUD contains `steps to` and `5 ft`, and the actions-remaining count is one lower than before the click.
- **Illegal square.** Aim with `T` again and click a dark floor square only if you are proving the rejection. The modal reads `That square is out of the step.` Dismiss it with `.cursor/skills/verify-genesis/scripts/control-genesis key Escape`.
- **Proof.** Keep the HUD text from `click --gold`. It is the action and the resulting state. The feature id is `step-square` and the entry point is `T`, then a click on a lit square.

## Gotchas

- The bottom bar can be missing until the first `T` or `M` after the trigger. `T` both aims and rebuilds the panel. Do not fail the fight for a missing bar before that key.
- Gold squares exist only on a player turn, and only while Step is aimed. On the guard's turn, wait. The guard acts on its own and does not Step.
- A click on the gold border of the panel is not a square. `click --gold` searches the room, inside the floor, and skips the panels.
- The destination square depends on where the party was standing when the trigger fired. Assert `steps to`, `5 ft`, and a lower action count, not a hardcoded coordinate, unless this same process just recorded that coordinate.
- `Escape` clears the aim. A following click does not move anyone until `T` is pressed again.
- A lit Step square is also a legal Stride. After the Step, press `M` and click a lit square to confirm the log still says `strides to`.
