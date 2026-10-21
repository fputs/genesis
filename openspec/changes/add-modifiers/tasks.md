# Tasks

## 1. Modifier construction

- [ ] 1.1 Add `crates/rules/src/modifiers.rs` with `BonusType`, `PenaltyType`, `SourceId`, `ConditionId`, `Modifier`, and `ModifierError::{UntypedBonus, Zero, Sign, Overflow}`. `Modifier::bonus` requires a value greater than 0. `Modifier::penalty` requires a value less than 0. Both return `Result`. There is no proficiency variant and no untyped-bonus variant. Re-export the public items from `crates/rules/src/lib.rs`. Add no crate dependency. Verify `cargo test -p rules --lib` compiles and `crates/rules/Cargo.toml` still has no dependencies
- [ ] 1.2 Add `an_untyped_bonus_is_rejected`, `a_zero_modifier_is_rejected`, `a_negative_bonus_is_rejected`, and `a_positive_penalty_is_rejected`. A +2 untyped bonus returns `ModifierError::UntypedBonus`. A circumstance 0 returns `ModifierError::Zero`. A circumstance bonus of -1 and a circumstance penalty of +1 each return `ModifierError::Sign`. None of them returns a modifier. Cite Player Core pg. 400 and pg. 442. Verify `cargo test -p rules an_untyped_bonus_is_rejected a_zero_modifier_is_rejected a_negative_bonus_is_rejected a_positive_penalty_is_rejected` passes
- [ ] 1.3 Add `an_item_bonus_is_accepted` and `proficiency_is_not_a_modifier_type`. Two item bonuses, +2 and +1, from different sources total +2. The listed bonus types are circumstance, item, and status, and the listed penalty types are circumstance, item, status, and untyped. Verify `cargo test -p rules an_item_bonus_is_accepted proficiency_is_not_a_modifier_type` passes

## 2. Fold

- [ ] 2.1 Add `total` and `dc_from_total`. `total` reduces repeated conditions, then same-source modifiers, then keeps the highest bonus and worst penalty of each type and adds every untyped penalty with `checked_add`. When different sources tie for that bonus or penalty, the reported modifier has the lowest sequence number. Overflow returns `ModifierError::Overflow` and does not saturate. `dc_from_total` is 10 plus the total and does not clamp. Verify `cargo test -p rules two_circumstance_bonuses_keep_the_highest` passes
- [ ] 2.2 Add `two_circumstance_bonuses_keep_the_highest`, `plus_2_circumstance_with_minus_2_circumstance_equals_0`, `frightened_2_and_sickened_1_equal_minus_2`, `untyped_penalties_stack`, `the_same_modifiers_total_twice`, `a_tied_highest_bonus_reports_the_lowest_sequence`, and `a_tied_worst_penalty_reports_the_lowest_sequence`. Two +2 circumstance bonuses with sequences 4 and 1 report sequence 1. Two -2 circumstance penalties with sequences 7 and 3 report sequence 3. Cite Player Core pg. 400, frightened pg. 444, and sickened pg. 446. Verify `cargo test -p rules two_circumstance_bonuses_keep_the_highest plus_2_circumstance_with_minus_2_circumstance_equals_0 frightened_2_and_sickened_1_equal_minus_2 untyped_penalties_stack the_same_modifiers_total_twice a_tied_highest_bonus_reports_the_lowest_sequence a_tied_worst_penalty_reports_the_lowest_sequence` passes
- [ ] 2.3 Add `a_duplicate_same_source_untyped_penalty_counts_once`, `a_tied_source_keeps_the_newer_modifier`, `a_higher_level_beats_a_higher_value`, and `a_higher_value_beats_a_newer_lower_value`. Level wins, otherwise the algebraically higher value, otherwise the greater sequence number, before the sum. Cite Player Core pg. 398. Verify `cargo test -p rules a_duplicate_same_source_untyped_penalty_counts_once a_tied_source_keeps_the_newer_modifier a_higher_level_beats_a_higher_value a_higher_value_beats_a_newer_lower_value` passes
- [ ] 2.4 Add `a_repeated_condition_keeps_its_highest_rank` and `off_guard_from_two_causes_applies_once`. Frightened 1 and frightened 2 leave one status penalty of -2 from rank 2. Off-guard from prone and from flanking is -2 once. Cite Player Core pg. 442. Verify `cargo test -p rules a_repeated_condition_keeps_its_highest_rank off_guard_from_two_causes_applies_once` passes
- [ ] 2.5 Add `a_negative_total`, `a_dc_below_10`, `a_flat_check_ignores_modifiers`, `insertion_order_does_not_change_the_total`, and `an_overflowing_modifier_sum_returns_an_error`. A +1 circumstance bonus and untyped penalties of -7 total -6. That total's DC is 4. A flat check at DC 11 with a scripted natural 10 and a +9 modifier total is still 10 and a failure. Either insertion order of status +1 and circumstance -2 totals -1. Untyped -2147483648 and -1 return `ModifierError::Overflow` and do not return a saturated total. Cite Player Core pg. 401 and pg. 400. Verify `cargo test -p rules a_negative_total a_dc_below_10 a_flat_check_ignores_modifiers insertion_order_does_not_change_the_total an_overflowing_modifier_sum_returns_an_error` passes

## 3. Phase 1 producers

- [ ] 3.1 Add `attack_modifiers`, `ac_modifiers`, and `perception_modifiers`. `attack_modifiers` calls `multiple_attack_penalty` and does not contain its own -5/-10 or -4/-8 table. A zero multiple attack penalty or a zero Aid result is omitted. Prone is circumstance -2. Cover is circumstance +1, +2, or +4. Off-guard is circumstance -2. Unconscious is status -4 on Armor Class and on Perception, in separate slices. No function reads `shield_ac_bonus`. Verify `rg -n "multiple_attack_penalty" crates/rules/src/modifiers.rs` shows the call and `rg -n -- "-10|-8" crates/rules/src/modifiers.rs` finds no penalty table
- [ ] 3.2 Add `map_minus_5_plus_range_minus_2_equals_minus_7`, `map_combines_with_circumstance_and_status`, `agile_map_stays_the_existing_untyped_penalty`, `an_aid_success_adds_plus_1_circumstance`, and `an_aid_failure_adds_no_modifier`. The -5 and the -4 come from `multiple_attack_penalty`. A synthetic untyped range penalty of -2 makes the untyped total -7. MAP -5, circumstance -2, and status -4 include -11. Aid +1 is kept and Aid 0 adds nothing. Do not implement the Aid action. Cite Player Core pg. 402 and pg. 416. Verify `cargo test -p rules map_minus_5_plus_range_minus_2_equals_minus_7 map_combines_with_circumstance_and_status agile_map_stays_the_existing_untyped_penalty an_aid_success_adds_plus_1_circumstance an_aid_failure_adds_no_modifier` passes
- [ ] 3.3 Add `cover_is_a_circumstance_bonus`, `off_guard_is_minus_2_circumstance_to_armor_class`, `unconscious_is_minus_4_status_to_armor_class_and_perception`, `a_shield_bonus_is_not_applied_here`, `a_prone_attacker_and_an_off_guard_defender_both_apply`, `damage_keeps_the_highest_circumstance_bonus`, and `phase_1_produces_no_item_modifier`. Cite Player Core pg. 424, pg. 445, pg. 446, and pg. 406. Verify `cargo test -p rules cover_is_a_circumstance_bonus off_guard_is_minus_2_circumstance_to_armor_class unconscious_is_minus_4_status_to_armor_class_and_perception a_shield_bonus_is_not_applied_here a_prone_attacker_and_an_off_guard_defender_both_apply damage_keeps_the_highest_circumstance_bonus phase_1_produces_no_item_modifier` passes
- [ ] 3.4 Implement `attack_check_total` by folding modifiers so its signature and the existing dice-and-checks totals stay. A 0 penalty argument is omitted. Verify `cargo test -p rules map_stacks_with_a_circumstance_penalty map_stacks_with_another_untyped_penalty` passes

## 4. Creature fold and damage

- [ ] 4.1 Add `FoldError::Statistics`. After a successful sum, `fold_contributions` rejects Hit Points of 0 or less and a negative Speed with that variant and does not return a Creature. Speed 0 is accepted. `try_new` still returns `TemplateError::NonPositiveHitPoints` and `TemplateError::NegativeSpeed`. `derive` of a valid build stays equal to folding its base. Verify `cargo test -p rules derive_agrees_with_folding_the_base speed_zero_is_accepted` passes
- [ ] 4.2 Add `a_folded_zero_hit_points_is_rejected`, `a_folded_negative_speed_is_rejected`, and `folded_speed_zero_is_accepted`. Update `zero_hit_points_is_rejected`, `negative_hit_points_are_rejected`, and `negative_speed_is_rejected` so the assertion names `TemplateError::NonPositiveHitPoints` or `TemplateError::NegativeSpeed`. Do not delete those tests. Cite Player Core pg. 410 and pg. 420. Verify `cargo test -p rules a_folded_zero_hit_points_is_rejected a_folded_negative_speed_is_rejected folded_speed_zero_is_accepted zero_hit_points_is_rejected negative_hit_points_are_rejected negative_speed_is_rejected` passes
- [ ] 4.3 Add `damage_dealt`, and change `apply_dice_damage` so it returns `Result<(), HpError>`. Fold the modifiers, and when that total is 0 or below, deal 1 before doubling a critical hit. A rolled 4 and a penalty of -6 deal 1, or 2 when critical. Overflow returns `HpError::Overflow` or `ModifierError::Overflow` and does not change Hit Points. Do not apply resistance. Add `apply_healing`, which caps current Hit Points at the maximum and returns `HpError::Overflow` without changing the state when the add does not fit. Cite Player Core pg. 406 and pg. 407. Verify `cargo test -p rules penalties_reduce_damage_to_1 minimum_critical_damage_is_2` passes
- [ ] 4.4 Add `penalties_reduce_damage_to_1`, `minimum_critical_damage_is_2`, `an_overflowing_damage_total_returns_an_error`, `damage_and_healing_keep_current_hit_points_in_range`, and `healing_that_does_not_fit_returns_an_error`. Keep `a_negative_total_does_not_raise_hit_points`, `healing_past_i32_max_leaves_hit_points_unchanged`, and `damage_past_i32_floors_hit_points_at_0`, and change their assertions to this rule: a negative total deals 1, and an overflowing total leaves Hit Points unchanged and returns the overflow error. Do not delete them. Verify `cargo test -p rules penalties_reduce_damage_to_1 minimum_critical_damage_is_2 an_overflowing_damage_total_returns_an_error damage_and_healing_keep_current_hit_points_in_range healing_that_does_not_fit_returns_an_error a_negative_total_does_not_raise_hit_points` passes

## 5. One test per scenario

- [ ] 5.1 Add one `#[cfg(test)]` test for every scenario below that sections 1–4 do not already name. The name is the scenario title in snake_case. A leading plus is written `plus` and a leading minus is written `minus`. Each test comment cites the Player Core section on that requirement. A scenario that rolls uses `ScriptedDice`. A scenario that does not roll still builds a `ScriptedDice` and asserts it is unused, except a test that only lists types or rejects construction. Satisfy each name once:
  - `an_item_bonus_is_accepted`
  - `proficiency_is_not_a_modifier_type`
  - `phase_1_produces_no_item_modifier`
  - `two_circumstance_bonuses_keep_the_highest`
  - `plus_2_circumstance_with_minus_2_circumstance_equals_0`
  - `frightened_2_and_sickened_1_equal_minus_2`
  - `untyped_penalties_stack`
  - `map_minus_5_plus_range_minus_2_equals_minus_7`
  - `an_overflowing_modifier_sum_returns_an_error`
  - `the_same_modifiers_total_twice`
  - `a_tied_highest_bonus_reports_the_lowest_sequence`
  - `a_tied_worst_penalty_reports_the_lowest_sequence`
  - `an_untyped_bonus_is_rejected`
  - `a_zero_modifier_is_rejected`
  - `a_negative_bonus_is_rejected`
  - `a_positive_penalty_is_rejected`
  - `a_duplicate_same_source_untyped_penalty_counts_once`
  - `a_tied_source_keeps_the_newer_modifier`
  - `a_higher_level_beats_a_higher_value`
  - `a_higher_value_beats_a_newer_lower_value`
  - `a_repeated_condition_keeps_its_highest_rank`
  - `off_guard_from_two_causes_applies_once`
  - `a_negative_total`
  - `a_dc_below_10`
  - `a_flat_check_ignores_modifiers`
  - `insertion_order_does_not_change_the_total`
  - `a_prone_attacker_and_an_off_guard_defender_both_apply`
  - `map_combines_with_circumstance_and_status`
  - `agile_map_stays_the_existing_untyped_penalty`
  - `an_aid_success_adds_plus_1_circumstance`
  - `an_aid_failure_adds_no_modifier`
  - `cover_is_a_circumstance_bonus`
  - `off_guard_is_minus_2_circumstance_to_armor_class`
  - `unconscious_is_minus_4_status_to_armor_class_and_perception`
  - `a_shield_bonus_is_not_applied_here`
  - `damage_keeps_the_highest_circumstance_bonus`
  - `zero_hit_points_is_rejected`
  - `negative_hit_points_are_rejected`
  - `negative_speed_is_rejected`
  - `speed_zero_is_accepted`
  - `a_folded_zero_hit_points_is_rejected`
  - `a_folded_negative_speed_is_rejected`
  - `folded_speed_zero_is_accepted`
  - `damage_and_healing_keep_current_hit_points_in_range`
  - `healing_that_does_not_fit_returns_an_error`
  - `penalties_reduce_damage_to_1`
  - `minimum_critical_damage_is_2`
  - `an_overflowing_damage_total_returns_an_error`

## 6. Changelog, proof, and archive

- [ ] 6.1 Add one bullet under `### Added` in `## [Unreleased]` of root `CHANGELOG.md`. The bullet names modifier stacking and the damage minimum of 1 before critical doubling, and it links to the implementation pull request. Verify the new bullet is under `### Added` and contains that link
- [ ] 6.2 Run the AGENTS.md proof commands and verify each exits 0. Fallible functions return `Result`, sums use checked arithmetic, overflow is a typed error, library code does not panic, and no dependency was added.
  - `cargo fmt --all --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo build --workspace`
  - `cargo test --workspace`
  - `npx @fission-ai/openspec@1.14.3 validate --all --strict`
- [ ] 6.3 As the final commit of the implementation pull request, run `openspec archive add-modifiers --yes` and verify the change is no longer active under `openspec/changes/` and sits under `openspec/changes/archive/`
