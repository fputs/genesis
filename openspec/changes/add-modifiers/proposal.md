# Proposal

## Why

Phase 1 checks add penalties as loose numbers, and a negative damage total is clamped to 0. Player Core keeps the highest bonus and the worst penalty of one type, adds every untyped penalty, and still deals 1 when penalties would reduce damage to 0 or below. A folded creature can also currently skip the Hit Point and Speed checks that template validation already applies. This change specifies that folding, and it makes the public fold obey the same checks.

## What Changes

- Add a `modifiers` capability. Bonus types are circumstance, item, and status. Penalty types are circumstance, item, status, and untyped. Proficiency is out of scope. The item type exists and is tested; Phase 1 produces none.
- Same type: keep the highest bonus and the worst penalty. A bonus and a penalty of the same type both apply. Untyped penalties all add, including the existing multiple attack penalty and a caller-supplied range penalty.
- Reject an untyped bonus and a zero value at construction, each with a typed error.
- Each modifier carries a source id and a sequence number. One source contributes once: higher level, otherwise higher value, otherwise the newer sequence. This happens before the sum. A repeated condition keeps its highest rank before that sum.
- Totals and DCs are not clamped and may be negative. A DC is 10 plus the total. Flat checks take no modifier total. Attack rolls, Armor Class, Perception, and damage keep separate collections and do not compete.
- Phase 1 applies the existing multiple attack penalty (including agile), prone -2 circumstance, and Aid circumstance on attack rolls; cover +1/+2/+4 circumstance, off-guard -2 circumstance, and unconscious -4 status on Armor Class; unconscious -4 status on Perception; and the same stacking on damage. Shield bonuses stay in the shields change. This change does not define a second multiple attack penalty.
- On `creature-template`, folding contributions uses the same maximum Hit Points and Speed checks as template validation and returns a `FoldError` instead of an invalid Creature. Current Hit Points on the damage and heal paths stay from 0 through the maximum.
- On `creature-template`, when modifiers or penalties reduce a damage total to 0 or below, deal 1, and apply that floor before critical doubling, so the smallest critical hit deals 2. This replaces the clamp-to-0 behavior in `apply_dice_damage`. That behavior is code, not a named requirement, so the delta adds the requirement. Resistance that reduces damage to 0 stays out of scope.
- Fallible operations return `Result`. Arithmetic is checked, and overflow is a typed error. No new dependency.

## Capabilities

### New Capabilities

- `modifiers`: Typed bonuses and penalties, same-type stacking, untyped sums, duplicate-effect and repeated-condition reduction, and the Phase 1 totals for attack rolls, Armor Class, Perception, and damage.

### Modified Capabilities

- `creature-template`: Validation of maximum Hit Points and Speed also applies to the public fold. A non-positive damage total after modifiers or penalties deals 1 before a critical hit doubles it.

## Impact

- New module `crates/rules/src/modifiers.rs`, re-exported from `crates/rules/src/lib.rs`. No new crate and no new dependency. `crates/rules` stays free of Bevy, serde, files, network, clock, and global randomness.
- `attack_check_total` keeps its signature and folds modifiers, so the existing multiple-attack-penalty scenarios still pass. The -5/-10 and agile -4/-8 table stays in `multiple_attack_penalty`.
- `fold_contributions` gains a `FoldError` variant for a creature `try_new` would reject. `derive` of a valid build is unchanged.
- `apply_dice_damage` returns a `Result`. A negative modifier total deals 1 instead of 0. Overflow leaves Hit Points unchanged and returns a typed error. A heal path caps current Hit Points at the maximum.
- The implementation pull request adds an Unreleased changelog bullet. This pull request is the spec only.
