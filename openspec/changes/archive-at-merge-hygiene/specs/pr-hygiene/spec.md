# pr-hygiene

## ADDED Requirements

### Requirement: Review skips archive-last

On pull_request, the job SHALL NOT apply archive-last and SHALL NOT log `archive commit is not last`. An unarchived implementation pull request SHALL pass that check. A commit after an archive commit SHALL NOT fail it.

#### Scenario: An unarchived implementation pull request passes review

- **GIVEN** a pull_request whose diff changes `crates/rules/src/lib.rs`
- **AND** the head adds an Unreleased bullet absent from the base
- **AND** the head section has more bullets than the base section
- **AND** the merge-base and the head both have `openspec/changes/add-foo/`
- **WHEN** the pull_request pr-hygiene run executes
- **THEN** the job passes
- **AND** the log does not include `archive commit is not last`

#### Scenario: A fix after archive passes review

- **GIVEN** a pull_request whose diff removes `openspec/changes/add-foo/`
- **AND** the commit before the head is `openspec archive add-foo --yes`
- **AND** the head commit message is `fix typo`
- **AND** the head commit changes `docs/producer/process.md`
- **WHEN** the pull_request pr-hygiene run executes
- **THEN** the job passes
- **AND** the log does not include `archive commit is not last`

### Requirement: Non-exempt archive at merge

A non-exempt implementation pull request is an implementation pull request that is not bullet-exempt. The merge-time run SHALL fail that pull request when no active change id was removed. The log SHALL include `archive commit is not last`. A bullet-exempt pull request, and a pull request that is not an implementation pull request, SHALL pass that run when no active id was removed.

#### Scenario: An unarchived implementation pull request fails at merge

- **GIVEN** a merge group whose second parent changes `crates/rules/src/lib.rs`
- **AND** that parent adds an Unreleased bullet absent from the base
- **AND** that parent's section has more bullets than the base section
- **AND** the merge-base and that parent both have `openspec/changes/add-foo/`
- **WHEN** the merge-time pr-hygiene run executes
- **THEN** the job fails
- **AND** the log includes `archive commit is not last`

#### Scenario: A lockfile-only pull request passes at merge

- **GIVEN** a merge group whose second parent's only diff is `Cargo.lock`
- **AND** no Unreleased bullet is added
- **AND** no active change id was removed
- **WHEN** the merge-time pr-hygiene run executes
- **THEN** the job passes
- **AND** the log does not include `archive commit is not last`

#### Scenario: A scripts-only pull request passes at merge without an archive

- **GIVEN** a merge group whose second parent's only diff is `scripts/note.mjs`
- **AND** that parent adds an Unreleased bullet absent from the base
- **AND** that parent's section has more bullets than the base section
- **AND** no active change id was removed
- **WHEN** the merge-time pr-hygiene run executes
- **THEN** the job passes
- **AND** the log does not include `archive commit is not last`

## MODIFIED Requirements

### Requirement: PR-only hygiene job

The repository SHALL run a job named `pr-hygiene` that invokes `scripts/pr-hygiene.mjs` on pull_request types `opened`, `synchronize`, `reopened`, and `edited` targeting `main`, and on `merge_group`. The `merge_group` run is the merge-time run. That job SHALL NOT run on push. Its permissions SHALL be `contents: read` only.

#### Scenario: Job runs on the four pull request types

- **GIVEN** `.github/workflows/pr-hygiene.yml`
- **WHEN** its triggers are read
- **THEN** the `pr-hygiene` job runs on pull_request `opened`, `synchronize`, `reopened`, and `edited` targeting `main`
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

#### Scenario: The job runs on the merge group

- **GIVEN** `.github/workflows/pr-hygiene.yml`
- **WHEN** its triggers are read
- **THEN** the workflow runs on `merge_group`
- **AND** it has no push trigger

### Requirement: Cargo and spec-drift stay

The `pr-hygiene` job SHALL be the only job in `.github/workflows/pr-hygiene.yml`. `.github/workflows/ci.yml` SHALL keep the Cargo job and the `spec-drift` job and their steps. Its triggers SHALL add `merge_group` to the existing push and pull_request triggers, with no `edited` type and no skip `if`. `.github/workflows/spec-drift-edited.yml` SHALL run only a job named `spec-drift` on pull_request `edited`. `pr-hygiene` SHALL NOT be a job in `ci.yml`.

#### Scenario: Cargo and spec-drift stay

- **GIVEN** `.github/workflows/ci.yml` after this change is applied
- **WHEN** the Cargo job and the `spec-drift` job are read
- **THEN** their steps stay, and `ci.yml` adds `merge_group` to its push and pull_request triggers
- **AND** neither job gains an `edited` type or a skip `if`
- **AND** `.github/workflows/spec-drift-edited.yml` runs only a job named `spec-drift` on pull_request `edited`
- **AND** `pr-hygiene` is not a job in that file

### Requirement: Fail closed on parse errors

When the event payload, a diff, a NUL record, the base, the head, the head's first parent, or the merge-base cannot be parsed, the job SHALL exit non-zero and SHALL NOT treat the pull request as exempt. A merge group head without exactly two parents SHALL fail closed.

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

#### Scenario: A one-parent merge group fails closed

- **GIVEN** a merge group head with only one parent
- **WHEN** the merge-time pr-hygiene run executes
- **THEN** the job fails

### Requirement: Pull request head from the event

Every diff SHALL use `git merge-base` of the base sha and the pull request head, for classification, the bullet diff, and the base `## [Unreleased]` section. The job SHALL NOT diff the base sha directly. On pull_request both shas are the event shas. On merge_group the base is `merge_group.base_sha` and the pull request head is the second parent of the group head. Checks SHALL use that head and its first parent, not the group head or a runner merge. No merge-base SHALL fail the job closed.

#### Scenario: The check uses the event head

- **GIVEN** a pull_request event whose head sha differs from a merge commit the runner created
- **WHEN** the `pr-hygiene` job runs
- **THEN** the bullet diff ends at that event head sha
- **AND** its other side is the merge-base of the event base sha and that head
- **AND** archive-last is not applied to the runner merge

#### Scenario: main advanced after branching

- **GIVEN** main gains a crate change and an Unreleased bullet after the branch point
- **AND** the pull request changes only `docs/architect/decisions.md` since that merge-base
- **AND** the pull request adds no Unreleased bullet
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes

#### Scenario: The merge group uses the pull request head

- **GIVEN** a merge group head whose subject is `Merge pull request`
- **AND** its second parent deletes `openspec/changes/add-foo/` and adds `openspec/changes/archive/2026-10-21-add-foo/`
- **AND** that parent's subject is `openspec archive add-foo --yes`
- **AND** the diff from the merge-base to that parent changes `crates/rules/src/lib.rs`
- **AND** that parent adds an Unreleased bullet absent from the base
- **AND** that parent's section has more bullets than the base section
- **WHEN** the merge-time pr-hygiene run executes
- **THEN** the job passes
- **AND** the group head is not the archive commit

### Requirement: Each archived change

On the merge-time run, each removed id SHALL be checked once. The trimmed subject SHALL be every exact command `openspec archive <id> --yes` joined by ` && ` in ascending id order. A command only on a later line SHALL NOT count. Otherwise the log SHALL include `archive commit is not last`.

#### Scenario: Two archived ids pass

- **GIVEN** the diff removes `openspec/changes/add-foo/` and `openspec/changes/add-bar/`
- **AND** the head commit deletes both directories and adds `openspec/changes/archive/2026-10-21-add-foo/` and `openspec/changes/archive/2026-10-21-add-bar/`
- **AND** the subject is `openspec archive add-bar --yes && openspec archive add-foo --yes`
- **WHEN** the merge-time pr-hygiene run executes
- **THEN** the job passes

#### Scenario: Two archived ids fail

- **GIVEN** the diff removes `openspec/changes/add-foo/` and `openspec/changes/add-bar/`
- **AND** the head commit adds `openspec/changes/archive/2026-10-21-add-foo/` and `openspec/changes/archive/2026-10-21-add-bar/`
- **AND** the subject is `openspec archive add-foo --yes`
- **AND** a later line is `openspec archive add-bar --yes`
- **WHEN** the merge-time pr-hygiene run executes
- **THEN** the job fails
- **AND** the log includes `archive commit is not last`

#### Scenario: One of two archive phrases is missing

- **GIVEN** the diff removes `openspec/changes/add-foo/` and `openspec/changes/add-bar/`
- **AND** the subject is `openspec archive add-foo --yes`
- **AND** the message has no `openspec archive add-bar --yes`
- **WHEN** the merge-time pr-hygiene run executes
- **THEN** the job fails
- **AND** the log includes `archive commit is not last`

#### Scenario: A duplicate reference counts once

- **GIVEN** the body contains `OpenSpec-Change: add-foo` twice
- **AND** the head commit deletes `openspec/changes/add-foo/` and adds `openspec/changes/archive/2026-10-21-add-foo/`
- **AND** the subject is `openspec archive add-foo --yes`
- **WHEN** the merge-time pr-hygiene run executes
- **THEN** the job passes

### Requirement: The bullet lands before the archive commit

On the merge-time run, an implementation pull request that also archives SHALL keep `CHANGELOG.md` off the archive commit. The added Unreleased bullet SHALL already be in an earlier commit.

#### Scenario: The bullet is added before the archive commit

- **GIVEN** a pull_request whose diff changes `crates/rules/src/lib.rs` and adds an Unreleased bullet
- **AND** the body line is `OpenSpec-Change: add-foo`
- **AND** the diff removes `openspec/changes/add-foo/`
- **AND** the head commit only archives that id
- **AND** the bullet was added by the parent commit
- **WHEN** the merge-time pr-hygiene run executes
- **THEN** the job passes

#### Scenario: A bullet inside the archive commit is not last

- **GIVEN** a pull_request body line `OpenSpec-Change: add-foo`
- **AND** the diff removes `openspec/changes/add-foo/`
- **AND** the head message contains `openspec archive add-foo`
- **AND** the head diff also edits `CHANGELOG.md`
- **WHEN** the merge-time pr-hygiene run executes
- **THEN** the job fails
- **AND** the log includes `archive commit is not last`

### Requirement: Every failed check is reported

The pull_request run SHALL run the bullet check and SHALL NOT apply archive-last. The merge-time run SHALL run the bullet check and the archive-last check. When both fail, the log SHALL include each failure line.

#### Scenario: Both checks fail together

- **GIVEN** a pull_request whose diff changes `crates/rules/src/lib.rs` and removes `openspec/changes/add-foo/`
- **AND** the body line is `OpenSpec-Change: add-foo`
- **AND** no Unreleased bullet is added
- **AND** the head commit also changes `crates/rules/src/lib.rs`
- **WHEN** the merge-time pr-hygiene run executes
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`
- **AND** the log includes `archive commit is not last`

### Requirement: Archive-last ignores the body

On the merge-time run, archive-last SHALL NOT read the pull request body. Extra words, a fenced `OpenSpec-Change` line, or a dropped line SHALL NOT exempt a removed change and SHALL NOT be required for the check.

#### Scenario: Extra words are not a change reference

- **GIVEN** a pull_request whose diff removes `openspec/changes/add-foo/`
- **AND** the only body line is `OpenSpec-Change: add-foo extra`
- **AND** the head commit also changes `README.md`
- **AND** the head has no `openspec/changes/archive/2026-10-21-add-foo/`
- **WHEN** the merge-time pr-hygiene run executes
- **THEN** the job fails
- **AND** the log includes `archive commit is not last`

#### Scenario: A fenced line does not exempt a removal

- **GIVEN** a pull_request whose diff removes `openspec/changes/add-foo/`
- **AND** the body contains a fenced `OpenSpec-Change: add-foo` line
- **AND** the head commit message does not contain `openspec archive add-foo`
- **AND** the head has no `openspec/changes/archive/2026-10-21-add-foo/`
- **WHEN** the merge-time pr-hygiene run executes
- **THEN** the job fails
- **AND** the log includes `archive commit is not last`

### Requirement: Head commit performs the archive

When the merge-base has `openspec/changes/<id>/` and the head does not, the head commit SHALL delete that directory and SHALL add `openspec/changes/archive/<YYYY>-<MM>-<DD>-<id>/` unless that directory is already on the merge-base. The dated remainder SHALL equal `<id>`. The trimmed subject SHALL be the exact command `openspec archive <id> --yes` for each removed id, joined by ` && `. Any other path SHALL be under `openspec/changes/archive/`, `openspec/specs/`, or the removed directory.

#### Scenario: The archive commit passes

- **GIVEN** the merge-base has `openspec/changes/add-foo/`
- **AND** the body has no `OpenSpec-Change` line
- **AND** the head commit deletes that directory, adds `openspec/changes/archive/2026-10-21-add-foo/`, and edits `openspec/specs/changelog/spec.md`
- **AND** the subject is `openspec archive add-foo --yes`
- **WHEN** the merge-time pr-hygiene run executes
- **THEN** the job passes

#### Scenario: An earlier archive add is not last

- **GIVEN** the merge-base has `openspec/changes/add-foo/` and no archive directory for it
- **AND** the parent commit adds `openspec/changes/archive/2026-10-21-add-foo/` and leaves the active directory
- **AND** the head commit only deletes `openspec/changes/add-foo/`
- **AND** the subject is `openspec archive add-foo --yes`
- **WHEN** the merge-time pr-hygiene run executes
- **THEN** the job fails
- **AND** the log includes `archive commit is not last`

#### Scenario: An empty head commit fails

- **GIVEN** the merge-base has `openspec/changes/add-foo/`
- **AND** the parent of the head already moved that directory to `openspec/changes/archive/2026-10-21-add-foo/`
- **AND** the head commit changes no path
- **AND** the subject is `openspec archive add-foo --yes`
- **WHEN** the merge-time pr-hygiene run executes
- **THEN** the job fails
- **AND** the log includes `archive commit is not last`

#### Scenario: A specs-only head commit fails

- **GIVEN** the merge-base has `openspec/changes/add-foo/`
- **AND** the parent of the head already removed that directory
- **AND** the head commit's only path is `openspec/specs/unrelated/spec.md`
- **AND** the subject is `openspec archive add-foo --yes`
- **WHEN** the merge-time pr-hygiene run executes
- **THEN** the job fails
- **AND** the log includes `archive commit is not last`

#### Scenario: A phrase below the subject does not count

- **GIVEN** the head commit deletes `openspec/changes/add-foo/` and adds `openspec/changes/archive/2026-10-21-add-foo/`
- **AND** the subject is `Record the archive`
- **AND** the next line is `openspec archive add-foo --yes`
- **WHEN** the merge-time pr-hygiene run executes
- **THEN** the job fails
- **AND** the log includes `archive commit is not last`

#### Scenario: A path outside archive and specs fails

- **GIVEN** the head commit deletes `openspec/changes/add-foo/`, adds `openspec/changes/archive/2026-10-21-add-foo/`, and changes `docs/producer/process.md`
- **AND** the subject is `openspec archive add-foo --yes`
- **WHEN** the merge-time pr-hygiene run executes
- **THEN** the job fails
- **AND** the log includes `archive commit is not last`

#### Scenario: A later fix-typo commit is not last

- **GIVEN** a pull_request whose diff removes `openspec/changes/add-foo/`
- **AND** the commit before the head is `openspec archive add-foo --yes`
- **AND** the head commit message is `fix typo`
- **AND** the head commit changes `docs/producer/process.md`
- **WHEN** the merge-time pr-hygiene run executes
- **THEN** the job fails
- **AND** the log includes `archive commit is not last`

#### Scenario: A missing archive phrase fails

- **GIVEN** the head commit deletes `openspec/changes/add-foo/` and adds `openspec/changes/archive/2026-10-21-add-foo/`
- **AND** the subject is `archive the change`
- **WHEN** the merge-time pr-hygiene run executes
- **THEN** the job fails
- **AND** the log includes `archive commit is not last`

#### Scenario: A longer id does not cover a shorter id

- **GIVEN** the head commit deletes `openspec/changes/add-foo/` and adds `openspec/changes/archive/2026-10-21-add-foo/`
- **AND** the subject is `openspec archive add-foo-extra --yes`
- **WHEN** the merge-time pr-hygiene run executes
- **THEN** the job fails
- **AND** the log includes `archive commit is not last`

#### Scenario: A body edit does not excuse a bad archive

- **GIVEN** a pull_request `edited` event
- **AND** the diff removes `openspec/changes/add-foo/`
- **AND** the edited body adds `OpenSpec-Change: add-foo`
- **AND** the head subject is `fix typo`
- **AND** the head commit does not add `openspec/changes/archive/2026-10-21-add-foo/`
- **WHEN** the merge-time pr-hygiene run executes
- **THEN** the job fails
- **AND** the log includes `archive commit is not last`

#### Scenario: An edited body drops the change reference

- **GIVEN** a pull_request `edited` event
- **AND** the diff removes `openspec/changes/add-foo/`
- **AND** the edited body has no `OpenSpec-Change: add-foo` line
- **AND** the head subject is `fix typo`
- **AND** the head commit also changes `README.md`
- **AND** the head has no archive directory for `add-foo`
- **WHEN** the merge-time pr-hygiene run executes
- **THEN** the job fails
- **AND** the log includes `archive commit is not last`

#### Scenario: A pre-existing archive allows the removal commit

- **GIVEN** the merge-base has `openspec/changes/add-foo/` and `openspec/changes/archive/2026-10-21-add-foo/`
- **AND** the head commit only deletes `openspec/changes/add-foo/`
- **AND** the subject is `openspec archive add-foo --yes`
- **WHEN** the merge-time pr-hygiene run executes
- **THEN** the job passes

### Requirement: Removal without an archive directory fails

On the merge-time run, archive-last SHALL NOT fail while `openspec/changes/<id>/` remains on the head, except a non-exempt implementation pull request that removed no active id, which SHALL fail. `<id>` SHALL be a direct child other than `archive`. The check SHALL fail when that directory is removed and the head has no `openspec/changes/archive/<YYYY>-<MM>-<DD>-<id>/`. A missing `OpenSpec-Change` line SHALL NOT exempt the removal. The log SHALL include `archive commit is not last`.

#### Scenario: A named change that stays active skips archive-last

- **GIVEN** a pull_request body line `OpenSpec-Change: add-foo`
- **AND** the head still has `openspec/changes/add-foo/`
- **AND** the head commit edits `openspec/changes/add-foo/proposal.md`
- **WHEN** the merge-time pr-hygiene run executes
- **THEN** the job passes
- **AND** the log does not include `archive commit is not last`

#### Scenario: A removal without a reference fails

- **GIVEN** a pull_request whose diff removes `openspec/changes/add-foo/`
- **AND** the body has no `OpenSpec-Change: add-foo` line
- **AND** the head has no `openspec/changes/archive/2026-10-21-add-foo/`
- **AND** the head commit also changes `README.md`
- **WHEN** the merge-time pr-hygiene run executes
- **THEN** the job fails
- **AND** the log includes `archive commit is not last`

