# Design

## Context

See proposal.md for why. `AGENTS.md` section 1 and `docs/architect/decisions.md` still say Evan merges a spec pull request and Architect opens a second one. `scripts/spec-drift-check.mjs` exports `changeReferences`. `isCodePath` is private there and copied in `scripts/pr-hygiene.mjs`. Merged pull requests use the GitHub login `fputs`. There is no `CODEOWNERS` file. Normative behavior is `specs/single-pr-flow/spec.md`, `specs/spec-approved/spec.md`, and `specs/spec-drift-check/spec.md`.

## Goals / Non-Goals

**Goals:**

- Compare a spec digest, not the head sha, so tests, code, and archive do not by themselves revoke approval.
- Fail closed when the allow-list or the comment thread cannot be read in full.
- Let one kebab refactor line satisfy spec-drift without `OpenSpec-Change`, including the `CHANGELOG.md` path the bullet needs.
- Keep the constitution and the workflow docs on the same flow.

**Non-Goals:**

- GitHub branch protection, rulesets, or required reviews. Evan turns those on. The decisions note lists that step. It is not a task.
- Changing `pr-hygiene` archive-last, the Cargo job, or spec-drift triggers.
- A CI check that commit order is spec, then tests, then code. Review checks that order.
- Deciding that a refactor left behavior unchanged. Architect and QA do that in the combined review.
- A new npm dependency, or any edit under `crates/rules` for this change's own implementation.

## Decisions

### 1. The approval compares spec bytes

The digest covers `proposal.md`, `design.md` when it exists, and every blob under `specs/`. Paths are relative to the change directory, sorted by UTF-8 bytes. Each file is the path, a NUL, an 8-byte big-endian length, and the raw blob. The digest is lowercase hex SHA-256. `tasks.md` is omitted, so checkbox edits do not matter. An archive move keeps the same relative paths and the same digest.

The active directory wins. If it is absent, exactly one archive directory whose stripped id matches is used, with spec-drift's dated and undated rules. Two matches fail with `spec-approved: ambiguous change path`. A missing `proposal.md` fails with `spec-approved: proposal missing`.

Alternative considered: require the comment sha to equal the head. Rejected. Every later test commit would need a new approval even when the spec bytes did not change.

### 2. The allow-list is a file

`openspec/spec-approved-authors.txt` has one login per line. Blank lines and `#` comments are ignored. The shipped file contains only `fputs`, the login on merged pull requests and the owner named in CODEOWNERS. Match `user.login` case-sensitively. An empty, missing, or unreadable file fails with `spec-approved: authors unavailable`. The file lives under `/openspec/`, which `@fputs` owns.

Alternative considered: hard-code the login in the script. Rejected. The list has to change without a script edit.

### 3. Which comment counts

A valid line is `spec-approved:`, one U+0020, and 40 lowercase hex digits, after trim. Fenced lines do not count. The job reads issue comments and pull request review bodies. It ignores review line comments and the pull request body. Among allowed authors, the greatest `updated_at` wins, then the greatest numeric id. The digest at that sha must equal the digest at the head.

The job lists comments and reviews with `GITHUB_TOKEN` and follows pagination to the end. A missing token, a non-2xx response, or a missing next page fails with `spec-approved: comments unavailable`. Tests inject the list and do not call GitHub.

Alternative considered: trust only the event that woke the job. Rejected. A later push must still see the earlier comment, and deleting the comment must remove the approval.

### 4. The job has its own workflow

`issue_comment` and `pull_request_review` must not re-run Cargo. `.github/workflows/spec-approved.yml` is a separate workflow, as `.github/workflows/spec-drift-edited.yml` is. For an issue comment, the job resolves the pull request head from the pulls API before checkout, with full history, so the approved sha is local. A failed lookup fails with `spec-approved: pull request unavailable`.

Alternative considered: add those events to `ci.yml`. Rejected. That file's Cargo job would run on a comment.

### 5. Spec-approved reuses spec-drift's path rules

`spec-approved.mjs` imports `changeReferences` and `isCodePath`. This change exports `isCodePath`. It does not make `pr-hygiene.mjs` switch its private copy. Changed paths are spec-drift's tree diff from the pull request base sha to the head sha, including both sides of a rename. No code path, or a pull request that meets the qualifying refactor paths, passes with no comment. A conflicting, malformed, or repeated refactor line does not.

Every returned id is hashed. An invalid token fails with `spec-approved: invalid change id`. An unknown id fails with `spec-approved: unknown change <id>`. Zero ids on a non-exempt code path fail with `spec-approved: missing OpenSpec-Change`. No allowed approval fails with `spec-approved: approval missing`. A sha that is not a commit fails with `spec-approved: approved sha unavailable`. A hash difference fails with `spec-approved: spec hash mismatch`.

### 6. The refactor exemption follows pull request 79

The delta in `specs/spec-drift-check/spec.md` is the text and scenarios from the draft on pull request 79. This change does not edit that pull request. Where the draft is stricter than the original brief, the draft wins.

The roadmap item is one kebab token, `^[a-z0-9]+(?:-[a-z0-9]+)*$`, after exactly one U+0020. The brief allowed any non-empty item, including spaces. The draft is stricter. Extra words, uppercase, a slash, and a double hyphen log `spec-drift: invalid refactor item` plus the escaped remainder. The item is not looked up.

A test file is a `crates/` path containing `/tests/`. Status `M`, `D`, or `T` on that path, and a rename whose old path is a test file, log `spec-drift: refactor changes existing tests`. Status `A` is allowed. `crates/rules/src/tests.rs` is not a test file. A filename rule ending in `tests.rs` would have classified it. The draft does not. The script does not parse Rust.

Both an `OpenSpec-Refactor` line and an `OpenSpec-Change` line fail with `spec-drift: refactor line conflicts with OpenSpec-Change`, including on a docs-only pull request. The brief would have kept the change id and ignored the refactor line. The draft is stricter. A repeated refactor line counts again and logs `spec-drift: multiple OpenSpec-Refactor lines`.

The brief required every path under `crates/`. The draft also keeps the Unreleased bullet the current changelog and pr-hygiene specs already require for a `crates/` diff. Those two cannot both hold unless exactly `CHANGELOG.md` may sit in the same diff. This change follows the draft.

Any other path outside `crates/` logs `spec-drift: refactor is not crates-only`. An `openspec/` path also logs `spec-drift: refactor changes openspec`. Spec-drift does not read the bullet. `OpenSpec-Refactor` is not an own-change line. Those two specs are not edited. If `amend-changelog-policy` later exempts a tests-only diff, that conflict is sent back rather than silently widening this exemption.

Path lines are logged only for one valid refactor line, no change line, and a code path. Their order is crates-only, then openspec, then existing tests, after the token-order lines. Push still skips the reference check. Fenced lines count. The spec-drift workflows and their concurrency group stay as they are. `spec-approved` skips only when the qualifying paths hold.

Alternative considered: a free-text item and one `refactor exemption refused` log. Rejected. Pull request 79 already closed the holes the change-id rule closed, and it named the failure lines.

### 7. Docs say the same thing as the spec

`AGENTS.md` section 1 replaces the two-pull-request sentences and keeps the review, git, and port rules. The config `tasks` rule ends with the sentence in the single-pr-flow spec. The three workflow docs drop "only specs merged to main" and record Evan's branch-protection step. A small clarification still cites Designer. Because it edits hashed files, `spec-approved` fails until a new comment matches.

## Risks / Trade-offs

- [An issue comment checks out the default branch] → Resolve the head sha before checkout, and fetch full history.

- [A short comment page looks like no approval] → A missing next page fails closed.

- [Archive rewrites only `tasks.md`] → That file is outside the digest, and paths are relative.

- [The API login casing differs from the file] → Ship `fputs` as the API spells it, and compare case-sensitively.

- [A refactor edits a source file that contains tests] → A `/tests/` path is the only test file. Inline modules stay with Architect and QA.

- [Strict validation fails a requirement past 500 characters] → Each new requirement description stays at or under 500. Detail lives in scenarios.

## Migration Plan

DevOps implements the scripts and the workflow on this branch after the `spec-approved` comment. Tests are committed before that code. Architect then updates the docs. The implementation adds its Unreleased bullet before the archive commit. The merger runs `openspec archive adopt-single-pr-flow --yes` as the last commit and merges when CI is green. There is no second pull request. Not merging is the rollback. Evan's branch-protection click happens after this change, outside the pull request.
