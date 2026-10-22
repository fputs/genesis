# pr-hygiene Specification

## Purpose
The pr-hygiene capability fails a pull request when an implementation change adds no Unreleased changelog bullet, or when a named OpenSpec change is removed by a commit that is not the archive.

## Requirements

### Requirement: Cargo and spec-drift stay

The `pr-hygiene` job SHALL be the only job in `.github/workflows/pr-hygiene.yml`. `.github/workflows/ci.yml` SHALL keep the Cargo job and the `spec-drift` job, their steps, and their current triggers, with no `edited` type and no skip `if`. `.github/workflows/spec-drift-edited.yml` SHALL run only a job named `spec-drift` on pull_request `edited`. `pr-hygiene` SHALL NOT be a job in `ci.yml`.

#### Scenario: Cargo and spec-drift stay

- **GIVEN** `.github/workflows/ci.yml` after this change is applied
- **WHEN** the Cargo job and the `spec-drift` job are read
- **THEN** their steps and triggers are unchanged
- **AND** neither job gains an `edited` type or a skip `if`
- **AND** `.github/workflows/spec-drift-edited.yml` runs only a job named `spec-drift` on pull_request `edited`
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

### Requirement: Cancel the previous run

The `pr-hygiene` workflow SHALL set concurrency group `pr-hygiene-<pull request number>` and SHALL cancel an in-progress run in that group.

#### Scenario: An in-progress run is cancelled

- **GIVEN** `.github/workflows/pr-hygiene.yml`
- **WHEN** its concurrency block is read
- **THEN** the group is `pr-hygiene-` plus the pull request number
- **AND** `cancel-in-progress` is true

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

An implementation pull request SHALL be a diff that includes a spec-drift code path: exactly `Cargo.toml`, `Cargo.lock`, or `rust-toolchain.toml`, or a path starting with `crates/` or `.github/workflows/`. A path under `scripts/` SHALL NOT be a spec-drift code path and SHALL NOT by itself be an implementation pull request. A scripts-only diff SHALL still fail the bullet check when no new Unreleased bullet was added.

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
- **THEN** the diff is not an implementation pull request
- **AND** it is not a spec-drift code path
- **AND** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

#### Scenario: A nested Cargo.toml is not a root manifest

- **GIVEN** a pull_request whose only diff is `docs/Cargo.toml`
- **AND** `CHANGELOG.md` gains no Unreleased bullet
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes

### Requirement: Unreleased changelog bullet

An implementation pull request that is not exempt, and a pull request whose diff includes any path under `scripts/`, SHALL add at least one new bullet under `## [Unreleased]` in `CHANGELOG.md`, using the Changelog bullet shape rule. Otherwise the job SHALL fail and the log SHALL include `implementation PR missing CHANGELOG bullet`. A missing `CHANGELOG.md`, or a missing `## [Unreleased]` heading, on the head SHALL fail that pull request the same way.

#### Scenario: An added Unreleased bullet passes

- **GIVEN** a pull_request whose diff changes `crates/rules/src/lib.rs`
- **AND** the head `## [Unreleased]` section has a bullet whose text appears nowhere in the base `CHANGELOG.md`
- **AND** that section has more bullets than the base section
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
- **AND** the base `CHANGELOG.md` has no bullet text `Record the gate`
- **AND** the new line under `## [Unreleased]` is `- Record the gate` with no pull request link
- **AND** the head section has more bullets than the base section
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes

#### Scenario: A missing head changelog fails

- **GIVEN** an implementation pull request whose head has no `CHANGELOG.md`, or whose `CHANGELOG.md` has no `## [Unreleased]` heading
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

#### Scenario: A docs-only pull request with no changelog passes

- **GIVEN** a pull_request whose only diff is `docs/architect/decisions.md`
- **AND** the head has no `CHANGELOG.md`
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes

#### Scenario: Edited still checks the bullet

- **GIVEN** a pull_request `edited` event
- **AND** the diff changes `crates/rules/src/lib.rs`
- **AND** no Unreleased bullet is added
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

#### Scenario: A scripts-only pull request needs a changelog bullet

- **GIVEN** a pull_request whose only diff is `scripts/note.mjs`
- **AND** no Unreleased bullet is added
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

### Requirement: Changelog bullet shape

A bullet SHALL be an unfenced trimmed markdown list item with text in the first unfenced `## [Unreleased]` section, before the next unfenced `## ` heading. Its text is that line without the list marker, trimmed. It SHALL count as new only when that text is not a bullet anywhere in the base `CHANGELOG.md` and the head section has more bullets than the base. Edited or moved bullets SHALL NOT count. A missing base file or heading SHALL leave the base set empty.

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

#### Scenario: A moved bullet is not new

- **GIVEN** a pull_request whose diff changes `crates/rules/src/lib.rs`
- **AND** the base `CHANGELOG.md` has the bullet text `Record the gate` under `## [0.1.0]`
- **AND** the head places that same text under `## [Unreleased]`
- **AND** the head adds no bullet text absent from the base file
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

#### Scenario: An edited bullet is not new

- **GIVEN** a pull_request whose diff changes `Cargo.toml`
- **AND** the base `## [Unreleased]` section's only bullet is `- Record the gate`
- **AND** the head changes that bullet to `- Record the gate today`
- **AND** the head section has the same number of bullets as the base section
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

### Requirement: Bullet exemptions

The bullet check SHALL pass with no new bullet when every changed path is exempt. Exempt paths are those under `docs/`, `openspec/`, or `.cursor/`, plus exactly `AGENTS.md`, `CHANGELOG.md`, `README.md`, `Cargo.lock`, and `rust-toolchain.toml`. Categories SHALL combine. Any other path SHALL remove the exemption.

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

#### Scenario: Exempt categories together pass without a bullet

- **GIVEN** a pull_request whose diff is `docs/architect/decisions.md`, `openspec/specs/changelog/spec.md`, `.cursor/rules/example.md`, `README.md`, `Cargo.lock`, and `rust-toolchain.toml`
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

### Requirement: Each archived change

Each removed id SHALL be checked once. The trimmed subject SHALL be every exact command `openspec archive <id> --yes` joined by ` && ` in ascending id order. A command only on a later line SHALL NOT count. Otherwise the log SHALL include `archive commit is not last`.

#### Scenario: Two archived ids pass

- **GIVEN** the diff removes `openspec/changes/add-foo/` and `openspec/changes/add-bar/`
- **AND** the head commit deletes both directories and adds `openspec/changes/archive/2026-10-21-add-foo/` and `openspec/changes/archive/2026-10-21-add-bar/`
- **AND** the subject is `openspec archive add-bar --yes && openspec archive add-foo --yes`
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes

#### Scenario: Two archived ids fail

- **GIVEN** the diff removes `openspec/changes/add-foo/` and `openspec/changes/add-bar/`
- **AND** the head commit adds `openspec/changes/archive/2026-10-21-add-foo/` and `openspec/changes/archive/2026-10-21-add-bar/`
- **AND** the subject is `openspec archive add-foo --yes`
- **AND** a later line is `openspec archive add-bar --yes`
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `archive commit is not last`

#### Scenario: One of two archive phrases is missing

- **GIVEN** the diff removes `openspec/changes/add-foo/` and `openspec/changes/add-bar/`
- **AND** the subject is `openspec archive add-foo --yes`
- **AND** the message has no `openspec archive add-bar --yes`
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `archive commit is not last`

#### Scenario: A duplicate reference counts once

- **GIVEN** the body contains `OpenSpec-Change: add-foo` twice
- **AND** the head commit deletes `openspec/changes/add-foo/` and adds `openspec/changes/archive/2026-10-21-add-foo/`
- **AND** the subject is `openspec archive add-foo --yes`
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes

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

The job SHALL run the bullet check on every pull request run. It SHALL run archive-last only when `pull_request.draft` is not true. When both fail, the log SHALL include each failure line.

#### Scenario: Both checks fail together

- **GIVEN** a pull_request whose diff changes `crates/rules/src/lib.rs` and removes `openspec/changes/add-foo/`
- **AND** `pull_request.draft` is not true
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

### Requirement: Fenced changelog blocks

Fenced lines SHALL be ignored when finding `## [Unreleased]` and when collecting bullets in either changelog. A trimmed line of three or more backticks SHALL toggle the fence. A fenced heading SHALL NOT open or close the section. A fenced list item SHALL NOT be a bullet.

#### Scenario: A fenced bullet is not new

- **GIVEN** a pull_request whose diff changes `crates/rules/src/lib.rs`
- **AND** the only list item added under `## [Unreleased]` sits inside a fenced code block
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

#### Scenario: A fenced heading does not open the section

- **GIVEN** an implementation pull request
- **AND** the only `## [Unreleased]` line sits inside a fenced code block
- **AND** no unfenced `## [Unreleased]` heading exists
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

#### Scenario: A fenced heading does not close the section

- **GIVEN** an implementation pull request
- **AND** a fenced `## [0.1.0]` sits inside the unfenced `## [Unreleased]` section
- **AND** after that fence the section has a new bullet whose text is absent from the base changelog
- **AND** the head section has more unfenced bullets than the base section
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes

### Requirement: Git locale is C

Every `git` subprocess started by `scripts/pr-hygiene.mjs` SHALL have `LC_ALL=C`. Missing-path checks SHALL match the C-locale phrases `does not exist` and `exists on disk, but not in` even when the parent locale is not C.

#### Scenario: A missing path uses C-locale git text

- **GIVEN** the parent process has `LC_ALL=fr_FR.UTF-8`
- **AND** a stand-in `git` prints a non-English missing-path line unless `LC_ALL=C`
- **AND** the head of an implementation pull request has no `CHANGELOG.md`
- **WHEN** `scripts/pr-hygiene.mjs` asks git for that file
- **THEN** the git child is started with `LC_ALL=C`
- **AND** the missing file is treated as missing
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

### Requirement: Archive-last ignores the body

Archive-last SHALL read the pull request body only to find the pull request's own change id for the non-draft failure. Extra words, a fenced `OpenSpec-Change` line, or a dropped line SHALL NOT exempt a removed change and SHALL NOT be required for the check.

#### Scenario: Extra words are not a change reference

- **GIVEN** a pull_request whose diff removes `openspec/changes/add-foo/`
- **AND** the only body line is `OpenSpec-Change: add-foo extra`
- **AND** the head commit also changes `README.md`
- **AND** the head has no `openspec/changes/archive/2026-10-21-add-foo/`
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `archive commit is not last`

#### Scenario: A fenced line does not exempt a removal

- **GIVEN** a pull_request whose diff removes `openspec/changes/add-foo/`
- **AND** the body contains a fenced `OpenSpec-Change: add-foo` line
- **AND** the head commit message does not contain `openspec archive add-foo`
- **AND** the head has no `openspec/changes/archive/2026-10-21-add-foo/`
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `archive commit is not last`

### Requirement: Head commit performs the archive

When the merge-base has `openspec/changes/<id>/` and the head does not, the head commit SHALL delete that directory and SHALL add `openspec/changes/archive/<YYYY>-<MM>-<DD>-<id>/` unless that directory is already on the merge-base. The dated remainder SHALL equal `<id>`. The trimmed subject SHALL be the exact command `openspec archive <id> --yes` for each removed id, joined by ` && `. Any other path SHALL be under `openspec/changes/archive/`, `openspec/specs/`, or the removed directory.

#### Scenario: The archive commit passes

- **GIVEN** the merge-base has `openspec/changes/add-foo/`
- **AND** the body has no `OpenSpec-Change` line
- **AND** the head commit deletes that directory, adds `openspec/changes/archive/2026-10-21-add-foo/`, and edits `openspec/specs/changelog/spec.md`
- **AND** the subject is `openspec archive add-foo --yes`
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes

#### Scenario: An earlier archive add is not last

- **GIVEN** the merge-base has `openspec/changes/add-foo/` and no archive directory for it
- **AND** the parent commit adds `openspec/changes/archive/2026-10-21-add-foo/` and leaves the active directory
- **AND** the head commit only deletes `openspec/changes/add-foo/`
- **AND** the subject is `openspec archive add-foo --yes`
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `archive commit is not last`

#### Scenario: An empty head commit fails

- **GIVEN** the merge-base has `openspec/changes/add-foo/`
- **AND** the parent of the head already moved that directory to `openspec/changes/archive/2026-10-21-add-foo/`
- **AND** the head commit changes no path
- **AND** the subject is `openspec archive add-foo --yes`
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `archive commit is not last`

#### Scenario: A specs-only head commit fails

- **GIVEN** the merge-base has `openspec/changes/add-foo/`
- **AND** the parent of the head already removed that directory
- **AND** the head commit's only path is `openspec/specs/unrelated/spec.md`
- **AND** the subject is `openspec archive add-foo --yes`
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `archive commit is not last`

#### Scenario: A phrase below the subject does not count

- **GIVEN** the head commit deletes `openspec/changes/add-foo/` and adds `openspec/changes/archive/2026-10-21-add-foo/`
- **AND** the subject is `Record the archive`
- **AND** the next line is `openspec archive add-foo --yes`
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `archive commit is not last`

#### Scenario: A path outside archive and specs fails

- **GIVEN** the head commit deletes `openspec/changes/add-foo/`, adds `openspec/changes/archive/2026-10-21-add-foo/`, and changes `docs/producer/process.md`
- **AND** the subject is `openspec archive add-foo --yes`
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `archive commit is not last`

#### Scenario: A later fix-typo commit is not last

- **GIVEN** a pull_request whose diff removes `openspec/changes/add-foo/`
- **AND** the commit before the head is `openspec archive add-foo --yes`
- **AND** the head commit message is `fix typo`
- **AND** the head commit changes `docs/producer/process.md`
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `archive commit is not last`

#### Scenario: A missing archive phrase fails

- **GIVEN** the head commit deletes `openspec/changes/add-foo/` and adds `openspec/changes/archive/2026-10-21-add-foo/`
- **AND** the subject is `archive the change`
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `archive commit is not last`

#### Scenario: A longer id does not cover a shorter id

- **GIVEN** the head commit deletes `openspec/changes/add-foo/` and adds `openspec/changes/archive/2026-10-21-add-foo/`
- **AND** the subject is `openspec archive add-foo-extra --yes`
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `archive commit is not last`

#### Scenario: A body edit does not excuse a bad archive

- **GIVEN** a pull_request `edited` event
- **AND** the diff removes `openspec/changes/add-foo/`
- **AND** the edited body adds `OpenSpec-Change: add-foo`
- **AND** the head subject is `fix typo`
- **AND** the head commit does not add `openspec/changes/archive/2026-10-21-add-foo/`
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `archive commit is not last`

#### Scenario: An edited body drops the change reference

- **GIVEN** a pull_request `edited` event
- **AND** the diff removes `openspec/changes/add-foo/`
- **AND** the edited body has no `OpenSpec-Change: add-foo` line
- **AND** the head subject is `fix typo`
- **AND** the head commit also changes `README.md`
- **AND** the head has no archive directory for `add-foo`
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `archive commit is not last`

#### Scenario: A pre-existing archive allows the removal commit

- **GIVEN** the merge-base has `openspec/changes/add-foo/` and `openspec/changes/archive/2026-10-21-add-foo/`
- **AND** the head commit only deletes `openspec/changes/add-foo/`
- **AND** the subject is `openspec archive add-foo --yes`
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes

### Requirement: Removal without an archive directory fails

Archive-last SHALL NOT fail while `openspec/changes/<id>/` remains on the head. `<id>` SHALL be a direct child other than `archive`. The check SHALL fail when that directory is removed and the head has no `openspec/changes/archive/<YYYY>-<MM>-<DD>-<id>/`. A missing `OpenSpec-Change` line SHALL NOT exempt the removal. The log SHALL include `archive commit is not last`.

#### Scenario: A named change that stays active skips archive-last

- **GIVEN** a pull_request body line `OpenSpec-Change: add-foo`
- **AND** the head still has `openspec/changes/add-foo/`
- **AND** the head commit edits `openspec/changes/add-foo/proposal.md`
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes
- **AND** the log does not include `archive commit is not last`

#### Scenario: A removal without a reference fails

- **GIVEN** a pull_request whose diff removes `openspec/changes/add-foo/`
- **AND** `pull_request.draft` is not true
- **AND** the body has no `OpenSpec-Change: add-foo` line
- **AND** the head has no `openspec/changes/archive/2026-10-21-add-foo/`
- **AND** the head commit also changes `README.md`
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `archive commit is not last`

### Requirement: Draft skips archive-last

When `pull_request.draft` is true, the job SHALL NOT apply archive-last and SHALL NOT log `archive commit is not last`. The bullet check SHALL still run.

#### Scenario: A draft unarchived implementation pull request passes

- **GIVEN** a draft pull_request whose diff changes `crates/rules/src/lib.rs`
- **AND** the body line is `OpenSpec-Change: add-foo`
- **AND** the merge-base and the head both have `openspec/changes/add-foo/`
- **AND** the head adds an Unreleased bullet absent from the base
- **AND** the head section has more bullets than the base section
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes
- **AND** the log does not include `archive commit is not last`

#### Scenario: Converting back to draft relaxes archive-last

- **GIVEN** a non-draft pull_request whose body line is `OpenSpec-Change: add-foo`
- **AND** `openspec/changes/add-foo/` is still on the head
- **AND** the diff changes `crates/rules/src/lib.rs` and adds an Unreleased bullet
- **WHEN** the pull request is `converted_to_draft` and `pull_request.draft` is true
- **THEN** the job passes
- **AND** the log does not include `archive commit is not last`

### Requirement: Ready pull request enforces archive-last

When `pull_request.draft` is not true, including when absent, the job SHALL apply archive-last. Own-change lines SHALL follow spec-drift's "Change reference line" on unfenced lines only: exact `OpenSpec-Change: <id>`, every matching line counts. A non-exempt implementation pull request SHALL fail when such an id is still active on the head. The log SHALL include `archive commit is not last`. No such line SHALL NOT fail because another change is active. A removed id SHALL satisfy archive-last.

#### Scenario: Archive then mark ready passes

- **GIVEN** a draft implementation pull request whose head subject is `openspec archive add-foo --yes`
- **AND** that head deletes `openspec/changes/add-foo/` and adds `openspec/changes/archive/2026-10-21-add-foo/`
- **AND** an earlier commit changed `crates/rules/src/lib.rs` and added an Unreleased bullet absent from the base
- **WHEN** `gh pr ready` sends `ready_for_review` and `pull_request.draft` is false
- **THEN** the job passes
- **AND** the log does not include `archive commit is not last`

#### Scenario: A non-draft unarchived implementation pull request fails

- **GIVEN** a pull_request whose `pull_request.draft` is false
- **AND** the body line is `OpenSpec-Change: add-foo`
- **AND** the diff changes `crates/rules/src/lib.rs`
- **AND** the merge-base and the head both have `openspec/changes/add-foo/`
- **AND** the head adds an Unreleased bullet absent from the base
- **AND** the head section has more bullets than the base section
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `archive commit is not last`

#### Scenario: A non-draft implementation pull request with no change of its own passes

- **GIVEN** a pull_request whose `pull_request.draft` is false
- **AND** the body has no `OpenSpec-Change:` line
- **AND** the diff changes `crates/rules/src/lib.rs`
- **AND** the merge-base and the head both have `openspec/changes/add-foo/`
- **AND** the head adds an Unreleased bullet absent from the base
- **AND** the head section has more bullets than the base section
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes
- **AND** the log does not include `archive commit is not last`

#### Scenario: A fenced or extra-word line is not its own change

- **GIVEN** a pull_request whose `pull_request.draft` is false
- **AND** its only `OpenSpec-Change:` line is fenced, or is `OpenSpec-Change: add-foo extra`
- **AND** the diff changes `crates/rules/src/lib.rs`
- **AND** the merge-base and the head both have `openspec/changes/add-foo/`
- **AND** the head adds an Unreleased bullet absent from the base
- **AND** the head section has more bullets than the base section
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes
- **AND** the log does not include `archive commit is not last`

#### Scenario: A push after ready re-runs the full check

- **GIVEN** a non-draft pull request whose archive commit was last and whose job passed
- **WHEN** a `synchronize` push adds a head commit `fix typo` that changes `crates/rules/src/lib.rs` and adds no Unreleased bullet
- **THEN** the job fails
- **AND** the log includes `archive commit is not last`

### Requirement: Pull-request hygiene job

The repository SHALL run a job named `pr-hygiene` that invokes `scripts/pr-hygiene.mjs` on pull_request types `opened`, `synchronize`, `reopened`, `edited`, `ready_for_review`, and `converted_to_draft` targeting `main`. That job SHALL NOT run on push. Its permissions SHALL be `contents: read` only.

#### Scenario: Job runs on its pull request types

- **GIVEN** `.github/workflows/pr-hygiene.yml`
- **WHEN** its triggers are read
- **THEN** the `pr-hygiene` job runs on pull_request `opened`, `synchronize`, `reopened`, `edited`, `ready_for_review`, and `converted_to_draft` targeting `main`
- **AND** it invokes `scripts/pr-hygiene.mjs`

#### Scenario: The job does not run on push

- **GIVEN** `.github/workflows/pr-hygiene.yml`
- **WHEN** its triggers are read
- **THEN** the workflow has no push trigger

#### Scenario: Permissions are contents read only

- **GIVEN** `.github/workflows/pr-hygiene.yml`
- **WHEN** its permissions are read
- **THEN** they are `contents: read` only
- **AND** they do not grant `pull-requests: read`
