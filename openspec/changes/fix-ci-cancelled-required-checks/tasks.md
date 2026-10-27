# Tasks

Genesis DevOps agent implements this change after a new `spec-approved: <sha>` review of this revision. Do not edit `crates/rules`. Do not edit `scripts/spec-drift-check.mjs`, `scripts/pr-hygiene.mjs`, or `scripts/spec-approved.mjs`. Do not add a dependency. Do not add a YAML parser. Do not edit branch protection or a ruleset. Do not add an `edited` type or a skip `if` to `ci.yml`. Do not add a concurrency key to the Cargo `ci` job. Do not add `queue` or `cancel-in-progress` to any workflow. The body includes `OpenSpec-Change: fix-ci-cancelled-required-checks`.

## 1. Workflow file tests

- [x] 1.1 In `scripts/spec-drift-check.test.mjs`, replace the shared-group constant and the assertions in "The edited workflow reuses the spec-drift job name", "Both spec-drift jobs share a pull request group", and "A push does not join a pull request group". Add cases named "The spec-drift jobs declare no concurrency", "A push does not cancel a pull request run", "The Cargo job has no concurrency key", "Three same-sha events leave no cancelled check", and "No required check declares concurrency". Read `.github/workflows/ci.yml`, `.github/workflows/spec-drift-edited.yml`, `.github/workflows/pr-hygiene.yml`, and `.github/workflows/spec-approved.yml` as text. None of the four files contains `concurrency:`, `cancel-in-progress`, or `queue:`. The `spec-drift` jobs and the Cargo `ci` job have no concurrency key. "The edited workflow reuses the spec-drift job name" still checks the job name, the test-then-script order, permissions, and the absence of a concurrency block. Commit this task before task 2.1. Until task 2.1, these cases fail on the current YAML. That failure is the gate. Do not edit the workflow files in this commit.

- [x] 1.2 In `scripts/pr-hygiene.test.mjs`, replace "An in-progress run is cancelled" with "The pr-hygiene workflow declares no concurrency". The file contains no `concurrency:`, `cancel-in-progress`, or `queue:` line. Commit this task before task 2.1. The case fails on the current YAML until task 2.1.

- [x] 1.3 In `scripts/spec-approved.test.mjs`, replace "A new push cancels the previous run" with "The spec-approved job declares no concurrency". The job block and the workflow header contain no `concurrency:` line, and the file contains no `cancel-in-progress` or `queue:` line. Commit this task before task 2.1. The case fails on the current YAML until task 2.1.

## 2. Delete the concurrency blocks and add the changelog bullet

- [x] 2.1 Delete the job-level concurrency block from the `spec-drift` job in `.github/workflows/ci.yml` and in `.github/workflows/spec-drift-edited.yml`. Delete the workflow-level concurrency block from `.github/workflows/pr-hygiene.yml`. Delete the job-level concurrency block from the `spec-approved` job in `.github/workflows/spec-approved.yml`. Do not add a replacement block. Leave the Cargo `ci` job without a concurrency key. Leave triggers, permissions, and steps as they are. Verify `node --test scripts/spec-drift-check.test.mjs`, `node --test scripts/pr-hygiene.test.mjs`, and `node --test scripts/spec-approved.test.mjs` exit 0, including the task 1.1, 1.2, and 1.3 names.

- [x] 2.2 In that same implementation commit, add one bullet under `### Fixed` in `## [Unreleased]` of `CHANGELOG.md`. The bullet states that `spec-drift`, `pr-hygiene`, and `spec-approved` no longer declare concurrency, so a same-sha burst does not leave a cancelled required check. The bullet links `https://github.com/fputs/genesis/pull/` plus this pull request's number. Verify the new bullet is under `### Fixed` and contains that URL. Do not add the bullet in a spec-only commit.

## 3. Proof and archive

- [x] 3.1 Run the AGENTS.md proof commands and verify each exits 0: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo build --workspace`, `cargo test --workspace`, and `npx @fission-ai/openspec@1.14.3 validate --all --strict`.

- [ ] 3.2 As the final commit of this pull request, after Architect and QA have both passed the previous head, run `openspec archive fix-ci-cancelled-required-checks --yes` and check this box in that same commit. The commit deletes `openspec/changes/fix-ci-cancelled-required-checks/` and adds `openspec/changes/archive/<YYYY>-<MM>-<DD>-fix-ci-cancelled-required-checks/`. It does not modify `CHANGELOG.md`. Leave this box open until that commit so the change is not a completed unarchived change. Verify the change is no longer an active directory under `openspec/changes/`.

## Workflow follow-up

- Task 3.2 is the archive commit, right before merge, and only after Architect and QA have passed the head that was archived.
