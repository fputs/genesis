# Design

## Context

See proposal.md for why. `AGENTS.md` section 1 and `docs/architect/decisions.md` still say Evan merges a spec pull request and Architect opens a second one. `scripts/spec-drift-check.mjs` exports `changeReferences`. `isCodePath` is private there and copied in `scripts/pr-hygiene.mjs`.

`unfencedBody` in `scripts/pr-hygiene.mjs` is the CommonMark fence helper. The GitHub user id for login `fputs` is `239839957`. There is no `CODEOWNERS` file. Normative behavior is the three delta specs under this change.

## Goals / Non-Goals

**Goals:**

- Compare a spec digest, not the head sha, so tests, code, and archive do not by themselves revoke approval.
- Fail closed when the allow-list or the review list cannot be read in full.
- Let one kebab refactor line satisfy spec-drift without `OpenSpec-Change`, including the `CHANGELOG.md` path the bullet needs.
- Keep the constitution and the workflow docs on the same flow.

**Non-Goals:**

- GitHub branch protection, rulesets, or required checks. Evan turns those on. The decisions note lists both admin steps, not as tasks.
- Changing `pr-hygiene` archive-last, the Cargo job, or spec-drift triggers.
- A CI check that commit order is spec, then tests, then code. Review checks that order.
- Deciding that a refactor left behavior unchanged. Architect and QA do that in the combined review.
- A new npm dependency, or any edit under `crates/rules` for this change's own implementation.

## Decisions

### 1. The approval compares spec bytes

The digest covers `proposal.md`, `design.md` when it exists, and every blob under `specs/`. Paths are relative to the change directory, sorted by UTF-8 bytes. Each file is the path, a NUL, an 8-byte big-endian length, and the raw blob. The digest is lowercase hex SHA-256. `tasks.md` is omitted, so checkbox edits do not matter. An archive move keeps the same relative paths and the same digest.

The active directory wins. If it is absent, exactly one archive directory whose stripped id matches is used, with spec-drift's dated and undated rules. Two matches fail with `spec-approved: ambiguous change path`. A missing `proposal.md` fails with `spec-approved: proposal missing`.

Alternative considered: require the comment sha to equal the head. Rejected. Every later test commit would need a new approval even when the spec bytes did not change.

### 2. The allow-list is user ids on the base

`openspec/spec-approved-authors.txt` has one decimal GitHub user id per line. Blank lines and `#` comments are ignored. The shipped file contains only `239839957`, the user id behind login `fputs`. Match `user.id` numerically. A login match does not count. The job reads that file from the pull request base sha, not the head. An empty, missing, or unreadable file fails with `spec-approved: authors unavailable`. The file lives under `/openspec/`, which `@fputs` owns.

Alternative considered: match `user.login`. Rejected. Logins change, and a head edit could add a login the base never allowed.

### 3. Approval is one review body

Option a: the only approval source is a `pull_request_review` body, submitted or edited, in any state. Issue comments do not approve, and the workflow does not run on `issue_comment`. The line, after one trailing CR and with no trim, is exactly `spec-approved: ` plus 40 lowercase hex digits. Uppercase hex, a short sha, wrong spacing, and trailing text are `spec-approved: approval malformed`.

Fences are CommonMark, via `unfencedBody` exported from `scripts/pr-hygiene.mjs`. A `~~~` fence hides the line. The job does not reimplement that helper. The pull request body and review line comments do not count.

The list is `GET pulls/{n}/reviews?per_page=100`, following Link `rel=next`. Among allowed user ids, the greatest `submitted_at` wins, then the greatest review id. `updated_at` does not reorder. A newer review from a disallowed id does not displace an allowed one.

The latest allowed review is the only source. A malformed line fails with `spec-approved: approval malformed` and does not fall back. No line there fails with `spec-approved: approval missing` and does not fall back. A missing token, a non-2xx response, or a skipped next page fails with `spec-approved: reviews unavailable`.

The sha must pass `git merge-base --is-ancestor` against `pull_request.head.sha`. A rebase or force-push that drops it fails with `spec-approved: approved sha unavailable`. The digest at that sha must equal the digest at the head.

Alternative considered: issue comments, ordered by `updated_at`. Rejected. Reviews are the approval record, and an edit must not make an older review newer.

### 4. The script runs from the base sha

`.github/workflows/spec-approved.yml` is its own workflow so a review does not re-run Cargo. The check context name is `spec-approved`. Concurrency is `spec-approved-${{ github.event.pull_request.number }}` with `cancel-in-progress: true`.

Every event checks out `pull_request.head.sha` and fetches `refs/pull/<n>/head` with full history. A fork gets the read-only token and no secrets, and the same checkout. A failed lookup fails with `spec-approved: pull request unavailable`. Permissions are `contents: read` and `pull-requests: read`. The workflow declares no secrets and does not use `pull_request_target`.

The steps execute `scripts/spec-approved.mjs` and read the authors file from the base sha. `isCodePath` and `unfencedBody` load from the base blobs. The head is only the diff and the hash. A head that adds another user id, and a review from that id, still fails with `spec-approved: approval missing`.

When the base has no `scripts/spec-approved.mjs` blob, the job passes and logs `bootstrap: no base script`. It does not run the head's script and does not special-case a change id. That is this pull request, whose base is `main` before the script exists. After merge, the base has the blob and the notice does not apply.

Alternative considered: run the head's script. Rejected. The head could replace the allow-list and the program together.

### 5. Spec-approved reuses spec-drift's path rules

`spec-approved.mjs` imports `changeReferences` and `isCodePath`. This change exports `isCodePath`. Changed paths are `git diff` from `git merge-base` of the base sha and the head sha to the head, including both sides of a rename. That is the same base pr-hygiene uses. The job does not diff `pull_request.base.sha` directly.

No merge-base fails with `spec-approved: no merge-base`. No code path, or a pull request that meets the qualifying refactor paths, passes with no review. A conflicting, malformed, or repeated refactor line does not.

Every returned id is hashed. An invalid token fails with `spec-approved: invalid change id`. An unknown id fails with `spec-approved: unknown change <id>`. Zero ids on a non-exempt code path fail with `spec-approved: missing OpenSpec-Change`. A hash difference fails with `spec-approved: spec hash mismatch`.

### 6. The refactor exemption follows pull request 79

`Code paths` and `Change reference line` are MODIFIED so the merged spec states the carve-out. The path diff for those requirements and for the refactor rule is `git merge-base` of the base sha and the head, matching pr-hygiene. Pull request 79 is not edited. Where that draft is stricter than the original brief, the draft wins. The delta keeps the new rules and drops requirements that only restated unchanged workflow behavior or the human gate.

The roadmap item is one kebab token, `^[a-z0-9]+(?:-[a-z0-9]+)*$`, after exactly one U+0020. The brief allowed any non-empty item, including spaces. The draft is stricter. Extra words, uppercase, a slash, and a double hyphen log `spec-drift: invalid refactor item` plus the escaped remainder. The item is not looked up.

A test file is a `crates/` path containing `/tests/`. Status `M`, `D`, or `T` on that path, and a rename whose old path is a test file, log `spec-drift: refactor changes existing tests`. Status `A` is allowed. `crates/rules/src/tests.rs` is not a test file. A filename rule ending in `tests.rs` would have classified it. The draft does not. The script does not parse Rust.

Both an `OpenSpec-Refactor` line and an `OpenSpec-Change` line fail with `spec-drift: refactor line conflicts with OpenSpec-Change`, including on a docs-only pull request. The brief would have kept the change id and ignored the refactor line. The draft is stricter. A repeated refactor line counts again and logs `spec-drift: multiple OpenSpec-Refactor lines`.

The brief required every path under `crates/`. Exactly `CHANGELOG.md` may also sit in the diff, so a bullet those other specs require can land in the same change. This change follows that draft.

Passing the exemption does not waive a bullet the changelog and pr-hygiene specs require. This job does not check the bullet. Main changed after pull request 76, so this delta does not restate a `/tests/` exemption or its absence. `OpenSpec-Refactor` is not an own-change line. Those two specs are not edited.

Path lines are logged only for one valid refactor line, no change line, and a code path. Their order is crates-only, then openspec, then existing tests, after the token-order lines. Push still skips the reference check and does not read refactor lines. Fenced refactor lines still count. `spec-approved` skips only when the qualifying paths hold. Unchanged behavior stays the human gate in `single-pr-flow`, not a second spec-drift requirement.

Alternative considered: a free-text item and one `refactor exemption refused` log. Rejected. Pull request 79 already closed the holes the change-id rule closed, and it named the failure lines.

### 7. Docs say the same thing as the spec

`AGENTS.md` section 1 replaces the two-pull-request sentences and keeps the review, git, and port rules. The config `tasks` rule ends with the sentence in the single-pr-flow spec. The three workflow docs drop "only specs merged to main".

They record two admin steps: Evan enables code-owner review, and Evan makes the `spec-approved` check required. CODEOWNERS does nothing until both are on. This change does not edit branch protection. A small clarification still cites Designer. Because it edits hashed files, `spec-approved` fails until a new review matches.

## Risks / Trade-offs

- [Checkout uses the default branch] → Check out `pull_request.head.sha` and fetch `refs/pull/<n>/head` with full history, including forks.

- [A short review page looks like no approval] → A missing Link `rel=next` fails closed.

- [Archive rewrites only `tasks.md`] → That file is outside the digest, and paths are relative.

- [The head adds its own author] → Read the authors file and the script from the base sha. Match `user.id`.

- [A refactor edits a source file that contains tests] → A `/tests/` path is the only test file. Inline modules stay with Architect and QA.

- [Strict validation fails a requirement past 500 characters] → Each new requirement description stays at or under 500. Detail lives in scenarios.

## Migration Plan

DevOps implements the scripts and the workflow on this branch after the `spec-approved` review. Tests are committed before that code. Architect then updates the docs. The implementation adds its Unreleased bullet before the archive commit.

The merger runs `openspec archive adopt-single-pr-flow --yes` as the last commit and merges when CI is green. There is no second pull request. Not merging is the rollback. Until that merge, the base has no script, so the job logs `bootstrap: no base script` and passes. Evan's two admin clicks happen after this change, outside the pull request.
