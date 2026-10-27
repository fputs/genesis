# Tasks

Each scenario has one test. The name is the title in snake_case. Spaces, hyphens, and apostrophes become `_`. Other punctuation is dropped. A title that says `minus 2` keeps those words. Use `ScriptedDice`, and assert an unused face when the scenario says so. Cite that requirement's Player Core section. The spec commit does not edit `docs/designer/`. Task 3.6 does, in the code phase. No file on `v1-reference` is named, so none is ported.

Write section 2 before filling the production bodies. The tests call the items in design.md. They may share a commit with section 3. Do not start section 2 until task 1.2 is true.

## 1. Spec

- [ ] 1.1 These files are the behavior. The spec commit adds no Rust, no test, and no changelog bullet. Verify `npx @fission-ai/openspec@1.14.3 validate --all --strict` exits 0

- [ ] 1.2 Do not start section 2 until this pull request has Evan's comment `spec-approved: <sha>` and that sha is an ancestor of HEAD. Designer rules on the spec in this pull request

## 2. Tests

- [ ] 2.1 Add `the_same_conditions_twice_match`, `frightened_2_is_stored_at_value_2`, `a_second_frightened_instance_stays_stored`, `frightened_without_a_value_is_rejected`, and `frightened_0_is_rejected` in `crates/rules/src/conditions.rs`. Verify `cargo test -p rules -- the_same_conditions_twice_match frightened_2_is_stored_at_value_2 a_second_frightened_instance_stays_stored frightened_without_a_value_is_rejected frightened_0_is_rejected` passes

- [ ] 2.2 Add `off_guard_with_a_value_is_rejected`, `a_round_duration_of_0_is_rejected`, `the_active_value_is_the_higher_frightened`, `the_higher_frightened_ending_leaves_the_lower_one`, and `reduce_lowers_every_frightened_instance`. Verify `cargo test -p rules -- off_guard_with_a_value_is_rejected a_round_duration_of_0_is_rejected the_active_value_is_the_higher_frightened the_higher_frightened_ending_leaves_the_lower_one reduce_lowers_every_frightened_instance` passes

- [ ] 2.3 Add `frightened_1_reduced_by_1_ends`, `reduce_on_an_absent_condition_changes_nothing`, `reduce_by_0_changes_nothing`, `off_guard_rejects_a_reduction`, and `prone_rejects_a_reduction`. Verify `cargo test -p rules -- frightened_1_reduced_by_1_ends reduce_on_an_absent_condition_changes_nothing reduce_by_0_changes_nothing off_guard_rejects_a_reduction prone_rejects_a_reduction` passes

- [ ] 2.4 Add `remove_clears_frightened_and_leaves_sickened`, `remove_on_an_absent_condition_changes_nothing`, `frightened_2_and_sickened_1_are_a_status_penalty_of_minus_2`, `frightened_does_not_penalize_damage`, and `tracked_frightened_2_and_a_caller_frightened_1_are_minus_2`. Verify `cargo test -p rules -- remove_clears_frightened_and_leaves_sickened remove_on_an_absent_condition_changes_nothing frightened_2_and_sickened_1_are_a_status_penalty_of_minus_2 frightened_does_not_penalize_damage tracked_frightened_2_and_a_caller_frightened_1_are_minus_2` passes

- [ ] 2.5 Add `a_status_bonus_and_frightened_both_apply`, `frightened_and_off_guard_both_apply_to_armor_class`, `off_guard_is_minus_2_circumstance_to_armor_class`, `prone_penalizes_attacks_and_armor_class`, and `prone_and_off_guard_penalize_armor_class_once`. Verify `cargo test -p rules -- a_status_bonus_and_frightened_both_apply frightened_and_off_guard_both_apply_to_armor_class off_guard_is_minus_2_circumstance_to_armor_class prone_penalizes_attacks_and_armor_class prone_and_off_guard_penalize_armor_class_once` passes

- [ ] 2.6 Add `clumsy_2_penalizes_reflex_and_a_ranged_attack`, `clumsy_2_does_not_penalize_a_strength_melee_attack`, `enfeebled_2_penalizes_a_strength_melee_attack_and_its_damage`, and `enfeebled_2_does_not_penalize_armor_class`. Verify `cargo test -p rules -- clumsy_2_penalizes_reflex_and_a_ranged_attack clumsy_2_does_not_penalize_a_strength_melee_attack enfeebled_2_penalizes_a_strength_melee_attack_and_its_damage enfeebled_2_does_not_penalize_armor_class` passes

- [ ] 2.7 Add `the_end_tick_lowers_frightened_and_not_sickened`, `frightened_1_ends_on_the_end_tick`, `a_due_frightened_3_leaves_the_other_frightened_at_1`, `an_end_that_is_not_yet_due_survives_one_tick`, `the_end_tick_leaves_persistent_damage`, and `a_due_end_removes_the_condition`. Verify `cargo test -p rules -- the_end_tick_lowers_frightened_and_not_sickened frightened_1_ends_on_the_end_tick a_due_frightened_3_leaves_the_other_frightened_at_1 an_end_that_is_not_yet_due_survives_one_tick the_end_tick_leaves_persistent_damage a_due_end_removes_the_condition` passes

- [ ] 2.8 Add `one_start_tick_lowers_3_rounds_to_2`, `the_third_start_tick_ends_a_3_round_duration`, `another_creature_s_start_does_not_lower_that_duration`, and `the_start_tick_does_not_lower_frightened`. Verify `cargo test -p rules -- one_start_tick_lowers_3_rounds_to_2 the_third_start_tick_ends_a_3_round_duration another_creature_s_start_does_not_lower_that_duration the_start_tick_does_not_lower_frightened` passes

- [ ] 2.9 Add `a_3_round_duration_decreases_on_the_clock_s_next_start`, `the_bearer_s_start_does_not_decrease_another_clock`, and `a_skipped_dead_slot_still_ticks_its_round_clock` beside the turn tests. Verify `cargo test -p rules -- a_3_round_duration_decreases_on_the_clock_s_next_start the_bearer_s_start_does_not_decrease_another_clock a_skipped_dead_slot_still_ticks_its_round_clock an_alive_start_rolls_nothing` passes

- [ ] 2.10 Add `an_open_turn_s_end_is_not_the_next_end`, `a_closed_turn_s_next_end_removes_it`, `ending_the_turn_lowers_frightened_and_leaves_hit_points`, and `the_persistent_hook_runs_before_frightened_decreases`. Verify `cargo test -p rules -- an_open_turn_s_end_is_not_the_next_end a_closed_turn_s_next_end_removes_it ending_the_turn_lowers_frightened_and_leaves_hit_points the_persistent_hook_runs_before_frightened_decreases ending_early_drops_leftover_actions_and_keeps_the_reaction` passes

- [ ] 2.11 Add `frightened_2_lowers_the_attack_total_by_2`, `frightened_2_and_enfeebled_2_stay_at_minus_2`, `clumsy_2_does_not_lower_a_strength_melee_attack`, `clumsy_2_lowers_a_dexterity_melee_attack`, and `enfeebled_2_does_not_lower_a_dexterity_melee_attack`. Verify `cargo test -p rules -- frightened_2_lowers_the_attack_total_by_2 frightened_2_and_enfeebled_2_stay_at_minus_2 clumsy_2_does_not_lower_a_strength_melee_attack clumsy_2_lowers_a_dexterity_melee_attack enfeebled_2_does_not_lower_a_dexterity_melee_attack` passes

- [ ] 2.12 Add `clumsy_2_lowers_a_ranged_attack`, `enfeebled_2_lowers_strength_melee_damage`, `enfeebled_2_does_not_lower_ranged_damage`, `frightened_2_on_the_target_changes_the_degree`, `frightened_2_and_sickened_1_do_not_sum_on_armor_class`, and `frightened_2_and_off_guard_both_lower_armor_class`. Verify `cargo test -p rules -- clumsy_2_lowers_a_ranged_attack enfeebled_2_lowers_strength_melee_damage enfeebled_2_does_not_lower_ranged_damage frightened_2_on_the_target_changes_the_degree frightened_2_and_sickened_1_do_not_sum_on_armor_class frightened_2_and_off_guard_both_lower_armor_class armor_class_in_the_caller_list_does_not_change_the_result` passes

- [ ] 2.13 Add `frightened_2_and_sickened_1_lower_initiative_by_2` and `a_returned_turn_does_not_lower_frightened_again`. Verify `cargo test -p rules -- frightened_2_and_sickened_1_lower_initiative_by_2 a_returned_turn_does_not_lower_frightened_again perception_is_added_to_the_d20` passes

- [ ] 2.14 Add `delay_lowers_frightened_once` and `a_missed_return_ticks_the_round_duration_on_the_next_start` beside the delay tests. Verify `cargo test -p rules -- delay_lowers_frightened_once a_missed_return_ticks_the_round_duration_on_the_next_start return_leaves_temporary_hit_points_as_the_start_left_them a_lowered_shield_stays_down` passes

- [ ] 2.15 Add `frightened_2_lowers_fortitude_by_2`, `frightened_2_and_sickened_1_lower_will_by_2`, `clumsy_2_lowers_reflex_and_not_fortitude`, `enfeebled_2_does_not_lower_fortitude`, and `frightened_1_and_unconscious_reflex_stay_at_minus_4` beside the save tests. Verify `cargo test -p rules -- frightened_2_lowers_fortitude_by_2 frightened_2_and_sickened_1_lower_will_by_2 clumsy_2_lowers_reflex_and_not_fortitude enfeebled_2_does_not_lower_fortitude frightened_1_and_unconscious_reflex_stay_at_minus_4` passes

- [ ] 2.16 Extend `drop_prone_leaves_the_combatant_state` so the condition list is empty. Add `a_prone_condition_does_not_change_drop_prone` and `prone_does_not_stop_a_stride`. Verify `cargo test -p rules -- drop_prone_leaves_the_combatant_state a_prone_condition_does_not_change_drop_prone prone_does_not_stop_a_stride` passes

- [ ] 2.17 Add `a_strength_melee_profile_keeps_the_old_fields`, `a_fresh_combatant_has_no_conditions`, `a_stored_frightened_2_can_be_read`, and `removing_frightened_leaves_persistent_damage`. Update `the_public_names_are_this_set` and `the_public_method_set_is_unchanged` to the sets in the api-facade spec, including the names and `gain_persistent_damage` from add-resistances-weaknesses-persistent. Verify `cargo test -p rules -- a_strength_melee_profile_keeps_the_old_fields a_fresh_combatant_has_no_conditions a_stored_frightened_2_can_be_read removing_frightened_leaves_persistent_damage the_public_names_are_this_set the_public_method_set_is_unchanged` passes and `cargo test -p rules --doc` passes

## 3. Code

- [ ] 3.1 Add `crates/rules/src/conditions.rs` with the types, `try_new`, `apply`, `reduce`, `remove`, `start_tick`, `end_tick`, and `condition_modifiers` from design.md. Add `ConditionSlot::Instance` to the enum pull request #93 owns. Do not change `PersistentDamage`. Re-export `AttackBasis`, `ConditionDuration`, `ConditionError`, and `ConditionKind` from the crate root. `ConditionSlot` is already exported. Keep `#![forbid(unsafe_code)]`. Verify the tests from 2.1 through 2.8 pass and `crates/rules/Cargo.toml` has no dependencies

- [ ] 3.2 Add `ConditionId::OFF_GUARD`, `PRONE`, `CLUMSY`, and `ENFEEBLED` at 1, 5, 6, and 7. Point the private off-guard id at `OFF_GUARD`. Add the six `ModifierSource` variants and sequences from design.md, including both `source_ord` matches. Do not add `with_condition` checks for the new ids. Verify the tests from 2.4 through 2.6 pass and the old frightened, sickened, and unconscious modifier tests still pass

- [ ] 3.3 Add `basis` to `StrikeProfile` and keep `damage_type`. Set `AttackBasis::StrengthMelee` where this change adds a literal that does not name another basis, including the doctest. Add `apply_condition`, `reduce_condition`, and `remove_condition`. `apply_condition` overwrites `due` from the open turn and never writes `PersistentDamage`. Unknown bearer or clock is `UnknownCombatant` and stores nothing. Map `ConditionError` to `EncounterError::Condition`. Add `ReadyEncounter::Condition` and one arm in `ready_from_encounter`, `from_spend`, and `as_damage_overflow`. Leave `end_turn`'s dice argument and `gain_persistent_damage` in place. Verify the tests from 2.17 pass

- [ ] 3.4 Tick skipped dead clocks in `start_turn` before `open_at`. Tick the opening creature in `start_of_turn_step` before actions return, and skip that tick on `SkipStartHook`. Call the decrement immediately after the persistent hook from #93 returns, including on Delay, unless the slot is `Returned` or `SkipStartHook`. Do not move that hook and do not roll inside the decrement. Fold Perception modifiers into initiative before any d20. Fold attack, Armor Class, and damage modifiers into Strike. Fold the save's modifiers into `basic_save`. A modifier overflow draws no attack d20. Do not write conditions from Drop Prone, Stand, Stride, Step, or Take Cover. Verify the tests from 2.9 through 2.16 pass

- [ ] 3.5 Do not add a second persistent container or a second end-of-turn hook. A comment on the decrement cites Rules 2430 and says the persistent hook already ran. Verify `rg "Rules 2430" crates/rules/src` finds that comment and `rg "enum ConditionSlot" -A 8 crates/rules/src` still shows `PersistentDamage` unchanged beside `Instance`

- [ ] 3.6 In the code phase, add DEV-73 from this design to `docs/designer/rules-deviations.md`. Do not add DEV-59 through DEV-72. Those are reserved: skills 59 through 61, resistances 62 and 63, spells 64 through 72. DEV-29, DEV-31, DEV-32, and DEV-36 stay. DEV-33 to DEV-35 and DEV-47 to DEV-49 stay unused. The spec commit does not edit that file

## 4. Changelog and proof

- [ ] 4.1 Add one Added bullet for conditions with durations and one **BREAKING** Changed bullet for `ConditionSlot::Instance`, `StrikeProfile.basis`, the three methods, and `EncounterError::Condition`, both under `## [Unreleased]`, both linking this pull request. The bullet SHALL NOT say this change introduced `ConditionSlot` or `PersistentDamage`. Commit that before the archive commit. Do not add either bullet in the spec commit. Verify both bullets contain the pull request link

- [ ] 4.2 Run the AGENTS.md proof commands and verify each exits 0. Confirm `crates/rules/Cargo.toml` still has no dependencies
  - `cargo fmt --all --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo build --workspace`
  - `cargo test --workspace`
  - `npx @fission-ai/openspec@1.14.3 validate --all --strict`

## 5. Archive

- [ ] 5.1 This is the last commit, right before merge, and only after Architect and QA have both passed that head. Run `openspec archive add-conditions-with-durations --yes` with that exact commit subject. Check this box only in that commit. Do not archive the spec draft. Verify the change is under `openspec/changes/archive/` and is no longer active, and that the archive commit does not edit `CHANGELOG.md`
