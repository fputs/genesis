# Tasks

## 1. Dice

- [x] 1.1 Port `Dice`, `SeededDice`, and `ScriptedDice` from `v1-reference` `crates/rules/src/dice.rs` into `crates/rules/src/dice.rs`, including the xorshift and the seed-0 replacement. Drop v1's `assert!(faces >= 1)` on `SeededDice`. `SeededDice::roll` and every production path never panic; a request for 0 faces returns 1 and does not divide by zero. v1 has no dice unit test to port. Add `the_same_seed_repeats` and verify `cargo test -p rules the_same_seed_repeats` passes with two sources of seed 42 rolling d20, d8, and d6. Cite Player Core "Checks". The generator state is a field advanced through `&mut self`, not a `Cell`, `RefCell`, or `Mutex`
- [x] 1.2 Add `scripted_dice_panic_when_exhausted`, `scripted_dice_panic_out_of_range` (values 0 and 21 on a d20, and 6 on a d4), and `scripted_bounds_are_legal` (a scripted 1 then 20 on a d20 returns those faces). Verify the two panic tests with `#[should_panic]`. Document those panics in a `# Panics` section on `ScriptedDice`. Cite Player Core "Checks"
- [x] 1.3 Add a `test-support` feature to `crates/rules/Cargo.toml` with an empty dependency list. Compile the whole `ScriptedDice` type and its re-export only under `#[cfg(any(test, feature = "test-support"))]`. Do not put a `#[cfg(test)]` branch inside `roll`. Verify `cargo test -p rules` passes, `cargo check -p rules --features test-support` passes, and `rg "struct ScriptedDice" crates/rules/src` finds the type only under that cfg

## 2. Degrees, DCs, and flat checks

- [x] 2.1 Port v1's `Degree` enum as `DegreeOfSuccess`, and port `degree_of_success` from `v1-reference` `crates/rules/src/check.rs`. Make `improve` and `worsen` public. Port the test `degrees_follow_dc_bands_then_natural_20_and_1` from that file, including the eight assertions listed in `design.md`, with a comment citing Player Core "Step 4: Degree of Success" and "Natural 1 and Natural 20". Add `a_later_upgrade_sees_the_degree_after_the_natural_20` and `a_later_upgrade_does_not_see_the_degree_from_before_the_natural_1` for the DC 25 natural 20 and the DC 15 natural 1 cases, each with scripted dice. Verify `cargo test -p rules -- degrees_follow_dc_bands_then_natural_20_and_1` and `cargo test -p rules -- a_later_upgrade_sees_the_degree_after_the_natural_20` and `cargo test -p rules -- a_later_upgrade_does_not_see_the_degree_from_before_the_natural_1` pass
- [x] 2.2 Add `fallback_unlisted_critical` and `missing_critical_entry_falls_back`, using scripted naturals 15 and 5 against DC 15. Cite Player Core "Step 4: Degree of Success". Verify `cargo test -p rules missing_critical_entry_falls_back` passes
- [x] 2.3 Add `dc_from_modifier` and `dc_from_modifier_is_ten_plus` for +7, 0, and -3, plus a scripted natural 12 with Perception +5 against DC 17. Cite Player Core "Getting a DC from a Modifier". Verify `cargo test -p rules dc_from_modifier_is_ten_plus` passes
- [x] 2.4 Add `flat_check` and `flat_checks_for_same_thing`, and tests `flat_check_ignores_modifiers`, `flat_check_auto_success_and_failure`, and `flat_checks_for_the_same_thing_roll_once`, with the scripted values in the spec (10 beside an ignored +9, unused 4 at DC 1 and DC 0, unused 20 at DC 21, rolled 20 at DC 20, one roll of 10 against DC 15 leaving 20 unused, and separate rolls 10 and 4). Cite Player Core "Flat Checks". Verify `cargo test -p rules flat_check` passes

## 3. Multiple attack penalty

- [x] 3.1 Replace v1 `multiple_attack_penalty(prior_strikes: u8)` with `multiple_attack_penalty(MapInput)` and `attack_check_total`. Port `map_is_zero_then_minus_five_then_minus_ten` from `v1-reference` `crates/rules/src/check.rs` as the on-turn non-agile cases (prior 0, 1, 2, and 3 yield 0, -5, -10, -10). Extend that test for agile -4/-8, the current weapon, a reaction on another turn, and a non-Strike attack-trait check, using the scripted naturals in the spec. Cite Player Core "Multiple Attack Penalty". Verify `cargo test -p rules map_is_zero_then_minus_five_then_minus_ten` passes and the old one-argument signature is gone
- [x] 3.2 Add `map_stacks_with_a_circumstance_penalty` (scripted natural 12, attack +9, MAP -5, circumstance -2, DC 16, total 14, failure) and `map_stacks_with_another_untyped_penalty` (the same total with a second untyped -2). Do not port `prone_attack_penalty_does_not_stack_with_map` from `v1-reference` `crates/rules/tests/procedures.rs`. Cite Player Core "Penalties". Verify `cargo test -p rules map_stacks_with_` passes and `rg prone_attack_penalty_does_not_stack_with_map crates/rules` finds nothing
- [x] 3.3 Do not port `later_strikes_take_map_and_a_miss_does_not_roll_damage_dice` or `reactive_strike_ignores_map` from `v1-reference` `crates/rules/tests/procedures.rs`. Verify `rg` of those two names under `crates/rules` finds nothing

## 4. Crate surface

- [x] 4.1 Re-export `Dice`, `SeededDice`, `DegreeOfSuccess`, `CheckRoll`, `FlatRoll`, `MapInput`, and the public functions from `crates/rules/src/lib.rs`. Re-export `ScriptedDice` under the same `#[cfg(any(test, feature = "test-support"))]` gate as the type. Add `#![forbid(unsafe_code)]` to that file. Give every new public item a `///` comment, citing the Player Core section when the item states a rule. Verify `cargo test -p rules` passes and the ported tests still cite the Player Core section in a comment

## 5. One test per scenario

- [x] 5.1 Add one `#[cfg(test)]` test for every scenario below that sections 1–3 do not already name. The name is the scenario title in snake_case, with a leading minus written as `minus`. Each test comment cites the Player Core section on that requirement. A scenario that rolls uses `ScriptedDice` with that scenario's values. `the_same_seed_repeats` uses `SeededDice` with seed 42. A scenario that does not roll still builds a `ScriptedDice` and asserts it is unused. The ported v1 tests stay; they do not replace these tests. Satisfy each name once:
  - `the_same_seed_repeats`
  - `scripted_dice_panic_when_exhausted`
  - `scripted_dice_panic_out_of_range`
  - `scripted_bounds_are_legal`
  - `total_equal_to_the_dc_is_a_success`
  - `beating_the_dc_by_9_is_a_success`
  - `beating_the_dc_by_10_is_a_critical_success`
  - `missing_the_dc_by_9_is_a_failure`
  - `missing_the_dc_by_10_is_a_critical_failure`
  - `natural_20_turns_a_failure_into_a_success`
  - `natural_20_turns_a_success_into_a_critical_success`
  - `natural_20_does_not_raise_a_critical_success`
  - `natural_20_turns_a_critical_failure_into_a_failure`
  - `natural_1_turns_a_success_into_a_failure`
  - `natural_1_turns_a_critical_success_into_a_success`
  - `natural_1_does_not_lower_a_critical_failure`
  - `a_later_upgrade_sees_the_degree_after_the_natural_20`
  - `a_later_upgrade_does_not_see_the_degree_from_before_the_natural_1`
  - `no_critical_success_entry`
  - `no_critical_failure_entry`
  - `listed_critical_entries_stay`
  - `positive_modifier`
  - `zero_modifier`
  - `negative_modifier`
  - `a_check_uses_the_derived_dc`
  - `modifiers_are_ignored`
  - `dc_1_succeeds_without_rolling`
  - `dc_0_succeeds_without_rolling`
  - `dc_21_fails_without_rolling`
  - `dc_20_still_rolls`
  - `the_same_thing_rolls_once_at_the_highest_dc`
  - `different_things_roll_separately`
  - `the_first_check_on_your_turn_has_no_penalty`
  - `the_second_non_agile_check_takes_minus_5`
  - `the_third_and_later_non_agile_checks_take_minus_10`
  - `a_fourth_non_agile_check_stays_at_minus_10`
  - `the_second_agile_check_takes_minus_4`
  - `the_third_agile_check_takes_minus_8`
  - `the_current_weapon_sets_the_penalty`
  - `a_non_agile_weapon_after_an_agile_weapon_uses_minus_5`
  - `a_reaction_on_another_turn_takes_no_penalty_and_does_not_count`
  - `an_attack_trait_check_that_is_not_a_strike_still_counts`
  - `the_penalty_stacks_with_a_circumstance_penalty`
  - `the_penalty_stacks_with_another_untyped_penalty`

## 6. Proof and archive

- [x] 6.1 Run the AGENTS.md proof commands and verify each exits 0. Add no crate dependency. The only `Cargo.toml` change is the empty `test-support` feature.
  - `cargo fmt --all --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo build --workspace`
  - `cargo test --workspace`
  - `npx @fission-ai/openspec@1.14.3 validate --all --strict`
- [x] 6.2 Archive this change with `openspec archive add-dice-and-checks --yes` and verify it moves under `openspec/changes/archive/`
