# Proposal

## Why

Phase 1 checks add penalties as loose numbers, and a negative damage total is clamped to 0. Player Core keeps the highest bonus and the worst penalty of one type, adds every untyped penalty, and still deals 1 when penalties would reduce damage to 0 or below. A folded creature can also currently skip the Hit Point and Speed checks that template validation already applies. This change specifies that folding, and it makes the public fold obey the same checks.

## What Changes

- Add a `modifiers` capability. Bonus types are circumstance, item, and status. Penalty types are circumstance, item, status, and untyped. Proficiency is excluded because AC, attack, Perception, and DCs are baked (DEV-01). The item type exists and is tested; Phase 1 produces none.
- Same type: keep the highest bonus and the worst penalty. When different sources tie, the lowest sequence number is the one reported as applied. A bonus and a penalty of the same type both apply. Untyped penalties all add, including the existing multiple attack penalty and a caller-supplied range penalty. A sum that overflows is a typed error and does not saturate.
- A bonus value is greater than 0 and a penalty value is less than 0. Any other value, including zero and an untyped bonus, is a typed construction error.
- Each modifier carries a source id and a sequence number. One source contributes once: the higher level or rank, otherwise the newer sequence, meaning the greater one. Value does not break that tie. A newer untyped -5 beats an older untyped -2 from the same source. This happens before the sum. A repeated condition keeps its highest rank before that sum.
- Totals and DCs are not clamped and may be negative. A DC is 10 plus the total. Flat checks take no modifier total. Attack rolls, Armor Class, Perception, and damage keep separate collections and do not compete.
- Phase 1 applies the existing multiple attack penalty (including agile), prone -2 circumstance, and an Aid circumstance modifier on attack rolls; cover +1/+2/+4 circumstance, off-guard -2 circumstance, and unconscious -4 status on Armor Class; unconscious -4 status on Perception; and the same stacking on damage. The Aid action is out of scope. Shield bonuses stay in the shields change. This change does not define a second multiple attack penalty.
- On `creature-template`, folding contributions uses the same maximum Hit Points and Speed checks as template validation and returns a `FoldError` instead of an invalid Creature. Speed 0 stays valid. A Speed that is not a multiple of 5 is not checked here; DEV-21 moves to add-grid-movement. Current Hit Points on the damage and heal paths stay from 0 through the maximum. Healing cannot raise current Hit Points above the maximum (Rules 2320, Player Core pg. 410).
- On `creature-template`, when penalties on an attack reduce a damage total to 0 or below, deal 1, and apply that floor before critical doubling, so the smallest critical hit deals 2. RAW (Rules 2302, Player Core pg. 406, and Rules 2307, Player Core pg. 407) does not state that order; DEV-14 is our ruling. This replaces the clamp-to-0 behavior in `apply_dice_damage`. That behavior is code, not a named requirement, so the delta adds the requirement. Resistance has a separate floor of 0 (Rules 2318) and stays out of scope.
- Fallible operations return `Result`. Arithmetic is checked. Overflow while summing is a typed error and never saturates. No new dependency.

## Capabilities

### New Capabilities

- `modifiers`: Typed bonuses and penalties, same-type stacking, untyped sums, duplicate-effect and repeated-condition reduction, and the Phase 1 totals for attack rolls, Armor Class, Perception, and damage.

### Modified Capabilities

- `creature-template`: Validation of maximum Hit Points and Speed also applies to the public fold. Speed stays greater than or equal to 0. Penalties on an attack that reduce damage to 0 or below deal 1 before a critical hit doubles it (DEV-14).

## Impact

- New module `crates/rules/src/modifiers.rs`, re-exported from `crates/rules/src/lib.rs`. No new crate and no new dependency. `crates/rules` stays free of Bevy, serde, files, network, clock, and global randomness.
- `attack_check_total` keeps its signature and folds modifiers, so the existing multiple-attack-penalty scenarios still pass. The -5/-10 and agile -4/-8 table stays in `multiple_attack_penalty`.
- `fold_contributions` gains a `FoldError` variant for a creature `try_new` would reject. `derive` of a valid build is unchanged.
- `apply_dice_damage` returns a `Result`. A negative modifier total deals 1 instead of 0. Overflow leaves Hit Points unchanged and returns a typed error. A heal path caps current Hit Points at the maximum.
- The implementation pull request adds an Unreleased changelog bullet. This pull request is the spec only.
