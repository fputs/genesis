# Proposal

## Why

The rules crate can roll a check, fold modifiers, and measure a square, and it still cannot open a fight. Initiative, the turn, and Strike are what spend the three actions and turn a degree of success into damage. This change is that slice of encounter-core. Dying and victory stay in `add-dying-and-victory`.

## What Changes

- Add the `encounter-turns` capability: one Perception roll per combatant, highest total first; a PC–enemy tie gives the enemy the earlier turn; any other tie uses the lower `CombatantId` (DEV-05). Each combatant rolls separately.
- A turn runs an empty start step, then regains 3 actions and 1 reaction. An empty end step drops leftover actions and keeps an unspent reaction. Reactions are ready from encounter start (DEV-06). Spending the last action ends the turn. The next turn starts only when a start is requested.
- Strike spends 1 action, rolls against the target's baked Armor Class, and applies the multiple attack penalty. The degree after a natural 20 or natural 1 selects damage: a success deals one damage roll, a critical success doubles that total, and a miss deals none.
- Strike damage follows R-07 and DEV-14. Dice and modifiers, then penalties, then the floor of 1 or a clamp at 0, then doubling. The smallest floored crit is 2. A total clamped at 0 stays 0 on a crit. R-06 applies only where Strike deals damage: temporary Hit Points are removed first. Gaining temporary Hit Points, durations, and healing stay out of this change.
- A reaction's multiple attack penalty is 0, and the check does not change the creature's own-turn count. R-05, the readied-attack snapshot, belongs to a separate spec.
- **BREAKING**: `attack_check_total` takes a public caller-modifier list again and returns `Result`. The list is folded with the internal terms by the modifier stacking rules. Commit `d6da856` dropped the list because no spec covered it, and the tests then read a private helper. This change puts the list back so those cases are tested through the public function (AGENTS.md §8).
- Out of scope: dying and wounded values, unconscious, massive damage, victory, Stride, Step, reach, flanking, prone, shields, and resolving a reaction activity.

## Capabilities

### New Capabilities

- `encounter-turns`: Initiative, the round and the turn, action accounting, and Strike damage.

### Modified Capabilities

- `modifiers`: Remove "Attack check internal terms use reserved sources" and add "Attack check total takes caller modifiers", so the reserved-source cases are specified on the public function.

## Impact

`crates/rules` gains an encounter module and re-exports it. No new crate and no new dependency. Existing `attack_check_total` call sites pass the list and handle `Result`. `CombatantState` keeps its fields; the encounter updates actions and the reaction. This change names no file on `v1-reference`, so none is ported. This pull request is spec-only: no changelog bullet and no archive. The implementation pull request adds the Unreleased bullet, then archives as its last commit.
