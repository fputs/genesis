# pr-hygiene

## MODIFIED Requirements

### Requirement: Cargo and spec-drift stay

The `pr-hygiene` job SHALL be the only job in `.github/workflows/pr-hygiene.yml`. `.github/workflows/ci.yml` SHALL keep the Cargo job and the `spec-drift` job, their steps, and their current triggers, with no `edited` type and no skip `if`. A separate workflow SHALL run only spec-drift on pull_request `edited`. `pr-hygiene` SHALL NOT be a job in `ci.yml`.

#### Scenario: Cargo and spec-drift stay

- **GIVEN** `.github/workflows/ci.yml` after this change is applied
- **WHEN** the Cargo job and the `spec-drift` job are read
- **THEN** their steps and triggers are unchanged
- **AND** neither job gains an `edited` type or a skip `if`
- **AND** a separate workflow runs only spec-drift on pull_request `edited`
- **AND** `pr-hygiene` is not a job in that file

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

## ADDED Requirements

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

Archive-last SHALL NOT read the pull request body. Extra words, a fenced `OpenSpec-Change` line, or a dropped line SHALL NOT exempt a removed change and SHALL NOT be required for the check.

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
- **AND** the body has no `OpenSpec-Change: add-foo` line
- **AND** the head has no `openspec/changes/archive/2026-10-21-add-foo/`
- **AND** the head commit also changes `README.md`
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `archive commit is not last`

## REMOVED Requirements

### Requirement: Change reference line

**Reason**: "A fenced line still counts" is the old wording.
**Migration**: Replaced by "Archive-last ignores the body".

### Requirement: Archive commit is last

**Reason**: "Edited re-reads the body" contradicts a check that does not read the body.
**Migration**: Replaced by "Head commit performs the archive".

### Requirement: Archive does not apply without a removal

**Reason**: "A removal without a reference skips archive-last" now fails.
**Migration**: Replaced by "Removal without an archive directory fails".
