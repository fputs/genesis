# Proposal

## Why

An `OpenSpec-Refactor:` pull request may change `crates/` and `CHANGELOG.md`, and it may not edit an existing file under a `tests/` directory. The spec-drift spec never says whether editing or moving an in-file `#[cfg(test)]` module inside a `src/` file counts as editing existing tests. That gap is what this change closes.

## What Changes

- A verbatim move of an in-file `#[cfg(test)]` block into a new test file is allowed on an `OpenSpec-Refactor` pull request. A new test file is an added path the Test file requirement classifies as a test file, so `src/tests.rs` does not qualify. A uniform change in the moved block's leading indentation is not a change. Any other whitespace change is. The dropped `#[cfg(test)] mod tests {` and `}` lines, and an added `#[cfg(test)] mod …;` declaration in a non-test path, are not changes to the moved block. `crates/<crate>/tests/` is allowed only if the block compiles unchanged apart from use/path lines. Asserting logic may not change, in place or in the move. Comments stay verbatim. Owner review on pull request 86.
- An existing file under a `tests/` directory stays untouchable. Modify, delete, typechange, and rename of that file still fail the job.
- The job does not parse Rust, so it cannot tell a verbatim move from an edit. Review enforces the verbatim move and the unchanged asserting logic. The job must not reject the move shape: a non-test `crates/` file loses a `#[cfg(test)]` block and a new test file is added.
- `AGENTS.md` section 1 and section 8, and the refactor paragraph in `docs/architect/decisions.md`, gain the same sentences so the constitution matches this spec. No new package. `crates/rules` is untouched. `scripts/pr-hygiene.mjs` is untouched.

This pull request's first commit is the spec only. Scripts, tests, and those docs wait for `spec-approved`.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `spec-drift-check`: `Test file` gains the scenario where a source file loses a `#[cfg(test)]` block and a new test file is added, and the job passes. Added requirement `Inline test block move` states the verbatim move, forbids asserting-logic edits, keeps an existing test file untouchable, and assigns the verbatim check to review so the job does not falsely reject the move.

## Impact

- DevOps adds the job cases in `scripts/spec-drift-check.test.mjs` and does not teach `scripts/spec-drift-check.mjs` to parse Rust or to compare a removed block with a new test file. The script source stays free of the text `cfg(test)`.
- `AGENTS.md` and `docs/architect/decisions.md` state the same move rule. The Unreleased bullet is `### Changed` and links this pull request, in a commit before the archive commit.
- No dependency is added. The job's existing-tests log line is unchanged.
