# Proposal

## Why

Required checks `spec-drift`, `pr-hygiene`, and `spec-approved` use `cancel-in-progress: true` on a pull-request-number group. A later event on the same head sha cancels the in-flight run and leaves `conclusion: cancelled` beside the run that then succeeds. The `Main` ruleset reads that cancelled check and refuses the merge with HTTP 405. Bots cannot rerun Actions. Pull request 87 (CI run 40144650992, head `ac160bce06cc8fe1b03c5f7edd7ff7d0f6da8ca2`) and pull request 89 (head `6306905cb06432ceb90718dfccbfdefe67193e07`, `mergeStateStatus: BLOCKED`) both hit this. Pull request 85 worked around it with an empty commit.

## What Changes

- A superseded run SHALL NOT leave a cancelled required check on a head sha that a later run on that same sha completes.
- **BREAKING** for the shared spec-drift group. `ci.yml` and `spec-drift-edited.yml` no longer share `spec-drift-${{ github.event.pull_request.number || github.sha }}` with `cancel-in-progress: true`. Each required-check concurrency group is the workflow, the head sha, and the event type, with `cancel-in-progress: false` and `queue: max`, so a same-sha rerun waits and a new sha is a different group.
- The Cargo `ci` job stays outside every concurrency group. Triggers, the `edited` workflow, and permissions stay. No ruleset edit and no skip `if`.
- This pull request is the spec only. Workflows and tests wait for `spec-approved`.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `spec-drift-check`: same-sha `spec-drift` runs queue, and the two workflows no longer share a cancelling group. This covers the cross-workflow race between `ci.yml` `spec-drift` and `spec-drift-edited`.
- `pr-hygiene`: a same-sha `edited` or `synchronize` event queues `pr-hygiene` instead of cancelling it.
- `spec-approved`: a same-sha review or edit queues `spec-approved` instead of cancelling it.

## Impact

- DevOps edits the concurrency blocks in `.github/workflows/ci.yml` (the `spec-drift` job only), `.github/workflows/spec-drift-edited.yml`, `.github/workflows/pr-hygiene.yml`, and `.github/workflows/spec-approved.yml`, plus the workflow assertions in `scripts/spec-drift-check.test.mjs`, `scripts/pr-hygiene.test.mjs`, and `scripts/spec-approved.test.mjs`.
- No new dependency. `crates/rules` is untouched. Branch protection and the ruleset stay as they are. The Cargo job's steps and triggers stay.
- The implementation commit adds one `### Fixed` bullet under `## [Unreleased]` that links this pull request. The archive commit does not edit `CHANGELOG.md`.
