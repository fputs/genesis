# Design

## Context

See proposal.md for why. `scripts/spec-drift-check.mjs` treats a test file as a `crates/` path that contains `/tests/`. `changesExistingTest` fails a refactor only for status `M`, `D`, or `T` on that path, or a rename whose old path is a test file. Status `A` is allowed. The script does not parse Rust, and `scripts/spec-drift-check.test.mjs` asserts the script source does not contain `cfg(test)`. `AGENTS.md` section 8 still says every pure function's tests live in the same file, which would contradict a move until that sentence gains the exception below. Normative behavior is the delta spec.

## Goals / Non-Goals

**Goals:**

- Keep the job from rejecting a non-test `crates/` edit that drops a `#[cfg(test)]` block when a new test file is added.
- Leave the verbatim check, and any edit to asserting logic, to review.
- Keep an existing test file failing the job.

**Non-Goals:**

- No Rust parser, no search for `#[cfg(test)]`, and no comparison of removed text to an added file.
- No change to `scripts/pr-hygiene.mjs`, the changelog category rules, or rename scoring.
- No edit to `crates/rules`.

## Decisions

### 1. The job stays path-shaped

The move the job must accept is already a non-test path plus an added test file. Do not teach the script to find the attribute or to diff the block. A parser that misses a wrapped attribute, or that treats a `use` edit as a logic edit, would reject a verbatim move. The existing `assertNoRustParse` check stays: the script source must not contain `cfg(test)`.

Alternative considered: extract the `#[cfg(test)]` region and compare it to the new file with a use-line allowance. Rejected. The spec says the job shall not distinguish a verbatim move from an edit, and a wrong compare is a false reject.

### 2. Review owns verbatim

A use line, after leading whitespace is removed, starts with `use ` or `pub use `. A path line differs only by a module path written with `crate::`, `super::`, or `self::`. Any other difference in the moved block, including an assertion, an expected value, control flow, a comment, or whitespace in the test body, is a change to asserting logic. Review rejects that, an in-place assertion edit, and a dropped block that no new test file contains. A new test that was not in the removed block is not a change to the moved block. The unchanged-behavior gate in the single-pr-flow spec still applies to the pull request as a whole.

Alternative considered: make CI fail in-place assertion edits. Rejected. That requires parsing, which decision 1 refuses.

### 3. Existing test files stay on the current status rule

`changesExistingTest` is unchanged. A legal move beside a modified, deleted, typechanged, or renamed test file still logs `spec-drift: refactor changes existing tests`. Adding the new file does not excuse that record.

### 4. The constitution states the same rule

Implementation edits `AGENTS.md` section 1 after "Do not modify a `crates/` path that contains `/tests/`.", section 8 after the same-file sentence, and the refactor paragraph in `docs/architect/decisions.md`. Those sentences are the exception that keeps the constitution from contradicting the spec. They are not a second rule.

## Risks / Trade-offs

- [CI stays green when asserting logic changes inside a `#[cfg(test)]` block] → Review rejects it. The job must not grow a parser to close that hole.
- [A use line can name a different item] → The unchanged-behavior gate still applies. The verbatim rule allows that line to differ.
- [rustfmt rewrites the moved body] → Format the block before the move. Whitespace in the test body is a change review rejects.
- [The script already passes the move, so the production file may not change] → Add the tests anyway. Do not add a dead check that searches for `cfg(test)`.

## Migration Plan

No data migration. The job's pass/fail for this diff does not change. Rollback is reverting this change. The `### Changed` bullet is added before the archive commit and links this pull request.
