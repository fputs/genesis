# Proposal

## Why

A change is split across a spec pull request and a later implementation pull request, so approval and code do not share a branch. Evan decided, relayed by Chief of Staff, to keep OpenSpec and use one draft pull request per change.

## What Changes

- **BREAKING** The separate spec pull request goes away. One draft pull request carries the spec, the tests, and the code.

- The first commit adds only `openspec/changes/<id>/`. Designer rules on that pull request. An allowed user id approves by a review whose body line is `spec-approved: <sha>`. An issue comment does not approve.

- Tests, then code, follow on the same branch. Architect and QA review in parallel. The merger archives once, right before merge. Architect merges when CI is green.

- A new `spec-approved` job hashes `proposal.md`, `design.md`, and `specs/**`, ignoring `tasks.md` and treating an archive move as the same content.

- The job compares that hash to the tree at the sha in the latest allowed review. It re-runs on `pull_request` and `pull_request_review`. The check context name is `spec-approved`. A pull request with no code path is exempt, by the same rule as `isCodePath`.

- The script and the authors file run from a worktree of `pull_request.base.sha`. The head is only the diff and the hash. Authors match GitHub user id `239839957`. Bootstrap applies only when the base lacks both the script and the workflow file.

- `.github/CODEOWNERS` gives `/openspec/`, `/AGENTS.md`, `/.github/`, and `/scripts/` to `@fputs`. The workflow file runs from the merge ref. The owner rules hold only after Evan enables required code-owner review through Chief of Staff and makes `spec-approved` required.

- **BREAKING** for the reference check: one `OpenSpec-Refactor:` line plus a kebab roadmap item stands in for `OpenSpec-Change` only when every path starts with `crates/` or is exactly `CHANGELOG.md`, no path is under `openspec/`, and no existing `/tests/` file is modified or removed. Adding a test file is allowed.

- A second refactor line, or any `OpenSpec-Change` line beside it, fails. The item is not free text. A qualifying refactor still needs an Unreleased bullet. `spec-approved` does not apply to that pull request. Unchanged behavior stays a human merge gate.

- `AGENTS.md` section 1 drops the separate-spec-PR wording. A small spec clarification stays in the pull request, cites Designer, and re-triggers `spec-approved`. `openspec/config.yaml` and the workflow docs under `docs/` match this flow.

## Capabilities

### New Capabilities

- `single-pr-flow`: one draft pull request per change, the approval comment, review, archive, merge, CODEOWNERS, and the constitution and workflow docs.
- `spec-approved`: the CI job that checks the approved spec hash and fails closed.

### Modified Capabilities

- `spec-drift-check`: one kebab `OpenSpec-Refactor` line can satisfy the reference check, using the path, test-file, and bullet rules drafted on pull request 79.

## Impact

- After approval, DevOps adds `scripts/spec-approved.mjs`, its tests, and `.github/workflows/spec-approved.yml` on this same branch.

- DevOps also adds `openspec/spec-approved-authors.txt` and `.github/CODEOWNERS`, and extends the spec-drift script and its tests. No new npm dependency.

- Architect then updates `AGENTS.md` section 1, `openspec/config.yaml`, `docs/architect/decisions.md`, `docs/producer/process.md`, and `docs/qa/test-plan.md`.

- This pull request is the spec only. It does not change scripts, workflows, or `crates/rules`.

- The implementation adds an Unreleased bullet before the archive commit on this same branch.

- Branch protection is not edited. Evan turns on code-owner review in the GitHub settings.

This change does not add a ruling. The log is [docs/designer/rules-rulings.md](../../../docs/designer/rules-rulings.md).
