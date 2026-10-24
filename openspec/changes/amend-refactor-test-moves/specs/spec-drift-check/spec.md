# spec-drift-check

## MODIFIED Requirements

### Requirement: Test file

A test file SHALL be a changed path that starts with `crates/` and contains the substring `/tests/`. `crates/rules/src/tests.rs` and `crates/rules/src/lib.rs` SHALL NOT be test files. The job SHALL NOT parse Rust. A `#[cfg(test)]` module in a non-test path SHALL NOT make that path a test file.

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

#### Scenario: An inline test module is not classified

- **GIVEN** a pull_request to `main` whose diff modifies `crates/rules/src/lib.rs`
- **AND** that file contains `#[cfg(test)]`
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **AND** validation exits zero
- **AND** the completed-change check passes
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes
- **AND** the job does not parse the file to find `#[cfg(test)]`

#### Scenario: A source file losing a cfg test block plus a new test file passes

- **GIVEN** a pull_request to `main` whose diff modifies `crates/rules/src/lib.rs`
- **AND** that file loses a `#[cfg(test)]` block
- **AND** the diff adds `crates/rules/src/encounter/tests/new_tests.rs`
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **AND** validation exits zero
- **AND** the completed-change check passes
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes
- **AND** the log does not include `spec-drift: refactor changes existing tests`
- **AND** the job does not parse either file

## ADDED Requirements

### Requirement: Inline test block move

On an OpenSpec-Refactor pull request, an in-file `#[cfg(test)]` block in a non-test path SHALL be allowed to move verbatim into a new test file. Only a use line or a path line SHALL differ. Asserting logic SHALL NOT change. An existing test file SHALL stay untouchable. The job SHALL NOT distinguish a verbatim move from an edit. Review SHALL enforce the verbatim move and the unchanged asserting logic. The job SHALL NOT reject that move.

#### Scenario: The job leaves the move to review

- **GIVEN** the pull request in "A source file losing a cfg test block plus a new test file passes"
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes
- **AND** the log does not include `spec-drift: refactor changes existing tests`
- **AND** review, not the job, decides whether the move is verbatim

#### Scenario: Review allows a verbatim move

- **GIVEN** an `OpenSpec-Refactor` pull request
- **AND** `crates/rules/src/check.rs` loses a `#[cfg(test)]` block
- **AND** the diff adds `crates/rules/src/encounter/tests/check_tests.rs`
- **AND** the added block matches the removed block except a use line or a path line
- **AND** a use line, after leading whitespace is removed, starts with `use ` or `pub use `
- **AND** a path line differs only by a module path written with `crate::`, `super::`, or `self::`
- **WHEN** review checks the move
- **THEN** the move is allowed
- **AND** asserting logic is unchanged

#### Scenario: Review rejects changed asserting logic

- **GIVEN** an `OpenSpec-Refactor` pull request that moves a `#[cfg(test)]` block into a new test file
- **AND** the added block changes an assertion, an expected value, control flow, a comment, or whitespace in the test body
- **WHEN** review checks the move
- **THEN** review rejects the change
- **AND** the `spec-drift` job does not log `spec-drift: refactor changes existing tests`

#### Scenario: Review rejects an in-place assertion edit

- **GIVEN** a pull_request to `main` whose diff modifies `crates/rules/src/lib.rs`
- **AND** that file keeps a `#[cfg(test)]` block and changes an assertion inside it
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **AND** validation exits zero
- **AND** the completed-change check passes
- **WHEN** review checks the block
- **THEN** review rejects the change
- **AND** the `spec-drift` job passes
- **AND** the job does not parse the file to find `#[cfg(test)]`

#### Scenario: Review rejects a dropped inline block

- **GIVEN** a pull_request to `main` whose diff modifies `crates/rules/src/lib.rs`
- **AND** that file loses a `#[cfg(test)]` block
- **AND** no added test file contains that block
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **AND** validation exits zero
- **AND** the completed-change check passes
- **WHEN** review checks the diff
- **THEN** review rejects the change
- **AND** the `spec-drift` job passes
- **AND** the log does not include `spec-drift: refactor changes existing tests`

#### Scenario: A moved block does not excuse an existing tests file

- **GIVEN** a pull_request to `main` whose diff modifies `crates/rules/src/lib.rs` by removing a `#[cfg(test)]` block
- **AND** the diff adds `crates/rules/src/encounter/tests/new_tests.rs`
- **AND** the diff modifies `crates/rules/src/encounter/tests/delay_ready_tests.rs`
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: refactor changes existing tests`

#### Scenario: A new test beside the moved block is not a changed assertion

- **GIVEN** a verbatim move of a `#[cfg(test)]` block into a new test file
- **AND** that file also contains a test that was not in the removed block
- **WHEN** review checks the move
- **THEN** that new test is not a change to the moved asserting logic
- **AND** the moved block is allowed
