# Tasks

Each scenario has one test. The name is the title in snake_case. Spaces, hyphens, and apostrophes become `_`. Other punctuation is dropped. A title that says `plus 4` or `minus 5` keeps those words. Use `ScriptedDice`, and assert an unused face when the scenario says so. Cite that requirement's Player Core section. The spec commit does not edit `docs/designer/`. Task 3.6 does, in the code phase. No file on `v1-reference` is named, so none is ported.

Write section 2 before filling the production bodies. The tests call the items in design.md. They may share a commit with section 3. Do not start section 2 until task 1.2 is true.

## 1. Spec

- [ ] 1.1 These files are the behavior. The spec commit adds no Rust, no test, and no changelog bullet. Verify `npx @fission-ai/openspec@1.14.3 validate --all --strict` exits 0

- [ ] 1.2 Do not start section 2 until this pull request has Evan's comment `spec-approved: <sha>` and that sha is an ancestor of HEAD. Designer rules on the spec in this pull request

## 2. Tests

- [ ] 2.1 Add `the_same_burst_twice_matches`, `a_5_foot_burst_is_the_four_touching_squares`, `a_10_foot_burst_is_twelve_squares`, `a_20_foot_burst_reaches_the_far_corner_boundary`, and `a_radius_of_0_4_or_negative_5_is_rejected`. Verify `cargo test -p rules -- the_same_burst_twice_matches a_5_foot_burst_is_the_four_touching_squares a_10_foot_burst_is_twelve_squares a_20_foot_burst_reaches_the_far_corner_boundary a_radius_of_0_4_or_negative_5_is_rejected` passes

- [ ] 2.2 Add `east_is_the_seven_squares`, `north_is_that_template_turned`, `south_is_that_template_turned`, `west_is_that_template_turned`, `a_30_foot_cone_is_rejected`, `northeast_is_the_six_squares`, `southeast_is_the_six_squares`, `southwest_is_the_six_squares`, and `northwest_is_the_six_squares`. Verify `cargo test -p rules -- east_is_the_seven_squares north_is_that_template_turned south_is_that_template_turned west_is_that_template_turned a_30_foot_cone_is_rejected northeast_is_the_six_squares southeast_is_the_six_squares southwest_is_the_six_squares northwest_is_the_six_squares` passes

- [ ] 2.3 Add `a_5_foot_emanation_is_the_eight_adjacent_squares`, `the_caller_can_include_the_origin`, `a_10_foot_emanation_includes_2_1_and_excludes_2_2`, `a_30_foot_east_line_is_six_squares`, `a_30_foot_northeast_line_is_four_squares`, `a_length_of_0_is_rejected`, and `a_large_creature_is_rejected`. Verify `cargo test -p rules -- a_5_foot_emanation_is_the_eight_adjacent_squares the_caller_can_include_the_origin a_10_foot_emanation_includes_2_1_and_excludes_2_2 a_30_foot_east_line_is_six_squares a_30_foot_northeast_line_is_four_squares a_length_of_0_is_rejected a_large_creature_is_rejected` passes

- [ ] 2.4 Add `trained_at_level_1_with_plus_4_is_plus_7_and_dc_17`, `expert_at_level_5_with_plus_3_is_plus_12_and_dc_22`, `untrained_at_level_20_with_plus_4_is_plus_4_and_dc_14`, `a_negative_attribute_lowers_the_dc`, `an_attack_bonus_does_not_raise_the_spell_dc`, `a_spellattack_status_bonus_raises_the_roll_and_the_dc`, `two_circumstance_bonuses_keep_the_highest`, and `a_modifier_that_does_not_fit_draws_nothing`. Verify `cargo test -p rules -- trained_at_level_1_with_plus_4_is_plus_7_and_dc_17 expert_at_level_5_with_plus_3_is_plus_12_and_dc_22 untrained_at_level_20_with_plus_4_is_plus_4_and_dc_14 a_negative_attribute_lowers_the_dc an_attack_bonus_does_not_raise_the_spell_dc a_spellattack_status_bonus_raises_the_roll_and_the_dc two_circumstance_bonuses_keep_the_highest a_modifier_that_does_not_fit_draws_nothing` passes

- [ ] 2.5 Add `spellattack_and_attack_do_not_share_a_stack` and `unconscious_adds_nothing_to_spellattack` beside the modifier tests. Verify `cargo test -p rules -- spellattack_and_attack_do_not_share_a_stack unconscious_adds_nothing_to_spellattack` passes

- [ ] 2.6 Add `omitted_spellcasting_stays_omitted`, `a_present_entry_is_copied`, and `the_same_spellcasting_build_twice_matches` beside the creature tests. Verify `cargo test -p rules -- omitted_spellcasting_stays_omitted a_present_entry_is_copied the_same_spellcasting_build_twice_matches` passes

- [ ] 2.7 Add `breathe_fire_before_the_turn_starts_rolls_nothing`, `a_projectile_off_the_active_turn_rolls_nothing`, `an_unconscious_caster_rolls_nothing`, `an_unknown_caster_rolls_nothing`, `one_action_cannot_cast_either_spell`, and `no_spellcasting_rolls_nothing`. Verify `cargo test -p rules -- breathe_fire_before_the_turn_starts_rolls_nothing a_projectile_off_the_active_turn_rolls_nothing an_unconscious_caster_rolls_nothing an_unknown_caster_rolls_nothing one_action_cannot_cast_either_spell no_spellcasting_rolls_nothing` passes

- [ ] 2.8 Add `six_orthogonal_squares_are_in_range`, `seven_orthogonal_squares_spend_nothing`, `the_caster_may_target_their_own_square`, `a_dead_target_spends_nothing`, `an_unknown_target_spends_nothing`, `a_lost_concealed_check_spends_the_cast`, `a_concealed_success_then_rolls_the_attack`, `empty_detection_draws_no_flat_check`, and `an_undetected_target_spends_nothing`. Verify `cargo test -p rules -- six_orthogonal_squares_are_in_range seven_orthogonal_squares_spend_nothing the_caster_may_target_their_own_square a_dead_target_spends_nothing an_unknown_target_spends_nothing a_lost_concealed_check_spends_the_cast a_concealed_success_then_rolls_the_attack empty_detection_draws_no_flat_check an_undetected_target_spends_nothing` passes

- [ ] 2.9 Add `the_same_projectile_twice_matches`, `the_projectile_leaves_one_action`, `the_projectile_is_ranged`, `the_second_attack_takes_minus_5_and_the_dc_stays_17`, `an_agile_strike_does_not_make_the_spell_agile`, `a_natural_10_against_armor_class_16_is_a_success`, `armor_class_in_the_caller_list_does_not_change_the_result`, `a_natural_10_against_armor_class_18_rolls_no_damage_die`, `a_natural_10_against_armor_class_16_hits_for_7`, and `a_natural_20_doubles_7_to_14_once`. Verify `cargo test -p rules -- the_same_projectile_twice_matches the_projectile_leaves_one_action the_projectile_is_ranged the_second_attack_takes_minus_5_and_the_dc_stays_17 an_agile_strike_does_not_make_the_spell_agile a_natural_10_against_armor_class_16_is_a_success armor_class_in_the_caller_list_does_not_change_the_result a_natural_10_against_armor_class_18_rolls_no_damage_die a_natural_10_against_armor_class_16_hits_for_7 a_natural_20_doubles_7_to_14_once` passes

- [ ] 2.10 Add `the_same_cone_twice_matches`, `breathe_fire_leaves_one_action`, `breathe_fire_is_not_ranged`, `breathe_fire_leaves_the_attack_count`, `a_concealed_creature_in_the_cone_draws_no_flat_check`, `failure_deals_7_and_success_deals_3`, `a_critical_success_deals_0`, `a_critical_failure_doubles_7_to_14`, `the_excluded_adjacent_squares_are_safe`, `a_dead_creature_in_the_cone_draws_no_save`, `a_duplicate_id_rolls_nothing`, `no_save_training_in_the_cone_rolls_nothing`, and `no_save_training_outside_the_cone_still_casts`. Verify `cargo test -p rules -- the_same_cone_twice_matches breathe_fire_leaves_one_action breathe_fire_is_not_ranged breathe_fire_leaves_the_attack_count a_concealed_creature_in_the_cone_draws_no_flat_check failure_deals_7_and_success_deals_3 a_critical_success_deals_0 a_critical_failure_doubles_7_to_14 the_excluded_adjacent_squares_are_safe a_dead_creature_in_the_cone_draws_no_save a_duplicate_id_rolls_nothing no_save_training_in_the_cone_rolls_nothing no_save_training_outside_the_cone_still_casts` passes

## 3. Code

- [ ] 3.1 Make the existing diagonal measurement `pub(crate)` in `crates/rules/src/grid.rs` without copying the formula. Add `crates/rules/src/spell.rs` with area membership for burst, 15-foot cone, emanation, and line, and with `spell_attack_modifier` and `spell_dc`. Re-export the design.md public types from `lib.rs`. Keep `#![forbid(unsafe_code)]`. Verify the tests from 2.1 through 2.4 pass and `crates/rules/Cargo.toml` has no dependencies

- [ ] 3.2 Add `Statistic::SpellAttack` after Will and a bucket that does not share a stack. Unconscious adds nothing to it. Verify the tests from 2.5 pass and the old Reflex unconscious tests still pass

- [ ] 3.3 Add `spellcasting: Option<Spellcasting>` to `BuildParts`, `Contribution`, and `Creature`. `derive` copies it and does not roll. Folding replaces it when `Some`. Set `spellcasting: None` on every existing literal, including the doctest in `lib.rs`. Verify the tests from 2.6 pass

- [ ] 3.4 Add `Encounter::telekinetic_projectile` and `EncounterError::NoSpellcasting` and `OutOfSpellRange`. Spend 2 actions only after the pre-die checks. Apply the multiple attack penalty on the roll only. Call `resolve_targeting`. Do not call `strike`, `shield_block`, or `resolve_reactions`. On a hit, roll 2d6 and pass a positive amount through `apply_resolved_blow`. Verify the tests from 2.7 through 2.9 pass

- [ ] 3.5 Add `Encounter::breathe_fire`, `UnsupportedCone`, and `DuplicateAreaTarget`. A living creature in the cone uses `Encounter::basic_save` for Reflex against the spell DC. Roll 2d6 once, before the saves, in ascending `CombatantId` order. Do not increment the attack count. Verify the tests from 2.10 pass

- [ ] 3.6 Add `Aim`, `AreaCreature`, `BreatheFireOutcome`, `SpellAttackHit`, `SpellAttackOutcome`, and `Spellcasting` to the `PUBLIC` list in `crates/rules/src/facade_check.rs`. Keep `breathe_fire` and `telekinetic_projectile` public on `Encounter` and absent from the private-name list. Verify `cargo test -p rules -- root_exports_match_the_facade` passes

- [ ] 3.7 In the code phase, add rows to `docs/designer/rules-deviations.md` for DEV-59 through DEV-67 from this design. DEV-47 and DEV-49 stay unused. The spec commit does not edit that file

## 4. Changelog and proof

- [ ] 4.1 Add one Added bullet for spells and areas under `## [Unreleased]`, linking this pull request. Commit that before the archive commit. Do not add the bullet in the spec commit. Verify the bullet contains the pull request link

- [ ] 4.2 Run the AGENTS.md proof commands and verify each exits 0. Confirm `crates/rules/Cargo.toml` still has no dependencies
  - `cargo fmt --all --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo build --workspace`
  - `cargo test --workspace`
  - `npx @fission-ai/openspec@1.14.3 validate --all --strict`

## 5. Archive

- [ ] 5.1 This is the last commit, right before merge, and only after Architect and QA have both passed that head. Run `openspec archive add-spells-and-areas --yes` with that exact commit subject. Check this box only in that commit. Do not archive the spec draft. Verify the change is under `openspec/changes/archive/` and is no longer active, and that the archive commit does not edit `CHANGELOG.md`
