# Tasks

## 1. Equal-value condition id

- [ ] 1.1 In `crates/rules/src/modifiers.rs`, change `the_lower_condition_id_wins_an_equal_value` so the circumstance bonuses use condition ids 8 and 4, and the applied condition id is 4. Leave `ConditionId::SICKENED` at 3. Leave every other test alone. The test comment still cites Player Core pg. 398. Keep the existing `ScriptedDice` and assert it is unused. Verify `cargo test -p rules --lib the_lower_condition_id_wins_an_equal_value` passes
- [ ] 1.2 Add one bullet under `## [Unreleased]` of root `CHANGELOG.md`. The bullet says the equal-value condition-id tie uses condition id 4, and it links to the implementation pull request. Verify the new bullet is under `## [Unreleased]` and contains that link
- [ ] 1.3 Run the AGENTS.md proof commands and verify each exits 0. Library code does not panic, sums stay checked, and no dependency was added.
  - `cargo fmt --all --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo build --workspace`
  - `cargo test --workspace`
  - `npx @fission-ai/openspec@1.14.3 validate --all --strict`

## Workflow follow-up

- As the final commit of the implementation pull request, run `openspec archive amend-condition-id-scenario --yes`. Verify the change is no longer active under `openspec/changes/` and sits under `openspec/changes/archive/`.
