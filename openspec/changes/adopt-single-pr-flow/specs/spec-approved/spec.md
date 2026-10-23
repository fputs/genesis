# spec-approved

## Purpose

The spec-approved check fails a code-path pull request whose spec digest does not match the latest allowed review, and it fails closed when that comparison cannot be made.

## ADDED Requirements

### Requirement: Spec-approved job

The repository SHALL run a job named `spec-approved` from `.github/workflows/spec-approved.yml` on Node 22. That name SHALL be the check context a required check matches. It SHALL test, then run, the base blobs of `scripts/spec-approved.test.mjs` and `scripts/spec-approved.mjs`. The script SHALL have no npm dependencies. Permissions SHALL be only `contents: read` and `pull-requests: read`. Git children SHALL use `LC_ALL=C`.

#### Scenario: The job tests the base script first

- **GIVEN** `.github/workflows/spec-approved.yml`
- **WHEN** the job runs and the base sha has `scripts/spec-approved.mjs`
- **THEN** it uses Node 22
- **AND** the check run name is `spec-approved`
- **AND** `node --test` of the base test blob runs before the base script blob
- **AND** permissions are `contents: read` and `pull-requests: read` only
- **AND** the workflow declares no secrets

### Requirement: Events that re-run the job

The job SHALL run on pull_request `opened`, `synchronize`, `reopened`, `edited`, and `ready_for_review` targeting `main`, and on pull_request_review `submitted` and `edited`. It SHALL NOT filter on review state. It SHALL NOT run on `issue_comment` or on push.

#### Scenario: A review re-runs the job

- **GIVEN** a pull request targeting `main`
- **WHEN** a pull request review is submitted or edited in any state
- **THEN** the `spec-approved` job runs

#### Scenario: An issue comment does not run the job

- **GIVEN** an `issue_comment` on a pull request
- **WHEN** workflows are selected
- **THEN** `spec-approved` does not run

#### Scenario: A push to main does not run the job

- **GIVEN** a push to `main`
- **WHEN** workflows are selected
- **THEN** `spec-approved` does not run

### Requirement: Checkout

For every event the job SHALL check out `pull_request.head.sha` and SHALL fetch `refs/pull/<n>/head` with full history. A fork SHALL use the read-only token and no secrets, and SHALL follow this same checkout. The workflow SHALL NOT use `pull_request_target`. A missing pull request, a missing head sha, or a failed fetch SHALL fail with `spec-approved: pull request unavailable`.

#### Scenario: A review checks out the pull request head

- **GIVEN** a `pull_request_review` on pull request 9
- **WHEN** the job checks out code
- **THEN** it checks out `pull_request.head.sha`
- **AND** it fetches `refs/pull/9/head` with full history
- **AND** it does not check out the default branch

#### Scenario: A fork uses the same checkout

- **GIVEN** a pull request from a fork
- **WHEN** the job checks out code
- **THEN** it checks out that pull request's `head.sha`
- **AND** the token is read-only
- **AND** no secret is available to the job

#### Scenario: A failed pull request lookup fails closed

- **GIVEN** the event has no `pull_request.head.sha`, or fetching `refs/pull/<n>/head` fails
- **WHEN** the job runs
- **THEN** the job fails
- **AND** the log includes `spec-approved: pull request unavailable`

### Requirement: Concurrency

The job SHALL use concurrency group `spec-approved-${{ github.event.pull_request.number }}` with `cancel-in-progress: true`.

#### Scenario: A new push cancels the previous run

- **GIVEN** `.github/workflows/spec-approved.yml`
- **WHEN** the job concurrency is read
- **THEN** the group is `spec-approved-${{ github.event.pull_request.number }}`
- **AND** `cancel-in-progress` is true

### Requirement: Base script and authors file

The job SHALL execute `scripts/spec-approved.mjs` from the pull request base sha and SHALL read `openspec/spec-approved-authors.txt` from that sha. It SHALL load `isCodePath` and `unfencedBody` from the base blobs. The head SHALL be used only to diff and to hash. A head edit to the script or the authors file SHALL NOT change the program or the allow-list.

#### Scenario: A head edit to the authors file is ignored

- **GIVEN** the base authors file lists only user id `239839957`
- **AND** the head adds user id `2` to `openspec/spec-approved-authors.txt`
- **AND** user id `2` submits a review whose body line is `spec-approved: <sha>`
- **AND** the pull request changes a code path
- **WHEN** the job runs
- **THEN** the job fails
- **AND** the log includes `spec-approved: approval missing`

#### Scenario: The head script is not executed

- **GIVEN** the base sha has `scripts/spec-approved.mjs`
- **AND** the head replaces that file with a script that always exits zero
- **WHEN** the job runs
- **THEN** the process is the base blob
- **AND** the head blob is not the program

### Requirement: Bootstrap when the base has no script

When the base sha has no blob `scripts/spec-approved.mjs`, the job SHALL pass and SHALL log `bootstrap: no base script`. It SHALL NOT run a copy from the head. The job SHALL NOT special-case a change id. This notice is how this pull request passes before that blob exists on the base.

#### Scenario: This pull request bootstraps

- **GIVEN** the base sha has no `scripts/spec-approved.mjs`
- **AND** the head adds that script
- **WHEN** the job runs
- **THEN** the job passes
- **AND** the log includes `bootstrap: no base script`
- **AND** the head script does not run

### Requirement: Change id and exemptions

The job SHALL take change ids from `changeReferences`. It SHALL import `isCodePath` from the base spec-drift script and SHALL NOT reimplement it. Changed paths SHALL be the merge-base diff of the base sha and the head. Both rename sides count. No merge-base SHALL fail with `spec-approved: no merge-base`. No code path, or qualifying refactor paths, SHALL pass with no review. Any other refactor line SHALL NOT skip this job. Zero ids SHALL fail with `spec-approved: missing OpenSpec-Change`.

#### Scenario: A docs-only pull request passes without a review

- **GIVEN** a pull_request whose merge-base diff is only `docs/architect/decisions.md`
- **AND** there is no `spec-approved` review
- **WHEN** the job runs
- **THEN** the job passes

#### Scenario: A qualifying refactor skips spec-approved

- **GIVEN** a pull_request whose merge-base diff changes `crates/rules/src/lib.rs` and `CHANGELOG.md`
- **AND** the only reference line is `OpenSpec-Refactor: dice-and-checks`
- **AND** there is no `spec-approved` review
- **WHEN** the job runs
- **THEN** the job passes
- **AND** the log does not include `spec-approved: missing OpenSpec-Change`

#### Scenario: A conflicting refactor line does not skip spec-approved

- **GIVEN** a pull_request whose diff changes `crates/rules/src/lib.rs`
- **AND** the body contains `OpenSpec-Refactor: dice-and-checks` and `OpenSpec-Change: add-foo`
- **AND** `add-foo` exists and has `proposal.md`
- **AND** there is no allowed review
- **WHEN** the job runs
- **THEN** the job fails
- **AND** the log includes `spec-approved: approval missing`

#### Scenario: A code path with no change line fails

- **GIVEN** a pull_request whose diff changes `crates/rules/src/lib.rs`
- **AND** the body has no `OpenSpec-Change` line
- **AND** the qualifying refactor paths do not hold
- **WHEN** the job runs
- **THEN** the job fails
- **AND** the log includes `spec-approved: missing OpenSpec-Change`

### Requirement: Spec digest

The job SHALL hash `proposal.md`, `design.md` when present, and every blob under `specs/` of the named change at the head and at the approved sha. Paths SHALL be relative to the change directory and sorted by UTF-8 bytes. `tasks.md` and any other file SHALL be omitted. The active directory SHALL be used when it exists, otherwise exactly one archive directory whose stripped id matches. The same bytes SHALL match after an archive move.

#### Scenario: Checking tasks does not change the digest

- **GIVEN** an approved digest for change `add-foo`
- **WHEN** only checkbox state in `tasks.md` changes
- **THEN** the digest is unchanged
- **AND** the job passes

#### Scenario: Archiving alone stays neutral

- **GIVEN** an approved digest for active `add-foo`
- **WHEN** the only spec-file change is moving that directory under `openspec/changes/archive/` with the same relative bytes
- **THEN** the digest is unchanged
- **AND** the job passes

#### Scenario: Editing a spec file mismatches

- **GIVEN** an approved digest for change `add-foo`
- **WHEN** `proposal.md`, `design.md`, or a file under `specs/` changes
- **THEN** the job fails
- **AND** the log includes `spec-approved: spec hash mismatch`

#### Scenario: Two copies of the change fail closed

- **GIVEN** both `openspec/changes/add-foo/` and an archive directory for `add-foo` exist
- **WHEN** the job resolves the change path
- **THEN** the job fails
- **AND** the log includes `spec-approved: ambiguous change path`

### Requirement: Spec digest bytes

The digest SHALL be lowercase hex SHA-256. Each hashed file SHALL be encoded as its relative path, a NUL byte, an 8-byte big-endian length, and the raw blob bytes, in sorted path order.

#### Scenario: The digest is sha-256 of the canonical bytes

- **GIVEN** a change directory with `proposal.md` and `specs/aid/spec.md` and no `design.md`
- **WHEN** the digest is computed
- **THEN** it is the lowercase hex SHA-256 of those two files in UTF-8 path order
- **AND** each file contributes its path, a NUL, an 8-byte big-endian length, and the raw bytes

### Requirement: Approval line

After one trailing CR is removed, a line SHALL be valid only when it is exactly `spec-approved: ` plus 40 lowercase hex digits and nothing else. The job SHALL NOT trim. Uppercase hex, a shorter sha, a second space, a missing space, and trailing text SHALL be invalid. Fenced lines SHALL be removed first by pr-hygiene's `unfencedBody`, which the job SHALL import and SHALL NOT reimplement. The pull request body and review line comments SHALL NOT count. Only a `pull_request_review` body SHALL count.

#### Scenario: Uppercase hex is invalid

- **GIVEN** the latest allowed review body is `spec-approved: ` plus 40 uppercase hex digits
- **AND** the pull request changes a code path
- **WHEN** the job runs
- **THEN** the job fails
- **AND** the log includes `spec-approved: approval malformed`

#### Scenario: A short sha is invalid

- **GIVEN** the latest allowed review body is `spec-approved: abcdef`
- **AND** the pull request changes a code path
- **WHEN** the job runs
- **THEN** the job fails
- **AND** the log includes `spec-approved: approval malformed`

#### Scenario: Wrong spacing is invalid

- **GIVEN** the latest allowed review body is `spec-approved:` plus two U+0020 spaces plus 40 lowercase hex digits
- **WHEN** the job runs
- **THEN** the job fails
- **AND** the log includes `spec-approved: approval malformed`

#### Scenario: Trailing text is invalid

- **GIVEN** the latest allowed review body is `spec-approved: ` plus 40 lowercase hex digits plus ` thanks`
- **WHEN** the job runs
- **THEN** the job fails
- **AND** the log includes `spec-approved: approval malformed`

#### Scenario: A malformed line does not fall back

- **GIVEN** an earlier allowed review has a valid `spec-approved: ` line
- **AND** the latest allowed review has a malformed `spec-approved:` line
- **WHEN** the job runs
- **THEN** the job fails
- **AND** the log includes `spec-approved: approval malformed`
- **AND** the earlier review is not used

#### Scenario: A tilde fence hides the line

- **GIVEN** the only approval line sits between a line `~~~` and a closing line `~~~`
- **AND** the pull request changes a code path
- **WHEN** the job runs
- **THEN** the job fails
- **AND** the log includes `spec-approved: approval missing`
- **AND** the fence decision is pr-hygiene's `unfencedBody`

### Requirement: Latest allowed review

The job SHALL GET `pulls/{n}/reviews` with `per_page=100` and SHALL follow every Link `rel=next` until none remains. Among reviews whose `user.id` is allowed, the greatest `submitted_at`, then the greatest numeric review id, SHALL be the latest. `updated_at` SHALL NOT order them. A review whose id is not allowed SHALL NOT displace an allowed one. A missing token, a non-2xx response, or a next page that is not fetched SHALL fail with `spec-approved: reviews unavailable`.

#### Scenario: The later submitted review wins

- **GIVEN** user id `239839957` submitted `spec-approved: <older-sha>` and later `spec-approved: <newer-sha>`
- **AND** the head digest matches `<newer-sha>` and not `<older-sha>`
- **WHEN** the job runs
- **THEN** the job passes

#### Scenario: The review id breaks a submitted_at tie

- **GIVEN** two allowed reviews share one `submitted_at`
- **AND** the greater review id has the sha whose digest matches the head
- **WHEN** the job runs
- **THEN** that review supplies the sha

#### Scenario: Editing a review does not make it newer

- **GIVEN** an allowed review submitted earlier is edited so its `updated_at` is later
- **AND** another allowed review was submitted after the first and before that edit
- **WHEN** the job runs
- **THEN** the later `submitted_at` wins
- **AND** the edit does not reorder them

#### Scenario: A newer disallowed review does not displace

- **GIVEN** user id `239839957` submitted a valid `spec-approved: <sha>` whose digest matches the head
- **AND** a later review from user id `2` has a different sha
- **AND** user id `2` is not in the base authors file
- **WHEN** the job runs
- **THEN** the job passes
- **AND** the disallowed review is not the approval

#### Scenario: A truncated review page fails closed

- **GIVEN** the reviews API returns a Link `rel=next` that the job does not fetch
- **WHEN** the job runs
- **THEN** the job fails
- **AND** the log includes `spec-approved: reviews unavailable`

### Requirement: Approved sha is an ancestor

The sha SHALL be a commit that `git merge-base --is-ancestor` reports as an ancestor of `pull_request.head.sha`, including equality. The digest at that sha SHALL equal the digest at the head. A missing commit, or a rebase or force-push that drops the sha, SHALL fail with `spec-approved: approved sha unavailable`.

#### Scenario: The approved sha need not be the head

- **GIVEN** user id `239839957` approved the spec commit's sha
- **AND** that sha is an ancestor of the head
- **AND** later commits add tests and code without changing the hashed files
- **WHEN** the job runs
- **THEN** the job passes

#### Scenario: A rebase that drops the sha fails closed

- **GIVEN** the latest allowed review names a sha that is not an ancestor of the head
- **WHEN** the job runs
- **THEN** the job fails
- **AND** the log includes `spec-approved: approved sha unavailable`
- **AND** the check is `git merge-base --is-ancestor`

### Requirement: Authors by user id

`openspec/spec-approved-authors.txt` on the base sha SHALL list one GitHub user id per line. Blank lines and lines whose first non-whitespace character is `#` SHALL be ignored. The file SHALL ship with exactly `239839957`. Match SHALL be numeric equality with `user.id`. A login SHALL NOT match. Zero ids, a missing file, or an unreadable file SHALL fail with `spec-approved: authors unavailable`.

#### Scenario: The shipped file allows one user id

- **GIVEN** `openspec/spec-approved-authors.txt`
- **WHEN** its ids are read
- **THEN** the only id is `239839957`

#### Scenario: A login match does not approve

- **GIVEN** the authors file lists `239839957`
- **AND** a review from login `fputs` has a different `user.id`
- **AND** the pull request changes a code path
- **WHEN** the job runs
- **THEN** the job fails
- **AND** the log includes `spec-approved: approval missing`

#### Scenario: An empty authors file fails closed

- **GIVEN** the authors file has no id after comments and blank lines are ignored
- **WHEN** the job runs
- **THEN** the job fails
- **AND** the log includes `spec-approved: authors unavailable`

### Requirement: Fail closed

The job SHALL fail closed when the authors list, the pull request, the review list, the approved sha, or the change path cannot be read completely. An invalid `OpenSpec-Change` token, an unknown id, a missing `proposal.md`, or a non-blob hashed path SHALL fail the job. Every id from `changeReferences` SHALL be checked. The latest allowed review with no approval line SHALL fail with `spec-approved: approval missing` and SHALL NOT fall back.

#### Scenario: An unknown change id fails closed

- **GIVEN** a code-path pull request whose body line is `OpenSpec-Change: missing-change`
- **AND** that id is neither active nor archived
- **WHEN** the job runs
- **THEN** the job fails
- **AND** the log includes `spec-approved: unknown change missing-change`

#### Scenario: A missing proposal fails closed

- **GIVEN** the resolved change directory has no `proposal.md`
- **WHEN** the job hashes the change
- **THEN** the job fails
- **AND** the log includes `spec-approved: proposal missing`

### Requirement: Fixture tests

The repository SHALL contain a fixture test for every pass and fail scenario of this capability. Each case SHALL build a temporary git repository and inject the review list, with no network call. A failing fixture SHALL fail the job.

#### Scenario: Each case is a fixture repository

- **GIVEN** `scripts/spec-approved.test.mjs`
- **WHEN** a table case runs
- **THEN** the test builds a temporary git repository
- **AND** it injects reviews instead of calling GitHub
- **AND** it asserts the exit status and the failure line
