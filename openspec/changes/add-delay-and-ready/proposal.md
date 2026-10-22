# Proposal

## Why

The encounter can spend one reaction and can Strike on the open turn, but a creature cannot leave the initiative order or prepare one action for a later trigger. Delay and Ready are the last Phase 1 procedures (Player Core pg. 416 and pg. 417). The encounter still refuses to keep the attack count Ready needs, which ruling R-05 and backfill B-06 already settled.

## What Changes

- Add the `delay` capability. Delay is a free action with the trigger "your turn begins" (https://2e.aonprd.com/Actions.aspx?ID=2294, Player Core pg. 416). The creature leaves the order. It returns, as a free action, at the end of any other creature's turn, and that return permanently changes its place. The Perception total stays the number rolled. While it is out of the order it cannot use reactions (FIX-04). The reaction itself was regained by the normal start step (https://2e.aonprd.com/Rules.aspx?ID=2428, Player Core pg. 435): `reaction_ready` stays true, and a spend returns `Delayed` without clearing it. Returning does not spend the reaction. If the creature does not return before its slot comes up again, the delayed turn's actions are lost and the arrival is the next turn at the original place.
- Effects that already ran at the start of the turn are not run again on return. The merged encounter start changes no Hit Points and no temporary Hit Points, and this change does not call `dying::start_of_turn` or add a tick (DEV-32 stays where dying-and-victory put it). A raised shield lowers in the shields change's start, not on return. Persistent damage is not modeled (DEV-29), and the end step still changes no Hit Points, so this change adds no end-of-turn damage order. Delay does not change the multiple attack penalty and does not write the attack count.
- Add the `ready` capability. Ready costs 2 actions, has the concentrate trait, stores one trigger and one action, then ends the turn (https://2e.aonprd.com/Actions.aspx?ID=2299, Player Core pg. 417). The action is one action or one free action. A free action that already has a trigger cannot be readied. The stored action is used later as a reaction, and that use spends the reaction. Ready expires at the start of the creature's next turn.
- Ready snapshots the attack count when it resolves. A readied attack calls `multiple_attack_penalty` with `readied_attack_count: Some(snapshot)` and with agile from the readied weapon (R-05, B-06). It does not increment the own-turn count. A Dying or Unconscious target is still a legal Strike. A Dead target spends nothing.
- Triggers are a closed set the engine can already observe: the subject enters one square within the readier's reach, or the subject begins Strike, Stride, Step, Drop Prone, or Stand. The readied action is Strike, Stride, Step, or Dismiss. Proposed DEV-52, DEV-53, and DEV-54 are in design.md. This change does not edit `docs/designer/`.
- Amend `encounter-turns` requirement "A reaction attack penalty stays 0" so a readied attack is the exception. The scenario title "A reaction after two Strikes keeps penalty 0" stays verbatim, and that scenario stays the non-readied case.

Out of this change, and not given a wrong substitute: any trigger outside those two kinds, any readied action outside those four, a readied Strike against someone other than the trigger subject, walking a whole path for reach, persistent damage, frightened, slowed, stunned, and a concentrate condition. Reactive Strike, Shield Block, and Aid keep their own reactions. They share the one reaction, and they fail with `Delayed` while the creature is out of the order. Classes, spells, and feats stay out.

This change does not add a ruling. The log is [docs/designer/rules-rulings.md](../../../docs/designer/rules-rulings.md). R-05 is cited, not copied.

## Capabilities

### New Capabilities

- `delay`: Leave the initiative order at the start of your turn, return at the end of another creature's turn, keep the regained reaction unusable until then, and lose the delayed turn if you do not return in time.
- `ready`: A 2-action concentrate Ready, a typed trigger, one readied action used as a reaction, the attack-count snapshot, and expiry at the next start.

### Modified Capabilities

- `encounter-turns`: Requirement "A reaction attack penalty stays 0" exempts a readied attack. The scenario title "A reaction after two Strikes keeps penalty 0" stays verbatim for a reaction that is not readied.

## Impact

- `Encounter::delay`, `Encounter::return_from_delay`, and `Encounter::spend_reaction` gain the delayed state. `spend_reaction` returns `EncounterError::Delayed` while that creature is out of the order and does not clear `reaction_ready`. `start_turn` plays the next turn at the original slot when a delayed creature's slot arrives.
- Ready stores the snapshot on the encounter, keyed by `CombatantId`. Firing a readied Strike calls `multiple_attack_penalty` with `readied_attack_count: Some(snapshot)`. That field is `amend-readied-attack-map`. This change does not add a second penalty table. `attack_trait_penalty` stays the non-readied reader and still returns 0 off the open turn.
- Stride and Step stay `grid::stride` and `grid::step`. A readied move uses a fresh `DiagonalCounter` (DEV-15). Dismiss stays `positioning::dismiss`. Reach stays `in_reach`. Damage stays `apply_resolved_blow`. No new crate and no new dependency. No file from `v1-reference` is ported.
- This pull request is spec only, so it adds no changelog bullet and does not archive. The implementation pull request adds the Unreleased bullet, then archives with `openspec archive add-delay-and-ready --yes` as its last commit.
