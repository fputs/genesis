# Tasks

Each scenario has one test. The name is the title in snake_case. Spaces, hyphens, and apostrophes become `_`. Other punctuation is dropped. Use `ScriptedDice`, and assert an unused face when the scenario says so. Cite that requirement's Player Core section. The spec commit does not edit `docs/designer/`. Task 3.5 does, in the code phase. No file on `v1-reference` is named, so none is ported.

Write section 2 before filling the production bodies. The tests call the items in design.md. They may share a commit with section 3. Do not start section 2 until task 1.2 is true.

## 1. Spec

- [x] 1.1 These files are the behavior. The spec commit adds no Rust, no test, and no changelog bullet. Verify `npx @fission-ai/openspec@1.14.3 validate --all --strict` exits 0

- [x] 1.2 Do not start section 2 until this pull request has Evan's review `spec-approved: <sha>` and that sha is an ancestor of HEAD. Designer rules on the spec in this pull request

## 2. Tests

- [x] 2.1 Add `fire_and_slashing_are_different_instances`, `immunity_to_fire_ignores_10_fire`, `immunity_beats_weakness_to_the_same_type`, `immunity_to_fire_does_not_affect_slashing`, and `weakness_5_fire_raises_7_fire_to_12`. Verify `cargo test -p rules -- fire_and_slashing_are_different_instances immunity_to_fire_ignores_10_fire immunity_beats_weakness_to_the_same_type immunity_to_fire_does_not_affect_slashing weakness_5_fire_raises_7_fire_to_12` passes

- [x] 2.2 Add `two_fire_weaknesses_use_10_once`, `a_slashing_weakness_does_not_raise_fire`, `weakness_then_resistance_leaves_3`, `resistance_10_fire_reduces_8_fire_to_0`, and `two_fire_resistances_use_10`. Verify `cargo test -p rules -- two_fire_weaknesses_use_10_once a_slashing_weakness_does_not_raise_fire weakness_then_resistance_leaves_3 resistance_10_fire_reduces_8_fire_to_0 two_fire_resistances_use_10` passes

- [x] 2.3 Add `slashing_resistance_does_not_reduce_fire`, `a_floored_1_is_resisted_to_0`, `a_critical_6_fire_with_weakness_5_deals_17`, `a_critical_6_fire_with_resistance_5_deals_7`, and `the_same_fire_adjustment_matches`. Verify `cargo test -p rules -- slashing_resistance_does_not_reduce_fire a_floored_1_is_resisted_to_0 a_critical_6_fire_with_weakness_5_deals_17 a_critical_6_fire_with_resistance_5_deals_7 the_same_fire_adjustment_matches` passes

- [x] 2.4 Add `weakness_on_the_maximum_amount_overflows`, `two_maximum_persistent_amounts_overflow_the_end_step`, `weakness_0_is_rejected`, `a_negative_resistance_is_rejected`, and `the_strike_type_and_the_defenses_are_kept`. Verify `cargo test -p rules -- weakness_on_the_maximum_amount_overflows two_maximum_persistent_amounts_overflow_the_end_step weakness_0_is_rejected a_negative_resistance_is_rejected the_strike_type_and_the_defenses_are_kept` passes

- [x] 2.5 Add `a_fire_strike_adds_weakness_5`, `empty_defenses_deal_the_rolled_total`, `a_miss_does_not_apply_weakness`, `a_readied_fire_hit_adds_weakness_5`, `a_reactive_fire_hit_subtracts_resistance_5`, `dying_1_and_immune_fire_8_stays_dying_1`, and `dying_1_and_resisted_fire_8_stays_dying_1`. Verify `cargo test -p rules -- a_fire_strike_adds_weakness_5 empty_defenses_deal_the_rolled_total a_miss_does_not_apply_weakness a_readied_fire_hit_adds_weakness_5 a_reactive_fire_hit_subtracts_resistance_5 dying_1_and_immune_fire_8_stays_dying_1 dying_1_and_resisted_fire_8_stays_dying_1` passes

- [x] 2.6 Add `fire_resistance_does_not_reduce_a_basic_save`, `a_fresh_creature_has_no_persistent_damage`, `fire_5_is_stored`, `a_critical_persistent_fire_2_is_stored_as_4`, `a_non_positive_amount_is_rejected`, `an_unknown_combatant_is_rejected`, and `a_dead_combatant_cannot_gain_it`. Verify `cargo test -p rules -- fire_resistance_does_not_reduce_a_basic_save a_fresh_creature_has_no_persistent_damage fire_5_is_stored a_critical_persistent_fire_2_is_stored_as_4 a_non_positive_amount_is_rejected an_unknown_combatant_is_rejected a_dead_combatant_cannot_gain_it` passes

- [x] 2.7 Add `fire_8_replaces_fire_5`, `fire_5_does_not_replace_fire_8`, `an_equal_fire_amount_stays`, `fire_and_bleed_are_both_stored`, and `fire_5_deals_5_then_waits_on_the_flat_check`. Verify `cargo test -p rules -- fire_8_replaces_fire_5 fire_5_does_not_replace_fire_8 an_equal_fire_amount_stays fire_and_bleed_are_both_stored fire_5_deals_5_then_waits_on_the_flat_check` passes

- [x] 2.8 Add `fire_and_bleed_deal_8_as_one_blow`, `two_types_while_dying_1_increase_dying_once`, `persistent_damage_to_0_sets_dying_1`, `without_dying_rules_0_hit_points_is_death`, and `temporary_hit_points_are_spent_once`. Verify `cargo test -p rules -- fire_and_bleed_deal_8_as_one_blow two_types_while_dying_1_increase_dying_once persistent_damage_to_0_sets_dying_1 without_dying_rules_0_hit_points_is_death temporary_hit_points_are_spent_once` passes

- [x] 2.9 Add `persistent_slashing_leaves_the_shield_and_the_reaction`, `the_other_creature_s_fire_does_not_tick`, `resistance_10_leaves_hit_points_unchanged`, `immune_fire_5_still_rolls_the_flat_check`, `a_natural_15_ends_fire_after_the_damage`, and `a_natural_14_keeps_fire`. Verify `cargo test -p rules -- persistent_slashing_leaves_the_shield_and_the_reaction the_other_creature_s_fire_does_not_tick resistance_10_leaves_hit_points_unchanged immune_fire_5_still_rolls_the_flat_check a_natural_15_ends_fire_after_the_damage a_natural_14_keeps_fire` passes

- [x] 2.10 Add `a_natural_20_ends_fire`, `a_natural_1_keeps_fire`, `bleed_is_checked_before_fire`, `massive_persistent_damage_draws_no_flat_check`, `dying_3_and_persistent_bleed_2_draws_no_flat_check`, and `a_dead_creature_s_end_step_draws_nothing`. Verify `cargo test -p rules -- a_natural_20_ends_fire a_natural_1_keeps_fire bleed_is_checked_before_fire massive_persistent_damage_draws_no_flat_check dying_3_and_persistent_bleed_2_draws_no_flat_check a_dead_creature_s_end_step_draws_nothing` passes

- [x] 2.11 Add `a_heal_to_maximum_removes_bleed_and_keeps_fire`, `a_short_heal_leaves_bleed`, `the_end_step_rolls_one_d20_and_spends_no_action`, `a_fire_strike_does_not_block`, `immunity_to_slashing_does_not_block`, `resistance_to_0_does_not_spend_the_reaction`, and `resistance_is_applied_before_hardness`. Verify `cargo test -p rules -- a_heal_to_maximum_removes_bleed_and_keeps_fire a_short_heal_leaves_bleed the_end_step_rolls_one_d20_and_spends_no_action a_fire_strike_does_not_block immunity_to_slashing_does_not_block resistance_to_0_does_not_spend_the_reaction resistance_is_applied_before_hardness` passes

- [x] 2.12 Update `strike_fields_are_kept` and `the_agile_flag_is_kept` for the damage type. Update `ending_early_drops_leftover_actions_and_keeps_the_reaction` and `end_before_a_start_returns_an_error` so `end_turn` takes the die and still draws nothing. Add `non_positive_persistent_keeps_its_own_arms` beside the existing conversion tests. Verify `cargo test -p rules -- strike_fields_are_kept the_agile_flag_is_kept ending_early_drops_leftover_actions_and_keeps_the_reaction end_before_a_start_returns_an_error non_positive_persistent_keeps_its_own_arms` passes

## 3. Code

- [x] 3.1 Add `DamageType`, `Weakness`, `Resistance`, `DamageDefenses`, and `adjust_damage` in `crates/rules/src`. Re-export the four types from the crate root. Do not re-export `adjust_damage`. Implement immunity, then the highest weakness, then the highest resistance, minimum 0. Keep `#![forbid(unsafe_code)]`. Verify the tests from 2.1 through 2.3 pass and `crates/rules/Cargo.toml` has no dependencies

- [x] 3.2 Add `damage_type` to `MeleeStrike` and `StrikeProfile`. Add `defenses` to `BuildParts`, `Contribution`, and `Creature`. `derive` copies them and does not roll. `try_new` rejects a value below 1 with `TemplateError::InvalidDefenses`. Set `DamageType::Slashing` and empty defenses on every existing literal. `ConditionSlot` becomes the design.md enum. Verify the tests from 2.4 pass and the old creature tests still pass

- [x] 3.3 Call `adjust_damage` from Strike, Reactive Strike, and a readied Strike after `damage_dealt` and before Shield Block. Skip the block when the type is not physical, the target is immune, or the adjusted amount is 0. An adjusted amount of 0 skips the blow. A physical request that is not immune still returns `NotRaised` before the roll. Do not call `adjust_damage` from `basic_save`, `apply_resolved_blow`, or `apply_dice_damage`. Verify the tests from 2.5 and the new shield tests in 2.11 pass

- [x] 3.4 Add `Encounter::gain_persistent_damage` and the dice argument on `end_turn`. Store the amount the caller passes. A critical hit's caller passes twice the listed persistent amount, so persistent Fire 2 is stored as 4, not 6. Do not double inside `gain_persistent_damage`. Implement the one ordinary blow, the DC 15 flat checks in `DamageType` order, death clearing the slots, and bleed ending at full Hit Points. Map `NonPositivePersistent` through the three arms in design.md. Update every `end_turn` call. Verify the tests from 2.6 through 2.12 pass

- [x] 3.5 In the code phase, add DEV-62 and DEV-63 to `docs/designer/rules-deviations.md`. Amend the persistent-damage clause of DEV-09 and DEV-29. In that edit, note the reserved ranges: DEV-59 through DEV-61 are `add-skills` (pull request 91), and DEV-64 through DEV-72 are `add-spells-and-areas` (pull request 92). DEV-73 and later stay with `add-conditions-with-durations`. DEV-33 through DEV-35 and DEV-47 through DEV-49 stay unused. The spec commit does not edit that file

## 4. Changelog and proof

- [x] 4.1 Add one Added bullet for damage types, resistance, weakness, immunity, and persistent damage, and one **BREAKING** Changed bullet for the strike type, defenses, `end_turn`, and `ConditionSlot`, both under `## [Unreleased]`, both linking this pull request. Commit that before the archive commit. Do not add either bullet in the spec commit. Verify both bullets contain the pull request link

- [x] 4.2 Run the AGENTS.md proof commands and verify each exits 0. Confirm `crates/rules/Cargo.toml` still has no dependencies
  - `cargo fmt --all --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo build --workspace`
  - `cargo test --workspace`
  - `npx @fission-ai/openspec@1.14.3 validate --all --strict`

## 5. Archive

- [ ] 5.1 This is the last commit, right before merge, and only after Architect and QA have both passed that head. Run `openspec archive add-resistances-weaknesses-persistent --yes` with that exact commit subject. Check this box only in that commit. Do not archive the spec draft. Verify the change is under `openspec/changes/archive/` and is no longer active, and that the archive commit does not edit `CHANGELOG.md`
