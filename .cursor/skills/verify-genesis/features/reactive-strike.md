# Reactive Strike

When the Hall Guard leaves a square in a fighter's reach, that fighter strikes as a reaction. The log shows `reactive strike`, the total `vs AC`, and the guard's `HP left`. The guard has no Reactive Strike of their own. A Step does not provoke.

## Sub-features

- `reactive-leave` is the guard's opening Stride off a threatened square once a fighter is adjacent.
- `reactive-line` names the reacting fighter, `reactive strike`, `vs AC`, and `HP left`.
- `reactive-step` is a Step away from the guard, which does not write `reactive strike`.

## How to get to it (user POV)

- Start the fight and spend player turns until a fighter stands next to the Hall Guard.
- Wait on the Hall Guard's turn. Do not press `M`, `T`, or `F` while the guard is acting.
- To see that a Step does not provoke, on your turn press `T` and click a lit square away from the guard instead of waiting.

## Driving it with control-genesis

Preconditions:

- The encounter has started. `control-genesis read-hud` contains `Initiative:`.
- A fighter is on a square next to the Hall Guard. If `F` lights no square, Stride closer with `M` and `click --gold` until a fighter is adjacent.
- No modal is open. If one is, run `.cursor/skills/verify-genesis/scripts/control-genesis key Escape`.

- **Wait.** When the HUD names Hall Guard, poll `.cursor/skills/verify-genesis/scripts/control-genesis read-hud` until it contains `reactive strike`. The guard spends about half a second per action. The same text contains `vs AC` and `HP left`.
- **Step does not provoke.** On a player turn, while a fighter is adjacent, press `T` and click a lit square. Run `.cursor/skills/verify-genesis/scripts/control-genesis key T`, then `.cursor/skills/verify-genesis/scripts/control-genesis click --gold`. The HUD contains `steps to` and does not contain `reactive strike`.
- **Proof.** Read the HUD as soon as the reaction line appears, before the next player action. The feature id is `reactive-line` and the entry point is waiting on the Hall Guard's turn after a fighter is adjacent.

## Gotchas

- The Hall Guard does not have Reactive Strike. A fighter's own Stride does not draw a reaction from the guard. The line appears when the guard leaves a fighter's reach.
- A Step never provokes, even from an adjacent square. A missing line after `T` is the Step result, not a failed leave.
- Read the HUD on the guard's turn, before the next player action. The reaction line stays on screen with the Stride line.
- OCR may wrap the line or drop the dash. Assert `reactive strike`, `vs AC`, and `HP left`.
- Seed 115 does not fix the square the guard leaves toward. Assert the line, not a coordinate.
- The guard still Strikes on a later action of that turn. The fight can still end with `The guard falls. The hall is quiet.`
