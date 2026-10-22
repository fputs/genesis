# Tasks

## 1. Frozen attack count

- [ ] 1.1 Add `readied_attack_count: Option<u8>` to `MapInput` in `crates/rules/src/check.rs`, as in design.md. `Some(count)` uses `count` and `current_weapon_agile` in the existing match (0, then -5, then -10, or 0, then -4, then -8) and does not return 0 because `on_own_turn` is false. `None` with `on_own_turn` false returns 0. `None` with `on_own_turn` true uses `prior_attack_trait_checks_this_turn`. The function takes `MapInput` by value and returns `i32`. It does not increment a count and does not panic. Set `readied_attack_count: None` on every existing `MapInput` literal in `check.rs`, `modifiers.rs`, and `encounter.rs`. Do not change `attack_modifiers`'s signature or body. Do not change `attack_trait_penalty`'s off-turn result. Do not add a dependency. Do not edit `docs/designer/`. Do not port a file from `v1-reference`. Verify `cargo test -p rules map_is_zero_then_minus_five_then_minus_ten a_reaction_after_two_strikes_keeps_penalty_0` passes and `rg -n "readied_attack_count: None" crates/rules/src` shows those literals

## 2. Readied scenarios

- [ ] 2.1 Add one `#[cfg(test)]` test in `crates/rules/src/check.rs` for each new scenario under "Multiple attack penalty on your turn". The test name is the scenario title in snake_case: lowercase, spaces become underscores, a hyphen before a number becomes `minus`, and other punctuation is dropped. The names are `a_readied_non_agile_attack_after_one_check_takes_minus_5`, `a_readied_attack_with_no_prior_check_takes_no_penalty`, `a_readied_agile_attack_after_two_checks_takes_minus_8`, `a_readied_non_agile_attack_after_two_checks_takes_minus_10`, `a_readied_attack_leaves_the_next_own_turn_at_no_penalty`, and `a_reactive_strike_on_the_same_round_takes_no_penalty`. Cite Player Core "Multiple Attack Penalty" and Player Core "Ready". Use `ScriptedDice` for each scripted natural and assert the penalty, total, and degree the scenario names. Assert the penalty is not -4, and is not -10, where the scenario says so. For the next-own-turn scenario, assert both checks. For that scenario and the Reactive Strike scenario, hold a `u8`, pass `Some` of it, and assert the `u8` is unchanged. Each new test calls `multiple_attack_penalty` twice with the same `MapInput` and asserts the two penalties are equal. Keep `a_reaction_on_another_turn_takes_no_penalty_and_does_not_count` and pass `readied_attack_count: None` there. Verify `cargo test -p rules a_readied_non_agile_attack_after_one_check_takes_minus_5 a_readied_attack_with_no_prior_check_takes_no_penalty a_readied_agile_attack_after_two_checks_takes_minus_8 a_readied_non_agile_attack_after_two_checks_takes_minus_10 a_readied_attack_leaves_the_next_own_turn_at_no_penalty a_reactive_strike_on_the_same_round_takes_no_penalty a_reaction_on_another_turn_takes_no_penalty_and_does_not_count` passes

## 3. Changelog, proof, and archive

- [ ] 3.1 Add one bullet under `### Changed` in `## [Unreleased]` of root `CHANGELOG.md`. The bullet says a readied attack uses the multiple attack penalty for the attack count frozen when Ready resolved, and it links to the implementation pull request. Verify the new bullet is under `### Changed` and contains that link
- [ ] 3.2 Run the AGENTS.md proof commands and verify each exits 0.
  - `cargo fmt --all --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo build --workspace`
  - `cargo test --workspace`
  - `npx @fission-ai/openspec@1.14.3 validate --all --strict`
- [ ] 3.3 As the final commit of the implementation pull request, run `openspec archive amend-readied-attack-map --yes` and verify the change is no longer active under `openspec/changes/` and sits under `openspec/changes/archive/`
