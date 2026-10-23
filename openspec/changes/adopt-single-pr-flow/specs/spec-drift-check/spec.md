# spec-drift-check

## MODIFIED Requirements

### Requirement: Code paths

A changed path is a code path when it is exactly `Cargo.toml`, `Cargo.lock`, or `rust-toolchain.toml`, or when it starts with `crates/` or `.github/workflows/`. Changed paths SHALL be the additions, modifications, deletions, and both sides of renames from `git merge-base` of the event base sha and the head sha to the head. The job SHALL NOT diff `pull_request.base.sha` directly. No merge-base SHALL fail the job closed. No other path is a code path.

A qualifying refactor exemption SHALL NOT clear that code-path classification and SHALL NOT log `spec-drift: code change missing OpenSpec-Change`. Every other pull request with a code path and no valid change reference SHALL still log that line.

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

#### Scenario: The diff starts at the merge-base

- **GIVEN** a pull_request whose base sha edits `crates/rules/src/lib.rs` after the merge-base
- **AND** the head matches the merge-base for that file
- **AND** the body has no change reference
- **AND** validation exits zero
- **AND** no active change is fully checked
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes
- **AND** the path diff is `git diff` from `git merge-base` of the base sha and the head to the head

#### Scenario: No merge-base fails closed

- **GIVEN** a pull_request whose base sha and head sha have no merge-base
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: no merge-base`

#### Scenario: A qualifying refactor does not log a missing change line

- **GIVEN** a pull_request to `main` whose merge-base diff changes only `crates/rules/src/lib.rs`
- **AND** the only reference line is `OpenSpec-Refactor: dice-and-checks`
- **AND** validation exits zero
- **AND** the completed-change check passes
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes
- **AND** `crates/rules/src/lib.rs` is still a code path
- **AND** the log does not include `spec-drift: code change missing OpenSpec-Change`

### Requirement: Change reference line

A body line, after one trailing CR is removed and surrounding whitespace is trimmed, that starts with the exact case-sensitive `OpenSpec-Change:` SHALL be a reference. It SHALL be valid if and only if it is that prefix, then exactly one U+0020, then one kebab id. Otherwise it SHALL be an invalid id and SHALL NOT be skipped or case-folded. Fenced lines SHALL count the same. This SHALL be the AGENTS.md line. The id SHALL be one or more `[a-z0-9]` groups joined by single hyphens.

An `OpenSpec-Refactor:` line SHALL NOT be a change reference. Exactly one valid refactor line, and no `OpenSpec-Change` line, SHALL satisfy the missing-reference check when the qualifying refactor paths hold. A second refactor line, a malformed refactor line, or any `OpenSpec-Change` line beside a refactor line SHALL NOT satisfy it.

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
- **AND** the log line is `spec-drift: invalid change id` immediately followed by `\u0009add-foo`
- **AND** the log does not include `spec-drift: code change missing OpenSpec-Change`

#### Scenario: Two spaces before the id are invalid

- **GIVEN** a pull_request to `main` whose diff is only `docs/architect/decisions.md`
- **AND** the body line is `OpenSpec-Change:` plus two U+0020 spaces plus `add-foo`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log line is `spec-drift: invalid change id` immediately followed by those two U+0020 spaces and `add-foo`
- **AND** nothing in that remainder was stripped
- **AND** the log does not include `spec-drift: code change missing OpenSpec-Change`

#### Scenario: A lowercase prefix is ignored

- **GIVEN** a pull_request to `main` that changes `crates/rules/src/lib.rs`
- **AND** the only body line is `openspec-change: foo`
- **AND** `openspec/changes/foo/` exists
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: code change missing OpenSpec-Change`
- **AND** the log does not include `spec-drift: invalid change id`
- **AND** the log does not include `spec-drift: unknown change foo`

#### Scenario: A refactor line contributes no change id

- **GIVEN** a body whose only reference line is `OpenSpec-Refactor: dice-and-checks`
- **WHEN** `changeReferences` parses it
- **THEN** that line contributes no change id

## ADDED Requirements

### Requirement: Refactor line

A body line, after one trailing CR is removed and surrounding whitespace is trimmed, that starts with exact case-sensitive `OpenSpec-Refactor:` SHALL be a refactor line. It SHALL be valid only as that prefix, one U+0020, and one roadmap item matching `^[a-z0-9]+(?:-[a-z0-9]+)*$`. Fenced lines SHALL count. `openspec-refactor:` SHALL NOT be a refactor line. The job SHALL NOT look the item up. On push the job SHALL NOT read refactor lines.

#### Scenario: A trimmed refactor line passes

- **GIVEN** a pull_request to `main` that changes `crates/rules/src/lib.rs`
- **AND** the body line is `  OpenSpec-Refactor: dice-and-checks  `
- **AND** validation exits zero
- **AND** the completed-change check passes
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes
- **AND** the log does not include `spec-drift: code change missing OpenSpec-Change`

#### Scenario: An item absent from the decision log passes

- **GIVEN** a pull_request to `main` that changes `crates/rules/src/lib.rs`
- **AND** the body line is `OpenSpec-Refactor: not-a-real-item`
- **AND** validation exits zero
- **AND** the completed-change check passes
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes
- **AND** the log does not include `spec-drift: unknown change not-a-real-item`

#### Scenario: A fenced refactor line counts

- **GIVEN** a pull_request to `main` that changes `crates/rules/src/lib.rs`
- **AND** the only refactor line sits inside a fenced code block and is `OpenSpec-Refactor: grid-movement`
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

#### Scenario: A push ignores a malformed refactor line

- **GIVEN** a push to `main` that changes `crates/rules/src/lib.rs`
- **AND** the event body is `OpenSpec-Refactor:`
- **AND** validation exits zero
- **AND** no active change is fully checked
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes
- **AND** the log does not include `spec-drift: invalid refactor item`

### Requirement: Invalid refactor item

A malformed refactor line SHALL fail on every pull request. The log line SHALL be `spec-drift: invalid refactor item` plus the whole remainder after `OpenSpec-Refactor:`, with nothing stripped. Each UTF-16 code unit outside U+0020 through U+007E, and a backslash, SHALL be `\u` plus four lowercase hex digits. Copy every other unit. The line SHALL NOT use JSON quotes, and SHALL NOT also log `spec-drift: code change missing OpenSpec-Change`.

#### Scenario: An empty refactor line is invalid

- **GIVEN** a pull_request to `main` whose diff is only `docs/architect/decisions.md`
- **AND** the body contains a line `OpenSpec-Refactor:`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes a line whose text is exactly `spec-drift: invalid refactor item`

#### Scenario: Extra words are an invalid refactor item

- **GIVEN** a pull_request to `main` whose diff is only `docs/architect/decisions.md`
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks extra`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: invalid refactor item dice-and-checks extra`

#### Scenario: A tab after the refactor colon is escaped

- **GIVEN** a pull_request to `main` whose diff is only `docs/architect/decisions.md`
- **AND** the body line is `OpenSpec-Refactor:` immediately followed by a tab and `dice`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log line is `spec-drift: invalid refactor item` immediately followed by `\u0009dice`

### Requirement: Multiple refactor lines

Two or more refactor lines SHALL fail, including two copies of one valid line. Duplicates SHALL NOT collapse. The log SHALL include `spec-drift: multiple OpenSpec-Refactor lines`. The job SHALL NOT also log the path lines or `spec-drift: code change missing OpenSpec-Change`.

#### Scenario: A repeated refactor line fails

- **GIVEN** a pull_request to `main` that changes `crates/rules/src/lib.rs`
- **AND** the body contains `OpenSpec-Refactor: dice-and-checks` twice
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: multiple OpenSpec-Refactor lines`
- **AND** the log does not include `spec-drift: refactor is not crates-only`

### Requirement: Refactor line conflicts with OpenSpec-Change

Any refactor line plus any `OpenSpec-Change:` line SHALL fail, including on a docs-only pull request. The log SHALL include `spec-drift: refactor line conflicts with OpenSpec-Change`. A valid change id SHALL NOT win. The job SHALL NOT also log the path lines or the missing-reference line.

#### Scenario: Both lines fail on a docs-only pull request

- **GIVEN** a pull_request to `main` whose diff is only `docs/architect/decisions.md`
- **AND** the body contains `OpenSpec-Refactor: dice-and-checks` and `OpenSpec-Change: add-foo`
- **AND** `openspec/changes/add-foo/` exists
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: refactor line conflicts with OpenSpec-Change`
- **AND** the log does not include `spec-drift: code change missing OpenSpec-Change`

### Requirement: Qualifying refactor paths

Exactly one valid refactor line, and no `OpenSpec-Change:` line, SHALL satisfy the reference check only when every changed path starts with `crates/` or is exactly `CHANGELOG.md`. Both sides of a rename SHALL count. The diff SHALL be `git diff -z --name-status --find-renames` from `git merge-base` of the event base sha and the head sha to the head, with no `--find-copies`.

#### Scenario: A crate file with a changelog passes

- **GIVEN** a pull_request to `main` whose merge-base diff changes `crates/rules/src/lib.rs` and `CHANGELOG.md`
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **AND** validation exits zero
- **AND** the completed-change check passes
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes

#### Scenario: Adding a test file passes

- **GIVEN** a pull_request to `main` whose diff adds `crates/rules/src/encounter/tests/new_tests.rs`
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **AND** validation exits zero
- **AND** the completed-change check passes
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes

### Requirement: Refactor path failures

When that single valid line is the only reference and the diff has a code path, a path outside `crates/` and not exactly `CHANGELOG.md` SHALL log `spec-drift: refactor is not crates-only`. A path under `openspec/` SHALL also log `spec-drift: refactor changes openspec`. Order SHALL be crates-only, then openspec, then existing tests. The missing-reference line SHALL NOT be added. Otherwise path lines SHALL be omitted.

#### Scenario: A crate file plus README is not a refactor

- **GIVEN** a pull_request to `main` whose diff changes `crates/rules/src/lib.rs` and `README.md`
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
- **AND** `spec-drift: refactor is not crates-only` appears before `spec-drift: refactor changes openspec`

#### Scenario: Renaming into crates counts both sides

- **GIVEN** a pull_request to `main` whose diff renames `docs/note.md` to `crates/rules/note.md`
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: refactor is not crates-only`

#### Scenario: A docs-only valid refactor line passes

- **GIVEN** a pull_request to `main` whose diff is only `docs/architect/decisions.md`
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **AND** validation exits zero
- **AND** no active change is fully checked
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes
- **AND** the log does not include `spec-drift: refactor is not crates-only`

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

### Requirement: Existing test change

An existing test file SHALL count as modified or removed when a record status is `M`, `D`, or `T` and that path is a test file, or when a status starting with `R` has a test file as its old path. Status `A` on a test file SHALL be allowed. A rename whose old path is not a test file SHALL be allowed when the new path is a test file. The status token includes a score suffix such as `R100`.

#### Scenario: Modifying a test file fails

- **GIVEN** a pull_request to `main` whose diff status for `crates/rules/tests/foo.rs` is `M`
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: refactor changes existing tests`

#### Scenario: Adding a test file is allowed

- **GIVEN** a pull_request to `main` whose diff status for `crates/rules/tests/foo.rs` is `A`
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **AND** validation exits zero
- **AND** the completed-change check passes
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes

#### Scenario: A typechange of a test file fails

- **GIVEN** a pull_request to `main` whose diff status for `crates/rules/tests/foo.rs` is `T`
- **AND** that record replaces the file with a symlink
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: refactor changes existing tests`

#### Scenario: Renaming a test file fails

- **GIVEN** a pull_request to `main` whose diff status starts with `R100`
- **AND** the old path is `crates/rules/tests/foo.rs`
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: refactor changes existing tests`

#### Scenario: Renaming a non-test file into tests passes

- **GIVEN** a pull_request to `main` whose diff renames `crates/rules/src/lib.rs` to `crates/rules/src/encounter/tests/lib.rs`
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **AND** validation exits zero
- **AND** the completed-change check passes
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes

### Requirement: Refactor token order

Invalid change ids SHALL be logged before invalid refactor items, in body order, a duplicate remainder once. Then `spec-drift: multiple OpenSpec-Refactor lines` when more than one refactor line is present, then the conflict line when both prefixes are present. Unknown ids and the missing-reference line follow only when that existing rule still applies. Path lines follow the token lines.

#### Scenario: Mixed body failures keep that order

- **GIVEN** a pull_request to `main` that changes `crates/rules/src/lib.rs`
- **AND** the body is `OpenSpec-Change: Add-Foo`, then `OpenSpec-Refactor: NOT-AN-ITEM`, then `OpenSpec-Refactor: dice-and-checks`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** `Add-Foo` is logged before `NOT-AN-ITEM`
- **AND** that item is logged before `spec-drift: multiple OpenSpec-Refactor lines`
- **AND** that line is logged before `spec-drift: refactor line conflicts with OpenSpec-Change`

#### Scenario: Token failures omit path lines

- **GIVEN** the pull request in the mixed body failure
- **WHEN** the `spec-drift` job runs
- **THEN** the log does not include `spec-drift: refactor is not crates-only`

### Requirement: Refactor does not skip the changelog

Passing this exemption SHALL NOT waive a bullet the changelog and pr-hygiene specs require; this job SHALL NOT check the bullet. `OpenSpec-Refactor` SHALL NOT be an own-change line.

#### Scenario: Spec-drift ignores a missing bullet

- **GIVEN** a pull_request to `main` whose diff changes `crates/rules/src/lib.rs` and adds no Unreleased bullet
- **AND** the body line is `OpenSpec-Refactor: dice-and-checks`
- **AND** validation exits zero
- **AND** the completed-change check passes
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes
- **AND** the log does not include `implementation PR missing CHANGELOG bullet`

#### Scenario: The bullet stays with the other specs

- **GIVEN** `openspec/specs/pr-hygiene/spec.md` and `openspec/specs/changelog/spec.md`
- **WHEN** those requirements are read
- **THEN** this exemption does not waive a bullet those specs require
- **AND** this job does not check the bullet

#### Scenario: A refactor line is not an own-change line

- **GIVEN** a pull request body whose only reference line is `OpenSpec-Refactor: dice-and-checks`
- **WHEN** own-change lines are parsed by the `OpenSpec-Change` rule
- **THEN** that line contributes no change id
