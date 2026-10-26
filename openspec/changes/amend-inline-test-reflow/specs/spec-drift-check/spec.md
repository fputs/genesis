# spec-drift-check

## MODIFIED Requirements

### Requirement: Inline test block move

On an OpenSpec-Refactor pull request, an in-file `#[cfg(test)]` block in a non-test path SHALL be allowed to move verbatim into a new test file. A new test file SHALL be an added path that the Test file requirement classifies as a test file. Asserting logic SHALL NOT change. An existing test file SHALL stay untouchable. The job SHALL NOT distinguish a verbatim move from an edit. Review SHALL enforce the verbatim move and the unchanged asserting logic. The job SHALL NOT reject that move.

#### Scenario: The job leaves the move to review

- **GIVEN** a pull_request to `main` whose diff modifies `crates/rules/src/lib.rs`
- **AND** that file loses a `#[cfg(test)]` block
- **AND** the diff adds `crates/rules/tests/moved_tests.rs`
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **AND** validation exits zero
- **AND** the completed-change check passes
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes
- **AND** the log does not include `spec-drift: refactor changes existing tests`
- **AND** review, not the job, decides whether the move is verbatim
- **AND** the job does not parse either file

#### Scenario: Review allows a verbatim move

- **GIVEN** the only differences are use lines, path lines, a uniform leading-indentation change, and the dropped or added `mod` lines
- **AND** this is an `OpenSpec-Refactor` pull request
- **AND** `crates/rules/src/check.rs` loses a `#[cfg(test)]` block
- **AND** the diff adds `crates/rules/src/encounter/tests/check_tests.rs`
- **AND** that added path is a new test file
- **AND** a use line, after leading whitespace is removed, starts with `use ` or `pub use `
- **AND** a path line differs only by a module path written with `crate::`, `super::`, or `self::`
- **AND** moving the block out of `mod tests` re-indents it
- **WHEN** review checks the move
- **THEN** the move is allowed
- **AND** a uniform change in the moved block's leading indentation is not a change
- **AND** a rustfmt-only reflow of a moved block is not a change, as long as a whitespace-insensitive token check shows the code is unchanged
- **AND** the pull request body carries that token-check output
- **AND** Pull request 88 is the precedent for carrying the check script and output in the body; its tokenizer does not satisfy this rule unless it also compares comments.
- **AND** any other whitespace change is a change
- **AND** the dropped `#[cfg(test)] mod tests {` and `}` lines are not a change to the moved block
- **AND** an added `#[cfg(test)] mod …;` declaration in a non-test path is not a change to the moved block
- **AND** asserting logic is unchanged

#### Scenario: Review allows a rustfmt-only reflow

- **GIVEN** an `OpenSpec-Refactor` pull request that moves a `#[cfg(test)]` block into a new test file
- **AND** the moved block differs by a rustfmt-only reflow
- **AND** a whitespace-insensitive token check, as defined in this scenario's THEN steps, shows the code is unchanged
- **WHEN** review checks the move
- **THEN** the move is allowed
- **AND** a rustfmt-only reflow of a moved block is not a change, as long as a whitespace-insensitive token check shows the code is unchanged
- **AND** Comments are tokens for this check: each comment's text, trimmed of leading/trailing whitespace, must match. Rustfmt does not rewrite comments.
- **AND** The check covers only the moved block: the #[cfg(test)] mod tests body at the merge-base against the added test files.
- **AND** It drops the mod wrapper lines and use items and compares the rest token by token, whitespace ignored.
- **AND** String and char literals are single tokens compared byte for byte.
- **AND** Path lines are compared after the allowed crate::/super::/self:: prefix difference.
- **AND** Items may be paired by name when split across files.
- **AND** The PR body carries the script (or a link at a fixed sha) and its output, ending in an explicit match/mismatch line.
- **AND** Pull request 88 is the precedent for carrying the check script and output in the body; its tokenizer does not satisfy this rule unless it also compares comments.

#### Scenario: Review rejects changed asserting logic

- **GIVEN** an `OpenSpec-Refactor` pull request that moves a `#[cfg(test)]` block into a new test file
- **AND** the added block changes an assertion, an expected value, control flow, a comment, or whitespace other than a uniform change in the moved block's leading indentation and other than a rustfmt-only reflow of a moved block when a whitespace-insensitive token check shows the code is unchanged and the pull request body carries that token-check output
- **WHEN** review checks the move
- **THEN** review rejects the change
- **AND** a uniform change in the moved block's leading indentation is not a change
- **AND** a rustfmt-only reflow of a moved block is not a change, as long as a whitespace-insensitive token check shows the code is unchanged
- **AND** the pull request body carries that token-check output
- **AND** Pull request 88 is the precedent for carrying the check script and output in the body; its tokenizer does not satisfy this rule unless it also compares comments.
- **AND** any other whitespace change is a change
- **AND** the dropped `#[cfg(test)] mod tests {` and `}` lines are not a change to the moved block
- **AND** an added `#[cfg(test)] mod …;` declaration in a non-test path is not a change to the moved block
- **AND** the `spec-drift` job does not log `spec-drift: refactor changes existing tests`

#### Scenario: A sibling tests.rs path is not a new test file

- **GIVEN** an `OpenSpec-Refactor` pull request
- **AND** the diff adds `crates/rules/src/tests.rs`
- **WHEN** review classifies that path
- **THEN** it is not a new test file
- **AND** the Test file requirement does not classify `crates/rules/src/tests.rs` as a test file

#### Scenario: An integration test move compiles apart from use and path lines

- **GIVEN** an `OpenSpec-Refactor` pull request
- **AND** the diff adds `crates/rules/tests/moved_tests.rs`
- **AND** that path is under `crates/<crate>/tests/`
- **AND** the moved block compiles unchanged apart from use lines and path lines
- **WHEN** review checks the move
- **THEN** the destination is allowed

#### Scenario: An integration test move that needs other edits is rejected

- **GIVEN** an `OpenSpec-Refactor` pull request
- **AND** the diff adds `crates/rules/tests/moved_tests.rs`
- **AND** that path is under `crates/<crate>/tests/`
- **AND** the moved block does not compile unchanged apart from use lines and path lines
- **WHEN** review checks the move
- **THEN** review rejects the move

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
