# Proposal

## Why

The encounter can already spend one reaction, and a Strike on your own turn already rolls against baked Armor Class. Nothing yet makes the melee Strike that answers a manipulate action, a move action, or a ranged attack inside reach, and nothing disrupts that manipulate action on a critical hit. This change is that procedure.

## What Changes

- Add the `reactive-strike` capability. One call resolves one reactor against one triggering creature. It calls `in_reach`, `degree_of_success`, `attack_check_total`, `damage_dealt`, and `apply_resolved_blow`. It does not reimplement them, and it does not call `Encounter::strike` or `note_hit_points`.
- The trigger is the actor's `ActionCost` at the actor's square. Positioning's `ActionTrait` gains `Manipulate` and `Ranged` beside `Move`. A cost whose traits include `Move`, `Manipulate`, or `Ranged` triggers. A cost with both `Move` and `Manipulate` is one action. Interact is one manipulate action. Release is the free manipulate action and does not trigger. Seek has none of those traits. `MoveAction::Step` does not trigger. Drop Prone and Stand are move (FIX-06). A ranged attack triggers with `Ranged` and no manipulate trait (FIX-08). The caller passes Point Out with `Manipulate` (FIX-07). This change does not resolve Point Out, Stride, Drop Prone, or Stand.
- The reaction is spent only through `Encounter::spend_reaction`, and only when the Strike returns `Ok`. A second call is `NoReaction`. The spend takes no action from the reactor or the actor and does not end the turn. It is legal before the first turn, during another creature's turn, and on the reactor's own turn.
- The multiple attack penalty on this Strike is 0, whether or not the weapon is agile, on the reactor's turn and off it. The stored attack count does not change. The reactor's next own-turn Strike still uses the count from before the reaction. `encounter-turns` is modified so a Reactive Strike off the reactor's own turn spends no action, applies penalty 0, and leaves the count unchanged. `Encounter::strike` still spends an action and still returns `NotYourTurn` off the active turn.
- A critical hit on a cost that includes `Manipulate` returns `Disruption::Disrupted`. The actor's own turn keeps that action's cost spent, and the action's effects are voided. A success, a failure, a critical failure, a move-only cost, and a ranged cost return `Disruption::NotDisrupted`. This function does not spend the actor's action and does not refund it.
- A reactor that is not `Vital::Alive` returns `NotConscious`, draws no die, and leaves the reaction as it was. An actor that is not `Vital::Alive` returns `TargetNotConscious` in the slot `TargetDead` held. `TargetDead` is not a variant of this error. That is `Dying`, `Unconscious`, and `Dead`.
- Reach is the reactor's stored `ReachFeet`. Squares and sizes are caller values. The encounter does not gain a square field. Reach errors use positioning's `UnsupportedSize`, `Overflow`, and `InvalidReach`, with `Overflow` before `InvalidReach`.

Out of this change, and not given a wrong substitute: a creature who starts outside reach and leaves a square along a move (proposed DEV-45). Grid-movement already emits leave and enter events and does not resolve a reaction. This change does not read those events. The class that grants the reaction is proposed DEV-44. Hidden, concealed, and undetected targeting is proposed DEV-46: the caller uses `resolve_targeting`. Off-guard from just dropping prone is ignored because Armor Class is baked (DEV-01). Shield Block is not taken. Step's movement, Crawl's distance, and Tumble Through stay out. There is no second damage formula and no second dying formula.

This change does not add a ruling. The log is [docs/designer/rules-rulings.md](../../../docs/designer/rules-rulings.md). Proposed DEV-44, DEV-45, and DEV-46 are written in design.md. This change does not edit `docs/designer/`.

## Capabilities

### New Capabilities

- `reactive-strike`: The in-reach trigger, the one reaction spent through encounter-turns, multiple-attack-penalty independence, disruption of a manipulate action on a critical hit, and `NotConscious` when the reactor is not Alive.

### Modified Capabilities

- `encounter-turns`: `Strike spends one action against baked Armor Class` and `The multiple attack penalty follows this turn's count`. A Reactive Strike while it is not that reactor's open turn spends no action, applies a multiple attack penalty of 0, and leaves the stored attack count unchanged. `Encounter::strike` is unchanged.

## Impact

New module `crates/rules/src/reactive.rs`, re-exported from the crate root. `Encounter::reactive_strike` calls `spend_reaction`, `attack_check_total`, and `apply_resolved_blow`. It does not declare `Side`, `Vital`, `Size`, `Square`, `Agility`, `DegreeOfSuccess`, `StrikeOutcome`, `ActionCost`, or `ActionTrait`. It extends positioning's one `ActionTrait` with `Manipulate` and `Ranged`. `crates/rules` gains no dependency. This pull request is the spec only: no changelog bullet and no archive. The implementation pull request adds the Unreleased changelog bullet, then archives as its last commit.

Implementation starts only once dying-and-victory's `Vital::Dying` and `Vital::Unconscious` are public, and once `apply_resolved_blow` is in the crate. `NotConscious` and `TargetNotConscious` name those variants, and the critical-hit knockout is that function's `Dying` result. It also waits until positioning's `ActionCost` and `ActionTrait` are in the crate, so this change extends that enum. `wire-leave-square-into-reactive-strike` is the follow-up for proposed DEV-45. `wire-targeting-into-reactive-strike` is the follow-up for proposed DEV-46. `wire-shield-block-into-reactive-strike` is the follow-up for Shield Block.
