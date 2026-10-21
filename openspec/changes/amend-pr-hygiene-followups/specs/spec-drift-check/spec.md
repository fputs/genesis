# spec-drift-check

## MODIFIED Requirements

### Requirement: Spec-drift job

The repository SHALL run a `spec-drift` job in `.github/workflows/ci.yml` on the same push and pull_request events as the Cargo job. Those triggers SHALL stay unchanged, with no `edited` type. That job SHALL still invoke `scripts/spec-drift-check.mjs`. `.github/workflows/spec-drift-edited.yml` SHALL test, then run that script, on pull_request `edited`. The script SHALL have no npm dependencies. A code-path pull request that drops every `OpenSpec-Change` line on `edited` SHALL fail that re-run.

#### Scenario: Job triggers match the Cargo workflow

- **GIVEN** `.github/workflows/ci.yml` and `.github/workflows/spec-drift-edited.yml`
- **WHEN** the workflow triggers are read
- **THEN** `ci.yml` still runs the Cargo job and the `spec-drift` job on push and pull_request to `main`, with no `edited` type and no skip `if`
- **AND** the `spec-drift` job in `ci.yml` still invokes `scripts/spec-drift-check.mjs`
- **AND** `.github/workflows/spec-drift-edited.yml` runs `node --test scripts/spec-drift-check.test.mjs` before `node scripts/spec-drift-check.mjs` on pull_request `edited` targeting `main`

#### Scenario: Dropping the change line on edit fails

- **GIVEN** a pull_request to `main` that changes `crates/rules/src/lib.rs`
- **AND** `openspec/changes/add-dice-and-checks/` exists
- **AND** the body line `OpenSpec-Change: add-dice-and-checks` made the `spec-drift` job pass
- **WHEN** the pull request is `edited` and the body has no `OpenSpec-Change` line
- **THEN** `.github/workflows/spec-drift-edited.yml` runs
- **AND** the check fails
- **AND** the log includes `spec-drift: code change missing OpenSpec-Change`

### Requirement: Completed unarchived change fails

On push and on pull_request to `main`, the job SHALL fail when any active change in the tree under test has a fully checked `tasks.md`. The tree under test SHALL be the pushed commit on push and the checked-out result after the pull request's changes on pull_request. Push SHALL run this same check only as a backstop. The log SHALL include `spec-drift: completed unarchived change <id>` for each such id, in UTF-8 byte order. The check SHALL NOT compare `tasks.md` with the base commit.

#### Scenario: A pull request that leaves every box checked fails

- **GIVEN** a pull_request to `main` whose checked-out result has active change `add-dice-and-checks` with every task checkbox checked
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: completed unarchived change add-dice-and-checks`

#### Scenario: Push backstop fails that same tree

- **GIVEN** a push to `main` whose tree has active change `add-dice-and-checks` with every task checkbox checked
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: completed unarchived change add-dice-and-checks`

#### Scenario: An open task passes the push

- **GIVEN** a push to `main` whose only active change has one unchecked task checkbox
- **AND** validation exits zero
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes

#### Scenario: An archived completed change passes the push

- **GIVEN** a push to `main` with no active changes
- **AND** `openspec/changes/archive/2026-10-21-add-project-foundation/tasks.md` is fully checked
- **AND** validation exits zero
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes

#### Scenario: One of two active changes is complete

- **GIVEN** a push to `main` where `add-dice-and-checks` is fully checked and `add-spec-drift-check` has an open checkbox
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: completed unarchived change add-dice-and-checks`
- **AND** the log does not include `spec-drift: completed unarchived change add-spec-drift-check`

#### Scenario: Uppercase checked boxes fail the push

- **GIVEN** a push to `main` whose only active change has one checkbox line, `- [X] 1.1 Done`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log names that change id

#### Scenario: Indented boxes fail the push

- **GIVEN** a push to `main` whose only active change has one checkbox line, `  - [x] 1.1 Done`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails

#### Scenario: Completed changes are listed in byte order

- **GIVEN** a push to `main` whose active changes `Zed` and `alpha` are both fully checked
- **WHEN** the `spec-drift` job runs
- **THEN** `spec-drift: completed unarchived change Zed` appears before `spec-drift: completed unarchived change alpha`
- **AND** that order is UTF-8 byte order

### Requirement: Change reference line

A change reference SHALL be one trimmed body line, after one trailing carriage return is removed, of `OpenSpec-Change:` plus one U+0020 plus one token. A token of `a-z` and `0-9` groups separated by single hyphens, not starting or ending with a hyphen, SHALL be the id. Any other token SHALL fail, SHALL NOT be skipped or case-folded, and the log SHALL include `spec-drift: invalid change id <token>`. Fenced lines SHALL count the same. This SHALL be the AGENTS.md line.

#### Scenario: A trimmed line is a reference

- **GIVEN** a pull_request to `main` that changes `Cargo.toml`
- **AND** the body line is `  OpenSpec-Change: add-dice-and-checks  `
- **AND** `openspec/changes/add-dice-and-checks/` exists
- **AND** validation exits zero
- **AND** the completed-change check passes
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes

#### Scenario: Extra words are not a reference

- **GIVEN** a pull_request to `main` that changes `Cargo.toml`
- **AND** the only body line is `OpenSpec-Change: add-dice-and-checks extra`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: invalid change id add-dice-and-checks extra`
- **AND** the log does not include `spec-drift: code change missing OpenSpec-Change`
- **AND** the log does not include `spec-drift: unknown change`

#### Scenario: An uppercase id is not skipped

- **GIVEN** a pull_request to `main` that changes `crates/rules/src/lib.rs`
- **AND** the only body line is `OpenSpec-Change: Add-Foo`
- **AND** `openspec/changes/add-foo/` exists
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: invalid change id Add-Foo`
- **AND** the log does not include `spec-drift: code change missing OpenSpec-Change`
- **AND** the line is not accepted as `add-foo`

#### Scenario: A valid id does not hide an invalid token

- **GIVEN** a pull_request to `main` that changes `Cargo.toml`
- **AND** the body contains `OpenSpec-Change: add-dice-and-checks` and `OpenSpec-Change: Add-Foo`
- **AND** `openspec/changes/add-dice-and-checks/` exists
- **AND** validation exits zero
- **AND** the completed-change check passes
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: invalid change id Add-Foo`

#### Scenario: A tab after the colon is invalid

- **GIVEN** a pull_request to `main` whose diff is only `docs/architect/decisions.md`
- **AND** the body line is `OpenSpec-Change:` immediately followed by a tab and `add-foo`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: invalid change id \u0009add-foo`
- **AND** the log does not include `spec-drift: code change missing OpenSpec-Change`

#### Scenario: Two spaces before the id are invalid

- **GIVEN** a pull_request to `main` whose diff is only `docs/architect/decisions.md`
- **AND** the body line is `OpenSpec-Change:` plus two U+0020 spaces plus `add-foo`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: invalid change id` followed by three U+0020 spaces and `add-foo`
- **AND** the token is not wrapped in JSON quotes
- **AND** the log does not include `spec-drift: code change missing OpenSpec-Change`

### Requirement: Existing change id

An id SHALL exist only when it is an active change id or an archive-stripped id. An active id SHALL be a direct child of `openspec/changes/` other than `archive`. An archive-stripped id SHALL be an undated archive child's name, or the full remainder of `<YYYY>-<MM>-<DD>-<id>`. A dated archive folder name SHALL NOT be an id.

#### Scenario: An active id exists

- **GIVEN** a pull_request to `main` that changes `crates/rules/src/lib.rs`
- **AND** the body line is `OpenSpec-Change: add-dice-and-checks`
- **AND** `openspec/changes/add-dice-and-checks/` exists
- **AND** validation exits zero
- **AND** the completed-change check passes
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes

#### Scenario: A dated archive id exists

- **GIVEN** a pull_request to `main` that changes `crates/rules/src/lib.rs`
- **AND** the body line is `OpenSpec-Change: add-project-foundation`
- **AND** the only copy of that change is `openspec/changes/archive/2026-10-21-add-project-foundation/`
- **AND** validation exits zero
- **AND** the completed-change check passes
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes

#### Scenario: An undated archive directory exists

- **GIVEN** a pull_request to `main` that changes `Cargo.lock`
- **AND** the body line is `OpenSpec-Change: add-project-foundation`
- **AND** the only copy of that change is `openspec/changes/archive/add-project-foundation/`
- **AND** validation exits zero
- **AND** the completed-change check passes
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes

#### Scenario: An unknown id fails

- **GIVEN** a pull_request to `main` that changes `rust-toolchain.toml`
- **AND** the body line is `OpenSpec-Change: missing-change`
- **AND** that id is neither active nor archived
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: unknown change missing-change`

#### Scenario: A partial archive suffix does not match

- **GIVEN** a pull_request to `main` that changes `.github/workflows/ci.yml`
- **AND** the body line is `OpenSpec-Change: foundation`
- **AND** the archive contains only `2026-10-21-add-project-foundation`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: unknown change foundation`

#### Scenario: A dated archive folder name is not an id

- **GIVEN** a pull_request to `main` that changes `crates/rules/src/lib.rs`
- **AND** the body line is `OpenSpec-Change: 2026-10-21-foo`
- **AND** the only copy of that change is `openspec/changes/archive/2026-10-21-foo/`
- **AND** `openspec/changes/2026-10-21-foo/` does not exist
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: unknown change 2026-10-21-foo`

#### Scenario: The stripped id of a dated folder exists

- **GIVEN** a pull_request to `main` that changes `crates/rules/src/lib.rs`
- **AND** the body line is `OpenSpec-Change: foo`
- **AND** the only copy of that change is `openspec/changes/archive/2026-10-21-foo/`
- **AND** validation exits zero
- **AND** the completed-change check passes
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes

#### Scenario: An active dated directory name is still an id

- **GIVEN** a pull_request to `main` that changes `crates/rules/src/lib.rs`
- **AND** the body line is `OpenSpec-Change: 2026-10-21-foo`
- **AND** `openspec/changes/2026-10-21-foo/` exists
- **AND** validation exits zero
- **AND** the completed-change check passes
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes

### Requirement: Edited spec-drift workflow

`.github/workflows/spec-drift-edited.yml` SHALL run `node --test scripts/spec-drift-check.test.mjs` before `node scripts/spec-drift-check.mjs` on pull_request `edited` targeting `main`. Its only job SHALL be named exactly `spec-drift`. It SHALL set `permissions: contents: read`. Its job-level concurrency SHALL be the shared spec-drift group, with `cancel-in-progress: true`.

#### Scenario: The edited workflow reuses the spec-drift job name

- **GIVEN** `.github/workflows/spec-drift-edited.yml`
- **WHEN** its lines are checked
- **THEN** it runs on pull_request `edited` targeting `main`
- **AND** its only job is named `spec-drift`
- **AND** that job runs `node --test scripts/spec-drift-check.test.mjs` before `node scripts/spec-drift-check.mjs`
- **AND** permissions are `contents: read`
- **AND** that job's concurrency group is `spec-drift-${{ github.event.pull_request.number || github.sha }}`
- **AND** `cancel-in-progress` is true
- **AND** the workflow has no workflow-level concurrency group

## ADDED Requirements

### Requirement: Visible invalid change id

An invalid-id log line SHALL escape every UTF-16 code unit outside U+0020 through U+007E as `\u` plus four lowercase hex digits. Printable ASCII SHALL be copied unchanged, with no JSON quotes. An empty token SHALL omit the token.

#### Scenario: A zero-width space is visible

- **GIVEN** a pull_request to `main` whose diff is only `docs/architect/decisions.md`
- **AND** the body line is `OpenSpec-Change:` plus one U+0020 plus U+200B plus `add-foo`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: invalid change id \u200badd-foo`
- **AND** the log does not contain U+200B

#### Scenario: An NBSP is visible

- **GIVEN** a pull_request to `main` whose diff is only `docs/architect/decisions.md`
- **AND** the body line is `OpenSpec-Change:` plus one U+0020 plus U+00A0 plus `add-foo`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: invalid change id \u00a0add-foo`
- **AND** the log does not contain U+00A0

#### Scenario: An em space is visible

- **GIVEN** a pull_request to `main` whose diff is only `docs/architect/decisions.md`
- **AND** the body line is `OpenSpec-Change:` plus one U+0020 plus U+2003 plus `add-foo`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: invalid change id \u2003add-foo`
- **AND** the log does not contain U+2003

### Requirement: Malformed change line

A trimmed body line that starts with `OpenSpec-Change:` and is not exactly `OpenSpec-Change:` plus one U+0020 plus one non-whitespace token SHALL fail as an invalid id on every pull request. It SHALL NOT be skipped. A valid line beside it SHALL NOT make the check pass.

#### Scenario: A tabbed extra word beside a valid line fails

- **GIVEN** a pull_request to `main` that changes `Cargo.toml`
- **AND** the body contains `OpenSpec-Change: add-dice-and-checks` and a line of `OpenSpec-Change:` plus one U+0020 plus a tab plus `Add-Foo extra`
- **AND** `openspec/changes/add-dice-and-checks/` exists
- **AND** validation exits zero
- **AND** the completed-change check passes
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: invalid change id \u0009Add-Foo extra`
- **AND** the log does not include `spec-drift: code change missing OpenSpec-Change`

#### Scenario: A docs-only malformed line fails

- **GIVEN** a pull_request to `main` whose diff is only `README.md`
- **AND** the only body line is `OpenSpec-Change: add-dice-and-checks extra`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: invalid change id add-dice-and-checks extra`
- **AND** the log does not include `spec-drift: code change missing OpenSpec-Change`

### Requirement: Shared spec-drift concurrency

The `spec-drift` job in `.github/workflows/ci.yml` and in `.github/workflows/spec-drift-edited.yml` SHALL share job-level group `spec-drift-${{ github.event.pull_request.number || github.sha }}` with `cancel-in-progress: true`. The Cargo job SHALL NOT use that group. A push SHALL NOT join a pull request group.

#### Scenario: Both spec-drift jobs share a pull request group

- **GIVEN** `.github/workflows/ci.yml` and `.github/workflows/spec-drift-edited.yml`
- **WHEN** the `spec-drift` job in each file is read
- **THEN** each job sets concurrency group `spec-drift-${{ github.event.pull_request.number || github.sha }}`
- **AND** each sets `cancel-in-progress: true`
- **AND** `ci.yml` does not set that group at workflow level
- **AND** the Cargo `ci` job has no concurrency key
- **AND** `spec-drift-edited.yml` has no workflow-level concurrency group

#### Scenario: A push does not join a pull request group

- **GIVEN** a push event has no pull request number
- **WHEN** the `ci.yml` spec-drift concurrency group is evaluated
- **THEN** the group uses `github.sha`
- **AND** it is not a pull request group
