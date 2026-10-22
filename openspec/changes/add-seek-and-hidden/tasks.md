# Tasks

Each scenario in `specs/seek-and-hidden/spec.md` has exactly one test. The test name is the scenario title in snake_case. Spaces, hyphens, and apostrophes become `_`. `Off-guard` in a title contributes `off_guard`. A test comment cites the Player Core section on that requirement. A test that does not roll builds a `ScriptedDice` and asserts the held face is unused. Do not edit `check.rs`, `modifiers.rs`, `combatant.rs`, `positioning.rs`, `encounter.rs`, or anything under `docs/designer/`. Do not declare `OffGuard` or `Cover`. Import `OffGuard` from positioning and `Cover` from modifiers. Do not start these tasks until `OffGuard` is public in `crates/rules`. Add no dependency. Do not add a field to `Creature` or `CombatantState`. This change names no `v1-reference` file, so none is ported.

## 1. Module

- [ ] 1.1 Start only after `OffGuard` from `add-positioning` is public. Add `crates/rules/src/detection.rs` and `mod detection` in `lib.rs`. Re-export the public items from design.md. Keep `#![forbid(unsafe_code)]` and leave `crates/rules/Cargo.toml` with no dependencies. Fallible functions return `Result<_, DetectionError>`. No `unwrap`, `expect`, `panic!`, or unchecked indexing. Use `checked_add` and `checked_sub`. Give every public item a `///` comment citing the Player Core section when it states a rule. Derive `Debug, Clone, Copy, PartialEq, Eq` on the type sketches in design.md. Verify `cargo test -p rules --lib` compiles and `crates/rules/Cargo.toml` has no `dependencies` table entries

## 2. Relation

- [ ] 2.1 Add the caller-owned `Relation`. A missing pair is observed and not concealed. `store` returns `SameCreature` or `NoStealthDc` and writes nothing. Concealed is a flag and does not replace the detection. Cite Player Core "Detecting Creatures" and "Concealed". Verify `cargo test -p rules -- the_same_relation_twice_matches one_subject_is_hidden_from_one_observer_and_observed_by_another a_missing_relation_is_observed the_observer_and_the_subject_cannot_be_the_same an_observed_creature_can_be_concealed a_hidden_creature_can_be_concealed hidden_without_a_stealth_dc_is_rejected undetected_without_a_stealth_dc_is_rejected concealed_without_a_stealth_dc_is_stored` passes

## 3. Targeting flat checks

- [ ] 3.1 Call `flat_check` at DC 5 or DC 11, and `flat_checks_for_same_thing` with 5 and 11 when the subject is hidden and concealed. A critical success affects the target. A critical failure does not. The numeric degree is returned. An observed creature and an area effect draw no die. `CannotTarget` draws no die (proposed DEV-39). Cite Player Core "Concealed", "Hidden", "Flat Checks", and "Undetected". Verify `cargo test -p rules -- the_same_flat_check_twice_matches a_natural_5_at_dc_5_affects_the_target a_natural_4_at_dc_5_does_not_affect_the_target a_natural_1_at_dc_5_does_not_affect_the_target a_natural_20_at_dc_5_affects_the_target a_natural_11_at_dc_11_affects_the_target a_natural_10_at_dc_11_does_not_affect_the_target a_natural_20_at_dc_11_affects_the_target hidden_and_concealed_fail_one_dc_11_roll an_observed_target_rolls_nothing an_area_effect_rolls_nothing an_undetected_target_cannot_be_chosen` passes

## 4. Off-guard

- [ ] 4.1 `off_guard_against` returns positioning's `OffGuard::Yes` for hidden and for undetected, and returns `other` otherwise. Pass that one value to `ac_modifiers(None, yes, false)` and `total`. Do not add a second −2. The unconscious argument is `false`. Cite Player Core "Hidden", "Undetected", and "Off-Guard". Verify `cargo test -p rules -- hidden_is_off_guard_at_minus_2 undetected_is_off_guard_at_minus_2 concealed_is_not_off_guard hidden_and_concealed_is_still_off_guard hidden_plus_another_off_guard_is_still_minus_2` passes

## 5. Seek

- [ ] 5.1 Seek costs 1 action with `DetectionTrait::Concentrate`. Reject, in order, `DuplicateId`, `SameCreature`, `AreaTooLarge` (DEV-08), then `Overflow` from `CheckModifier::new`, and draw no die. Perception modifier `2147483647` is `Overflow`. An area of 35 by 5 by 5 feet with a duplicate subject is `DuplicateId`, not `AreaTooLarge`. The seeker listed twice is `DuplicateId`, not `SameCreature`. The seeker listed once in that 35-foot area is `SameCreature`. Cite Player Core "Seek". Verify `cargo test -p rules -- seek_reports_one_concentrate_action an_area_beyond_30_feet_rolls_nothing a_duplicate_subject_rolls_nothing the_seeker_as_a_subject_rolls_nothing the_seeker_listed_twice_rolls_nothing a_perception_modifier_that_does_not_fit_rolls_nothing` passes
- [ ] 5.2 Roll one `resolve_check` and compare it with each hidden or undetected Stealth DC, in ascending `CombatantId`. Seek lists a critical success and a success, and no critical failure. Use `fallback_unlisted_critical(degree, true, false)` for the effect. Copy the concealed flag. Invisibility does not keep the creature hidden (proposed DEV-40). If nobody qualifies, succeed and draw no die. Cite Player Core "Seek". Verify `cargo test -p rules -- the_same_seek_twice_matches a_critical_success_moves_hidden_to_observed a_critical_success_moves_undetected_to_observed a_success_moves_hidden_to_observed a_failure_leaves_undetected a_critical_failure_leaves_hidden a_natural_20_moves_hidden_to_observed a_success_leaves_the_concealed_flag_set one_perception_roll_meets_one_stealth_dc_and_misses_another observed_and_a_missing_stealth_dc_are_not_compared only_an_observed_subject_rolls_nothing` passes

## 6. Hide

- [ ] 6.1 Reject, in order, `DuplicateId`, `SameCreature`, `NoStealthDc`, then `RequirementNotMet`, and draw no die. Lesser cover does not qualify. No Stealth DC with lesser cover is `NoStealthDc`, not `RequirementNotMet`. Hiding from yourself twice is `DuplicateId`, not `SameCreature`. An observer without cover stays observed when another observer qualifies. Cite Player Core "Hide". Verify `cargo test -p rules -- lesser_cover_is_not_enough no_cover_and_not_concealed_is_not_enough hide_without_a_stealth_dc_rolls_nothing hiding_from_yourself_rolls_nothing hiding_from_yourself_twice_rolls_nothing one_observer_without_cover_is_left_observed a_duplicate_observer_rolls_nothing` passes
- [ ] 6.2 The modifier is the Stealth DC minus 10, plus 2, plus 4, or plus 0, through `CheckModifier`. The DC is `DcModifier` of the Perception modifier. Perception `2147483638` is `Overflow` and draws no die. Hide lists a success only. A critical success uses that success entry. An observed subject becomes hidden. Hidden and undetected stay. The cost is 1 action and the trait list is empty. The degree is on the roll (proposed DEV-38). Cite Player Core "Hide" and "Getting a DC from a Modifier". Verify `cargo test -p rules -- the_same_hide_twice_matches standard_cover_and_a_natural_6_hides standard_cover_and_a_natural_5_stays_observed a_critical_success_on_hide_hides greater_cover_adds_4 concealment_alone_and_a_natural_8_hides concealment_alone_and_a_natural_7_stays_observed one_stealth_roll_meets_one_perception_dc_and_misses_another a_success_keeps_hidden a_success_keeps_undetected a_perception_dc_that_does_not_fit_rolls_nothing` passes

## 7. Cover loss and Strike

- [ ] 7.1 `refresh` ends hidden when the subject has neither standard nor greater cover and is not concealed. Lesser cover does not keep hidden. Undetected stays. `note_strike` reads `OffGuard::Yes` before it changes the relation, then sets only that observer to observed and keeps the concealed flag. Another observer stays hidden. `note_action` for Stride leaves hidden (proposed DEV-41). An undetected Strike stays undetected and off-guard. Neither function rolls. Cite Player Core "Hide" and "Hidden". Verify `cargo test -p rules -- losing_cover_and_concealment_ends_hidden lesser_cover_does_not_keep_a_creature_hidden concealment_keeps_a_hidden_creature_hidden losing_cover_does_not_end_undetected a_hidden_strike_is_off_guard_then_that_observer_observes a_stride_does_not_end_hidden an_undetected_strike_stays_undetected_and_off_guard` passes

## 8. Changelog and proof

- [ ] 8.1 Add one bullet under `## [Unreleased]` `### Added` in `CHANGELOG.md` describing Seek, Hide, the targeting flat checks, and hidden off-guard, and link that bullet to the implementation pull request. Put that edit in a commit before the archive commit. Do not add the bullet in the spec-only pull request. Verify the new bullet is under `### Added`, contains the pull request link, and that the archive commit does not modify `CHANGELOG.md`
- [ ] 8.2 Run the AGENTS.md proof commands and verify each exits 0. Confirm `crates/rules/Cargo.toml` still has no dependencies
  - `cargo fmt --all --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo build --workspace`
  - `cargo test --workspace`
  - `npx @fission-ai/openspec@1.14.3 validate --all --strict`

## Workflow follow-up

- The final commit of the implementation pull request is `openspec archive add-seek-and-hidden --yes`. There is no separate archive pull request. That commit touches only `openspec/changes/archive/**`, `openspec/specs/**`, and the removed `openspec/changes/add-seek-and-hidden/**`, and its message contains `openspec archive add-seek-and-hidden`. The changelog bullet is already in an earlier commit.
