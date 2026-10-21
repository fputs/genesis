# pr-hygiene

## Purpose

The pr-hygiene capability fails a pull request when an implementation change adds no Unreleased changelog bullet, or when a named OpenSpec change is removed by a commit that is not the archive.

## ADDED Requirements

### Requirement: PR-only hygiene job

The repository SHALL run a job named `pr-hygiene` that invokes `scripts/pr-hygiene.mjs` on pull_request types `opened`, `synchronize`, `reopened`, and `edited` targeting `main`. That job SHALL NOT run on push. Its permissions SHALL be `contents: read` and `pull-requests: read`.

#### Scenario: Job runs on the four pull request types

- **GIVEN** `.github/workflows/pr-hygiene.yml`
- **WHEN** its triggers are read
- **THEN** the `pr-hygiene` job runs on pull_request `opened`, `synchronize`, `reopened`, and `edited` targeting `main`
- **AND** it invokes `scripts/pr-hygiene.mjs`

#### Scenario: The job does not run on push

- **GIVEN** `.github/workflows/pr-hygiene.yml`
- **WHEN** its triggers are read
- **THEN** the workflow has no push trigger

#### Scenario: Permissions are contents and pull requests read

- **GIVEN** `.github/workflows/pr-hygiene.yml`
- **WHEN** its permissions are read
- **THEN** they are `contents: read` and `pull-requests: read`

### Requirement: Cargo and spec-drift stay

The `pr-hygiene` job SHALL be the only job in `.github/workflows/pr-hygiene.yml`. `.github/workflows/ci.yml` SHALL keep the Cargo job and the `spec-drift` job, with their current steps and triggers.

#### Scenario: Cargo and spec-drift stay

- **GIVEN** `.github/workflows/ci.yml` after this change is applied
- **WHEN** the Cargo job and the `spec-drift` job are read
- **THEN** their steps and triggers are unchanged
- **AND** `pr-hygiene` is not a job in that file

### Requirement: No npm dependencies

The check SHALL be a script with no npm dependencies. The job SHALL NOT install an npm package, and SHALL run that script with Node.

#### Scenario: The job installs no npm package

- **GIVEN** the `pr-hygiene` job
- **WHEN** its steps are read
- **THEN** no step installs an npm package
- **AND** the script imports only Node built-ins

### Requirement: Node runs the suite first

The `pr-hygiene` job SHALL run on Node 22. It SHALL run `node --test` on the fixture suite before `scripts/pr-hygiene.mjs`. A failing test SHALL fail the job.

#### Scenario: The job runs the suite first

- **GIVEN** the `pr-hygiene` job
- **WHEN** it runs
- **THEN** it uses Node 22
- **AND** `node --test` runs before `scripts/pr-hygiene.mjs`
- **AND** a failing test fails the job

### Requirement: NUL-split diff paths

Changed paths SHALL be parsed from `git diff -z` output split on NUL, including additions, modifications, deletions, and both sides of a rename, as the spec-drift check parses them.

#### Scenario: NUL-split diff includes both rename sides

- **GIVEN** a pull_request whose diff renames `docs/note.md` to `crates/rules/note.md`
- **WHEN** changed paths are parsed
- **THEN** the parse uses NUL-separated `git diff -z` records
- **AND** both `docs/note.md` and `crates/rules/note.md` are changed paths

#### Scenario: A rename into crates is an implementation pull request

- **GIVEN** a pull_request whose diff renames `docs/note.md` to `crates/rules/note.md`
- **AND** `CHANGELOG.md` gains no Unreleased bullet
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

### Requirement: Fail closed on parse errors

When the event payload, a diff, a NUL record, the base, the head, the head's first parent, or the merge-base cannot be parsed, the job SHALL exit non-zero and SHALL NOT treat the pull request as exempt.

#### Scenario: A diff parse error fails closed

- **GIVEN** a pull_request whose `git diff -z` exits non-zero
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** it does not treat the pull request as exempt

#### Scenario: A missing base fails closed

- **GIVEN** a pull_request event with no base sha, or whose base and head have no merge-base
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails

#### Scenario: A missing head parent fails closed

- **GIVEN** a pull request head commit with no parent
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails

### Requirement: Pull request head from the event

Every pull request diff SHALL use `git merge-base` of the event base sha and head sha, including implementation classification, the bullet diff, and the base-side `## [Unreleased]` section. The job SHALL NOT diff `pull_request.base.sha` directly. Archive-last SHALL still read the event head message and its first-parent diff, not a runner merge commit. No merge-base SHALL fail the job closed.

#### Scenario: The check uses the event head

- **GIVEN** a pull_request event whose head sha differs from a merge commit the runner created
- **WHEN** the `pr-hygiene` job runs
- **THEN** the bullet diff ends at that event head sha
- **AND** its other side is the merge-base of the event base sha and that head
- **AND** archive-last reads that head commit, not the runner merge

#### Scenario: main advanced after branching

- **GIVEN** main gains a crate change and an Unreleased bullet after the branch point
- **AND** the pull request changes only `docs/architect/decisions.md` since that merge-base
- **AND** the pull request adds no Unreleased bullet
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes

### Requirement: Implementation pull request

An implementation pull request SHALL be a diff that includes a spec-drift code path: exactly `Cargo.toml`, `Cargo.lock`, or `rust-toolchain.toml`, or a path starting with `crates/` or `.github/workflows/`.

#### Scenario: A crate file is an implementation pull request

- **GIVEN** a pull_request whose diff changes `crates/rules/src/lib.rs`
- **AND** `CHANGELOG.md` gains no Unreleased bullet
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

#### Scenario: A workflow file is an implementation pull request

- **GIVEN** a pull_request whose diff changes `.github/workflows/ci.yml`
- **AND** `CHANGELOG.md` gains no Unreleased bullet
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

#### Scenario: Root Cargo.toml is an implementation pull request

- **GIVEN** a pull_request whose diff changes `Cargo.toml`
- **AND** `CHANGELOG.md` gains no Unreleased bullet
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

#### Scenario: A scripts-only pull request is not an implementation pull request

- **GIVEN** a pull_request whose only diff is `scripts/note.mjs`
- **AND** `CHANGELOG.md` gains no Unreleased bullet
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes

#### Scenario: A nested Cargo.toml is not a root manifest

- **GIVEN** a pull_request whose only diff is `docs/Cargo.toml`
- **AND** `CHANGELOG.md` gains no Unreleased bullet
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes

### Requirement: Unreleased changelog bullet

An implementation pull request that is not exempt SHALL add at least one bullet under `## [Unreleased]` in `CHANGELOG.md`. Otherwise the job SHALL fail and the log SHALL include `implementation PR missing CHANGELOG bullet`. A missing `CHANGELOG.md`, or a missing `## [Unreleased]` heading, on the head SHALL fail an implementation pull request the same way.

#### Scenario: An added Unreleased bullet passes

- **GIVEN** a pull_request whose diff changes `crates/rules/src/lib.rs`
- **AND** the head `## [Unreleased]` section has a list item the base section does not
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes

#### Scenario: A missing bullet fails

- **GIVEN** a pull_request whose diff changes `crates/rules/src/lib.rs`
- **AND** the Unreleased list items on the head match the base
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

#### Scenario: A bullet outside Unreleased fails

- **GIVEN** a pull_request whose diff changes `crates/rules/src/lib.rs`
- **AND** the only new list item is under `## [0.1.0]`
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

#### Scenario: Any Unreleased list item passes

- **GIVEN** a pull_request whose diff changes `.github/workflows/pr-hygiene.yml`
- **AND** the new line under `## [Unreleased]` is `- Record the gate` with no pull request link
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes

#### Scenario: A missing head changelog fails

- **GIVEN** an implementation pull request whose head has no `CHANGELOG.md`, or whose `CHANGELOG.md` has no `## [Unreleased]` heading
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

### Requirement: Changelog bullet shape

A bullet SHALL be a trimmed line inside `## [Unreleased]`, before the next `## ` heading, that is a markdown list item with text. Added means the head section has such a line the base section does not. A missing `CHANGELOG.md`, or a missing `## [Unreleased]` heading, on the base SHALL leave the base bullet set empty.

#### Scenario: An unchanged existing bullet fails

- **GIVEN** a pull_request whose diff changes `Cargo.toml`
- **AND** the only Unreleased list item is already on the base
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

#### Scenario: A missing base changelog leaves no bullets

- **GIVEN** an implementation pull request whose base has no `CHANGELOG.md`, or whose base `CHANGELOG.md` has no `## [Unreleased]` heading
- **AND** the head `## [Unreleased]` section has a list item
- **WHEN** the `pr-hygiene` job runs
- **THEN** the base bullet set is empty
- **AND** the job passes

### Requirement: Bullet exemptions

The bullet check SHALL pass with no new bullet when the diff is docs-only, openspec-only, or every path is under `.cursor/` or is exactly `README.md`, `Cargo.lock`, or `rust-toolchain.toml`. Docs-only paths SHALL be `docs/**`, `AGENTS.md`, and `CHANGELOG.md`. Openspec-only SHALL mean every path starts with `openspec/`.

#### Scenario: Docs-only passes without a bullet

- **GIVEN** a pull_request whose diff is `docs/architect/decisions.md`, `AGENTS.md`, and `CHANGELOG.md`
- **AND** no Unreleased bullet is added
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes

#### Scenario: Openspec-only passes without a bullet

- **GIVEN** a pull_request whose diff is only under `openspec/`
- **AND** no Unreleased bullet is added
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes

#### Scenario: A cursor-only pull request passes without a bullet

- **GIVEN** a pull_request whose only diff is `.cursor/rules/example.md`
- **AND** no Unreleased bullet is added
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes

#### Scenario: A README-only pull request passes without a bullet

- **GIVEN** a pull_request whose only diff is `README.md`
- **AND** no Unreleased bullet is added
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes

#### Scenario: A Cargo.lock-only pull request passes without a bullet

- **GIVEN** a pull_request whose only diff is `Cargo.lock`
- **AND** no Unreleased bullet is added
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes

#### Scenario: A rust-toolchain-only pull request passes without a bullet

- **GIVEN** a pull_request whose only diff is `rust-toolchain.toml`
- **AND** no Unreleased bullet is added
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes

#### Scenario: The exempt path set together passes without a bullet

- **GIVEN** a pull_request whose diff is `.cursor/rules/example.md`, `README.md`, `Cargo.lock`, and `rust-toolchain.toml`
- **AND** no Unreleased bullet is added
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes

#### Scenario: Docs plus a crate still needs a bullet

- **GIVEN** a pull_request whose diff changes `docs/architect/decisions.md` and `crates/rules/src/lib.rs`
- **AND** no Unreleased bullet is added
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

#### Scenario: Openspec plus a workflow file still needs a bullet

- **GIVEN** a pull_request whose diff changes `openspec/specs/changelog/spec.md` and `.github/workflows/ci.yml`
- **AND** no Unreleased bullet is added
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

#### Scenario: Cursor plus a crate still needs a bullet

- **GIVEN** a pull_request whose diff changes `.cursor/rules/example.md` and `crates/rules/src/lib.rs`
- **AND** no Unreleased bullet is added
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

#### Scenario: README plus root Cargo.toml still needs a bullet

- **GIVEN** a pull_request whose diff changes `README.md` and `Cargo.toml`
- **AND** no Unreleased bullet is added
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

#### Scenario: Cargo.lock plus a crate still needs a bullet

- **GIVEN** a pull_request whose diff changes `Cargo.lock` and `crates/rules/src/lib.rs`
- **AND** no Unreleased bullet is added
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

#### Scenario: rust-toolchain.toml plus a workflow file still needs a bullet

- **GIVEN** a pull_request whose diff changes `rust-toolchain.toml` and `.github/workflows/ci.yml`
- **AND** no Unreleased bullet is added
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

### Requirement: No opt-out label

No pull request label SHALL exempt an implementation pull request from the bullet check.

#### Scenario: A label does not skip the bullet

- **GIVEN** a pull_request whose diff changes `crates/rules/src/lib.rs`
- **AND** the pull request has a label meant to skip the changelog
- **AND** no Unreleased bullet is added
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

### Requirement: Change reference line

An `OpenSpec-Change: <id>` line SHALL be one trimmed body line of `OpenSpec-Change:` plus one spec-drift change id. Extra words SHALL NOT count. Fenced lines SHALL count the same.

#### Scenario: Extra words are not a change reference

- **GIVEN** a pull_request whose diff removes `openspec/changes/add-foo/`
- **AND** the only body line is `OpenSpec-Change: add-foo extra`
- **AND** the head commit also changes `README.md`
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes
- **AND** the log does not include `archive commit is not last`

#### Scenario: A fenced line still counts

- **GIVEN** a pull_request whose diff removes `openspec/changes/add-foo/`
- **AND** the body contains a fenced `OpenSpec-Change: add-foo` line
- **AND** the head commit message does not contain `openspec archive add-foo`
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `archive commit is not last`

### Requirement: Archive commit is last

When the body has `OpenSpec-Change: <id>` and the diff removes `openspec/changes/<id>/`, the head message SHALL contain `openspec archive <id>` with the id at a token end, and every path it touches SHALL sit under `openspec/changes/archive/`, under `openspec/specs/`, or that removed directory. Otherwise the log SHALL include `archive commit is not last`.

#### Scenario: The archive commit passes

- **GIVEN** a pull_request body line `OpenSpec-Change: add-foo`
- **AND** the diff removes `openspec/changes/add-foo/`
- **AND** the head message is `openspec archive add-foo --yes`
- **AND** the head diff only deletes that directory, writes `openspec/changes/archive/2026-10-21-add-foo/`, and edits `openspec/specs/changelog/spec.md`
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes

#### Scenario: A path outside archive and specs fails

- **GIVEN** a pull_request body line `OpenSpec-Change: add-foo`
- **AND** the diff removes `openspec/changes/add-foo/`
- **AND** the head message contains `openspec archive add-foo`
- **AND** the head diff also changes `docs/producer/process.md`
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `archive commit is not last`

#### Scenario: A missing archive phrase fails

- **GIVEN** a pull_request body line `OpenSpec-Change: add-foo`
- **AND** the diff removes `openspec/changes/add-foo/`
- **AND** the head paths are only that removal, `openspec/changes/archive/**`, and `openspec/specs/**`
- **AND** the head message is `archive the change`
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `archive commit is not last`

#### Scenario: A longer id does not cover a shorter id

- **GIVEN** a pull_request body line `OpenSpec-Change: add-foo`
- **AND** the diff removes `openspec/changes/add-foo/`
- **AND** the head message is `openspec archive add-foo-extra --yes`
- **AND** the head paths are only the allowed archive paths
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `archive commit is not last`

#### Scenario: A named change that stays active skips archive-last

- **GIVEN** a pull_request body line `OpenSpec-Change: add-foo`
- **AND** the head still has `openspec/changes/add-foo/`
- **AND** the head commit edits `openspec/changes/add-foo/proposal.md`
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes
- **AND** the log does not include `archive commit is not last`

#### Scenario: A removal without a reference skips archive-last

- **GIVEN** a pull_request whose diff removes `openspec/changes/add-foo/`
- **AND** the body has no `OpenSpec-Change: add-foo` line
- **AND** the head commit also changes `README.md`
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes
- **AND** the log does not include `archive commit is not last`

### Requirement: Each archived change

When several `OpenSpec-Change:` lines name ids the diff removes, the head commit SHALL meet archive-last for each of those ids.

#### Scenario: Two archived ids pass

- **GIVEN** body lines `OpenSpec-Change: add-foo` and `OpenSpec-Change: add-bar`
- **AND** the diff removes both `openspec/changes/add-foo/` and `openspec/changes/add-bar/`
- **AND** the head message contains `openspec archive add-foo` and `openspec archive add-bar`
- **AND** the head diff only removes those two directories and writes under `openspec/changes/archive/` and `openspec/specs/`
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes

#### Scenario: One of two archive phrases is missing

- **GIVEN** body lines `OpenSpec-Change: add-foo` and `OpenSpec-Change: add-bar`
- **AND** the diff removes both directories
- **AND** the head message contains only `openspec archive add-foo --yes`
- **AND** the head paths are otherwise allowed
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `archive commit is not last`

#### Scenario: A duplicate reference counts once

- **GIVEN** the body contains `OpenSpec-Change: add-foo` twice
- **AND** the diff removes `openspec/changes/add-foo/`
- **AND** the head message is `openspec archive add-foo --yes`
- **AND** the head paths are only the allowed archive paths
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes

### Requirement: Archive does not apply without a removal

Archive-last SHALL NOT fail a named id that is still present, and SHALL NOT fail a removed directory that no change-reference line names.

#### Scenario: Edited re-reads the body

- **GIVEN** a pull_request `edited` event
- **AND** the diff removes `openspec/changes/add-foo/`
- **AND** the edited body adds `OpenSpec-Change: add-foo`
- **AND** the head message does not contain `openspec archive add-foo`
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `archive commit is not last`

### Requirement: The bullet lands before the archive commit

An implementation pull request that also archives SHALL keep `CHANGELOG.md` off the archive commit. The added Unreleased bullet SHALL already be in an earlier commit.

#### Scenario: The bullet is added before the archive commit

- **GIVEN** a pull_request whose diff changes `crates/rules/src/lib.rs` and adds an Unreleased bullet
- **AND** the body line is `OpenSpec-Change: add-foo`
- **AND** the diff removes `openspec/changes/add-foo/`
- **AND** the head commit only archives that id
- **AND** the bullet was added by the parent commit
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes

#### Scenario: A bullet inside the archive commit is not last

- **GIVEN** a pull_request body line `OpenSpec-Change: add-foo`
- **AND** the diff removes `openspec/changes/add-foo/`
- **AND** the head message contains `openspec archive add-foo`
- **AND** the head diff also edits `CHANGELOG.md`
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `archive commit is not last`

### Requirement: No head SHA in the body

The job SHALL NOT fail because the pull request body has no commit SHA. The pull request head the workflow checks out is the record.

#### Scenario: A body without a head SHA passes

- **GIVEN** a pull_request whose diff changes `crates/rules/src/lib.rs`
- **AND** the head adds an Unreleased bullet
- **AND** the body has no commit SHA
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes

### Requirement: Every failed check is reported

The job SHALL run the bullet check and the archive-last check. When both fail, the log SHALL include each failure line.

#### Scenario: Both checks fail together

- **GIVEN** a pull_request whose diff changes `crates/rules/src/lib.rs` and removes `openspec/changes/add-foo/`
- **AND** the body line is `OpenSpec-Change: add-foo`
- **AND** no Unreleased bullet is added
- **AND** the head commit also changes `crates/rules/src/lib.rs`
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`
- **AND** the log includes `archive commit is not last`

### Requirement: Fixture tests

The repository SHALL contain a pass fixture and a fail fixture for the bullet rule, for archive-last, and for each bullet exemption. The job SHALL run those fixtures with `node --test` and SHALL fail when one fails.

#### Scenario: Each case is a fixture repository

- **GIVEN** the script test suite
- **WHEN** a table case runs
- **THEN** the test builds a temporary git repository for that case
- **AND** asserts the script's exit status and failure lines
