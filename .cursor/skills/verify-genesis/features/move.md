# Stride

Stride spends one action and lands on a lit square. The log names the creature, the destination square, the feet spent, and the actions still left.

## Sub-features

- `move-aim` lights the legal Stride squares.
- `move-step` moves the active creature onto the clicked square and spends one action.
- `move-illegal` rejects a click that is not a lit square.

## How to get to it (user POV)

- Be in the fight, on a player character's turn.
- Press `M`, or click the `Stride (M)` button on the bottom bar.
- Click a gold square.
- Press `Escape` to put the aim away without moving.

## Driving it with control-genesis

Preconditions:

- The encounter has started. `control-genesis read-hud` contains `Initiative:`.
- The HUD names a player character as the active creature, not Hall Guard. On a fresh fight with seed 115 that creature is Sera Voss, with three actions.
- No modal is open. If one is, run `.cursor/skills/verify-genesis/scripts/control-genesis key Escape`.

- **Aim.** Press `M`. Run `.cursor/skills/verify-genesis/scripts/control-genesis key M`. Gold squares appear in the room. The bottom bar shows `Stride (M)`, `Strike (F)`, and `End Turn (Space)`. The heading reads `Combat` after this rebuild.
- **Stride.** Click a lit square. Run `.cursor/skills/verify-genesis/scripts/control-genesis click --gold`. The printed HUD contains `strides to` and `ft.` and the actions-remaining count is one lower than before the click.
- **Illegal square.** Aim with `M` again and click a dark floor square only if you are proving the rejection. The modal reads `That square is out of the stride.` Dismiss it with `.cursor/skills/verify-genesis/scripts/control-genesis key Escape`. `click --gold` already dismisses that modal and tries a different lit square, so a normal stride does not stop there.
- **Proof.** Keep the HUD text from `click --gold`. It is the action and the resulting state. The feature id is `move-step` and the entry point is `M`, then a click on a lit square.

## Gotchas

- The bottom bar can be missing until the first `M` after the trigger. `M` both aims and rebuilds the panel. Do not fail the fight for a missing bar before that key.
- Gold squares exist only on a player turn, and only while Stride is aimed. On the guard's turn, wait. The guard acts on its own.
- A click on the gold border of the panel is not a square. `click --gold` searches the room, inside the floor, and skips the panels.
- The destination square depends on where the party was standing when the trigger fired. Assert `strides to` and a lower action count, not a hardcoded coordinate, unless this same process just recorded that coordinate.
- `Escape` clears the aim. A following click does not move anyone until `M` is pressed again.
