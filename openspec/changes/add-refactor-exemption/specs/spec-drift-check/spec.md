# spec-drift-check

## ADDED Requirements

### Requirement: Refactor line

A body line, after one trailing CR is removed and surrounding whitespace is trimmed, that starts with exact case-sensitive `OpenSpec-Refactor:` SHALL be a refactor line. It SHALL be valid only as that prefix, one U+0020, and one roadmap item. A roadmap item SHALL be one or more `[a-z0-9]` groups joined by single hyphens. Any other such line SHALL be malformed. Fenced lines SHALL count. `openspec-refactor:` SHALL NOT be a refactor line. The job SHALL NOT look the item up.

#### Scenario: A trimmed refactor line passes

- **GIVEN** a pull_request to `main` that changes `crates/rules/src/lib.rs`
- **AND** the body line is `  OpenSpec-Refactor: dice-and-checks  `
- **AND** validation exits zero
- **AND** the completed-change check passes
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes
- **AND** the log does not include `spec-drift: code change missing OpenSpec-Change`
- **AND** the log does not include `spec-drift: invalid refactor item`
- **AND** the log does not include `spec-drift: unknown change`

#### Scenario: A single-group roadmap item passes

- **GIVEN** a pull_request to `main` that changes `crates/rules/src/lib.rs`
- **AND** the body line is `OpenSpec-Refactor: shields`
- **AND** validation exits zero
- **AND** the completed-change check passes
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes

#### Scenario: An item absent from the decision log passes

- **GIVEN** a pull_request to `main` that changes `crates/rules/src/lib.rs`
- **AND** the body line is `OpenSpec-Refactor: not-a-real-item`
- **AND** that token names no change and no heading in `docs/architect/decisions.md`
- **AND** validation exits zero
- **AND** the completed-change check passes
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes
- **AND** the log does not include `spec-drift: unknown change not-a-real-item`

#### Scenario: A fenced refactor line counts

- **GIVEN** a pull_request to `main` that changes `crates/rules/src/lib.rs`
- **AND** the only refactor line sits inside a fenced code block
- **AND** that line is `OpenSpec-Refactor: grid-movement`
- **AND** validation exits zero
- **AND** the completed-change check passes
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes

#### Scenario: A lowercase refactor prefix is ignored

- **GIVEN** a pull_request to `main` that changes `crates/rules/src/lib.rs`
- **AND** the only body line is `openspec-refactor: dice-and-checks`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: code change missing OpenSpec-Change`
- **AND** the log does not include `spec-drift: invalid refactor item`

### Requirement: Invalid refactor item

A malformed refactor line SHALL fail on every pull request. The log line SHALL be `spec-drift: invalid refactor item` plus the whole remainder after `OpenSpec-Refactor:`, with nothing stripped. Each UTF-16 code unit outside U+0020 through U+007E, and a backslash, SHALL be `\u` plus four lowercase hex digits. Copy every other unit. The line SHALL NOT use JSON quotes, and SHALL NOT also log `spec-drift: code change missing OpenSpec-Change`.

#### Scenario: An empty refactor line is invalid

- **GIVEN** a pull_request to `main` whose diff is only `docs/architect/decisions.md`
- **AND** the body contains a line `OpenSpec-Refactor:`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes a line whose text is exactly `spec-drift: invalid refactor item`
- **AND** the log does not include `spec-drift: code change missing OpenSpec-Change`

#### Scenario: A spaceless refactor line is invalid

- **GIVEN** a pull_request to `main` whose diff is only `docs/architect/decisions.md`
- **AND** the body line is `OpenSpec-Refactor:dice-and-checks`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes a line whose text is exactly `spec-drift: invalid refactor itemdice-and-checks`

#### Scenario: Two spaces before the roadmap item are invalid

- **GIVEN** a pull_request to `main` whose diff is only `docs/architect/decisions.md`
- **AND** the body line is `OpenSpec-Refactor:` plus two U+0020 spaces plus `dice-and-checks`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log line is `spec-drift: invalid refactor item` immediately followed by those two U+0020 spaces and `dice-and-checks`
- **AND** nothing in that remainder was stripped

#### Scenario: A tab after the refactor colon is invalid

- **GIVEN** a pull_request to `main` whose diff is only `docs/architect/decisions.md`
- **AND** the body line is `OpenSpec-Refactor:` immediately followed by a tab and `dice-and-checks`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log line is `spec-drift: invalid refactor item` immediately followed by `\u0009dice-and-checks`

#### Scenario: Extra words are not a roadmap item

- **GIVEN** a pull_request to `main` whose diff is only `README.md`
- **AND** the only body line is `OpenSpec-Refactor: dice-and-checks extra`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: invalid refactor item dice-and-checks extra`
- **AND** the log does not include `spec-drift: code change missing OpenSpec-Change`

#### Scenario: An uppercase roadmap item is invalid

- **GIVEN** a pull_request to `main` that changes `crates/rules/src/lib.rs`
- **AND** the only body line is `OpenSpec-Refactor: Dice-And-Checks`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: invalid refactor item Dice-And-Checks`
- **AND** the line is not accepted as `dice-and-checks`

#### Scenario: A slash in the roadmap item is invalid

- **GIVEN** a pull_request to `main` that changes `crates/rules/src/lib.rs`
- **AND** the only body line is `OpenSpec-Refactor: positioning/flanking`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: invalid refactor item positioning/flanking`

#### Scenario: A double hyphen is not a roadmap item

- **GIVEN** a pull_request to `main` whose diff is only `docs/architect/decisions.md`
- **AND** the body line is `OpenSpec-Refactor: dice--checks`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: invalid refactor item dice--checks`

#### Scenario: A zero-width space in the roadmap item is visible

- **GIVEN** a pull_request to `main` whose diff is only `docs/architect/decisions.md`
- **AND** the body line is `OpenSpec-Refactor:` plus one U+0020 plus U+200B plus `dice-and-checks`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: invalid refactor item \u200bdice-and-checks`
- **AND** the log does not contain U+200B

#### Scenario: An NBSP in the roadmap item is visible

- **GIVEN** a pull_request to `main` whose diff is only `docs/architect/decisions.md`
- **AND** the body line is `OpenSpec-Refactor:` plus one U+0020 plus U+00A0 plus `dice-and-checks`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: invalid refactor item \u00a0dice-and-checks`
- **AND** the log does not contain U+00A0

#### Scenario: A typed backslash escape differs from a real NBSP

- **GIVEN** a pull_request to `main` whose diff is only `docs/architect/decisions.md`
- **AND** one body line is `OpenSpec-Refactor:` plus one U+0020 plus the characters `\u00a0` plus `dice-and-checks`
- **AND** another body line is `OpenSpec-Refactor:` plus one U+0020 plus U+00A0 plus `dice-and-checks`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: invalid refactor item \u005cu00a0dice-and-checks`
- **AND** the log includes `spec-drift: invalid refactor item \u00a0dice-and-checks`
- **AND** those two lines differ

#### Scenario: A duplicate malformed remainder is logged once

- **GIVEN** a pull_request to `main` whose diff is only `docs/architect/decisions.md`
- **AND** the body contains `OpenSpec-Refactor: NOT-AN-ITEM` twice
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: invalid refactor item NOT-AN-ITEM` once

### Requirement: Multiple refactor lines

More than one refactor line SHALL fail on every pull request. The log SHALL include `spec-drift: multiple OpenSpec-Refactor lines`. A repeated roadmap item SHALL count again. The job SHALL NOT also log `spec-drift: code change missing OpenSpec-Change`.

#### Scenario: The same refactor line twice fails

- **GIVEN** a pull_request to `main` that changes `crates/rules/src/lib.rs`
- **AND** the body contains `OpenSpec-Refactor: dice-and-checks` twice
- **AND** validation exits zero
- **AND** the completed-change check passes
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: multiple OpenSpec-Refactor lines`
- **AND** the log does not include `spec-drift: code change missing OpenSpec-Change`
- **AND** the log does not include `spec-drift: refactor is not crates-only`

#### Scenario: Two different roadmap items fail

- **GIVEN** a pull_request to `main` that changes `crates/rules/src/lib.rs`
- **AND** the body contains `OpenSpec-Refactor: dice-and-checks` and `OpenSpec-Refactor: grid-movement`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: multiple OpenSpec-Refactor lines`

#### Scenario: A valid line beside a malformed refactor line fails

- **GIVEN** a pull_request to `main` that changes `crates/rules/src/lib.rs` and `README.md`
- **AND** the body contains `OpenSpec-Refactor: dice-and-checks` and `OpenSpec-Refactor: NOT-AN-ITEM`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: invalid refactor item NOT-AN-ITEM`
- **AND** the log includes `spec-drift: multiple OpenSpec-Refactor lines`
- **AND** the log does not include `spec-drift: refactor is not crates-only`
- **AND** the log does not include `spec-drift: code change missing OpenSpec-Change`

### Requirement: Refactor line conflicts with OpenSpec-Change

A body that has at least one refactor line and at least one trimmed line starting with exact `OpenSpec-Change:` SHALL fail on every pull request. The log SHALL include `spec-drift: refactor line conflicts with OpenSpec-Change`. The job SHALL NOT also log `spec-drift: code change missing OpenSpec-Change`.

#### Scenario: Both lines fail even when each would pass alone

- **GIVEN** a pull_request to `main` that changes `crates/rules/src/lib.rs`
- **AND** the body contains `OpenSpec-Refactor: dice-and-checks` and `OpenSpec-Change: add-dice-and-checks`
- **AND** `openspec/changes/add-dice-and-checks/` exists
- **AND** validation exits zero
- **AND** the completed-change check passes
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: refactor line conflicts with OpenSpec-Change`
- **AND** the log does not include `spec-drift: code change missing OpenSpec-Change`
- **AND** the log does not include `spec-drift: unknown change`
- **AND** the log does not include `spec-drift: refactor is not crates-only`

#### Scenario: Both lines fail on a docs-only pull request

- **GIVEN** a pull_request to `main` whose diff is only `docs/architect/decisions.md`
- **AND** the body contains `OpenSpec-Refactor: dice-and-checks` and `OpenSpec-Change: add-dice-and-checks`
- **AND** validation exits zero
- **AND** the completed-change check passes
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: refactor line conflicts with OpenSpec-Change`

#### Scenario: A conflict also logs an invalid change id

- **GIVEN** a pull_request to `main` that changes `crates/rules/src/lib.rs`
- **AND** the body contains `OpenSpec-Refactor: dice-and-checks` and `OpenSpec-Change: Add-Foo`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: invalid change id Add-Foo`
- **AND** the log includes `spec-drift: refactor line conflicts with OpenSpec-Change`
- **AND** the log does not include `spec-drift: code change missing OpenSpec-Change`

#### Scenario: Change lines without a refactor line still pass

- **GIVEN** a pull_request to `main` that changes `crates/rules/src/lib.rs`
- **AND** the body contains `OpenSpec-Change: add-dice-and-checks` and `OpenSpec-Change: add-project-foundation`
- **AND** the first id is active and the second exists only under `openspec/changes/archive/2026-10-21-add-project-foundation/`
- **AND** the body has no refactor line
- **AND** validation exits zero
- **AND** the completed-change check passes
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes
- **AND** the log does not include `spec-drift: multiple OpenSpec-Refactor lines`
- **AND** the log does not include `spec-drift: refactor line conflicts with OpenSpec-Change`

### Requirement: Qualifying refactor paths

Exactly one valid refactor line, and no `OpenSpec-Change:` line, SHALL satisfy the reference check only when every changed path starts with `crates/` or is exactly `CHANGELOG.md`, and no changed path starts with `openspec/`. Both sides of a rename SHALL count. Changed paths SHALL come from `git diff -z --name-status --find-renames` between the pull request base and the checked-out commit. The command SHALL NOT add `--find-copies`.

#### Scenario: A crate file with a changelog bullet passes

- **GIVEN** a pull_request to `main` whose diff changes `crates/rules/src/lib.rs` and `CHANGELOG.md`
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **AND** validation exits zero
- **AND** the completed-change check passes
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes

#### Scenario: A crate file without a changelog passes spec-drift

- **GIVEN** a pull_request to `main` whose only diff is `crates/rules/src/lib.rs`
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **AND** the head has no `CHANGELOG.md`
- **AND** validation exits zero
- **AND** the completed-change check passes
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes

#### Scenario: Adding a test file passes

- **GIVEN** a pull_request to `main` whose diff adds `crates/rules/src/encounter/tests/new_tests.rs` and modifies `crates/rules/src/lib.rs`
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **AND** validation exits zero
- **AND** the completed-change check passes
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes
- **AND** the log does not include `spec-drift: refactor changes existing tests`

#### Scenario: Deleting a non-test crate file passes

- **GIVEN** a pull_request to `main` whose diff deletes `crates/rules/src/old.rs`
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **AND** validation exits zero
- **AND** the completed-change check passes
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes

#### Scenario: Renaming a non-test crate file passes

- **GIVEN** a pull_request to `main` whose diff renames `crates/rules/src/old.rs` to `crates/rules/src/new.rs`
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **AND** validation exits zero
- **AND** the completed-change check passes
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes

#### Scenario: Renaming a non-test file into tests passes

- **GIVEN** a pull_request to `main` whose diff renames `crates/rules/src/lib.rs` to `crates/rules/src/encounter/tests/lib.rs`
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **AND** validation exits zero
- **AND** the completed-change check passes
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes
- **AND** the log does not include `spec-drift: refactor changes existing tests`

### Requirement: Refactor path failures

When that single valid line is the only reference and the diff has a code path, a changed path that does not start with `crates/` and is not exactly `CHANGELOG.md` SHALL log `spec-drift: refactor is not crates-only`. A changed path that starts with `openspec/` SHALL log `spec-drift: refactor changes openspec`. When both match, the log SHALL include both lines. The job SHALL NOT also log `spec-drift: code change missing OpenSpec-Change`.

#### Scenario: Root Cargo.toml is not a refactor

- **GIVEN** a pull_request to `main` whose only diff is `Cargo.toml`
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: refactor is not crates-only`
- **AND** the log does not include `spec-drift: code change missing OpenSpec-Change`
- **AND** the log does not include `spec-drift: refactor changes openspec`

#### Scenario: A crate file plus README is not a refactor

- **GIVEN** a pull_request to `main` whose diff changes `crates/rules/src/lib.rs` and `README.md`
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: refactor is not crates-only`

#### Scenario: A crate file plus a workflow file is not a refactor

- **GIVEN** a pull_request to `main` whose diff changes `crates/rules/src/lib.rs` and `.github/workflows/ci.yml`
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: refactor is not crates-only`

#### Scenario: A crate file plus a script is not a refactor

- **GIVEN** a pull_request to `main` whose diff changes `crates/rules/src/lib.rs` and `scripts/note.mjs`
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: refactor is not crates-only`

#### Scenario: A crate file plus docs is not a refactor

- **GIVEN** a pull_request to `main` whose diff changes `crates/rules/src/lib.rs` and `docs/architect/decisions.md`
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: refactor is not crates-only`

#### Scenario: A lowercase changelog name is not the bullet file

- **GIVEN** a pull_request to `main` whose diff changes `crates/rules/src/lib.rs` and `changelog.md`
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: refactor is not crates-only`

#### Scenario: A crate file plus openspec logs both path lines

- **GIVEN** a pull_request to `main` whose diff changes `crates/rules/src/lib.rs` and `openspec/specs/spec-drift-check/spec.md`
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: refactor is not crates-only`
- **AND** the log includes `spec-drift: refactor changes openspec`
- **AND** `spec-drift: refactor is not crates-only` appears before `spec-drift: refactor changes openspec`
- **AND** the log does not include `spec-drift: code change missing OpenSpec-Change`

#### Scenario: Renaming into crates is not crates-only

- **GIVEN** a pull_request to `main` whose diff renames `docs/note.md` to `crates/rules/note.md`
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: refactor is not crates-only`

#### Scenario: Renaming out of crates is not crates-only

- **GIVEN** a pull_request to `main` whose diff renames `crates/rules/src/lib.rs` to `docs/lib.rs`
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: refactor is not crates-only`

#### Scenario: A crates prefix without a slash is not under crates

- **GIVEN** a pull_request to `main` whose diff changes `crates/rules/src/lib.rs` and `crates-extra/foo.rs`
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: refactor is not crates-only`

### Requirement: Test file

A test file SHALL be a changed path that starts with `crates/` and contains the substring `/tests/`. `crates/rules/src/tests.rs` and `crates/rules/src/lib.rs` SHALL NOT be test files. The job SHALL NOT parse Rust. A `#[cfg(test)]` module in a non-test path SHALL NOT make that path a test file. Reviewing that module SHALL stay with Architect and QA.

#### Scenario: A tests directory path is a test file

- **GIVEN** a pull_request to `main` whose diff modifies `crates/rules/src/encounter/tests/delay_ready_tests.rs`
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: refactor changes existing tests`

#### Scenario: A sibling tests.rs file is not a test file

- **GIVEN** a pull_request to `main` whose diff modifies `crates/rules/src/tests.rs`
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **AND** validation exits zero
- **AND** the completed-change check passes
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes
- **AND** the log does not include `spec-drift: refactor changes existing tests`

#### Scenario: An inline test module is not classified

- **GIVEN** a pull_request to `main` whose diff modifies `crates/rules/src/lib.rs`
- **AND** that file contains `#[cfg(test)]`
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **AND** validation exits zero
- **AND** the completed-change check passes
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes
- **AND** the job does not parse the file to find `#[cfg(test)]`
- **AND** the log does not include `spec-drift: refactor changes existing tests`

### Requirement: Existing test change

An existing test file SHALL count as modified or removed when a record status is `M`, `D`, or `T` and that path is a test file, or when a status starting with `R` has a test file as its old path. Status `A` on a test file SHALL be allowed. A rename whose old path is not a test file SHALL be allowed when the new path is a test file. The status field SHALL be the git name-status token, including a score suffix on `R`.

#### Scenario: Modifying a test file fails

- **GIVEN** a pull_request to `main` whose diff status for `crates/rules/tests/foo.rs` is `M`
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: refactor changes existing tests`
- **AND** the log does not include `spec-drift: code change missing OpenSpec-Change`

#### Scenario: Deleting a test file fails

- **GIVEN** a pull_request to `main` whose diff status for `crates/rules/src/encounter/tests/delay_ready_tests.rs` is `D`
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: refactor changes existing tests`

#### Scenario: A type change of a test file fails

- **GIVEN** a pull_request to `main` whose diff status for `crates/rules/tests/foo.rs` is `T`
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: refactor changes existing tests`

#### Scenario: Renaming a test file fails

- **GIVEN** a pull_request to `main` whose diff renames `crates/rules/src/encounter/tests/old.rs` to `crates/rules/src/encounter/tests/new.rs`
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: refactor changes existing tests`

#### Scenario: Adding a test file does not fail the test rule

- **GIVEN** a pull_request to `main` whose only diff adds `crates/rules/src/encounter/tests/new_tests.rs`
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **AND** validation exits zero
- **AND** the completed-change check passes
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes

#### Scenario: A modified test and a docs file log both lines

- **GIVEN** a pull_request to `main` whose diff modifies `crates/rules/tests/foo.rs` and `docs/architect/decisions.md`
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: refactor is not crates-only`
- **AND** the log includes `spec-drift: refactor changes existing tests`
- **AND** `spec-drift: refactor is not crates-only` appears before `spec-drift: refactor changes existing tests`

#### Scenario: A modified test and an openspec file log three lines

- **GIVEN** a pull_request to `main` whose diff modifies `crates/rules/tests/foo.rs` and `openspec/specs/spec-drift-check/spec.md`
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: refactor is not crates-only`
- **AND** the log includes `spec-drift: refactor changes openspec`
- **AND** the log includes `spec-drift: refactor changes existing tests`
- **AND** those three lines appear in that order

### Requirement: Path lines only for one valid line

Path-condition lines SHALL be logged only for exactly one valid refactor line, no `OpenSpec-Change:` line, and a diff that has a code path. A malformed item, more than one refactor line, or a conflict SHALL NOT also log `spec-drift: refactor is not crates-only`, `spec-drift: refactor changes openspec`, or `spec-drift: refactor changes existing tests`.

#### Scenario: A malformed line hides a mixed diff

- **GIVEN** a pull_request to `main` whose diff changes `crates/rules/src/lib.rs` and `docs/architect/decisions.md`
- **AND** the only body line is `OpenSpec-Refactor: NOT-AN-ITEM`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: invalid refactor item NOT-AN-ITEM`
- **AND** the log does not include `spec-drift: refactor is not crates-only`
- **AND** the log does not include `spec-drift: code change missing OpenSpec-Change`

#### Scenario: A conflict hides a mixed diff

- **GIVEN** a pull_request to `main` whose diff changes `crates/rules/src/lib.rs` and `README.md`
- **AND** the body contains `OpenSpec-Refactor: dice-and-checks` and `OpenSpec-Change: add-dice-and-checks`
- **AND** `openspec/changes/add-dice-and-checks/` exists
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: refactor line conflicts with OpenSpec-Change`
- **AND** the log does not include `spec-drift: refactor is not crates-only`

### Requirement: No code path skips refactor path rules

When the diff has no code path, a valid refactor line SHALL NOT be tested against the path rules and SHALL NOT fail the reference check. A malformed refactor line, more than one refactor line, or a conflict SHALL still fail.

#### Scenario: Docs-only with a valid refactor line passes

- **GIVEN** a pull_request to `main` whose diff is only `docs/architect/decisions.md`
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **AND** validation exits zero
- **AND** no active change is fully checked
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes
- **AND** the log does not include `spec-drift: refactor is not crates-only`

#### Scenario: Openspec-only with a valid refactor line passes

- **GIVEN** a pull_request to `main` whose diff is only under `openspec/`
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **AND** validation exits zero
- **AND** no active change is fully checked
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes
- **AND** the log does not include `spec-drift: refactor changes openspec`

#### Scenario: A changelog-only refactor line passes

- **GIVEN** a pull_request to `main` whose only diff is `CHANGELOG.md`
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **AND** validation exits zero
- **AND** no active change is fully checked
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes

### Requirement: Missing change line stays

A code-path pull request with no refactor line, no invalid change line, and no valid change id SHALL still fail with `spec-drift: code change missing OpenSpec-Change`. One or more valid `OpenSpec-Change` lines and no refactor line SHALL still pass when every id exists.

#### Scenario: Dropping the refactor line on edit fails

- **GIVEN** a pull_request to `main` that changes `crates/rules/src/lib.rs`
- **AND** the body line `OpenSpec-Refactor: dice-and-checks` made the `spec-drift` job pass
- **WHEN** the pull request is `edited` and the body has no refactor line and no `OpenSpec-Change` line
- **THEN** `.github/workflows/spec-drift-edited.yml` runs
- **AND** the check fails
- **AND** the log includes `spec-drift: code change missing OpenSpec-Change`

#### Scenario: Adding the refactor line on edit passes

- **GIVEN** a pull_request to `main` that changes only `crates/rules/src/lib.rs`
- **AND** the body had no reference line and the `spec-drift` job failed
- **WHEN** the pull request is `edited` and the body line is `OpenSpec-Refactor: dice-and-checks`
- **AND** validation exits zero
- **AND** the completed-change check passes
- **THEN** `.github/workflows/spec-drift-edited.yml` runs
- **AND** the check passes

#### Scenario: One existing change line still passes

- **GIVEN** a pull_request to `main` that changes `crates/rules/src/lib.rs`
- **AND** the body line is `OpenSpec-Change: add-dice-and-checks`
- **AND** the body has no refactor line
- **AND** `openspec/changes/add-dice-and-checks/` exists
- **AND** validation exits zero
- **AND** the completed-change check passes
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes

### Requirement: Push skips refactor lines

On push, the job SHALL NOT read refactor lines. A push that changes a code path SHALL NOT fail because a refactor line is missing, malformed, repeated, or paired with `OpenSpec-Change`.

#### Scenario: A push ignores a malformed refactor line

- **GIVEN** a push to `main` that changes `crates/rules/src/lib.rs`
- **AND** the event body is `OpenSpec-Refactor:`
- **AND** validation exits zero
- **AND** no active change is fully checked
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes
- **AND** the log does not include `spec-drift: invalid refactor item`
- **AND** the log does not include `spec-drift: code change missing OpenSpec-Change`

### Requirement: Behavior confirmation is not CI

Architect and QA confirming that behavior is unchanged SHALL be required before merge and SHALL NOT be enforced by this job. The job SHALL NOT read reviews, approvals, or Rust sources to decide that behavior changed. A qualifying refactor line and paths SHALL pass when that confirmation is absent. No workflow `if` SHALL encode that confirmation.

#### Scenario: A qualifying refactor passes with no review

- **GIVEN** a pull_request to `main` that changes `crates/rules/src/lib.rs`
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **AND** the pull request has no review and no approval
- **AND** validation exits zero
- **AND** the completed-change check passes
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes

#### Scenario: Workflows do not gate on a review

- **GIVEN** `.github/workflows/ci.yml` and `.github/workflows/spec-drift-edited.yml`
- **WHEN** their `spec-drift` jobs are read
- **THEN** neither job has an `if` that reads reviews, approvals, or a behavior confirmation
- **AND** neither script step inspects Rust source for behavior

### Requirement: Refactor does not skip the changelog

Passing this exemption SHALL NOT waive the Unreleased bullet. A path under `crates/` SHALL stay an implementation pull request under the current `changelog` and `pr-hygiene` specs and SHALL still need a new Unreleased bullet. Those specs SHALL NOT exempt a tests-only diff or unchanged behavior. This job SHALL NOT check the bullet. `OpenSpec-Refactor` SHALL NOT be an own-change line.

#### Scenario: Spec-drift ignores a missing bullet

- **GIVEN** a pull_request to `main` whose diff changes `crates/rules/src/lib.rs` and adds no Unreleased bullet
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **AND** validation exits zero
- **AND** the completed-change check passes
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes
- **AND** the log does not include `implementation PR missing CHANGELOG bullet`

#### Scenario: Current specs still require the bullet

- **GIVEN** `openspec/specs/pr-hygiene/spec.md` and `openspec/specs/changelog/spec.md`
- **WHEN** those requirements are read
- **THEN** a diff that changes `crates/rules/src/lib.rs` is an implementation pull request
- **AND** it needs a new Unreleased bullet
- **AND** neither spec exempts a path because it contains `/tests/`
- **AND** neither spec exempts a pull request because behavior is unchanged

#### Scenario: A refactor line is not an own-change line

- **GIVEN** a pull_request body whose only reference line is `OpenSpec-Refactor: dice-and-checks`
- **WHEN** own-change lines are parsed by the `OpenSpec-Change` rule
- **THEN** that line contributes no change id

### Requirement: Edited workflow and concurrency stay

The `spec-drift` jobs in `.github/workflows/ci.yml` and `.github/workflows/spec-drift-edited.yml` SHALL keep their triggers, names, permissions, and concurrency. Both SHALL still invoke `scripts/spec-drift-check.mjs`. The shared group SHALL stay `spec-drift-${{ github.event.pull_request.number || github.sha }}` with `cancel-in-progress: true`. Neither workflow SHALL add a workflow-level group. The Cargo job SHALL NOT join it. An edited body SHALL re-run that script.

#### Scenario: The refactor check uses the shared group

- **GIVEN** `.github/workflows/ci.yml` and `.github/workflows/spec-drift-edited.yml`
- **WHEN** the `spec-drift` job in each file is read
- **THEN** each job's concurrency group is `spec-drift-${{ github.event.pull_request.number || github.sha }}`
- **AND** each sets `cancel-in-progress: true`
- **AND** each invokes `scripts/spec-drift-check.mjs`
- **AND** `spec-drift-edited.yml` still runs on pull_request `edited` targeting `main`
- **AND** neither file has a workflow-level concurrency group
- **AND** the Cargo `ci` job has no concurrency key
- **AND** neither file names a refactor-only concurrency group

### Requirement: Other checks still run

The validation check and the completed-change check SHALL still run when a refactor line is present. A failure of either SHALL still fail the job and SHALL still be logged beside any refactor failure line.

#### Scenario: A completed change still fails beside a valid refactor

- **GIVEN** a pull_request to `main` that changes `crates/rules/src/lib.rs`
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **AND** active change `add-dice-and-checks` is fully checked
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: completed unarchived change add-dice-and-checks`
- **AND** the log does not include `spec-drift: code change missing OpenSpec-Change`

#### Scenario: Validation and an invalid refactor item both fail

- **GIVEN** a pull_request to `main` whose validation exits non-zero
- **AND** the body line is `OpenSpec-Refactor:`
- **AND** the diff changes `crates/rules/src/lib.rs`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: validate failed`
- **AND** the log includes a line whose text is exactly `spec-drift: invalid refactor item`

### Requirement: Refactor token order

Invalid change ids SHALL be logged before invalid refactor items. Each list SHALL follow body order and SHALL log a duplicate remainder once. `spec-drift: multiple OpenSpec-Refactor lines` SHALL follow those items when more than one refactor line is present. `spec-drift: refactor line conflicts with OpenSpec-Change` SHALL follow that line when both prefixes are present. Unknown change ids and the missing-reference line SHALL follow those, and only when that existing rule still applies.

#### Scenario: Mixed body failures keep that order

- **GIVEN** a pull_request to `main` that changes `crates/rules/src/lib.rs`
- **AND** the body contains, in order, `OpenSpec-Change: Add-Foo`, `OpenSpec-Refactor: NOT-AN-ITEM`, and `OpenSpec-Refactor: dice-and-checks`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** `spec-drift: invalid change id Add-Foo` appears before `spec-drift: invalid refactor item NOT-AN-ITEM`
- **AND** that invalid refactor line appears before `spec-drift: multiple OpenSpec-Refactor lines`
- **AND** that multiple line appears before `spec-drift: refactor line conflicts with OpenSpec-Change`
- **AND** the log does not include `spec-drift: refactor is not crates-only`
- **AND** the log does not include `spec-drift: code change missing OpenSpec-Change`

### Requirement: Path failure order

When path lines are logged, `spec-drift: refactor is not crates-only` SHALL precede `spec-drift: refactor changes openspec`, which SHALL precede `spec-drift: refactor changes existing tests`. A path line that does not apply SHALL be omitted. These lines SHALL follow any token-order lines.

#### Scenario: Path failures keep crates, openspec, then tests

- **GIVEN** a pull_request to `main` whose diff modifies `crates/rules/tests/foo.rs` and `openspec/specs/spec-drift-check/spec.md`
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** `spec-drift: refactor is not crates-only` appears before `spec-drift: refactor changes openspec`
- **AND** `spec-drift: refactor changes openspec` appears before `spec-drift: refactor changes existing tests`
