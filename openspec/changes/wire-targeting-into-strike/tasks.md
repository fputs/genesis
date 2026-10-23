# Tasks

Each new scenario has exactly one test. The test name is the scenario title in snake_case. Spaces and hyphens become `_`. `-5` becomes `minus_5`. Punctuation is dropped. A test comment cites the Player Core section on that requirement. A test that does not roll builds a `ScriptedDice` and asserts the held face is unused. Do not add a dependency. Do not add a field to `Encounter`, `Creature`, or `CombatantState`. Do not call `flat_check` from the encounter. Do not call `note_strike`. This change names no `v1-reference` file, so none is ported.

## 1. The flat face

- [ ] 1.1 Add `natural: Option<u8>` to `TargetingResult` and set it from the `FlatRoll` `resolve_targeting` already has. `None` when no die is drawn. Do not change the DC, the effect, or the spend flags, and do not call `spend_actions` or `strike`. Cite Player Core "Checks (Flat Checks)", "Concealed", and "Undetected". Verify `cargo test -p rules -- a_natural_4_at_dc_5_reports_that_face a_natural_5_at_dc_5_reports_that_face an_observed_target_reports_no_face an_undetected_target_reports_no_face a_natural_4_at_dc_5_does_not_affect_the_target a_natural_5_at_dc_5_affects_the_target an_observed_target_rolls_nothing an_undetected_target_cannot_be_chosen` passes

## 2. On-turn Strike

- [ ] 2.1 Add `LostStrike`, `StrikeDelivery`, and `EncounterError::CannotTarget`. `Encounter::strike` takes `&Detections` and returns `Result<StrikeDelivery, EncounterError>`. After `prepare_strike` and a requested Shield Block's readiness, call `resolve_targeting` with `area_effect` false. An affected target then rolls the attack d20 as today, inside `Rolled`. A lost check calls `finish_strike`, does not roll the attack d20 or a damage die, does not call `shield_block`, and returns `Lost`. `CannotTarget` spends nothing. Match `Some` for the face, the DC, and the degree. Do not `expect` and do not panic. An empty `Detections` draws no flat-check die. Give each new public item a `///` comment citing the Player Core section. Cite Player Core "Strike", "Concealed", "Hidden", "Undetected", and "Multiple Attack Penalty", and R-12 in `docs/designer/rules-rulings.md`. Verify `cargo test -p rules -- an_observed_target_draws_the_attack_die_first a_natural_5_at_dc_5_then_the_attack_roll a_natural_11_at_dc_11_then_the_attack_roll hidden_and_concealed_succeed_on_one_dc_11_roll the_same_lost_strike_twice_matches a_natural_4_at_dc_5_loses_the_strike a_natural_1_at_dc_5_loses_the_strike_and_counts a_natural_10_at_dc_11_makes_the_next_strike_take_minus_5 a_lost_strike_makes_the_next_agile_strike_take_minus_4 two_lost_strikes_make_the_third_take_minus_10 hidden_and_concealed_lose_on_one_dc_11_roll an_undetected_target_cannot_be_struck a_dead_undetected_target_is_target_dead no_actions_beats_a_concealed_flat_check an_attack_count_at_the_maximum_draws_no_flat_check a_shield_that_is_not_raised_draws_no_flat_check a_flat_check_success_can_still_overflow_the_attack the_first_strike_totals_the_d20_plus_the_attack_bonus` passes

## 3. Reactive Strike

- [ ] 3.1 `reactive_strike` takes `&Detections`. After the existing no-die gates, including `AttackOverflow`, call `resolve_targeting`. A lost check spends the reaction, leaves the attack count and actions unchanged, returns `Lost` and `Disruption::NotDisrupted`, and draws no attack d20. `ReactiveError::CannotTarget` spends nothing. `ResolvedReaction::ReactiveStrike` returns `Result<ReactiveOutcome, ReactiveError>`. Extend the existing `too_many_arguments` expect reason to include detections. Do not add a crate-level allow. Cite Player Core "Reactive Strike", "Concealed", "Hidden", and "Undetected". Verify `cargo test -p rules -- a_natural_5_at_dc_5_then_the_reactive_attack hidden_and_concealed_use_one_dc_11_roll a_natural_4_at_dc_5_loses_the_reactive_strike a_natural_10_at_dc_11_loses_the_reactive_strike a_lost_reactive_strike_does_not_count_toward_map an_undetected_actor_cannot_be_the_target no_reaction_beats_an_undetected_actor reactive_strike_does_not_use_map reactive_strike_does_not_count_toward_map` passes

## 4. Readied Strike

- [ ] 4.1 The readied Strike calls `resolve_targeting` after `fire_checks`. `resolve_reactions` takes `&Detections` and passes it to both paths. A lost check spends the reaction, clears the ready, and does not increment the attack count. `ReadySuccess::Lost` carries `LostStrike`. `ReadyError::CannotTarget` leaves the reaction and the ready in place. Replace `a_readied_strike_draws_no_flat_check`. Do not keep an assertion that a hidden target skips the flat check. Cite Player Core "Ready", "Concealed", "Hidden", and "Undetected", and R-05 in `docs/designer/rules-rulings.md`. Verify `cargo test -p rules -- an_observed_readied_strike_draws_the_attack_die_first a_natural_11_at_dc_11_then_the_readied_attack a_natural_5_at_dc_5_keeps_the_frozen_penalty each_reaction_draws_its_own_flat_check a_natural_4_at_dc_5_loses_the_readied_strike an_undetected_readied_strike_spends_nothing a_dead_undetected_target_is_target_dead a_readied_non_agile_strike_after_one_strike_takes_minus_5` passes

## 5. Retire DEV-46

- [ ] 5.1 In `docs/designer/rules-deviations.md`, set DEV-46's Our behavior to retired, in the style of DEV-03: on-turn Strike, a readied Strike, and Reactive Strike call `resolve_targeting` before the attack roll, a failed flat check loses the Strike, and undetected stays `CannotTarget` (DEV-41). Keep the RAW citation. Do not add a DEV id, and do not reuse DEV-47 through DEV-49. Do not edit `docs/designer/rules-rulings.md`. Verify the DEV-46 row contains `Retired` and the DEV-41 row still says `CannotTarget`

## 6. Changelog and proof

- [ ] 6.1 Add one **BREAKING** bullet under `## [Unreleased]` `### Changed` in `CHANGELOG.md`. It names the flat check before the attack roll, the lost Strike, and `StrikeDelivery`. Link it to the implementation pull request. Put that edit in a commit before the archive commit. Do not add the bullet in the spec-only pull request. Verify the new bullet is under `### Changed`, contains the pull request link, and that the archive commit does not modify `CHANGELOG.md`
- [ ] 6.2 Run the AGENTS.md proof commands and verify each exits 0
  - `cargo fmt --all --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo build --workspace`
  - `cargo test --workspace`
  - `npx @fission-ai/openspec@1.14.3 validate --all --strict`

## 7. Archive

- [ ] 7.1 The final commit of the implementation pull request is `openspec archive wire-targeting-into-strike --yes`. There is no separate archive pull request. That commit touches only `openspec/changes/archive/**`, `openspec/specs/**`, and the removed `openspec/changes/wire-targeting-into-strike/**`, and its message contains `openspec archive wire-targeting-into-strike`. The changelog bullet is already in an earlier commit. Do not archive the spec-only pull request
