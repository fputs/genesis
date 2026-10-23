# Proposal

## Why

A behavior-preserving edit under `crates/` has no OpenSpec change to name, so spec-drift fails it for a missing `OpenSpec-Change` line. Chief of Staff decided one `OpenSpec-Refactor: <roadmap item>` line can satisfy that check when the diff stays inside the path rules below.

## What Changes

- A pull request body line `OpenSpec-Refactor: <roadmap item>` satisfies the spec-drift reference check in place of `OpenSpec-Change:`, only when every changed path starts with `crates/` or is exactly `CHANGELOG.md`, no path is under `openspec/`, and no existing test file is modified or removed. Adding a test file is allowed. `CHANGELOG.md` is the one extra path so the bullet below can land in the same diff.
- The roadmap item uses the change-id line grammar: exactly one U+0020 after the colon, then one kebab token (`[a-z0-9]` groups joined by single hyphens), the same shape as Phase 1 names in `docs/architect/decisions.md` (`dice-and-checks`, `grid-movement`). Empty or any other shape is malformed. The invalid-item log uses the same remainder escaping as `spec-drift: invalid change id`. The item is not looked up in the decision log.
- Fail when both an `OpenSpec-Refactor` line and an `OpenSpec-Change` line are present, and when more than one refactor line is present. Exact failure lines are in the delta.
- A test file is a `crates/` path containing `/tests/`. Statuses come from the existing `git diff -z --name-status --find-renames` records. `#[cfg(test)]` modules inside other sources are not classified by CI.
- Architect and QA confirming behavior is unchanged is a human merge gate. This job does not check it.
- A qualifying refactor is still an implementation pull request under the current `changelog` and `pr-hygiene` specs and still needs a new Unreleased bullet. Those specs do not exempt tests-only diffs. This job does not check the bullet. `OpenSpec-Refactor` is not an own-change line.
- Both spec-drift workflows already run this script. Triggers, the job name, permissions, and the shared concurrency group stay as they are. Push still skips the reference check.

This pull request is the spec only. It does not change the script or the workflows.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `spec-drift-check`: an `OpenSpec-Refactor` line can satisfy the reference check for a crates refactor, with the failure lines, test-file rule, human gate, changelog interaction, and unchanged edited workflow.

## Impact

- DevOps edits `scripts/spec-drift-check.mjs` and `scripts/spec-drift-check.test.mjs` only. `changeReferences` stays the `OpenSpec-Change` parser `scripts/pr-hygiene.mjs` imports.
- No workflow edit, no new dependency, and no `crates/rules` edit in the implementation of this change.
- The implementation pull request adds one Unreleased bullet, then archives with `openspec archive add-refactor-exemption --yes` as its last commit.
