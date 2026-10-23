# Tasks

Each scenario has one test. The name is the title in snake_case. Spaces, hyphens, and apostrophes become `_`. Other punctuation is dropped. A title that says `minus 4` or `plus 7` keeps those words. Use `ScriptedDice`, and assert an unused face when the scenario says so. Cite that requirement's Player Core section. The spec commit does not edit `docs/designer/`. Task 3.5 does, in the code phase. No file on `v1-reference` is named, so none is ported.

Write section 2 before filling the production bodies. The tests call the items in design.md. They may share a commit with section 3. Do not start section 2 until task 1.2 is true.

## 1. Spec

- [x] 1.1 These files are the behavior. The spec commit adds no Rust, no test, and no changelog bullet. Verify `npx @fission-ai/openspec@1.14.3 validate --all --strict` exits 0

- [x] 1.2 Do not start section 2 until this pull request has Evan's comment `spec-approved: <sha>` and that sha is an ancestor of HEAD. Designer rules on the spec in this pull request

## 2. Tests

- [x] 2.1 Add `total_dc_minus_1_is_a_failure`, `natural_1_at_a_total_equal_to_the_dc_is_a_failure`, and `natural_20_at_dc_minus_11_is_a_failure`. Verify `cargo test -p rules -- total_dc_minus_1_is_a_failure natural_1_at_a_total_equal_to_the_dc_is_a_failure natural_20_at_dc_minus_11_is_a_failure` passes

- [x] 2.2 Add `critical_success_deals_no_damage`, `success_halves_5_to_2`, `success_halves_7_to_3`, `success_on_1_damage_stays_1`, and `success_on_0_damage_stays_0`. Verify `cargo test -p rules -- critical_success_deals_no_damage success_halves_5_to_2 success_halves_7_to_3 success_on_1_damage_stays_1 success_on_0_damage_stays_0` passes

- [x] 2.3 Add `failure_deals_the_full_amount`, `critical_failure_doubles_once`, `critical_success_on_1_damage_deals_0`, `critical_failure_on_0_damage_stays_0`, and `a_negative_amount_rolls_nothing`. Verify `cargo test -p rules -- failure_deals_the_full_amount critical_failure_doubles_once critical_success_on_1_damage_deals_0 critical_failure_on_0_damage_stays_0 a_negative_amount_rolls_nothing` passes

- [x] 2.4 Add `fortitude_is_constitution_plus_trained_at_level_1`, `reflex_is_dexterity_plus_expert_at_level_5`, `will_is_wisdom_plus_legendary_at_level_20`, `untrained_is_0_at_level_20`, and `trained_at_level_2_with_wisdom_0_is_plus_4`. Verify `cargo test -p rules -- fortitude_is_constitution_plus_trained_at_level_1 reflex_is_dexterity_plus_expert_at_level_5 will_is_wisdom_plus_legendary_at_level_20 untrained_is_0_at_level_20 trained_at_level_2_with_wisdom_0_is_plus_4` passes

- [x] 2.5 Add `an_item_bonus_adds_beside_proficiency`, `two_circumstance_bonuses_keep_the_highest`, `an_attack_penalty_does_not_change_fortitude`, `fortitude_and_will_do_not_share_a_bonus`, and `the_same_modifier_twice_matches`. Verify `cargo test -p rules -- an_item_bonus_adds_beside_proficiency two_circumstance_bonuses_keep_the_highest an_attack_penalty_does_not_change_fortitude fortitude_and_will_do_not_share_a_bonus the_same_modifier_twice_matches` passes

- [x] 2.6 Add `a_negative_total_against_dc_5_is_a_failure`, `a_status_bonus_adds`, `a_status_penalty_and_a_circumstance_penalty_both_apply`, `two_circumstance_penalties_keep_the_worst`, and `an_untyped_penalty_applies`. Verify `cargo test -p rules -- a_negative_total_against_dc_5_is_a_failure a_status_bonus_adds a_status_penalty_and_a_circumstance_penalty_both_apply two_circumstance_penalties_keep_the_worst an_untyped_penalty_applies` passes

- [x] 2.7 Add `unconscious_reflex_includes_minus_4`, `alive_reflex_does_not_include_minus_4`, `a_dying_creature_s_reflex_includes_minus_4`, `unconscious_fortitude_does_not_include_minus_4`, and `unconscious_will_does_not_include_minus_4`. Verify `cargo test -p rules -- unconscious_reflex_includes_minus_4 alive_reflex_does_not_include_minus_4 a_dying_creature_s_reflex_includes_minus_4 unconscious_fortitude_does_not_include_minus_4 unconscious_will_does_not_include_minus_4` passes

- [x] 2.8 Add `a_modifier_of_plus_7_has_dc_17`, `a_level_of_0_cannot_be_built`, `a_level_of_26_cannot_be_built`, `a_level_of_25_can_be_built`, and `legendary_at_level_25_is_plus_33_before_ability`. Verify `cargo test -p rules -- a_modifier_of_plus_7_has_dc_17 a_level_of_0_cannot_be_built a_level_of_26_cannot_be_built a_level_of_25_can_be_built legendary_at_level_25_is_plus_33_before_ability` passes

- [x] 2.9 Add `doubling_the_maximum_damage_writes_nothing`. Verify `cargo test -p rules -- doubling_the_maximum_damage_writes_nothing` passes

- [x] 2.10 Add `failure_to_0_sets_dying_1`, `critical_failure_to_0_sets_dying_2`, `failure_while_dying_1_increases_to_dying_2`, `critical_failure_while_dying_1_increases_by_2`, and `without_dying_rules_0_hit_points_is_death`. Verify `cargo test -p rules -- failure_to_0_sets_dying_1 critical_failure_to_0_sets_dying_2 failure_while_dying_1_increases_to_dying_2 critical_failure_while_dying_1_increases_by_2 without_dying_rules_0_hit_points_is_death` passes

- [x] 2.11 Add `critical_failure_spends_temporary_hit_points_first` and `failure_leaves_1_hit_point_and_no_dying`. Verify `cargo test -p rules -- critical_failure_spends_temporary_hit_points_first failure_leaves_1_hit_point_and_no_dying` passes

- [x] 2.12 Add `dying_1_and_a_critical_success_on_listed_5_stays_dying_1`, `dying_1_and_a_success_on_listed_0_stays_dying_1`, and `unconscious_at_0_hit_points_and_a_critical_success_is_not_a_knockout`. Verify `cargo test -p rules -- dying_1_and_a_critical_success_on_listed_5_stays_dying_1 dying_1_and_a_success_on_listed_0_stays_dying_1 unconscious_at_0_hit_points_and_a_critical_success_is_not_a_knockout` passes

- [x] 2.13 Add `a_raised_shield_does_not_block`, `a_damage_modifier_does_not_change_the_amount`, `the_same_save_twice_matches`, `an_unknown_defender_rolls_nothing`, `a_dead_defender_rolls_nothing`, and `no_save_training_rolls_nothing`. Verify `cargo test -p rules -- a_raised_shield_does_not_block a_damage_modifier_does_not_change_the_amount the_same_save_twice_matches an_unknown_defender_rolls_nothing a_dead_defender_rolls_nothing no_save_training_rolls_nothing` passes

- [x] 2.14 Add `a_creature_may_save_against_itself`, `a_delayed_creature_can_save`, and `an_unconscious_creature_can_save`. Verify `cargo test -p rules -- a_creature_may_save_against_itself a_delayed_creature_can_save an_unconscious_creature_can_save` passes

- [x] 2.15 Add `a_delayed_knockout_is_placed_before_the_open_creature` and `a_knockout_that_is_not_delayed_moves_before_the_open_creature`. Verify `cargo test -p rules -- a_delayed_knockout_is_placed_before_the_open_creature a_knockout_that_is_not_delayed_moves_before_the_open_creature` passes

- [x] 2.16 Add `no_saving_throws_keeps_its_own_arms` and `save_overflow_keeps_its_own_arms` in `encounter.rs`, beside the existing conversion tests. Verify `cargo test -p rules -- no_saving_throws_keeps_its_own_arms save_overflow_keeps_its_own_arms` passes

- [x] 2.17 Add `authored_armor_class_stays_authored`, `omitted_training_stays_omitted`, `the_same_build_twice_matches`, and `a_later_contribution_replaces_training` beside the creature tests. Verify `cargo test -p rules -- authored_armor_class_stays_authored omitted_training_stays_omitted the_same_build_twice_matches a_later_contribution_replaces_training` passes

- [x] 2.18 Add `fortitude_and_will_do_not_share_a_stack`, `unconscious_adds_nothing_to_fortitude`, `unconscious_adds_nothing_to_will`, and `reflex_keeps_the_unconscious_penalty` beside the modifier tests. Verify `cargo test -p rules -- fortitude_and_will_do_not_share_a_stack unconscious_adds_nothing_to_fortitude unconscious_adds_nothing_to_will reflex_keeps_the_unconscious_penalty` passes

- [x] 2.19 Add `a_save_after_a_strike_is_not_a_strike`, `a_save_draws_no_flat_check_die`, `a_save_before_any_turn_rolls`, and `a_save_off_the_active_turn_rolls`. Verify `cargo test -p rules -- a_save_after_a_strike_is_not_a_strike a_save_draws_no_flat_check_die a_save_before_any_turn_rolls a_save_off_the_active_turn_rolls` passes

## 3. Code

- [x] 3.1 Add `crates/rules/src/save.rs` and re-export the design.md items from `lib.rs`. Implement `Level`, `Proficiency`, `save_modifier`, `save_dc`, and `basic_save_amount`, including half of 1 staying 1. Keep `#![forbid(unsafe_code)]`. Verify the tests from 2.1 through 2.9 pass and `crates/rules/Cargo.toml` has no dependencies

- [x] 3.2 Add `saves: Option<SavingThrows>` to `BuildParts`, `Contribution`, and `Creature`. `derive` copies it and does not roll. Folding replaces it when `Some`. Set `saves: None` on every existing literal. Verify the tests from 2.17 pass

- [x] 3.3 Add `Statistic::Fortitude` and `Statistic::Will` after Reflex, with sort keys 5 and 6. Do not add `fortitude_modifiers` or `will_modifiers`. Leave `reflex_modifiers` as status -4 at sequence 10. Verify the tests from 2.18 pass and the old Reflex unconscious tests still pass

- [x] 3.4 Add `Encounter::basic_save` taking `Save`, plus `NoSavingThrows` and `SaveOverflow`, in design.md's order. Skip `apply_resolved_blow` when the amount is 0. Translate a positive amount. While a turn is open, place every knockout before the open creature. Add the three conversion arms from design.md. Do not call `strike`, `damage_dealt`, `shield_block`, or `resolve_targeting`. Verify the tests from 2.10 through 2.16 and 2.19 pass

- [x] 3.5 In the code phase, add rows to `docs/designer/rules-deviations.md`. Copy DEV-52, DEV-53, DEV-54, and DEV-55 from `openspec/changes/archive/2026-10-23-add-delay-and-ready/design.md`. Add DEV-56, DEV-57, and DEV-58 from this design. DEV-47 and DEV-49 stay unused, do not reuse. The spec commit does not edit that file

## 4. Changelog and proof

- [x] 4.1 Add one Added bullet for saving throws and one **BREAKING** Changed bullet for `saves` and the two statistics, both under `## [Unreleased]`, both linking this pull request. Commit that before the archive commit. Do not add either bullet in the spec commit. Verify both bullets contain the pull request link

- [x] 4.2 Run the AGENTS.md proof commands and verify each exits 0. Confirm `crates/rules/Cargo.toml` still has no dependencies
  - `cargo fmt --all --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo build --workspace`
  - `cargo test --workspace`
  - `npx @fission-ai/openspec@1.14.3 validate --all --strict`

## 5. Archive

- [ ] 5.1 This is the last commit, right before merge, and only after Architect and QA have both passed that head. Run `openspec archive add-saving-throws --yes` with that exact commit subject. Check this box only in that commit. Do not archive the spec draft. Verify the change is under `openspec/changes/archive/` and is no longer active, and that the archive commit does not edit `CHANGELOG.md`
