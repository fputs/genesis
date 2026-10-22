# Tasks

## 1. Concealed failure cost

- [ ] 1.1 In `resolve_targeting` in `crates/rules/src/detection.rs`, set `action_spent` and `counts_for_map` when a targeting flat check is rolled and the target is not affected. That covers a concealed DC 5 failure and a concealed DC 5 critical failure, and it keeps a hidden DC 11 failure and a hidden-and-concealed DC 11 failure reporting both. A success leaves both false. Do not add a field. Do not call `spend_actions` or `strike`. Do not change the relation. Update the comments on those two fields and on `resolve_targeting` so a concealed failure is named with the hidden failure. Cite Player Core "Concealed", "Hidden", and "Multiple Attack Penalty". Do not add a dependency. Do not edit `docs/designer/`. Do not port a file from `v1-reference`. In `a_natural_4_at_dc_5_does_not_affect_the_target`, keep the consumed roll, DC 5, failure, and the target not affected, and assert both flags are true. In `a_natural_1_at_dc_5_does_not_affect_the_target`, keep the critical failure, the effect resolving as a failure, and the target not affected, and assert both flags are true. Each of those two tests resolves the same inputs twice, each with its own script of that natural, and asserts the two results are equal. Leave `a_natural_5_at_dc_5_affects_the_target` asserting both flags are false. Verify `cargo test -p rules -- a_natural_4_at_dc_5_does_not_affect_the_target a_natural_1_at_dc_5_does_not_affect_the_target a_natural_5_at_dc_5_affects_the_target a_natural_10_at_dc_11_does_not_affect_the_target hidden_and_concealed_fail_one_dc_11_roll` passes

## 2. Changelog, proof, and archive

- [ ] 2.1 Add one bullet under `### Changed` in `## [Unreleased]` of root `CHANGELOG.md`, in a commit before the archive commit. The bullet says a failed concealed targeting flat check reports the action spent and the attack counting for MAP, and it links to the implementation pull request. Verify the new bullet is under `### Changed` and contains that link
- [ ] 2.2 Run the AGENTS.md proof commands and verify each exits 0.
  - `cargo fmt --all --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo build --workspace`
  - `cargo test --workspace`
  - `npx @fission-ai/openspec@1.14.3 validate --all --strict`
- [ ] 2.3 As the final commit of the implementation pull request, and not in the changelog commit, run `openspec archive amend-concealed-flat-check-cost --yes`. The commit subject is exactly `openspec archive amend-concealed-flat-check-cost --yes`. That commit does not edit `CHANGELOG.md`. Verify the change is no longer active under `openspec/changes/` and sits under `openspec/changes/archive/`
