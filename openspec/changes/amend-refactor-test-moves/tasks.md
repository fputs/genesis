# Tasks

Genesis DevOps agent implements this change after `spec-approved`. Do not edit `crates/rules`. Do not edit `scripts/pr-hygiene.mjs`. Do not add a dependency. Do not add a Rust parser, a search for `#[cfg(test)]`, or a text compare of a removed block to a new test file. `scripts/spec-drift-check.mjs` stays free of the text `cfg(test)`. Leave `changesExistingTest` as it is when the new fixtures already pass. The body includes `OpenSpec-Change: amend-refactor-test-moves`.

## 1. Job fixtures

- [x] 1.1 In `scripts/spec-drift-check.test.mjs`, add a passing case named "A source file losing a cfg test block plus a new test file passes". Use the harness helpers that "An inline test module is not classified" uses. Base `crates/rules/src/lib.rs` contains a `#[cfg(test)]` block. Head removes that block and adds `crates/rules/src/encounter/tests/new_tests.rs`. The body line is `OpenSpec-Refactor: dice-and-checks`. Set `assertNoRustParse`. The case exits 0 and the log does not include `spec-drift: refactor changes existing tests`. Verify `node --test scripts/spec-drift-check.test.mjs` runs that name and passes. Do not edit `scripts/spec-drift-check.mjs` for this case.
- [x] 1.2 Add a passing case named "The job leaves the move to review". Base `crates/rules/src/lib.rs` contains a `#[cfg(test)]` block. Head removes that block and adds `crates/rules/tests/moved_tests.rs`. The body line is `OpenSpec-Refactor: dice-and-checks`. Set `assertNoRustParse`. Verify `node --test scripts/spec-drift-check.test.mjs` runs that name, the case exits 0, and the log does not include `spec-drift: refactor changes existing tests`.
- [x] 1.3 Add a failing case named "A moved block does not excuse an existing tests file". Use the task 1.1 head, and also modify `crates/rules/src/encounter/tests/delay_ready_tests.rs` that already exists on the base. The body line is `OpenSpec-Refactor: dice-and-checks`. Verify `node --test scripts/spec-drift-check.test.mjs` runs that name, the case exits non-zero, and the log includes `spec-drift: refactor changes existing tests`.
- [x] 1.4 Confirm `scripts/spec-drift-check.mjs` does not contain `cfg(test)` and that "An inline test module is not classified" and "A tests directory path is a test file" still pass. Verify `node --test scripts/spec-drift-check.test.mjs` exits 0.

## 2. Constitution text

- [ ] 2.1 In `AGENTS.md` section 1, keep the refactor bullet. After the existing sentence that forbids modifying a crates/ path containing `/tests/`, and before `The Unreleased bullet is still required.`, insert the section 1 sentences in the block below. In section 8, append the section 8 sentence to the same-file bullet. In `docs/architect/decisions.md`, after the existing sentence that says a crates/ path containing `/tests/` is not modified, insert the decisions sentence. Change no other sentence in those files. Verify those sentences are present and that `node --test scripts/spec-drift-check.test.mjs` exits 0.

Section 1 sentences:

```
An in-file #[cfg(test)] block may be moved verbatim into a new test file. A use line or a path line may change, as the inline test block move requirement defines them. A uniform change in the moved block's leading indentation is not a change. Any other whitespace change is. The dropped `#[cfg(test)] mod tests {` and `}` lines, and an added `#[cfg(test)] mod …;` declaration, are not changes. A move into `crates/<crate>/tests/` is allowed only if the block compiles unchanged apart from use and path lines. Asserting logic may not change, in place or in the move. That includes an assertion, an expected value, control flow, or a comment. A dropped block that no new test file contains is a change. A new test that was not in the removed block is not a change to the moved block. Review enforces the verbatim move and the unchanged asserting logic. The spec-drift job does not distinguish a verbatim move from an edit and does not reject a non-test crates/ file that loses a #[cfg(test)] block when a new test file is added.
```

Section 8 sentence:

```
An OpenSpec-Refactor pull request may move that block verbatim into a new test file under the inline test block move rule.
```

Decisions sentence:

```
An in-file #[cfg(test)] block may be moved verbatim into a new test file. Only a use line or a path line may change, as the inline test block move requirement defines them. A uniform change in the moved block's leading indentation is not a change. Any other whitespace change is. Asserting logic may not change. Review enforces that move. The spec-drift job does not reject it.
```
- [ ] 2.2 Add a `### Changed` bullet under `## [Unreleased]` in a commit before the archive commit. The bullet states that an `OpenSpec-Refactor` pull request may move an in-file `#[cfg(test)]` block verbatim into a new test file, and that the spec-drift job still does not parse Rust or treat that move as an edit of an existing test file. The bullet links `https://github.com/fputs/genesis/pull/` plus this pull request's number. Verify the new bullet is under `### Changed` and contains that URL.

## 3. Proof and archive

- [ ] 3.1 Run the AGENTS.md proof commands and verify each exits 0: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo build --workspace`, `cargo test --workspace`, and `npx @fission-ai/openspec@1.14.3 validate --all --strict`.
- [ ] 3.2 As the final commit of this pull request, after Architect and QA have both passed the previous head, run `openspec archive amend-refactor-test-moves --yes` and check this box in that same commit. The commit deletes `openspec/changes/amend-refactor-test-moves/` and adds `openspec/changes/archive/<YYYY>-<MM>-<DD>-amend-refactor-test-moves/`. It does not modify `CHANGELOG.md`. Leave this box open until that commit so the change is not a completed unarchived change. Verify the change is no longer an active directory under `openspec/changes/`.

## Workflow follow-up

- Task 3.2 is the archive commit, right before merge, and only after Architect and QA have passed the head that was archived.
