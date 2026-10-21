# Tasks

## 1. Build and derive

- [x] 1.1 Add `crates/rules/src/combatant.rs` with private `CharacterBuild`, `try_new`, `Contribution`, `base_contribution`, `contributions`, `fold_contributions`, and `derive(&CharacterBuild) -> Creature`. Port `CombatantId(pub u32)` from `v1-reference` `crates/rules/src/encounter.rs` (`Display`, `Hash`, `PartialOrd`, `Ord`) but do not put it on the build. Verify `derive` takes only `&CharacterBuild` and `cargo test -p rules --lib` compiles
- [x] 1.2 Add `derive_is_deterministic`: two `derive` calls match, a clone matches, a rebuilt equal build matches, and a scripted d20 of 8 is unused. Cite Player Core Hit Points as the fixed statistic. Verify `cargo test -p rules derive_is_deterministic` passes
- [x] 1.3 Add `later_contribution_folds_after_the_base`: a base Armor Class of 18 plus a following contribution of +1 folds to 19, in that order, and Phase 1 `contributions` returns only the base. Verify `cargo test -p rules later_contribution_folds_after_the_base` passes
- [x] 1.4 Port `preparation_copies_fixture_numbers` from `v1-reference` `crates/rules/tests/procedures.rs` as `baked_statistics_stay_as_authored`. Drop `prepare_combatant`. Assert the derived Creature has HP 20, AC 18, Perception +5, Speed 25, attack bonus +7, damage 1d8+4, reach 5, and agile false, and that the build is unchanged. Cite Player Core Hit Points, Strike, and Speed. Verify `cargo test -p rules baked_statistics_stay_as_authored` passes
- [x] 1.5 Add `ShieldTemplate` and `shield_ac_bonus_is_not_rewritten_to_plus_two` for a +1 shield and a +2 shield on the derived Creature. Do not port `raise_shield_spends_one_action_and_adds_two_ac` from `v1-reference` `crates/rules/tests/procedures.rs`. Cite Player Core "Shield Statistics" and the buckler +1 example. Verify `cargo test -p rules shield_ac_bonus_is_not_rewritten_to_plus_two` passes and `rg raise_shield_spends_one_action_and_adds_two_ac crates/rules` finds nothing
- [x] 1.6 Add `stealth_dc_from_modifier` and `stealth_dc_from_modifier_is_ten_plus` (Stealth +7 derives 17; omitting it stores none). A Seek total uses a scripted natural 12 plus Perception +5 against DC 17 and leaves the Creature equal. Cite Player Core "Seek" and "Getting a DC from a Modifier". Verify `cargo test -p rules stealth_dc_from_modifier_is_ten_plus` passes
- [x] 1.7 Add `aid_check` and `aid_circumstance_bonus`, and `aid_check_degrees_use_the_fixed_critical_bonus` with scripted naturals 10, 18, 4, 1, and 20 against DC 15. Assert the Creature is unchanged. The test comment records the deviation: rule (crit +2 / master +3 / legendary +4), why (no rank on the baked modifier), substitute (always +2), intent (a crit still beats a success). Cite Player Core "Aid". Do not port `aid_success_gives_plus_one_including_a_diagonal_neighbor`, `aid_critical_success_gives_plus_two`, `aid_critical_failure_gives_minus_one`, `aid_failure_does_not_change_the_strike`, `two_aid_bonuses_keep_only_the_larger`, `aid_bonus_applies_beside_the_multiple_attack_penalty`, `spent_reaction_refuses_a_second_aid`, or `an_ally_who_is_not_adjacent_cannot_aid` from `v1-reference` `crates/rules/tests/procedures.rs`. Verify `cargo test -p rules aid_check_degrees_use_the_fixed_critical_bonus` passes and `rg aid_success_gives_plus_one crates/rules` finds nothing
- [x] 1.8 Add `template_does_not_store_aid_preparation`: `try_new` plus `derive` accepts aid modifier +6 with no prepared-to-help field. Verify `cargo test -p rules template_does_not_store_aid_preparation` passes and `rg prepared_to_help crates/rules` finds nothing

## 2. Combatant state

- [x] 2.1 Add `CombatantState` and `new_combatant_state`. Add `fresh_state_matches_and_mutation_does_not_change_creature`: two states for the same Creature and `CombatantId(7)` are equal, current Hit Points start at the Creature's maximum, temporary Hit Points are 0, the shield is not raised, and setting current Hit Points to 12 leaves the Creature equal. Verify `cargo test -p rules fresh_state_matches_and_mutation_does_not_change_creature` passes
- [x] 2.2 Add `any_roster_length_is_valid` for six states (`CombatantId(0)` through `CombatantId(5)`) and for `CombatantId(4000000000)`, all from one Creature, with no `[T; 4]` constructor. Verify `cargo test -p rules any_roster_length_is_valid` passes
- [x] 2.3 Add `validation_rejects_non_positive_hp_and_negative_speed`. `try_new` rejects HP 0, HP -1, and Speed -5, and accepts Speed 0 with HP 12. Cite Player Core "Getting Knocked Out" and "Movement". Verify `cargo test -p rules validation_rejects_non_positive_hp_and_negative_speed` passes
- [x] 2.4 Add `zero_hp_without_dying_rules_is_dead` and `zero_hp_with_dying_rules_enters_dying`. Omitting `uses_dying_rules` derives false. A scripted d8 of 8 on 1d8+1 from 9 current Hit Points sets the state to 0 and `Vital::Dead` with no dying when the flag is false, and `Vital::EnterDying` with no dying number when the flag is true. The Creature still has 9 Hit Points. A scripted d8 of 1 leaves the state at 7. Cite Player Core "Getting Knocked Out". Do not port `a_hit_that_drops_a_creature_sets_dying_and_wounded`, `a_critical_hit_that_drops_a_creature_sets_dying_two_plus_wounded`, `wounded_increases_the_next_dying_value`, `damage_while_dying_increases_dying_without_wounded`, `dying_four_kills_and_a_dead_party_slot_remains`, `recovery_checks_cover_each_degree`, or `massive_damage_kills_without_dying` from `v1-reference` `crates/rules/tests/procedures.rs`. Verify `cargo test -p rules zero_hp_` passes and `rg a_hit_that_drops_a_creature_sets_dying_and_wounded crates/rules` finds nothing

## 3. Crate surface

- [x] 3.1 Re-export `CharacterBuild`, `Creature`, `CombatantState`, `CombatantId`, `Contribution`, `MeleeStrike`, `ShieldTemplate`, and the public functions from `crates/rules/src/lib.rs`. Leave `#![forbid(unsafe_code)]` in place. Give every new public item a `///` comment, citing the Player Core section when the item states a rule. Verify `cargo test -p rules` passes and `baked_statistics_stay_as_authored` still cites the Player Core section

## 4. One test per scenario

- [x] 4.1 Add one `#[cfg(test)]` test for every scenario below that sections 1–2 do not already name. The name is the scenario title in snake_case, with a leading plus written as `plus` and a leading minus written as `minus`. Each test comment cites the Player Core section on that requirement. A scenario that rolls uses `ScriptedDice` with that scenario's values. A scenario that does not roll still builds a `ScriptedDice` and asserts it is unused. The ported v1 test stays; it does not replace these tests. Satisfy each name once:
  - `the_build_is_unchanged_by_derive`
  - `combat_does_not_write_the_build`
  - `calling_derive_twice_matches`
  - `a_clone_and_an_equal_build_match`
  - `phase_1_folds_only_the_base`
  - `a_later_contribution_is_folded_after_the_base`
  - `authored_numbers_are_kept`
  - `strike_fields_are_kept`
  - `the_agile_flag_is_kept`
  - `a_buckler_bonus_stays_plus_1`
  - `a_second_shield_keeps_its_own_bonus`
  - `no_shield_is_omitted`
  - `stealth_modifier_plus_10`
  - `seek_uses_the_derived_dc`
  - `omitted_stealth_dc_stores_nothing`
  - `success_grants_plus_1`
  - `critical_success_grants_plus_2`
  - `failure_grants_0`
  - `critical_failure_grants_minus_1`
  - `a_natural_20_uses_the_fixed_critical_bonus`
  - `no_prepared_flag_is_required`
  - `zero_hit_points_is_rejected`
  - `negative_hit_points_are_rejected`
  - `negative_speed_is_rejected`
  - `speed_zero_is_accepted`
  - `fresh_states_from_one_creature_match`
  - `mutating_state_leaves_the_creature_unchanged`
  - `two_ids_can_share_one_creature`
  - `more_than_four_states_are_valid`
  - `omitting_the_flag_defaults_to_false`
  - `without_the_flag_0_hit_points_is_death_on_the_state`
  - `with_the_flag_0_hit_points_enters_dying_on_the_state`
  - `damage_above_0_does_not_kill`

## 5. Proof and archive

- [x] 5.1 Run the AGENTS.md proof commands and verify each exits 0. Add no crate dependency; `crates/rules/Cargo.toml` stays dependency-free.
  - `cargo fmt --all --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo build --workspace`
  - `cargo test --workspace`
  - `npx @fission-ai/openspec@1.14.3 validate --all --strict`
- [x] 5.2 Archive this change with `openspec archive add-creature-template --yes` and verify it moves under `openspec/changes/archive/`
