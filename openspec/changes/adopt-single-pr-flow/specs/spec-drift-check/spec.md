# spec-drift-check

## ADDED Requirements

### Requirement: Refactor exemption

A trimmed body line that is `OpenSpec-Refactor:`, one U+0020, and a non-empty roadmap item SHALL satisfy the reference check in place of `OpenSpec-Change` only when every condition holds. Every changed path SHALL be under `crates/`. No changed path SHALL be under `openspec/`. No existing test file SHALL be modified or deleted. The body SHALL NOT also have an `OpenSpec-Change` line. Adding a test file SHALL be allowed.

#### Scenario: A crates-only refactor passes without a change id

- **GIVEN** a pull_request to `main` whose diff is only `crates/rules/src/lib.rs`
- **AND** that path is not a test file
- **AND** the body line is `OpenSpec-Refactor: modifier tie breaks`
- **AND** the body has no `OpenSpec-Change` line
- **AND** validation exits zero
- **AND** no active change is fully checked
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes
- **AND** the log does not include `spec-drift: code change missing OpenSpec-Change`

#### Scenario: A dropped change line with a refactor exemption passes

- **GIVEN** a pull_request to `main` that changes `crates/rules/src/lib.rs`
- **AND** that path is not a test file
- **WHEN** the pull request is `edited` and the body is only `OpenSpec-Refactor: modifier tie breaks`
- **THEN** the reference check passes
- **AND** the log does not include `spec-drift: code change missing OpenSpec-Change`

#### Scenario: A path outside crates refuses the exemption

- **GIVEN** a pull_request whose diff changes `crates/rules/src/lib.rs` and `README.md`
- **AND** the body line is `OpenSpec-Refactor: modifier tie breaks`
- **AND** the body has no `OpenSpec-Change` line
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: refactor exemption refused`
- **AND** the log does not include `spec-drift: code change missing OpenSpec-Change`

#### Scenario: An openspec path refuses the exemption

- **GIVEN** a pull_request whose diff changes `crates/rules/src/lib.rs` and `openspec/config.yaml`
- **AND** the body line is `OpenSpec-Refactor: modifier tie breaks`
- **AND** the body has no `OpenSpec-Change` line
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: refactor exemption refused`

#### Scenario: An empty roadmap item does not exempt

- **GIVEN** a pull_request whose diff is only `crates/rules/src/lib.rs`
- **AND** the body line is `OpenSpec-Refactor:` with nothing after the space
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: invalid refactor line`

#### Scenario: Adding a test file is allowed

- **GIVEN** a pull_request whose diff adds `crates/rules/src/encounter/tests/new_tests.rs` and edits `crates/rules/src/lib.rs`
- **AND** no existing test file is modified or deleted
- **AND** the body line is `OpenSpec-Refactor: split strike helpers`
- **AND** validation exits zero
- **AND** no active change is fully checked
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes

#### Scenario: Modifying an existing test file refuses the exemption

- **GIVEN** the merge-base has `crates/rules/src/encounter/tests/delay_ready_tests.rs`
- **AND** the pull request modifies that file and `crates/rules/src/lib.rs`
- **AND** the body line is `OpenSpec-Refactor: split strike helpers`
- **AND** the body has no `OpenSpec-Change` line
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: refactor exemption refused`

#### Scenario: Deleting a test file refuses the exemption

- **GIVEN** the merge-base has `crates/rules/src/check_tests.rs`
- **AND** the pull request deletes that file
- **AND** the body line is `OpenSpec-Refactor: fold check helpers`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: refactor exemption refused`

#### Scenario: Both lines leave the exemption unused

- **GIVEN** a pull_request whose diff is only `crates/rules/src/lib.rs`
- **AND** the body has `OpenSpec-Refactor: modifier tie breaks` and `OpenSpec-Change: add-foo`
- **AND** `add-foo` exists
- **AND** validation exits zero
- **AND** the completed-change check passes
- **WHEN** the `spec-drift` job runs
- **THEN** the refactor exemption does not apply
- **AND** the reference check uses `add-foo`
- **AND** the job passes

#### Scenario: A completed change still fails under a refactor line

- **GIVEN** a refactor exemption that otherwise holds
- **AND** active change `add-foo` has every task checkbox checked
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: completed unarchived change add-foo`

### Requirement: Refactor line refused

When one well-formed `OpenSpec-Refactor` line is present, some condition fails, and the line is not malformed, the job SHALL fail. The log SHALL include `spec-drift: refactor exemption refused` and SHALL NOT include `spec-drift: code change missing OpenSpec-Change`.

#### Scenario: A workflow file refuses the exemption

- **GIVEN** a pull_request whose diff changes `.github/workflows/ci.yml`
- **AND** the body line is `OpenSpec-Refactor: split a helper`
- **AND** the body has no `OpenSpec-Change` line
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: refactor exemption refused`
- **AND** the log does not include `spec-drift: code change missing OpenSpec-Change`

### Requirement: Refactor exemption limits

When the exemption holds, the job SHALL NOT log `spec-drift: code change missing OpenSpec-Change`, including on `edited`. The Spec-drift job sentence that a dropped change line fails SHALL NOT apply. Validation and the completed-change check SHALL still run.

#### Scenario: The edited re-run accepts the exemption

- **GIVEN** a code-path pull request edited down to only `OpenSpec-Refactor: modifier tie breaks`
- **AND** every refactor condition holds
- **WHEN** `.github/workflows/spec-drift-edited.yml` runs
- **THEN** the reference check passes
- **AND** the log does not include `spec-drift: code change missing OpenSpec-Change`

### Requirement: Refactor test files

A test file SHALL be a path under `crates/` whose filename is `tests.rs`, ends with `_test.rs` or `_tests.rs`, or has a `/tests/` directory segment. A `#[cfg(test)]` module in another file SHALL NOT make that file a test file. An existing test file SHALL be one present on the pull request base commit. A rename away from an existing test file SHALL count as a deletion.

#### Scenario: An inline test module is not a test file

- **GIVEN** a pull_request whose only diff edits `crates/rules/src/check.rs`
- **AND** that file contains `#[cfg(test)]`
- **AND** the body line is `OpenSpec-Refactor: rename a helper`
- **AND** validation exits zero
- **AND** no active change is fully checked
- **WHEN** the `spec-drift` job runs
- **THEN** the file is not a test file
- **AND** the job passes

#### Scenario: A rename of a test file refuses the exemption

- **GIVEN** the base commit has `crates/rules/src/encounter/tests/delay_ready_tests.rs`
- **AND** the diff renames that file to `crates/rules/src/encounter/tests/delay_tests.rs`
- **AND** the body line is `OpenSpec-Refactor: rename the test module`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: refactor exemption refused`

### Requirement: Malformed refactor lines

A line that starts with `OpenSpec-Refactor:` but is not one U+0020 plus a non-empty item SHALL fail on every pull request. More than one well-formed `OpenSpec-Refactor` line SHALL fail. The log SHALL include `spec-drift: invalid refactor line`. A valid `OpenSpec-Change` line beside it SHALL NOT make the check pass.

#### Scenario: A missing space is invalid

- **GIVEN** a pull_request whose diff is only `README.md`
- **AND** the body line is `OpenSpec-Refactor:item`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: invalid refactor line`

#### Scenario: Two refactor lines fail

- **GIVEN** a pull_request whose diff is only `crates/rules/src/lib.rs`
- **AND** the body has `OpenSpec-Refactor: one item` and `OpenSpec-Refactor: other item`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: invalid refactor line`

#### Scenario: A valid change line does not hide a bad refactor line

- **GIVEN** a pull_request that changes `Cargo.toml`
- **AND** the body has `OpenSpec-Change: add-foo` and `OpenSpec-Refactor:item`
- **AND** `openspec/changes/add-foo/` exists
- **AND** validation exits zero
- **AND** the completed-change check passes
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: invalid refactor line`
