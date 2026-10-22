# Proposal

## Why

Workflow v2 archives a change once, as the last commit right before merge. `pr-hygiene` still applies archive-last on every head, so a fix after that commit is red until the change is restored and archived again. The agent should keep the pull request a draft while fixing, archive as the final commit, then run `gh pr ready`.

## What Changes

- **BREAKING** for a non-draft, non-exempt implementation pull request whose `OpenSpec-Change:` line names a change still active on the head. `pr-hygiene` fails it. A draft with the same commits stays green. A pull request with no `OpenSpec-Change:` line does not fail because another change is active.
- While `pull_request.draft` is true, archive-last does not run. The bullet check still runs.
- When the pull request is not a draft, today's archive-last rules run.
- The workflow also runs on `ready_for_review` and `converted_to_draft`. A push after ready re-runs the full check. The bullet added before the archive still counts. Converting back to draft skips archive-last again.
- Spec-drift fixture cleanup retries `ENOTEMPTY` on `.git/objects` through an injectable remover, at most 5 calls with backoff (run 39705218352).

This pull request is the spec only. It does not change scripts, workflows, or the ruleset.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `pr-hygiene`: archive-last is skipped on a draft and enforced once the pull request is ready.
- `spec-drift-check`: fixture teardown retries `ENOTEMPTY`.

## Impact

- DevOps edits `scripts/pr-hygiene.mjs`, `scripts/pr-hygiene.test.mjs`, `.github/workflows/pr-hygiene.yml`, and `scripts/spec-drift-check.test.mjs`.
- No ruleset change, no `ci.yml` change, and no new dependency. `crates/rules` is untouched.
- `gh api repos/fputs/genesis --jq .owner.type` is `User`. Merge queue is not available on a user-owned repository.
- The implementation pull request adds its Unreleased bullet before the archive commit, then runs `openspec archive archive-at-merge-hygiene --yes`.
