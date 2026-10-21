# Proposal

## Why

Later actions need baked creature numbers, not a derived character sheet. Templates on `v1-reference` have no agile flag, no per-shield AC bonus, and no Aid modifier, and every creature that reaches 0 Hit Points enters the dying track. This change specifies that plain data, including `uses_dying_rules` defaulting to false so the creature dies at 0 Hit Points unless it uses the dying rules. The dying track itself stays in encounter-core.

## What Changes

- Bake Hit Points, Armor Class, Perception modifier, and Speed.
- Bake exactly one melee strike: attack bonus, damage dice, die faces, damage bonus, reach, and an agile flag.
- Bake an optional shield with hardness, Hit Points, broken threshold, and that shield's own AC bonus. Do not rewrite the bonus to a flat +2.
- Bake an optional `stealth_dc` of 10 plus the Stealth modifier, for Seek. Omit it and the creature has no Stealth DC.
- Bake an optional `aid_modifier` used for an Aid check against the typical DC 15. Critical success grants +2 circumstance, a recorded deviation from the master and legendary values. Do not store Aid's prepared-to-help state; that belongs to the later Aid change.
- Keep templates as plain data. A roster has no fixed party size. Identity is `CombatantId(u32)`.
- Reject Hit Points of 0 or less and a negative Speed.
- Add `uses_dying_rules: bool`, default false. At 0 Hit Points the creature is dead and does not gain dying. With the flag set, it gains dying as encounter-core specifies, and this change does not specify that track.
- Do not write `add-modifiers` here. That separate change specifies circumstance, status, and untyped stacking, and it comes next, before encounter-core.

## Capabilities

### New Capabilities

- `creature-template`: Baked creature statistics, one melee strike, and the choice between death at 0 Hit Points and the dying rules.

### Modified Capabilities

None.

## Impact

New module `crates/rules/src/combatant.rs`, re-exported from `crates/rules/src/lib.rs`. Port the baked-number idea and `preparation_copies_fixture_numbers` from `v1-reference` `crates/rules/src/combatant.rs` and `crates/rules/tests/procedures.rs`, and port the `CombatantId(u32)` newtype from `v1-reference` `crates/rules/src/encounter.rs`. Do not port `Side`, class features, `shield_block`, the synthetics pipeline, or the dying-track tests. Aid checks use the degree helper from `dice-and-checks`, so apply that change first. `add-modifiers` is a later change and is not part of this one.
