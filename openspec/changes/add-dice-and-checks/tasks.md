# Tasks

## 1. Dice

- [ ] 1.1 Port `Dice`, `SeededDice`, and `ScriptedDice` from `v1-reference` `crates/rules/src/dice.rs` into `crates/rules/src/dice.rs`, including the xorshift and the seed-0 replacement. v1 has no dice unit test to port. Add `seeded_dice_repeats_the_sequence_for_a_seed` and verify `cargo test -p rules seeded_dice_repeats_the_sequence_for_a_seed` passes with two sources of seed 42 rolling d20, d8, and d6
- [ ] 1.2 Add `scripted_dice_panics_when_exhausted` and `scripted_dice_panics_when_out_of_range` (values 0 and 21 on a d20, and 6 on a d4). Verify both `#[should_panic]` tests pass, and a scripted 1 then 20 on a d20 returns those faces

## 2. Degrees, DCs, and flat checks

- [ ] 2.1 Port `Degree` and `degree_of_success` from `v1-reference` `crates/rules/src/check.rs`, and make `improve` and `worsen` public. Port the test `degrees_follow_dc_bands_then_natural_20_and_1` from that file, including the eight assertions listed in `design.md`, with a comment citing Player Core "Step 4: Degree of Success" and "Natural 1 and Natural 20". Add `natural_20_is_applied_before_a_later_upgrade` for the DC 25 natural 20 and the DC 15 natural 1 cases. Verify `cargo test -p rules -- degrees_follow_dc_bands_then_natural_20_and_1` and `cargo test -p rules -- natural_20_is_applied_before_a_later_upgrade` both pass
- [ ] 2.2 Add `fallback_unlisted_critical` and `missing_critical_entry_falls_back`, using scripted naturals 15 and 5 against DC 15. Cite Player Core "Step 4: Degree of Success". Verify `cargo test -p rules missing_critical_entry_falls_back` passes
- [ ] 2.3 Add `dc_from_modifier` and `dc_from_modifier_is_ten_plus` for +7, 0, and -3, plus a scripted natural 12 with Perception +5 against DC 17. Cite Player Core "Getting a DC from a Modifier". Verify `cargo test -p rules dc_from_modifier_is_ten_plus` passes
- [ ] 2.4 Add `flat_check` and `flat_checks_for_same_thing`, and tests `flat_check_ignores_modifiers`, `flat_check_auto_success_and_failure`, and `flat_checks_for_the_same_thing_roll_once`, with the scripted values in the spec (10 beside an ignored +9, unused 4 at DC 1 and DC 0, unused 20 at DC 21, rolled 20 at DC 20, one roll of 10 against DC 15 leaving 20 unused, and separate rolls 10 and 4). Cite Player Core "Flat Checks". Verify `cargo test -p rules flat_check` passes

## 3. Multiple attack penalty

- [ ] 3.1 Replace v1 `multiple_attack_penalty(prior_strikes: u8)` with `multiple_attack_penalty(MapInput)` and `attack_check_total`. Port `map_is_zero_then_minus_five_then_minus_ten` from `v1-reference` `crates/rules/src/check.rs` as the on-turn non-agile cases (prior 0, 1, 2, and 3 yield 0, -5, -10, -10). Extend that test for agile -4/-8, the current weapon, a reaction on another turn, and a non-Strike attack-trait check, using the scripted naturals in the spec. Cite Player Core "Multiple Attack Penalty". Verify `cargo test -p rules map_is_zero_then_minus_five_then_minus_ten` passes and the old one-argument signature is gone
- [ ] 3.2 Add `map_stacks_with_a_circumstance_penalty` (scripted natural 12, attack +9, MAP -5, circumstance -2, DC 16, total 14, failure) and `map_stacks_with_another_untyped_penalty` (the same total with a second untyped -2). Do not port `prone_attack_penalty_does_not_stack_with_map` from `v1-reference` `crates/rules/tests/procedures.rs`. Cite Player Core "Penalties". Verify `cargo test -p rules map_stacks_with_` passes and `rg prone_attack_penalty_does_not_stack_with_map crates/rules` finds nothing
- [ ] 3.3 Do not port `later_strikes_take_map_and_a_miss_does_not_roll_damage_dice` or `reactive_strike_ignores_map` from `v1-reference` `crates/rules/tests/procedures.rs`. Verify `rg` of those two names under `crates/rules` finds nothing

## 4. Crate surface

- [ ] 4.1 Re-export `Dice`, `SeededDice`, `ScriptedDice`, `Degree`, `CheckRoll`, `FlatRoll`, `MapInput`, and the public functions from `crates/rules/src/lib.rs`. Verify `cargo test -p rules` passes and the ported tests still cite the Player Core section in a comment
