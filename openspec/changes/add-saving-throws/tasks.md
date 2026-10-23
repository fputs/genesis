# Tasks

Each scenario has one test. The name is the title in snake_case. Spaces, hyphens, and apostrophes become `_`. Other punctuation is dropped. A title that says `minus 4` or `plus 7` keeps those words. Use `ScriptedDice`, and assert an unused face when the scenario says so. Cite that requirement's Player Core section. Do not edit `docs/designer/`. No file on `v1-reference` is named, so none is ported.

Write section 2 before filling the production bodies. The tests call the items in design.md. They may share a commit with section 3. Do not start section 2 until task 1.2 is true.

## 1. Spec

- [ ] 1.1 These files are the behavior. The spec commit adds no Rust, no test, and no changelog bullet. Verify `npx @fission-ai/openspec@1.14.3 validate --all --strict` exits 0

- [ ] 1.2 Do not start section 2 until this pull request has Evan's comment `spec-approved: <sha>` and that sha is an ancestor of HEAD. Designer rules on the spec in this pull request

## 2. Tests

- [ ] 2.1 Add the degree tests `total_equal_to_the_dc_is_a_success`, `beating_the_dc_by_9_is_a_success`, `beating_the_dc_by_10_is_a_critical_success`, `missing_the_dc_by_9_is_a_failure`, and `missing_the_dc_by_10_is_a_critical_failure`. Verify `cargo test -p rules -- total_equal_to_the_dc_is_a_success beating_the_dc_by_9_is_a_success beating_the_dc_by_10_is_a_critical_success missing_the_dc_by_9_is_a_failure missing_the_dc_by_10_is_a_critical_failure` passes

- [ ] 2.2 Add the natural-20 tests `natural_20_turns_a_failure_into_a_success`, `natural_20_turns_a_success_into_a_critical_success`, `natural_20_does_not_raise_a_critical_success`, and `natural_20_turns_a_critical_failure_into_a_failure`. Verify `cargo test -p rules -- natural_20_turns_a_failure_into_a_success natural_20_turns_a_success_into_a_critical_success natural_20_does_not_raise_a_critical_success natural_20_turns_a_critical_failure_into_a_failure` passes

- [ ] 2.3 Add the natural-1 tests `natural_1_turns_a_success_into_a_failure`, `natural_1_turns_a_critical_success_into_a_success`, and `natural_1_does_not_lower_a_critical_failure`. Cite Player Core "Natural 1 and Natural 20". Verify `cargo test -p rules -- natural_1_turns_a_success_into_a_failure natural_1_turns_a_critical_success_into_a_success natural_1_does_not_lower_a_critical_failure` passes

- [ ] 2.4 Add `critical_success_deals_no_damage`, `success_halves_5_to_2`, `success_halves_7_to_3`, `success_on_1_damage_stays_1`, and `success_on_0_damage_stays_0`. Cite Player Core "Basic Saving Throws" and "Doubling and Halving Damage". Verify `cargo test -p rules -- critical_success_deals_no_damage success_halves_5_to_2 success_halves_7_to_3 success_on_1_damage_stays_1 success_on_0_damage_stays_0` passes

- [ ] 2.5 Add `failure_deals_the_full_amount`, `critical_failure_doubles_once`, `critical_success_on_1_damage_deals_0`, `critical_failure_on_0_damage_stays_0`, and `a_negative_amount_rolls_nothing`. Cite Player Core "Basic Saving Throws". Verify `cargo test -p rules -- failure_deals_the_full_amount critical_failure_doubles_once critical_success_on_1_damage_deals_0 critical_failure_on_0_damage_stays_0 a_negative_amount_rolls_nothing` passes

- [ ] 2.6 Add `fortitude_is_constitution_plus_trained_at_level_1`, `reflex_is_dexterity_plus_expert_at_level_5`, `will_is_wisdom_plus_legendary_at_level_20`, `untrained_is_0_at_level_20`, and `trained_at_level_2_with_wisdom_0_is_plus_4`. Verify `cargo test -p rules -- fortitude_is_constitution_plus_trained_at_level_1 reflex_is_dexterity_plus_expert_at_level_5 will_is_wisdom_plus_legendary_at_level_20 untrained_is_0_at_level_20 trained_at_level_2_with_wisdom_0_is_plus_4` passes

- [ ] 2.7 Add `an_item_bonus_adds_beside_proficiency`, `two_circumstance_bonuses_keep_the_highest`, `an_attack_penalty_does_not_change_fortitude`, `fortitude_and_will_do_not_share_a_bonus`, and `the_same_modifier_twice_matches`. Verify `cargo test -p rules -- an_item_bonus_adds_beside_proficiency two_circumstance_bonuses_keep_the_highest an_attack_penalty_does_not_change_fortitude fortitude_and_will_do_not_share_a_bonus the_same_modifier_twice_matches` passes

- [ ] 2.8 Add `unconscious_reflex_includes_minus_4`, `alive_reflex_does_not_include_minus_4`, `a_dying_creature_s_reflex_includes_minus_4`, `unconscious_fortitude_does_not_include_minus_4`, and `unconscious_will_does_not_include_minus_4`. Verify `cargo test -p rules -- unconscious_reflex_includes_minus_4 alive_reflex_does_not_include_minus_4 a_dying_creature_s_reflex_includes_minus_4 unconscious_fortitude_does_not_include_minus_4 unconscious_will_does_not_include_minus_4` passes

- [ ] 2.9 Add `a_modifier_of_plus_7_has_dc_17`, `trained_at_the_maximum_level_rolls_nothing`, `a_constitution_modifier_at_the_maximum_rolls_nothing`, and `doubling_the_maximum_damage_writes_nothing`. Cite Player Core "Saving Throws" and "Doubling and Halving Damage". Verify `cargo test -p rules -- a_modifier_of_plus_7_has_dc_17 trained_at_the_maximum_level_rolls_nothing a_constitution_modifier_at_the_maximum_rolls_nothing doubling_the_maximum_damage_writes_nothing` passes

- [ ] 2.10 Add `failure_to_0_sets_dying_1`, `critical_failure_to_0_sets_dying_2`, `failure_while_dying_1_increases_to_dying_2`, `critical_failure_while_dying_1_increases_by_2`, and `without_dying_rules_0_hit_points_is_death`. Verify `cargo test -p rules -- failure_to_0_sets_dying_1 critical_failure_to_0_sets_dying_2 failure_while_dying_1_increases_to_dying_2 critical_failure_while_dying_1_increases_by_2 without_dying_rules_0_hit_points_is_death` passes

- [ ] 2.11 Add `a_raised_shield_does_not_block`, `a_damage_modifier_does_not_change_the_amount`, `the_same_save_twice_matches`, `an_unknown_defender_rolls_nothing`, `a_dead_defender_rolls_nothing`, and `no_save_training_rolls_nothing`. Verify `cargo test -p rules -- a_raised_shield_does_not_block a_damage_modifier_does_not_change_the_amount the_same_save_twice_matches an_unknown_defender_rolls_nothing a_dead_defender_rolls_nothing no_save_training_rolls_nothing` passes

- [ ] 2.12 Add `a_creature_may_save_against_itself`, `a_delayed_creature_can_save`, `a_delayed_knockout_is_placed_before_the_open_creature`, and `a_knockout_that_is_not_delayed_leaves_the_order`. Cite Player Core "Saving Throws", "Delay", and "Getting Knocked Out". Verify `cargo test -p rules -- a_creature_may_save_against_itself a_delayed_creature_can_save a_delayed_knockout_is_placed_before_the_open_creature a_knockout_that_is_not_delayed_leaves_the_order` passes

- [ ] 2.13 Add `authored_armor_class_stays_authored`, `omitted_training_stays_omitted`, `the_same_build_twice_matches`, and `a_later_contribution_replaces_training` beside the creature tests. Cite Player Core "Saving Throws". Verify `cargo test -p rules -- authored_armor_class_stays_authored omitted_training_stays_omitted the_same_build_twice_matches a_later_contribution_replaces_training` passes

- [ ] 2.14 Add `fortitude_and_will_do_not_share_a_stack`, `unconscious_adds_nothing_to_fortitude`, `unconscious_adds_nothing_to_will`, and `reflex_keeps_the_unconscious_penalty` beside the modifier tests. Cite Player Core "Saving Throws" and "Unconscious". Verify `cargo test -p rules -- fortitude_and_will_do_not_share_a_stack unconscious_adds_nothing_to_fortitude unconscious_adds_nothing_to_will reflex_keeps_the_unconscious_penalty` passes

- [ ] 2.15 Add `a_save_after_a_strike_is_not_a_strike`, `a_save_draws_no_flat_check_die`, `a_save_before_any_turn_rolls`, and `a_save_off_the_active_turn_rolls`. Cite Player Core "Saving Throws" and "Strike". Verify `cargo test -p rules -- a_save_after_a_strike_is_not_a_strike a_save_draws_no_flat_check_die a_save_before_any_turn_rolls a_save_off_the_active_turn_rolls` passes

## 3. Code

- [ ] 3.1 Add `crates/rules/src/save.rs` and re-export its public items from `lib.rs`. Implement `save_modifier`, `save_dc`, and `basic_save_amount` as design.md states, including half of 1 staying 1. Keep `#![forbid(unsafe_code)]`. Verify the tests from 2.1 through 2.7 pass and `crates/rules/Cargo.toml` has no dependencies

- [ ] 3.2 Add `saves: Option<SavingThrows>` to `BuildParts`, `Contribution`, and `Creature`. `derive` copies it and does not roll. Folding replaces it when `Some`. Set `saves: None` on every existing literal. Verify the tests from 2.13 pass

- [ ] 3.3 Add `Statistic::Fortitude` and `Statistic::Will` after Reflex. Add empty `fortitude_modifiers` and `will_modifiers`. Leave `reflex_modifiers` as status -4 at sequence 10. Verify the tests from 2.14 pass and the old Reflex unconscious tests still pass

- [ ] 3.4 Add `Encounter::basic_save` and `EncounterError::NoSavingThrows` and `SaveOverflow`, in design.md's order. Translate the degree before `apply_resolved_blow`. Do not call `strike`, `damage_dealt`, `shield_block`, or `resolve_targeting`. Call `place_delayed_knockout` only while a turn is open. Verify the tests from 2.8 through 2.12 and 2.15 pass

## 4. Changelog and proof

- [ ] 4.1 Add one Added bullet for saving throws and one **BREAKING** Changed bullet for `saves` and the two statistics, both under `## [Unreleased]`, both linking this pull request. Commit that before the archive commit. Do not add either bullet in the spec commit. Verify both bullets contain the pull request link

- [ ] 4.2 Run the AGENTS.md proof commands and verify each exits 0. Confirm `crates/rules/Cargo.toml` still has no dependencies
  - `cargo fmt --all --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo build --workspace`
  - `cargo test --workspace`
  - `npx @fission-ai/openspec@1.14.3 validate --all --strict`

## 5. Archive

- [ ] 5.1 This is the last commit, right before merge, and only after Architect and QA have both passed that head. Run `openspec archive add-saving-throws --yes` with that exact commit subject. Check this box only in that commit. Do not archive the spec draft. Verify the change is under `openspec/changes/archive/` and is no longer active, and that the archive commit does not edit `CHANGELOG.md`
