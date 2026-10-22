# Proposal

## Why

A shield on the creature is only a stored maximum. The fight cannot raise it, add its circumstance bonus, or spend a reaction to block. This change is that procedure. `apply_resolved_blow` stays the only creature-damage formula.

## What Changes

- Add the `shields` capability: Raise a Shield, the raised shield's circumstance bonus, Shield Block, and the combatant's shield Hit Points and raised flag.
- Raise a Shield costs 1 action, spent through `spend_actions`, and sets the shield raised. The bonus is that shield's own listed circumstance bonus to Armor Class (a buckler is +1, a steel shield is +2), passed as `shield_ac_modifier` through positioning's `extra`, until the start of the defender's next turn. The start step lowers it only after the round increment has succeeded. Ending the turn does not lower it. Another creature's turn does not lower it. A Dead combatant is skipped, so that shield is never lowered and adds nothing.
- Shield Block is `ShieldBlock::{Requested, Declined}`. The trigger is physical damage from an attack while the shield is raised. Hardness is subtracted once from that blow. The defender and the shield each take the remainder. The shield breaks when its Hit Points are at or below its Broken Threshold and above 0, and it is destroyed at 0. The remainder, not a second formula, is what `apply_resolved_blow` receives. Declining passes the full amount. A requested block that is not legal returns a typed error and writes nothing. A miss returns `NotAHit` before those checks.
- A build rejects a shield whose hardness is below 0, whose AC bonus is below 1, or whose broken threshold is not below its Hit Points, as `InvalidShield`.
- Shield Hit Points and the raised flag stay on `CombatantState`. The `Creature` keeps the maximum. No new team type. `Side` stays `Side::{Pc, Enemy}`.

Out of scope: shield bash, shield boss, shield spikes, tower-shield cover and Take Cover's +4, Repair, a speed penalty, hands and disarm, damage types, resistance, and changing the Strike degree's baked Armor Class. An Unconscious creature that already has a raised shield is deferred to `wire-dying-into-turns`. Feats and classes stay out of Phase 1. Two substitutes are proposed as DEV entries in design.md. This change does not edit `docs/designer/`.

## Capabilities

### New Capabilities

- `shields`: Raise a Shield, the circumstance bonus while raised, Shield Block on an attack blow, and shield Hit Points on the combatant.

### Modified Capabilities

- `encounter-turns`: `Strike damage uses resolved blows`. Shield Hit Points stay unchanged unless a legal Shield Block is requested. The header is the one on main.
- `creature-template`: a shield is validated at build as `InvalidShield`. The authored buckler of Hit Points 8 and broken threshold 4 is unchanged.

## Impact

New module `crates/rules/src/shield.rs`, re-exported from the crate root. It calls `total` and `apply_resolved_blow`. It does not declare `Side`, `OffGuard`, `DegreeOfSuccess`, or `HpError::NotAHit`. `Encounter::raise_shield` and `Encounter::shield_block` call `spend_actions` and `spend_reaction`. `Encounter::strike` gains `ShieldBlock` and maps a blow or shield error into `EncounterError`. This alters Strike behavior specified by `add-encounter-turns` and `add-dying-and-victory`: a legal Shield Block is the only Strike that changes shield Hit Points, and the degree still uses baked Armor Class. `crates/rules` gains no dependency. This pull request is the spec only: no changelog bullet and no archive. The implementation pull request adds the Unreleased changelog bullet, then archives as its last commit.
