# Proposal

## Why

A shield on the creature is only a stored maximum. The fight cannot raise it, add its circumstance bonus, or spend a reaction to block. This change is that procedure. `apply_resolved_blow` stays the only creature-damage formula.

## What Changes

- Add the `shields` capability: Raise a Shield, the raised shield's circumstance bonus, Shield Block, and the combatant's shield Hit Points and raised flag.
- Raise a Shield costs 1 action and sets the shield raised. The bonus is that shield's own listed circumstance bonus to Armor Class (a buckler is +1, a steel shield is +2), folded through the existing modifier total, until the start of the defender's next turn. Ending the turn does not lower it. Another creature's turn does not lower it.
- Shield Block is a reaction. The trigger is an attack blow while the shield is raised. Hardness is subtracted once from that blow. The defender and the shield each take the remainder. The shield breaks when its Hit Points are at or below its Broken Threshold and above 0, and it is destroyed at 0. The remainder, not a second formula, is what `apply_resolved_blow` receives. A miss, a declined block, or a block that is not legal leaves shield Hit Points unchanged and still uses that function for a hit.
- Shield Hit Points and the raised flag stay on `CombatantState`. The `Creature` keeps the maximum. No new team type. `Side` stays `Side::{Pc, Enemy}`.

Out of scope: shield bash, shield boss, shield spikes, tower-shield cover, Repair, a speed penalty, hands and disarm, damage types, resistance, and changing the Strike degree's baked Armor Class. Feats and classes stay out of Phase 1. Two substitutes are proposed as DEV entries in design.md. This change does not edit `docs/designer/`.

## Capabilities

### New Capabilities

- `shields`: Raise a Shield, the circumstance bonus while raised, Shield Block on an attack blow, and shield Hit Points on the combatant.

### Modified Capabilities

None. No requirement in `openspec/specs/` changes. `add-encounter-turns`, `add-positioning`, and `add-dying-and-victory` keep the requirement headers they already have on main.

## Impact

New module `crates/rules/src/shield.rs`, re-exported from the crate root. It calls `total` and `apply_resolved_blow`. It does not declare `Side`, `DegreeOfSuccess`, or `HpError::NotAHit`. `crates/rules` gains no dependency. This pull request is the spec only: no changelog bullet and no archive. The implementation pull request adds the Unreleased changelog bullet, then archives as its last commit.
