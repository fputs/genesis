# spec-drift-check

## ADDED Requirements

### Requirement: Fixture cleanup retries ENOTEMPTY

Each spec-drift fixture case SHALL remove its temporary git directory when the case ends. When removal returns `ENOTEMPTY`, including `rmdir` of `.git/objects`, it SHALL retry until the directory is gone. That `ENOTEMPTY` SHALL NOT fail the test. The test SHALL fail if the directory remains after the retries.

#### Scenario: A transient ENOTEMPTY is retried

- **GIVEN** a fixture cleanup whose first removal of the temporary git directory returns `ENOTEMPTY` on `.git/objects`
- **AND** the next removal succeeds
- **WHEN** the case ends
- **THEN** the directory is gone
- **AND** the test does not fail

## MODIFIED Requirements

### Requirement: Spec-drift job

The repository SHALL run a `spec-drift` job in `.github/workflows/ci.yml` on the same push and pull_request events as the Cargo job. Triggers SHALL add only `merge_group`, not an `edited` type. That job SHALL still invoke `scripts/spec-drift-check.mjs`. `.github/workflows/spec-drift-edited.yml` SHALL test, then run that script, on pull_request `edited`. The script SHALL have no npm dependencies. A code-path pull request that drops every `OpenSpec-Change` line on `edited` SHALL fail that re-run.

#### Scenario: Job triggers match the Cargo workflow

- **GIVEN** `.github/workflows/ci.yml` and `.github/workflows/spec-drift-edited.yml`
- **WHEN** the workflow triggers are read
- **THEN** `ci.yml` still runs the Cargo job and the `spec-drift` job on push and pull_request to `main`, plus `merge_group`, with no `edited` type and no skip `if`
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

### Requirement: Cargo job remains

The workflow SHALL keep the existing job that runs `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo build --workspace`, and `cargo test --workspace`. `.github/workflows/ci.yml` SHALL add `merge_group` to its triggers and SHALL NOT skip that job. A body edit SHALL NOT replace a failed Cargo check. `.github/workflows/spec-drift-edited.yml` SHALL run a job named `spec-drift` on pull_request `edited`.

#### Scenario: Cargo steps stay

- **GIVEN** `.github/workflows/ci.yml` after this change is applied
- **WHEN** the Cargo job is read
- **THEN** it still runs fmt check, clippy with warnings denied, workspace build, and workspace test

#### Scenario: Editing the body does not replace a failed Cargo check

- **GIVEN** a pull_request to `main` whose Cargo job has failed
- **WHEN** the pull request body is edited
- **THEN** `.github/workflows/ci.yml` does not run for that `edited` event
- **AND** the failed Cargo check stays failed
- **AND** only `.github/workflows/spec-drift-edited.yml` runs

### Requirement: Completed unarchived change fails

On push, pull_request, and merge_group, the job SHALL fail when any active change in the checkout has a fully checked `tasks.md`. The tree under test SHALL be the pushed commit on push and the checked-out result after the pull request's changes on pull_request. Push and merge_group SHALL run this same check as a backstop. The log SHALL include `spec-drift: completed unarchived change <id>` for each such id, in UTF-8 byte order. The check SHALL NOT compare `tasks.md` with the base commit.

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
#### Scenario: A merge group backstop fails a completed change

- **GIVEN** a merge_group checkout whose tree has active change `add-dice-and-checks` with every task checkbox checked
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: completed unarchived change add-dice-and-checks`

### Requirement: Push skips the reference check

On push to `main` and on `merge_group`, the job SHALL NOT fail for a missing or unknown change reference, including when a code path changed. The merge-group run SHALL still run validation and the completed-change check.

#### Scenario: A code push has no pull request body

- **GIVEN** a push to `main` that changes `crates/rules/src/lib.rs`
- **AND** there is no pull request body
- **AND** validation exits zero
- **AND** no active change is fully checked
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes
- **AND** the log does not include `spec-drift: code change missing OpenSpec-Change`
#### Scenario: A merge group skips the reference check

- **GIVEN** a merge_group checkout whose diff changes `crates/rules/src/lib.rs`
- **AND** there is no pull request body
- **AND** validation exits zero
- **AND** no active change is fully checked
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes
- **AND** the log does not include `spec-drift: code change missing OpenSpec-Change`

