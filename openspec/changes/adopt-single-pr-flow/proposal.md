# Proposal

## Why

A change is split across a spec pull request and a later implementation pull request, so approval and code do not share a branch. Evan decided, relayed by Chief of Staff, to keep OpenSpec and use one draft pull request per change.

## What Changes

- **BREAKING** The separate spec pull request goes away. One draft pull request carries the spec, the tests, and the code.

- The first commit adds only `openspec/changes/<id>/`. Designer rules on that pull request. An authorized login approves by commenting `spec-approved: <sha>`.

- Tests, then code, follow on the same branch. Architect and QA review in parallel. The merger archives once, right before merge. Architect merges when CI is green.

- A new `spec-approved` job hashes `proposal.md`, `design.md`, and `specs/**`, ignoring `tasks.md` and treating an archive move as the same content.

- The job compares that hash to the tree at the sha in the latest allowed `spec-approved: <sha>` comment. It re-runs on `issue_comment` and `pull_request_review`. A pull request with no code path is exempt, by the same rule as `isCodePath`.

- Allowed logins are a configurable list. A missing list, an unreadable list, or an unread comment thread fails closed.

- `.github/CODEOWNERS` gives `/openspec/` and `/AGENTS.md` to `@fputs`. Branch protection stays out of scope and is an admin step for Evan.

- **BREAKING** for the reference check: `OpenSpec-Refactor: <roadmap item>` stands in for `OpenSpec-Change` only when the pull request is crates-only, does not touch `openspec/`, does not modify or delete an existing test file, names a non-empty item, and has no `OpenSpec-Change` line.

- `spec-approved` does not apply to that refactor pull request. Unchanged behavior stays a human merge gate.

- `AGENTS.md` section 1 drops the separate-spec-PR wording. A small spec clarification stays in the pull request, cites Designer, and re-triggers `spec-approved`. `openspec/config.yaml` and the workflow docs under `docs/` match this flow.

## Capabilities

### New Capabilities

- `single-pr-flow`: one draft pull request per change, the approval comment, review, archive, merge, CODEOWNERS, and the constitution and workflow docs.
- `spec-approved`: the CI job that checks the approved spec hash and fails closed.

### Modified Capabilities

- `spec-drift-check`: a crates-only refactor line can satisfy the reference check in place of `OpenSpec-Change`.

## Impact

- After approval, DevOps adds `scripts/spec-approved.mjs`, its tests, and `.github/workflows/spec-approved.yml` on this same branch.

- DevOps also adds `openspec/spec-approved-authors.txt` and `.github/CODEOWNERS`, and extends the spec-drift script and its tests. No new npm dependency.

- Architect then updates `AGENTS.md` section 1, `openspec/config.yaml`, `docs/architect/decisions.md`, `docs/producer/process.md`, and `docs/qa/test-plan.md`.

- This pull request is the spec only. It does not change scripts, workflows, or `crates/rules`.

- The implementation adds an Unreleased bullet before the archive commit on this same branch.

- Branch protection is not edited. Evan turns on code-owner review in the GitHub settings.

This change does not add a ruling. The log is [docs/designer/rules-rulings.md](../../../docs/designer/rules-rulings.md).
