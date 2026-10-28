# Tasks

Each scenario has one test. The name is the title in snake_case. Spaces, hyphens, and apostrophes become `_`. Other punctuation is dropped. A title that says `plus 7` or `minus 11` keeps those words. Use `ScriptedDice`, and assert an unused face when the scenario says so. Cite that requirement's Player Core section. The spec commit does not edit `docs/designer/`. Task 3.6 does, in the code phase. No file on `v1-reference` is named, so none is ported.

Write section 2 before filling the production bodies. Commit those tests before the section 3 code. The tests may fail until section 3. Do not start section 2 until task 1.2 is true.

## 1. Spec

- [x] 1.1 These files are the behavior. The spec commit adds no Rust, no test, and no changelog bullet. Verify `npx @fission-ai/openspec@1.14.3 validate --all --strict` exits 0

- [x] 1.2 Do not start section 2 until this pull request has Evan's review whose body line is `spec-approved: <sha>` and that sha is an ancestor of HEAD. Designer rules on the spec in this pull request

## 2. Tests

- [x] 2.1 Add `athletics_is_strength_plus_trained_at_level_1`, `acrobatics_is_dexterity_plus_expert_at_level_5`, `arcana_is_intelligence_plus_master_at_level_10`, `medicine_is_wisdom_plus_legendary_at_level_20`, and `diplomacy_is_charisma_plus_untrained`. Verify `cargo test -p rules -- athletics_is_strength_plus_trained_at_level_1 acrobatics_is_dexterity_plus_expert_at_level_5 arcana_is_intelligence_plus_master_at_level_10 medicine_is_wisdom_plus_legendary_at_level_20 diplomacy_is_charisma_plus_untrained` passes

- [x] 2.2 Add `an_item_bonus_adds_when_untrained`, `two_item_bonuses_keep_the_highest`, `a_circumstance_bonus_adds_beside_an_item_bonus`, `a_status_penalty_applies`, `an_item_bonus_and_an_item_penalty_both_apply`, `the_same_skill_modifier_twice_matches`, `frightened_2_on_trained_athletics_is_plus_1_and_dc_11`, `frightened_2_and_sickened_1_give_minus_2`, `two_untyped_penalties_of_minus_1_on_athletics_give_minus_2`, and `two_status_penalties_keep_only_the_worst`. Verify `cargo test -p rules -- an_item_bonus_adds_when_untrained two_item_bonuses_keep_the_highest a_circumstance_bonus_adds_beside_an_item_bonus a_status_penalty_applies an_item_bonus_and_an_item_penalty_both_apply the_same_skill_modifier_twice_matches frightened_2_on_trained_athletics_is_plus_1_and_dc_11 frightened_2_and_sickened_1_give_minus_2 two_untyped_penalties_of_minus_1_on_athletics_give_minus_2 two_status_penalties_keep_only_the_worst` passes

- [x] 2.3 Add `an_acrobatics_bonus_does_not_change_athletics`, `an_attack_bonus_does_not_change_athletics`, and `each_skill_uses_its_key_attribute`. Verify `cargo test -p rules -- an_acrobatics_bonus_does_not_change_athletics an_attack_bonus_does_not_change_athletics each_skill_uses_its_key_attribute` passes

- [x] 2.4 Add `trained_at_level_2_with_strength_0_is_plus_4`, `legendary_athletics_at_level_25_is_plus_33`, `a_skill_level_of_0_cannot_be_built`, `a_skill_level_of_26_cannot_be_built`, `a_skill_level_of_25_can_be_built`, and `a_skill_modifier_of_plus_7_has_dc_17`. Verify `cargo test -p rules -- trained_at_level_2_with_strength_0_is_plus_4 legendary_athletics_at_level_25_is_plus_33 a_skill_level_of_0_cannot_be_built a_skill_level_of_26_cannot_be_built a_skill_level_of_25_can_be_built a_skill_modifier_of_plus_7_has_dc_17` passes

- [x] 2.5 Add `total_dc_minus_1_on_a_skill_check_is_a_failure`, `natural_1_on_a_skill_check_at_a_total_equal_to_the_dc_is_a_failure`, `natural_20_on_a_skill_check_at_dc_minus_11_is_a_failure`, `beating_a_skill_dc_by_10_is_a_critical_success`, `a_negative_skill_total_against_dc_5_is_a_failure`, `natural_20_on_a_skill_check_at_the_dc_is_a_critical_success`, `natural_20_on_a_skill_check_at_dc_minus_10_is_a_failure`, `natural_1_on_a_skill_check_at_dc_plus_10_is_a_success`, and `a_total_of_exactly_dc_minus_10_is_a_critical_failure`. Verify `cargo test -p rules -- total_dc_minus_1_on_a_skill_check_is_a_failure natural_1_on_a_skill_check_at_a_total_equal_to_the_dc_is_a_failure natural_20_on_a_skill_check_at_dc_minus_11_is_a_failure beating_a_skill_dc_by_10_is_a_critical_success a_negative_skill_total_against_dc_5_is_a_failure natural_20_on_a_skill_check_at_the_dc_is_a_critical_success natural_20_on_a_skill_check_at_dc_minus_10_is_a_failure natural_1_on_a_skill_check_at_dc_plus_10_is_a_success a_total_of_exactly_dc_minus_10_is_a_critical_failure` passes

- [x] 2.6 Add `recall_knowledge_success_returns_that_degree`, `recall_knowledge_critical_success_returns_that_degree`, `recall_knowledge_failure_returns_that_degree`, `recall_knowledge_critical_failure_returns_that_degree`, `untrained_recall_knowledge_is_legal`, and `the_same_recall_twice_matches`. Verify `cargo test -p rules -- recall_knowledge_success_returns_that_degree recall_knowledge_critical_success_returns_that_degree recall_knowledge_failure_returns_that_degree recall_knowledge_critical_failure_returns_that_degree untrained_recall_knowledge_is_legal the_same_recall_twice_matches` passes

- [x] 2.7 Add `a_modifier_that_does_not_fit_rolls_nothing`, `no_skills_keeps_its_own_arms`, and `skill_overflow_keeps_its_own_arms`. Put the arm tests beside `no_saving_throws_keeps_its_own_arms`. Verify `cargo test -p rules -- a_modifier_that_does_not_fit_rolls_nothing no_skills_keeps_its_own_arms skill_overflow_keeps_its_own_arms` passes

- [x] 2.8 Add `recall_knowledge_spends_one_action`, `recall_knowledge_does_not_change_the_attack_count`, `a_skill_check_off_the_active_turn_rolls`, `a_skill_check_before_any_turn_rolls`, and `recall_knowledge_before_any_turn_rolls_nothing`. Verify `cargo test -p rules -- recall_knowledge_spends_one_action recall_knowledge_does_not_change_the_attack_count a_skill_check_off_the_active_turn_rolls a_skill_check_before_any_turn_rolls recall_knowledge_before_any_turn_rolls_nothing` passes

- [x] 2.9 Add `recall_knowledge_off_the_active_turn_rolls_nothing`, `an_unconscious_actor_rolls_nothing`, `an_unconscious_creature_can_still_roll_a_skill_check`, `insufficient_actions_roll_nothing`, `no_skill_training_rolls_nothing`, `a_skill_check_with_no_training_rolls_nothing`, `a_skill_check_draws_no_flat_check_die`, and `an_unknown_creature_rolls_nothing`. Verify `cargo test -p rules -- recall_knowledge_off_the_active_turn_rolls_nothing an_unconscious_actor_rolls_nothing an_unconscious_creature_can_still_roll_a_skill_check insufficient_actions_roll_nothing no_skill_training_rolls_nothing a_skill_check_with_no_training_rolls_nothing a_skill_check_draws_no_flat_check_die an_unknown_creature_rolls_nothing` passes

- [x] 2.10 Add `skill_training_leaves_armor_class_authored`, `omitted_skill_training_stays_omitted`, `the_same_skill_build_twice_matches`, `a_later_contribution_replaces_skill_training`, and `skill_dexterity_is_not_the_save_dexterity` beside the creature tests. Verify `cargo test -p rules -- skill_training_leaves_armor_class_authored omitted_skill_training_stays_omitted the_same_skill_build_twice_matches a_later_contribution_replaces_skill_training skill_dexterity_is_not_the_save_dexterity` passes

- [x] 2.11 Add `athletics_and_acrobatics_do_not_share_a_stack`, `unconscious_adds_nothing_to_athletics`, `a_skill_item_bonus_does_not_change_fortitude`, `frightened_2_reaches_every_skill_statistic`, and `sickened_1_reaches_every_skill_statistic` beside the modifier tests. Verify `cargo test -p rules -- athletics_and_acrobatics_do_not_share_a_stack unconscious_adds_nothing_to_athletics a_skill_item_bonus_does_not_change_fortitude frightened_2_reaches_every_skill_statistic sickened_1_reaches_every_skill_statistic` passes

- [x] 2.12 Update the crate-root doctest `An external crate starts and ends one turn` so the build sets `skills: None`. Update `The public names are this set` and `PUBLIC` in `crates/rules/src/facade_check.rs` to add `Skill`, `SkillRoll`, and `Skills`. Keep the method doctest heading `The public method set is unchanged` and name `Encounter::skill_check` and `Encounter::recall_knowledge` in that doctest. Add one `compile_fail,E0432` doctest each for `use rules::skill_modifier;` and `use rules::skill_dc;`. Verify `cargo test --doc --package rules` lists those fences. The new imports fail until task 3.5

## 3. Code

- [x] 3.1 Add `crates/rules/src/skill.rs` with `Skill`, `Skills`, `SkillRoll`, `skill_modifier`, and `skill_dc` from design.md. Call `proficiency_bonus`. Include the frightened and sickened status penalties from the existing valued-condition producer, including when the caller's entry names another statistic, and use that modifier for `skill_dc`. Do not copy the rank table and do not add a second level range. Keep `#![forbid(unsafe_code)]`. Verify the tests from 2.1 through 2.5 pass and `crates/rules/Cargo.toml` has no dependencies

- [x] 3.2 Add `skills: Option<Skills>` to `BuildParts`, `Contribution`, and `Creature`. `derive` copies it and does not roll. Folding replaces it when `Some`. Set `skills: None` on every existing literal. Do not read `SavingThrows` from the skill block. Verify the tests from 2.10 pass

- [x] 3.3 Add the sixteen skill variants to `Statistic` after Will, with sort keys 7 through 22 in `Skill` order. Give each its own bucket. Extend the frightened and sickened valued-condition producer so each skill statistic receives a status penalty equal to that value. Do not add an unconscious penalty. Update every `Statistic` match. Verify the tests from 2.11 pass and the existing Fortitude, Reflex, and Will tests still pass

- [x] 3.4 Add `Encounter::skill_check` and `Encounter::recall_knowledge` in `crates/rules/src/encounter/skills.rs`, plus `NoSkills` and `SkillOverflow`, in design.md's order. `skill_check` is a primitive for a caller-adjudicated check: it does not read vital and it does not apply an unconscious penalty. Recall Knowledge calls `spend_actions` only after `NoSkills` and `SkillOverflow`. Do not store Recall Knowledge's concentrate trait. Add the three conversion arms from design.md. Do not call `strike`, `resolve_reactions`, `damage_dealt`, `shield_block`, or `resolve_targeting`. Verify the tests from 2.6 through 2.9 pass

- [x] 3.5 Re-export `Skill`, `Skills`, and `SkillRoll` from `lib.rs`. Do not re-export `skill_modifier` or `skill_dc`. Verify `cargo test --doc --package rules` passes and `root_exports_match_the_facade` passes

- [x] 3.6 In the code phase, add DEV-59, DEV-60, and DEV-61 from this design to `docs/designer/rules-deviations.md`. DEV-59's row amends DEV-58. DEV-60's row cross-references DEV-08. The next free numbers are 59, 60, and 61. Do not reuse DEV-33, DEV-34, DEV-35, DEV-47, DEV-48, DEV-49, DEV-56, DEV-57, or DEV-58. The spec commit does not edit that file

## 4. Changelog and proof

- [x] 4.1 Add one Added bullet for skill checks and Recall Knowledge, and one **BREAKING** Changed bullet for `skills`, the skill statistics, and the two encounter methods, both under `## [Unreleased]`, both linking this pull request. Commit that before the archive commit. Do not add either bullet in the spec commit. Verify both bullets contain the pull request link

- [x] 4.2 Run the AGENTS.md proof commands and verify each exits 0. Confirm `crates/rules/Cargo.toml` still has no dependencies
  - `cargo fmt --all --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo build --workspace`
  - `cargo test --workspace`
  - `npx @fission-ai/openspec@1.14.3 validate --all --strict`

## 5. Archive

- [x] 5.1 This is the last commit, right before merge, and only after Architect and QA have both passed that head. Run `openspec archive add-skills --yes` with that exact commit subject. Check this box only in that commit. Do not archive the spec draft. Verify the change is under `openspec/changes/archive/` and is no longer active, and that the archive commit does not edit `CHANGELOG.md`
