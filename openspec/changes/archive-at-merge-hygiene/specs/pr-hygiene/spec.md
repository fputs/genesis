# pr-hygiene

## ADDED Requirements

### Requirement: Draft skips archive-last

When `pull_request.draft` is true, the job SHALL NOT apply archive-last and SHALL NOT log `archive commit is not last`. The bullet check SHALL still run.

#### Scenario: A draft unarchived implementation pull request passes

- **GIVEN** a draft pull_request whose diff changes `crates/rules/src/lib.rs`
- **AND** the merge-base and the head both have `openspec/changes/add-foo/`
- **AND** the head adds an Unreleased bullet absent from the base
- **AND** the head section has more bullets than the base section
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes
- **AND** the log does not include `archive commit is not last`

#### Scenario: Converting back to draft relaxes archive-last

- **GIVEN** a non-draft pull_request that fails archive-last because `openspec/changes/add-foo/` is still on the head
- **AND** the diff changes `crates/rules/src/lib.rs` and adds an Unreleased bullet
- **WHEN** the pull request is `converted_to_draft` and `pull_request.draft` is true
- **THEN** the job passes
- **AND** the log does not include `archive commit is not last`

### Requirement: Ready pull request enforces archive-last

When `pull_request.draft` is not true, including when the field is absent, the job SHALL apply archive-last. A non-exempt implementation pull request is an implementation pull request that is not bullet-exempt. That pull request SHALL fail when the merge-base has an active change id and the head removed none. The log SHALL include `archive commit is not last`. A removed id SHALL satisfy the archive-last requirements already in this capability.

#### Scenario: Archive then mark ready passes

- **GIVEN** a draft implementation pull request whose head subject is `openspec archive add-foo --yes`
- **AND** that head deletes `openspec/changes/add-foo/` and adds `openspec/changes/archive/2026-10-21-add-foo/`
- **AND** an earlier commit changed `crates/rules/src/lib.rs` and added an Unreleased bullet absent from the base
- **WHEN** `gh pr ready` sends `ready_for_review` and `pull_request.draft` is false
- **THEN** the job passes
- **AND** the log does not include `archive commit is not last`

#### Scenario: A non-draft unarchived implementation pull request fails

- **GIVEN** a pull_request whose `pull_request.draft` is false
- **AND** the diff changes `crates/rules/src/lib.rs`
- **AND** the merge-base and the head both have `openspec/changes/add-foo/`
- **AND** the head adds an Unreleased bullet absent from the base
- **AND** the head section has more bullets than the base section
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `archive commit is not last`

#### Scenario: A push after ready re-runs the full check

- **GIVEN** a non-draft pull request whose archive commit was last and whose job passed
- **WHEN** a `synchronize` push adds a head commit `fix typo` that changes `crates/rules/src/lib.rs` and adds no Unreleased bullet
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`
- **AND** the log includes `archive commit is not last`

## MODIFIED Requirements

### Requirement: PR-only hygiene job

The repository SHALL run a job named `pr-hygiene` that invokes `scripts/pr-hygiene.mjs` on pull_request types `opened`, `synchronize`, `reopened`, `edited`, `ready_for_review`, and `converted_to_draft` targeting `main`. That job SHALL NOT run on push. Its permissions SHALL be `contents: read` only.

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

#### Scenario: Ready and draft conversion re-run the job

- **GIVEN** `.github/workflows/pr-hygiene.yml`
- **WHEN** its triggers are read
- **THEN** the `pr-hygiene` job also runs on pull_request `ready_for_review` and `converted_to_draft` targeting `main`

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

### Requirement: Removal without an archive directory fails

Archive-last SHALL NOT fail while `openspec/changes/<id>/` remains on the head, except a non-draft non-exempt implementation pull request that removed no active id. `<id>` SHALL be a direct child other than `archive`. The check SHALL fail when that directory is removed and the head has no `openspec/changes/archive/<YYYY>-<MM>-<DD>-<id>/`, unless the pull request is a draft. A missing `OpenSpec-Change` line SHALL NOT exempt the removal. The log SHALL include `archive commit is not last`.

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
