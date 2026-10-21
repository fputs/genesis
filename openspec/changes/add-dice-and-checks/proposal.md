# Proposal

## Why

Phase 1 needs one injected die source and one degree procedure before any later action can roll. On `v1-reference`, `roll_attack` in `crates/rules/src/encounter.rs` treats the multiple attack penalty as a circumstance penalty and keeps only the worse of that penalty and any other circumstance penalty. Player Core lists the multiple attack penalty as untyped, so the numbers add. This change specifies injected dice, check degrees, flat checks, and that untyped penalty.

## What Changes

- Add a caller-injected `Dice` source: `SeededDice` (same seed, same sequence) and `ScriptedDice` (panics when exhausted or out of range).
- Resolve a d20 plus modifiers against a DC. Meeting the DC is a success, beating it by 10 is a critical success, and missing it by 10 is a critical failure. A natural 20 or natural 1 shifts one step and is applied before any other degree change. An effect with no critical entry treats that degree as the ordinary success or failure.
- Derive a DC from a modifier as 10 plus the modifier.
- Resolve flat checks with no modifiers. DC 1 or lower succeeds without a roll. DC 21 or higher fails without a roll. Several flat checks for the same thing roll once, at the highest DC.
- Define the multiple attack penalty as an untyped penalty: -5/-10, agile -4/-8 from the weapon used for the current check, only on the creature's own turn, on every attack-trait check. It adds to other untyped penalties and to a circumstance penalty. It is not a circumstance penalty.
- Do not write `add-modifiers` here. That separate change specifies circumstance, status, and untyped stacking, and it comes next, before encounter-core. This change only proves the multiple attack penalty is added in full beside a circumstance penalty.

## Capabilities

### New Capabilities

- `dice-and-checks`: Injected dice, d20 checks, flat checks, and the untyped multiple attack penalty.

### Modified Capabilities

None.

## Impact

New modules `crates/rules/src/dice.rs` and `crates/rules/src/check.rs`, re-exported from `crates/rules/src/lib.rs`. Port `crates/rules/src/dice.rs` and the degree helper in `crates/rules/src/check.rs` from `v1-reference`. Do not port the circumstance fold in `v1-reference` `crates/rules/src/encounter.rs` (`roll_attack`) or the test `prone_attack_penalty_does_not_stack_with_map` in `crates/rules/tests/procedures.rs`. No game or editor crates. `add-modifiers` is a later change and is not part of this one.
