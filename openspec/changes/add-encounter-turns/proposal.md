# Proposal

## Why

The rules crate can roll a check, fold modifiers, and measure a square, and it still cannot open a fight. Initiative, the turn, and Strike are what spend the three actions and turn a degree of success into damage. This change is that slice of encounter-core. Dying values, massive damage, and victory stay in `add-dying-and-victory`. Wiring those rules into the turn stays in `wire-dying-into-turns`.

## What Changes

- Add the `encounter-turns` capability: one Perception roll per combatant, highest total first. A PC–enemy tie gives the enemy the earlier turn (Player Core pg. 435). Any same-Side tie uses the lower `CombatantId` (DEV-05, amended to that id order). Each combatant rolls separately. `Side` is `Pc` or `Enemy`, and in Phase 1 a faction is a `Side` (DEV-26).
- A turn runs an empty start step, then an Alive creature regains exactly 3 actions and 1 reaction. A creature that is not Alive gets 0 actions and no reaction; its turn opens and closes empty. Dead creatures stay in the order and are skipped. An empty end step drops leftover actions and keeps an unspent reaction. Reactions of Alive creatures are ready from encounter start (DEV-06). Spending the last action is an engineering handoff: the caller calls `end_turn`, nothing further can be spent, and no end-of-turn effect fires before that call. `end_turn` runs the end step once and advances once. The next turn starts only when a start is requested.
- Strike spends 1 action, rolls against the target's baked Armor Class, and applies the multiple attack penalty from this turn's count. The degree after a natural 20 or natural 1 selects damage: a success deals one damage roll, a critical success doubles that total, and a miss deals none. A blow at Failure or CriticalFailure returns `HpError::NotAHit` and leaves Hit Points unchanged. `add-dying-and-victory` uses that variant.
- Strike damage follows R-07 and DEV-14. Dice and modifiers, then penalties, then the floor of 1 or a clamp at 0, then doubling. The smallest floored crit is 2. A total clamped at 0 stays 0 on a crit. R-06 applies only where Strike deals damage: temporary Hit Points are removed first. Gaining temporary Hit Points, durations, and healing stay out of this change. Current Hit Points floor at 0.
- A reaction's multiple attack penalty is 0, and the check does not change the creature's own-turn count. R-05, the readied-attack snapshot, belongs to a separate spec.
- **BREAKING**: `attack_check_total` takes a public caller-modifier list again and returns `Result`. The list is folded with the internal terms by the modifier stacking rules. Commit `d6da856` dropped the list because no spec covered it, and the tests then read a private helper. This change puts the list back so those cases are tested through the public function (AGENTS.md §8). `dice-and-checks` does not name that return type, so its spec is unchanged.
- Out of scope: dying and wounded values, unconscious, massive damage, victory, the turn wiring in `wire-dying-into-turns`, spending encounter actions on Stride and Step, reach, flanking, prone, shields, Delay, Ready, and resolving a reaction activity. Stride and Step already exist in grid-movement.

## Capabilities

### New Capabilities

- `encounter-turns`: Initiative, the round and the turn, action accounting, and Strike damage.

### Modified Capabilities

- `modifiers`: Remove "Attack check internal terms use reserved sources" and add "Attack check total takes caller modifiers", so the reserved-source cases are specified on the public function.

## Impact

`crates/rules` gains an encounter module and re-exports it. No new crate and no new dependency. Existing `attack_check_total` call sites pass the list and handle `Result`. `attack_check_parts` keeps its callers. `CombatantState` keeps its fields; the encounter is the only mutator of actions, the reaction, Hit Points from Strike, and the order. This change names no file on `v1-reference`, so none is ported. This pull request is spec-only: no changelog bullet and no archive. The implementation pull request adds the Unreleased bullet, then archives as its last commit.
