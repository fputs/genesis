# Tasks

## Definition of done

The implementation is done when the three rejection tests pass, the condition 10 scenario passes in place of the frightened-bonus scenario, the generator draws no frightened or sickened bonus, the proof commands below exit 0, and this change is archived only after `harden-modifier-validation`.

## 1. Reject a frightened or sickened bonus

- [ ] 1.1 In `crates/rules/src/modifiers.rs`, extend the R-03 construction check. A frightened or sickened bonus at any rank, including 0, returns `ModifierError::InvalidValuedCondition` and does not panic. Do not rewrite it into a penalty. A penalty of rank 0 stays `Ok` and still contributes nothing. A penalty of rank above 0 still applies as status `-rank`. Add `frightened_bonus_rank_0_plus_2_is_rejected`, `frightened_bonus_rank_2_plus_2_is_rejected`, and `sickened_bonus_rank_1_plus_1_is_rejected`. A frightened status bonus of +2 at rank 0, a frightened status bonus of +2 at rank 2, and a sickened status bonus of +1 at rank 1 each return `InvalidValuedCondition` and no modifier. Change `a_frightened_bonus_keeps_its_declared_value` so a frightened status bonus of +2 at rank 3 returns `InvalidValuedCondition` and is not totaled as +2. Do not delete that test. Cite Player Core pg. 444, pg. 446, and pg. 442. Build `ScriptedDice::new([8])` and assert `remaining() == 1` in each test. Do not edit `the_lower_condition_id_wins_an_equal_value`. Verify `cargo test -p rules --lib frightened_bonus_rank_0_plus_2_is_rejected frightened_bonus_rank_2_plus_2_is_rejected sickened_bonus_rank_1_plus_1_is_rejected a_frightened_bonus_keeps_its_declared_value` passes
- [ ] 1.2 In `a_frightened_bonus_does_not_hide_a_frightened_penalty`, keep the test name. Use condition 10, which has no value, from source 7 on the attack roll at sequence 4, with a status bonus of +2 at rank 3 and a status penalty of -1 at rank 1. In either order, both are applied and the attack-roll total is +1. Do not tag either modifier frightened. Cite Player Core pg. 400 and pg. 442. Build `ScriptedDice::new([8])` and assert `remaining() == 1`. Verify `cargo test -p rules --lib a_frightened_bonus_does_not_hide_a_frightened_penalty` passes

## 2. Generator and oracle

- [ ] 2.1 In `permuted_sets_match_the_stacking_oracle`, a frightened draw is a status penalty with rank `Some(0)`, `Some(1)`, or `Some(2)`. The generator never draws a frightened or sickened bonus. Sickened stays out of the generator. Assert that no generated modifier is a frightened or sickened bonus, and that every generated modifier builds without error. The oracle uses status `-rank` for frightened and sickened penalties and drops rank 0 entries. It has no branch that keeps a frightened status bonus. Do not call `derived_rank`, `same_duplicate_key`, `source_ord`, or the comparison chains. Keep the three stages, the tie order, the fixed seed, and the sets named in the requirement. Cite Player Core pg. 442 and pg. 444. Build one `ScriptedDice::new([8])` and assert `remaining() == 1`. Verify `cargo test -p rules --lib permuted_sets_match_the_stacking_oracle` passes

## 3. Changelog and proof

- [ ] 3.1 Add one bullet under `## [Unreleased]` of root `CHANGELOG.md`. The bullet says a frightened or sickened bonus is rejected at construction, and it links to the implementation pull request. Verify the new bullet is under `## [Unreleased]` and contains that link
- [ ] 3.2 Run the AGENTS.md proof commands and verify each exits 0. Library code does not panic, sums stay checked, and no dependency was added.
  - `cargo fmt --all --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo build --workspace`
  - `cargo test --workspace`
  - `npx @fission-ai/openspec@1.14.3 validate --all --strict`

## Workflow follow-up

- Archive only after `harden-modifier-validation` is archived. As the final commit of the implementation pull request, run `openspec archive reject-valued-condition-bonus --yes`. Verify the change is no longer active under `openspec/changes/` and sits under `openspec/changes/archive/`. Verify the synced modifiers spec still rejects a frightened or sickened bonus and does not contain "a frightened status bonus keeps its declared value".
