# Proposal

## Why

Required checks `spec-drift`, `pr-hygiene`, and `spec-approved` use `cancel-in-progress: true` on a pull-request-number group. A later event on the same head sha cancels the in-flight run and leaves `conclusion: cancelled` beside the run that then succeeds. The `Main` ruleset reads that cancelled check and refuses the merge with HTTP 405. Bots cannot rerun Actions. Pull request 87 (CI run 40144650992, head `ac160bce06cc8fe1b03c5f7edd7ff7d0f6da8ca2`) and pull request 89 (head `6306905cb06432ceb90718dfccbfdefe67193e07`, `mergeStateStatus: BLOCKED`) both hit this. Pull request 85 worked around it with an empty commit.

## What Changes

- A superseded run SHALL NOT leave a cancelled required check on a head sha that a later run on that same sha completes. Three events on one sha (`synchronize`, `edited`, review `submitted`) SHALL NOT leave a cancelled required check.
- **BREAKING** for the shared spec-drift group. Owner review of pull request 90 at `4087a9c`, with Architect's decision on that review: delete the concurrency blocks. The `spec-drift` jobs in `ci.yml` and `spec-drift-edited.yml`, the `pr-hygiene` workflow, and the `spec-approved` job SHALL NOT declare `concurrency`. `queue` is not used. The Cargo `ci` job stays as it is, with no concurrency key added.
- Triggers, permissions, and steps stay. No ruleset edit and no skip `if`.
- This pull request is the spec only. Workflows and tests wait for a new `spec-approved: <sha>` review.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `spec-drift-check`: both `spec-drift` jobs drop concurrency, so an `edited` run cannot cancel the `ci.yml` job. This covers the cross-workflow race between `ci.yml` `spec-drift` and `spec-drift-edited`.
- `pr-hygiene`: the workflow concurrency block is removed, so a same-sha edit does not cancel an in-flight run.
- `spec-approved`: the job concurrency block is removed, so a same-sha review does not cancel an in-flight run.

## Impact

- DevOps deletes those concurrency blocks and updates the workflow assertions in `scripts/spec-drift-check.test.mjs`, `scripts/pr-hygiene.test.mjs`, and `scripts/spec-approved.test.mjs`.
- No new dependency. `crates/rules` is untouched. Branch protection and the ruleset stay as they are. The Cargo job's steps and triggers stay.
- The implementation commit adds one `### Fixed` bullet under `## [Unreleased]` that links this pull request. The archive commit does not edit `CHANGELOG.md`.
