# Proposal

## Why

Workflow v2 archives an OpenSpec change once, as the last commit right before merge, and forbids un-archive/re-archive on each fix. `pr-hygiene` still applies archive-last on every pull request head, so a fix after that commit is red until the change is restored and archived again. Implementation PRs #25 and #28 did that restore step on every fix. The check should stay green through review and block only the merge.

## What Changes

- **BREAKING** for a non-exempt implementation pull request that merges without an archive commit. The merge queue rejects it. Review pushes stay green without one.
- On `pull_request`, `pr-hygiene` still enforces the Unreleased bullet and does not fail with `archive commit is not last`. An unarchived implementation pull request passes. A commit after an archive commit does not fail archive-last on that run.
- On `merge_group`, archive-last is required. The pull request head is the second parent of the merge group head. When an archive commit exists, it must be that head. A non-exempt implementation pull request that removed no active change id fails with `archive commit is not last`.
- `.github/workflows/ci.yml` also runs on `merge_group` so the required `ci` and `spec-drift` checks report on the queue. Cargo steps stay. `merge_group` skips the spec-drift reference check and still fails a fully checked active change.
- Spec-drift fixture cleanup retries `ENOTEMPTY` when deleting a temporary git directory, including `.git/objects`. Run 39705218352 failed `Copying to archive without removing the active change fails` on that race.

This pull request is the spec only. It does not change the scripts, the workflows, or the ruleset.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `pr-hygiene`: archive-last runs on the merge queue, not on review heads. A non-exempt implementation pull request must archive before merge. When an archive commit exists, it is last.
- `spec-drift-check`: `merge_group` skips the change-id reference check and still fails a fully checked active change. Fixture teardown retries `ENOTEMPTY`.

## Impact

- DevOps edits `scripts/pr-hygiene.mjs`, `scripts/pr-hygiene.test.mjs`, `scripts/spec-drift-check.test.mjs`, `.github/workflows/pr-hygiene.yml`, and the triggers of `.github/workflows/ci.yml`.
- DevOps adds a merge queue to the Main ruleset (`25563418`): merge method `MERGE`, one pull request per group. Required checks stay `ci`, `pr-hygiene`, and `spec-drift`.
- No new dependency. `crates/rules` is untouched. `AGENTS.md` already says to archive once, right before merge, so it needs no amendment.
- The implementation pull request adds its Unreleased bullet before the archive commit, then runs `openspec archive archive-at-merge-hygiene --yes`.
