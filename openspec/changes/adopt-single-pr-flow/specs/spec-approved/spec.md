# spec-approved

## Purpose

The spec-approved check fails a code-path pull request whose spec digest does not match the latest allowed approval comment, and it fails closed when that comparison cannot be made.

## ADDED Requirements

### Requirement: Spec-approved job

The repository SHALL run a job named `spec-approved` from `.github/workflows/spec-approved.yml` on Node 22. It SHALL run `node --test scripts/spec-approved.test.mjs` before `scripts/spec-approved.mjs`. The script SHALL have no npm dependencies. Permissions SHALL be only `contents: read`, `pull-requests: read`, and `issues: read`. The job SHALL check out the pull request head with full history. Git children SHALL use `LC_ALL=C`.

#### Scenario: The job tests the script first

- **GIVEN** `.github/workflows/spec-approved.yml`
- **WHEN** the job runs
- **THEN** it uses Node 22
- **AND** `node --test scripts/spec-approved.test.mjs` runs before `scripts/spec-approved.mjs`
- **AND** a failing test fails the job
- **AND** permissions are `contents: read`, `pull-requests: read`, and `issues: read` only

#### Scenario: An issue comment checks out the pull request head

- **GIVEN** an `issue_comment` on pull request 9
- **WHEN** the job checks out code
- **THEN** it checks out that pull request's head sha
- **AND** the history includes the approved sha
- **AND** it does not check out the default branch instead

### Requirement: Events that re-run the job

The job SHALL run on pull_request `opened`, `synchronize`, `reopened`, `edited`, and `ready_for_review` targeting `main`, on issue_comment `created`, `edited`, and `deleted`, and on pull_request_review `submitted` and `edited`. An issue comment that is not on a pull request SHALL NOT run the job. The job SHALL NOT run on push.

#### Scenario: A new approval comment re-runs the job

- **GIVEN** a pull request targeting `main`
- **WHEN** an issue comment is created, edited, or deleted
- **THEN** the `spec-approved` job runs

#### Scenario: A review re-runs the job

- **GIVEN** a pull request targeting `main`
- **WHEN** a pull request review is submitted or edited
- **THEN** the `spec-approved` job runs

#### Scenario: A push to main does not run the job

- **GIVEN** a push to `main`
- **WHEN** workflows are selected
- **THEN** `spec-approved` does not run

#### Scenario: An issue that is not a pull request is skipped

- **GIVEN** an `issue_comment` on an issue that is not a pull request
- **WHEN** workflows are selected
- **THEN** the `spec-approved` job does not run

### Requirement: Change id and exemptions

The job SHALL take change ids from spec-drift's `changeReferences`. It SHALL import `isCodePath` from `scripts/spec-drift-check.mjs` and SHALL NOT reimplement it. Changed paths SHALL be that script's diff from the base sha to the head, including renames. When no path is a code path, or the qualifying refactor paths hold, the job SHALL pass with no approval comment. No other refactor line SHALL skip this job. Zero ids otherwise SHALL fail with `spec-approved: missing OpenSpec-Change`.

#### Scenario: A docs-only pull request passes without a comment

- **GIVEN** a pull_request whose diff is only `docs/architect/decisions.md`
- **AND** there is no `spec-approved` comment
- **WHEN** the job runs
- **THEN** the job passes

#### Scenario: A scripts-only pull request passes without a comment

- **GIVEN** a pull_request whose only diff is `scripts/note.mjs`
- **AND** there is no `spec-approved` comment
- **WHEN** the job runs
- **THEN** the job passes

#### Scenario: A qualifying refactor skips spec-approved

- **GIVEN** a pull_request whose diff changes `crates/rules/src/lib.rs` and `CHANGELOG.md`
- **AND** the only reference line is `OpenSpec-Refactor: dice-and-checks`
- **AND** there is no `spec-approved` comment
- **WHEN** the job runs
- **THEN** the job passes
- **AND** the log does not include `spec-approved: missing OpenSpec-Change`

#### Scenario: A conflicting refactor line does not skip spec-approved

- **GIVEN** a pull_request whose diff changes `crates/rules/src/lib.rs`
- **AND** the body contains `OpenSpec-Refactor: dice-and-checks` and `OpenSpec-Change: add-foo`
- **AND** `add-foo` exists and has `proposal.md`
- **AND** there is no allowed `spec-approved` comment
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

#### Scenario: A workflow file still needs approval

- **GIVEN** a pull_request whose diff changes `.github/workflows/ci.yml`
- **AND** the body line is `OpenSpec-Change: add-foo`
- **AND** that change exists and has `proposal.md`
- **AND** there is no allowed `spec-approved` comment
- **WHEN** the job runs
- **THEN** the job fails
- **AND** the log includes `spec-approved: approval missing`

### Requirement: Spec digest

The job SHALL hash `proposal.md`, `design.md` when present, and every blob under `specs/` of the named change. Paths SHALL be relative to the change directory and sorted by UTF-8 bytes. `tasks.md` and any other file SHALL be omitted. The active directory SHALL be used when it exists, otherwise exactly one archive directory whose stripped id matches. The same bytes SHALL match after an archive move.

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

### Requirement: Latest allowed approval

A valid line, after one trailing CR is removed and the line is trimmed, SHALL be `spec-approved:`, one U+0020, and 40 lowercase hex digits. Fenced lines SHALL NOT count. The job SHALL read issue comments and pull request review bodies, and SHALL ignore review line comments and the pull request body. Among valid lines whose `user.login` is allowed, the greatest `updated_at`, then the greatest numeric id, SHALL supply the sha. The digest at that sha SHALL equal the digest at the head.

#### Scenario: The latest allowed comment wins

- **GIVEN** login `fputs` commented `spec-approved: <older-sha>` and later `spec-approved: <newer-sha>`
- **AND** the head digest matches `<newer-sha>` and not `<older-sha>`
- **WHEN** the job runs
- **THEN** the job passes

#### Scenario: A disallowed login does not approve

- **GIVEN** a code-path pull request whose body line is `OpenSpec-Change: add-foo`
- **AND** that change exists and its head digest matches the sha in the comment
- **AND** only login `someone-else` commented `spec-approved: <sha>`
- **AND** that login is not in the authors file
- **WHEN** the job runs
- **THEN** the job fails
- **AND** the log includes `spec-approved: approval missing`
- **AND** the log does not include `spec-approved: missing OpenSpec-Change`

#### Scenario: A fenced line is not an approval

- **GIVEN** the only `spec-approved: <sha>` line is inside a fence
- **AND** the pull request changes a code path
- **WHEN** the job runs
- **THEN** the job fails
- **AND** the log includes `spec-approved: approval missing`

#### Scenario: The approved sha need not be the head

- **GIVEN** `fputs` approved the spec commit's sha
- **AND** later commits add tests and code without changing the hashed files
- **WHEN** the job runs
- **THEN** the job passes

### Requirement: Configurable authors list

`openspec/spec-approved-authors.txt` SHALL list one GitHub login per line. Blank lines and lines whose first non-whitespace character is `#` SHALL be ignored. The file SHALL ship with exactly the login `fputs`. Comparison with `user.login` SHALL be case-sensitive. Zero logins, a missing file, or an unreadable file SHALL fail the job.

#### Scenario: The shipped file allows fputs

- **GIVEN** `openspec/spec-approved-authors.txt`
- **WHEN** its logins are read
- **THEN** the only login is `fputs`

#### Scenario: An empty authors file fails closed

- **GIVEN** the authors file has no login after comments and blank lines are ignored
- **WHEN** the job runs
- **THEN** the job fails
- **AND** the log includes `spec-approved: authors unavailable`
- **AND** the job does not pass

#### Scenario: A missing authors file fails closed

- **GIVEN** `openspec/spec-approved-authors.txt` is absent
- **WHEN** the job runs on a code-path pull request
- **THEN** the job fails
- **AND** the log includes `spec-approved: authors unavailable`

### Requirement: Fail closed

The job SHALL fail closed, and SHALL NOT pass, when the authors list, the pull request, the comment thread, the approved sha, or the change path cannot be read completely. A partial page list SHALL NOT count as the full thread. An invalid `OpenSpec-Change` token, an unknown id, a missing `proposal.md`, or a non-blob hashed path SHALL fail the job. Every id from `changeReferences` SHALL be checked.

#### Scenario: The comments API fails closed

- **GIVEN** a code-path pull request
- **AND** `GITHUB_TOKEN` is missing, or a comments or reviews request is not 2xx
- **WHEN** the job runs
- **THEN** the job fails
- **AND** the log includes `spec-approved: comments unavailable`
- **AND** the job does not treat the event payload as the full thread

#### Scenario: A truncated comment page fails closed

- **GIVEN** the comments API returns a next page that the job does not receive
- **WHEN** the job runs
- **THEN** the job fails
- **AND** the log includes `spec-approved: comments unavailable`

#### Scenario: An unknown approved sha fails closed

- **GIVEN** the latest allowed comment names a sha that is not a commit in the checkout
- **WHEN** the job runs
- **THEN** the job fails
- **AND** the log includes `spec-approved: approved sha unavailable`

#### Scenario: An invalid change line fails closed

- **GIVEN** a code-path pull request whose only body line is `OpenSpec-Change: Add-Foo`
- **WHEN** the job runs
- **THEN** the job fails
- **AND** the log includes `spec-approved: invalid change id`

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

The repository SHALL contain a fixture test for every pass and fail scenario of this capability. Each case SHALL build a temporary git repository and inject the comment list, with no network call. A failing fixture SHALL fail the job.

#### Scenario: Each case is a fixture repository

- **GIVEN** `scripts/spec-approved.test.mjs`
- **WHEN** a table case runs
- **THEN** the test builds a temporary git repository
- **AND** it injects comments instead of calling GitHub
- **AND** it asserts the exit status and the failure line
