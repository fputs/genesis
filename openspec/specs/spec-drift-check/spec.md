# spec-drift-check Specification

## Purpose
The spec-drift-check capability fails CI when OpenSpec validation fails, when an active change is fully checked but unarchived, or when a code pull request names no existing change id.

## Requirements

### Requirement: Spec-drift job

The repository SHALL run a `spec-drift` job in `.github/workflows/ci.yml` on push and pull_request events targeting `main`. The job SHALL invoke `scripts/spec-drift-check.mjs`. That script SHALL have no npm dependencies: it only needs Node, which the pinned OpenSpec CLI already requires, so the three checks stay testable with `node --test` and no new package.

#### Scenario: Job triggers match the Cargo workflow

- **GIVEN** `.github/workflows/ci.yml`
- **WHEN** the workflow triggers are read
- **THEN** the `spec-drift` job runs on push and pull_request to `main`
- **AND** it invokes `scripts/spec-drift-check.mjs`

### Requirement: Cargo job remains

The workflow SHALL keep the existing job that runs `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo build --workspace`, and `cargo test --workspace`.

#### Scenario: Cargo steps stay

- **GIVEN** `.github/workflows/ci.yml` after this change is applied
- **WHEN** the Cargo job is read
- **THEN** it still runs fmt check, clippy with warnings denied, workspace build, and workspace test

### Requirement: Pinned OpenSpec CLI

The `spec-drift` job SHALL run `@fission-ai/openspec` version `1.14.3` exactly, the same pin as the AGENTS.md proof command, so validation cannot float. It SHALL NOT use a floating tag or a version range for that package. This named CLI is the only added tool.

#### Scenario: Workflow pins the CLI

- **GIVEN** `.github/workflows/ci.yml`
- **WHEN** the `spec-drift` job is read
- **THEN** it installs `@fission-ai/openspec@1.14.3`
- **AND** it does not select `@latest` or a version range for that package

### Requirement: Local inputs only

The job SHALL decide using the local checkout, the local `openspec` binary, and, on pull_request, the pull request body from the workflow event payload. It SHALL NOT contact a network service. On pull_request the checkout SHALL include the base commit and the checked-out commit.

#### Scenario: A missing reference needs no network lookup

- **GIVEN** a pull_request to `main` whose event payload body has no change reference
- **AND** the diff changes `crates/rules/src/lib.rs`
- **AND** the runner cannot contact a network service
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: code change missing OpenSpec-Change`

### Requirement: Strict OpenSpec validation

The job SHALL run `openspec validate --all --strict` from the repository root. A non-zero exit SHALL fail the job. A zero exit SHALL pass the validation check. The log SHALL include the command output.

#### Scenario: Invalid tree fails validation

- **GIVEN** a push or pull_request to `main` whose tree makes `openspec validate --all --strict` exit non-zero
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: validate failed`
- **AND** the log includes that command's output

#### Scenario: Valid tree passes validation

- **GIVEN** a tree on which `openspec validate --all --strict` exits zero
- **WHEN** the validation check runs
- **THEN** the validation check passes

### Requirement: Active change

An active change SHALL be a direct child directory of `openspec/changes/` other than `archive`. The directory name SHALL be the change id.

#### Scenario: Archive children are not active

- **GIVEN** `openspec/changes/archive/2026-10-21-add-project-foundation/`
- **WHEN** active changes are listed
- **THEN** `add-project-foundation` is not an active change

#### Scenario: A sibling of archive is active

- **GIVEN** `openspec/changes/add-dice-and-checks/`
- **WHEN** active changes are listed
- **THEN** `add-dice-and-checks` is an active change

### Requirement: Fully checked tasks file

An active change's `tasks.md` SHALL be fully checked only when it has at least one tracked checkbox and every tracked checkbox is checked. A tracked checkbox SHALL be a line matching leading whitespace, `- [`, bracket text, `]`, then the end of the line or whitespace. Bracket text SHALL be checked only when it is optional whitespace around a single `x` or `X`. Any other bracket text SHALL be unchecked. A missing `tasks.md` SHALL contain no tracked checkbox.

#### Scenario: A titled box counts

- **GIVEN** a `tasks.md` whose only checkbox line is `- [x] 1.1 Record the decision`
- **WHEN** the file is classified
- **THEN** it is fully checked

#### Scenario: Space around the x still counts as checked

- **GIVEN** a `tasks.md` whose only checkbox line is `- [ x] 1.1 Done`
- **WHEN** the file is classified
- **THEN** it is fully checked

#### Scenario: An open box is not fully checked

- **GIVEN** a `tasks.md` with `- [x] 1.1 Done` and `- [ ] 1.2 Left`
- **WHEN** the file is classified
- **THEN** it is not fully checked

#### Scenario: No checkboxes is not fully checked

- **GIVEN** a `tasks.md` with prose and no task checkbox
- **WHEN** the file is classified
- **THEN** it is not fully checked

#### Scenario: A missing tasks file is not fully checked

- **GIVEN** an active change directory with no `tasks.md`
- **WHEN** the change is classified
- **THEN** it is not fully checked

#### Scenario: Other list markers are not checkboxes

- **GIVEN** a `tasks.md` whose only lists are `* [x] done` and `1. [x] done`
- **WHEN** the file is classified
- **THEN** it is not fully checked

#### Scenario: A checkbox needs whitespace before the title

- **GIVEN** a `tasks.md` whose only list line is `- [x]title`
- **WHEN** the file is classified
- **THEN** it is not fully checked

#### Scenario: A non-x marker is unchecked

- **GIVEN** a `tasks.md` whose only checkbox line is `- [~] 1.1 Done`
- **WHEN** the file is classified
- **THEN** it is not fully checked

#### Scenario: An empty box is unchecked

- **GIVEN** a `tasks.md` whose only checkbox line is `- [] 1.1 Done`
- **WHEN** the file is classified
- **THEN** it is not fully checked

### Requirement: Completed unarchived change fails

On push and on pull_request to `main`, the job SHALL fail when any active change in the tree under test has a fully checked `tasks.md`. The tree under test SHALL be the pushed commit on push and the checked-out result after the pull request's changes on pull_request. Push SHALL run this same check only as a backstop. The log SHALL include `spec-drift: completed unarchived change <id>` for each such id. The check SHALL NOT compare `tasks.md` with the base commit.

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

### Requirement: Keeping the active change fails

A pull_request to `main` SHALL fail when its checked-out result still has a fully checked active change, including when an archive copy of that change is also present.

#### Scenario: A later pull request still has the completed change

- **GIVEN** a pull_request to `main` whose checked-out result has active `add-dice-and-checks` fully checked
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: completed unarchived change add-dice-and-checks`

#### Scenario: Copying to archive without removing the active change fails

- **GIVEN** a pull_request to `main` whose result keeps active `add-dice-and-checks` fully checked
- **AND** the result also has `openspec/changes/archive/2026-10-21-add-dice-and-checks/`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: completed unarchived change add-dice-and-checks`

### Requirement: Implementation pull request archives in its last commit

The pull request that completes a change SHALL pass the completed-change check only when its checked-out result has no fully checked active change. That result SHALL be produced by a final commit that runs `openspec archive <id> --yes` for each change the pull request completed, the same final commit AGENTS.md requires. There SHALL be no separate archive pull request. The push of that result to `main` SHALL pass the same check, so `main` SHALL stay green.

#### Scenario: Checked tasks still active fails the pull request

- **GIVEN** a pull_request to `main` whose base has active `add-dice-and-checks` with an open checkbox
- **AND** the checked-out result has that change active and fully checked
- **AND** validation exits zero
- **AND** the diff does not change a code path
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: completed unarchived change add-dice-and-checks`

#### Scenario: A new fully checked active change fails

- **GIVEN** a pull_request to `main` whose checked-out result adds active `add-dice-and-checks` already fully checked
- **AND** validation exits zero
- **AND** the diff does not change a code path
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: completed unarchived change add-dice-and-checks`

#### Scenario: The last commit is the archive

- **GIVEN** an implementation pull request that completed `add-dice-and-checks`
- **WHEN** its commits are read
- **THEN** the last commit is `openspec archive add-dice-and-checks --yes`
- **AND** the checked-out result has no active `add-dice-and-checks` directory
- **AND** the result has `openspec/changes/archive/` dated with that id and a fully checked `tasks.md`

#### Scenario: The archived result passes the pull request

- **GIVEN** that archived checked-out result
- **AND** validation exits zero
- **AND** the diff does not change a code path
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes

#### Scenario: The following push stays green

- **GIVEN** the push to `main` of that same archived result
- **AND** validation exits zero
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes
- **AND** the log does not include `spec-drift: completed unarchived change`

### Requirement: Code paths

A changed path is a code path when it is exactly `Cargo.toml`, `Cargo.lock`, or `rust-toolchain.toml`, or when it starts with `crates/` or `.github/workflows/`. Changed paths SHALL be the additions, modifications, deletions, and both sides of renames between the pull request base commit and the checked-out commit. No other path is a code path.

#### Scenario: A crate file is a code path

- **GIVEN** a pull_request to `main` whose diff changes `crates/rules/src/lib.rs`
- **AND** the body has no change reference
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: code change missing OpenSpec-Change`

#### Scenario: Root Cargo.toml is a code path

- **GIVEN** a pull_request to `main` whose diff changes `Cargo.toml` and no other code path
- **AND** the body has no change reference
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: code change missing OpenSpec-Change`

#### Scenario: Cargo.lock is a code path

- **GIVEN** a pull_request to `main` whose diff changes `Cargo.lock` and no other code path
- **AND** the body has no change reference
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: code change missing OpenSpec-Change`

#### Scenario: rust-toolchain.toml is a code path

- **GIVEN** a pull_request to `main` whose diff changes `rust-toolchain.toml` and no other code path
- **AND** the body has no change reference
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: code change missing OpenSpec-Change`

#### Scenario: A workflow file is a code path

- **GIVEN** a pull_request to `main` whose diff changes `.github/workflows/ci.yml`
- **AND** the body has no change reference
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: code change missing OpenSpec-Change`

#### Scenario: Deleting a crate file is a code path

- **GIVEN** a pull_request to `main` whose diff deletes `crates/rules/src/lib.rs`
- **AND** the body has no change reference
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: code change missing OpenSpec-Change`

#### Scenario: Renaming into crates is a code path

- **GIVEN** a pull_request to `main` whose diff renames `docs/note.md` to `crates/rules/note.md`
- **AND** the body has no change reference
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: code change missing OpenSpec-Change`

#### Scenario: Renaming out of crates is a code path

- **GIVEN** a pull_request to `main` whose diff renames `crates/rules/src/lib.rs` to `docs/lib.rs`
- **AND** the body has no change reference
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: code change missing OpenSpec-Change`

#### Scenario: A nested Cargo.toml is not a root manifest

- **GIVEN** a pull_request to `main` whose only diff is `docs/Cargo.toml`
- **AND** the body has no change reference
- **AND** validation exits zero
- **AND** no active change is fully checked
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes

#### Scenario: A crates prefix without a slash is not a code path

- **GIVEN** a pull_request to `main` whose only diff is `crates-extra/foo.rs`
- **AND** the body has no change reference
- **AND** validation exits zero
- **AND** no active change is fully checked
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes

#### Scenario: Files outside the diff do not count

- **GIVEN** a pull_request to `main` whose diff is only `README.md`
- **AND** the tree contains `crates/rules/src/lib.rs`
- **AND** the body has no change reference
- **AND** validation exits zero
- **AND** no active change is fully checked
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes

### Requirement: Change reference line

A change reference SHALL be one body line that, after removing one trailing carriage return and trimming leading and trailing whitespace, is `OpenSpec-Change: ` plus a change id. A change id SHALL match one or more groups of `a-z` and `0-9` separated by single hyphens, and SHALL NOT start or end with a hyphen. Any other line SHALL NOT be a change reference. Lines inside fences count the same as other lines. This SHALL be the AGENTS.md `OpenSpec-Change: <id>` line.

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
- **AND** the log includes `spec-drift: code change missing OpenSpec-Change`
- **AND** the log does not include `spec-drift: unknown change`

### Requirement: Existing change id

A change id exists when `openspec/changes/<id>/` is an active change, or when a direct child of `openspec/changes/archive/` is named `<id>` or `<YYYY>-<MM>-<DD>-<id>` with a four-digit year and two-digit month and day. The dated name SHALL match only when the full remainder after that prefix equals the id.

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

### Requirement: Several change ids

When a code-path pull request has one or more change references, the reference check SHALL pass only if every distinct id exists. A repeated id SHALL count once. Each unknown id SHALL produce `spec-drift: unknown change <id>`. When at least one change reference is present, the job SHALL NOT also log `spec-drift: code change missing OpenSpec-Change`.

#### Scenario: Two existing ids pass

- **GIVEN** a pull_request to `main` that changes `crates/rules/src/lib.rs`
- **AND** the body contains `OpenSpec-Change: add-dice-and-checks` and `OpenSpec-Change: add-project-foundation`
- **AND** the first id is active and the second exists only under `openspec/changes/archive/2026-10-21-add-project-foundation/`
- **AND** validation exits zero
- **AND** the completed-change check passes
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes

#### Scenario: One of two ids is unknown

- **GIVEN** a pull_request to `main` that changes `Cargo.toml`
- **AND** the body contains `OpenSpec-Change: add-dice-and-checks` and `OpenSpec-Change: missing-change`
- **AND** only `add-dice-and-checks` exists
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: unknown change missing-change`
- **AND** the log does not include `spec-drift: code change missing OpenSpec-Change`

#### Scenario: A duplicate id counts once

- **GIVEN** a pull_request to `main` that changes `Cargo.toml`
- **AND** the body contains `OpenSpec-Change: add-dice-and-checks` twice
- **AND** that active change exists
- **AND** validation exits zero
- **AND** the completed-change check passes
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes

### Requirement: Docs-only and openspec-only pull requests

When the diff has no code path, the reference check SHALL pass and SHALL ignore the body. A docs-only diff and an openspec-only diff are this case. The job SHALL NOT fail those pull requests for a missing line. AGENTS.md still requires the line on the pull request body; this check only withholds the CI failure.

#### Scenario: Docs-only pull request passes without a reference

- **GIVEN** a pull_request to `main` whose diff is `docs/architect/decisions.md` and `README.md`
- **AND** the body has no change reference
- **AND** validation exits zero
- **AND** no active change is fully checked
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes

#### Scenario: Openspec-only pull request passes without a reference

- **GIVEN** a pull_request to `main` whose diff is only under `openspec/`
- **AND** the body has no change reference
- **AND** validation exits zero
- **AND** no active change is fully checked
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes

#### Scenario: An exempt pull request ignores an unknown id

- **GIVEN** a pull_request to `main` whose diff is only `README.md`
- **AND** the body line is `OpenSpec-Change: missing-change`
- **AND** validation exits zero
- **AND** no active change is fully checked
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes

#### Scenario: Docs plus a code path still requires a reference

- **GIVEN** a pull_request to `main` whose diff changes `README.md` and `crates/rules/src/lib.rs`
- **AND** the body has no change reference
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: code change missing OpenSpec-Change`

#### Scenario: Openspec plus a workflow file still requires a reference

- **GIVEN** a pull_request to `main` whose diff changes `openspec/changes/add-spec-drift-check/tasks.md` and `.github/workflows/ci.yml`
- **AND** the body has no change reference
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: code change missing OpenSpec-Change`

#### Scenario: Docs plus a code path passes with an existing id

- **GIVEN** a pull_request to `main` whose diff changes `docs/architect/decisions.md` and `Cargo.lock`
- **AND** the body line is `OpenSpec-Change: add-dice-and-checks`
- **AND** that active change exists
- **AND** validation exits zero
- **AND** the completed-change check passes
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes

### Requirement: Push skips the reference check

On push to `main`, the job SHALL NOT fail for a missing or unknown change reference, including when the push changes a code path.

#### Scenario: A code push has no pull request body

- **GIVEN** a push to `main` that changes `crates/rules/src/lib.rs`
- **AND** there is no pull request body
- **AND** validation exits zero
- **AND** no active change is fully checked
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes
- **AND** the log does not include `spec-drift: code change missing OpenSpec-Change`

### Requirement: Code-path implementation pull request archives too

An implementation pull request that changes a code path SHALL pass only when its body names an existing change id, validation passes, and its checked-out result has no fully checked active change. Its last commit SHALL be `openspec archive <id> --yes`. The push of that result SHALL pass the completed-change check.

#### Scenario: Archived implementation pull request passes

- **GIVEN** a pull_request to `main` whose last commit is `openspec archive add-spec-drift-check --yes`
- **AND** the checked-out result has that change only under `openspec/changes/archive/`
- **AND** the diff changes `.github/workflows/ci.yml`
- **AND** the body line is `OpenSpec-Change: add-spec-drift-check`
- **AND** validation exits zero
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes

#### Scenario: Push of that merge stays green

- **GIVEN** the push to `main` of that same archived result
- **AND** validation exits zero
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes
- **AND** the log does not include `spec-drift: completed unarchived change`

### Requirement: Every failed check is reported

The job SHALL run the validation check, the completed-change check, and the reference check. It SHALL fail if any of them fails. On push, the skipped reference check SHALL count as passed. When more than one check fails, the log SHALL include each of those failure lines.

#### Scenario: Validation and a completed change both fail

- **GIVEN** a push to `main` whose validation exits non-zero
- **AND** active change `add-dice-and-checks` is fully checked
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: validate failed`
- **AND** the log includes `spec-drift: completed unarchived change add-dice-and-checks`

#### Scenario: A clean tree passes

- **GIVEN** a pull_request to `main` with validation exiting zero, no fully checked active change on the result, and a diff of only `openspec/`
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes

### Requirement: Fixture tests for the script

The repository SHALL contain table-driven tests that build one temporary git repository per case and cover every pass and fail scenario of the validation, completed-change, and reference checks. The `spec-drift` job SHALL run those tests with `node --test` and SHALL fail when they fail.

#### Scenario: The job runs the suite first

- **GIVEN** the `spec-drift` job
- **WHEN** it runs
- **THEN** it executes `node --test` on the script suite
- **AND** a failing test fails the job

#### Scenario: Each case is a fixture repository

- **GIVEN** the script test suite
- **WHEN** a table case runs
- **THEN** the test builds a temporary git repository for that case
- **AND** asserts the script's exit status and failure lines
