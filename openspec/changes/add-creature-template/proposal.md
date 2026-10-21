# Proposal

## Why

The same character inputs must always produce the same creature, and combat must not write those inputs. Templates on `v1-reference` bake numbers and then mix them with encounter status, and every creature at 0 Hit Points enters dying. This change specifies an immutable CharacterBuild, a pure derive to a read-only Creature, and a separate CombatantState. uses_dying_rules defaults to false. The dying track stays in encounter-core.

## What Changes

- Split the character into an immutable `CharacterBuild` and a pure `derive(&CharacterBuild) -> Creature`. Phase 1 inputs are the baked stat block plus stealth DC, Aid modifier, shield, and `uses_dying_rules`. Later phases add ancestry, class, feat, attribute, and proficiency as further sources without changing `derive`'s callers.
- Fold an ordered list of contributions. Phase 1 has only the base contribution.
- `derive` does not roll Dice, do I/O, or read global state. The same build, a clone, or any equal build always yields an equal Creature.
- Keep runtime state on `CombatantState`, keyed by `CombatantId`: current Hit Points, temporary Hit Points, conditions, dying, wounded, shield Hit Points and raised, actions, and the reaction. Initialize it from a Creature. Never write it back into the build or the Creature.
- Carry one melee strike and an optional shield whose AC bonus is that shield's own value, not a flat +2.
- Use `aid_modifier` for an Aid check against DC 15. Critical success stays +2, a recorded deviation. Do not store Aid's prepared state.
- Reject Hit Points of 0 or less and a negative Speed.
- Default `uses_dying_rules` to false. At 0 current Hit Points the state is dead. With the flag set, the state enters dying and this change does not specify that track.
- Do not write `add-modifiers` here. That separate change specifies circumstance, status, and untyped stacking, and it comes next, before encounter-core.

## Capabilities

### New Capabilities

- `creature-template`: Immutable character build, pure derivation of a creature, and separate combatant state.

### Modified Capabilities

None.

## Impact

New module `crates/rules/src/combatant.rs`, re-exported from `crates/rules/src/lib.rs`. `CharacterBuild` and `derive` replace v1's `CreatureTemplate` plus `prepare_combatant`. Port the baked-number idea and `preparation_copies_fixture_numbers` from `v1-reference` `crates/rules/src/combatant.rs` and `crates/rules/tests/procedures.rs`, and port the `CombatantId(u32)` newtype from `v1-reference` `crates/rules/src/encounter.rs` onto `CombatantState` only. Do not port `Side`, class features, `shield_block`, the synthetics pipeline, or the dying-track tests. Aid checks use the degree helper from `dice-and-checks`; `derive` itself does not. Apply dice-and-checks first. `add-modifiers` is a later change and is not part of this one.
