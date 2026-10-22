# Proposal

## Why

The encounter can already spend one reaction, and a Strike on your own turn already rolls against baked Armor Class. Nothing yet makes the melee Strike that answers a manipulate action, a move action, or a ranged attack inside reach, and nothing disrupts that manipulate action on a critical hit. This change is that procedure.

## What Changes

- Add the `reactive-strike` capability. One call resolves one reactor against one triggering creature. It calls `in_reach`, `degree_of_success`, `attack_check_total`, `damage_dealt`, and the same Hit Point function `Encounter::strike` uses. It does not reimplement them, and it does not call `Encounter::strike`.
- The trigger is the actor's square when the action is used. `Activity::Manipulate`, `Activity::Move`, and `Activity::RangedAttack` trigger when that square is inside the reactor's stored melee reach. `Activity::Step` and `Activity::Other` do not. Drop Prone and Stand are `Activity::Move` (FIX-06). A ranged attack triggers with no manipulate trait and no move trait (FIX-08). The caller classifies Point Out as `Activity::Manipulate` (FIX-07). This change does not resolve Point Out, Stride, Drop Prone, or Stand.
- The reaction is spent only through `Encounter::spend_reaction`, and only when the Strike returns `Ok`. A second call is `NoReaction`. The spend takes no action from the reactor or the actor and does not end the turn. It is legal before the first turn and during another creature's turn.
- The multiple attack penalty on this Strike is 0, whether or not the weapon is agile, on the reactor's turn and off it. The stored attack count does not change. The reactor's next own-turn Strike still uses the count from before the reaction.
- A critical hit on a manipulate action returns `Disruption::Disrupted`. A success, a failure, a critical failure, a move, and a ranged attack return `Disruption::NotDisrupted`. The caller keeps the triggering action's cost: this function does not spend it and does not refund it.
- A reactor that is not `Vital::Alive` returns `NotConscious`, draws no die, and leaves the reaction as it was. That is `Dying`, `Unconscious`, and `Dead`.
- Reach is the reactor's stored `ReachFeet`. Squares and sizes are caller values. The encounter does not gain a square field.

Out of this change, and not given a wrong substitute: a creature who starts outside reach and leaves a square along a move (proposed DEV-45). Grid-movement already emits leave and enter events and does not resolve a reaction. This change does not read those events. Classes and feats are proposed DEV-44. Step's movement, Crawl's distance, and Tumble Through stay out. There is no second damage formula and no second dying formula.

This change does not add a ruling. The log is [docs/designer/rules-rulings.md](../../../docs/designer/rules-rulings.md). Proposed DEV-44 and DEV-45 are written in design.md. This change does not edit `docs/designer/`. No other spec's requirement header changes.

## Capabilities

### New Capabilities

- `reactive-strike`: The in-reach trigger, the one reaction spent through encounter-turns, multiple-attack-penalty independence, disruption of a manipulate action on a critical hit, and `NotConscious` when the reactor is not Alive.

### Modified Capabilities

None. `encounter-turns` already readies one reaction, spends it through `spend_reaction`, and keeps an off-turn attack-trait penalty at 0 without changing the count. Those requirements stay. `add-positioning` already names the move trait on Drop Prone and Stand and does not resolve this reaction. `add-dying-and-victory` already names `Vital` and the not-Alive `NotConscious` error. This change calls those results. Positioning and dying-and-victory are not under `openspec/specs/`, so a MODIFIED header cannot match main byte for byte.

## Impact

New module `crates/rules/src/reactive.rs`, re-exported from the crate root. `Encounter::reactive_strike` calls `spend_reaction` and the shared Strike math. It does not declare `Side`, `Vital`, `Size`, `Square`, `Agility`, `DegreeOfSuccess`, or `StrikeOutcome`. `crates/rules` gains no dependency. This pull request is the spec only: no changelog bullet and no archive. The implementation pull request adds the Unreleased changelog bullet, then archives as its last commit. Implementation starts once `Vital::Dying` and `Vital::Unconscious` are public in the crate. `wire-leave-square-into-reactive-strike` is the follow-up for proposed DEV-45.
