# spec-drift-check

## MODIFIED Requirements

### Requirement: Spec-drift job

The repository SHALL run a `spec-drift` job in `.github/workflows/ci.yml` on the same push and pull_request events as the Cargo job. Those triggers SHALL stay unchanged, with no `edited` type. That job SHALL still invoke `scripts/spec-drift-check.mjs`. A separate workflow SHALL run only that script on pull_request `edited` targeting `main`. The script SHALL have no npm dependencies. A code-path pull request that drops every `OpenSpec-Change` line on `edited` SHALL fail that re-run.

#### Scenario: Job triggers match the Cargo workflow

- **GIVEN** `.github/workflows/ci.yml` and the separate spec-drift workflow
- **WHEN** the workflow triggers are read
- **THEN** `ci.yml` still runs the Cargo job and the `spec-drift` job on push and pull_request to `main`, with no `edited` type and no skip `if`
- **AND** the `spec-drift` job in `ci.yml` still invokes `scripts/spec-drift-check.mjs`
- **AND** the separate workflow runs only that script on pull_request `edited` targeting `main`

#### Scenario: Dropping the change line on edit fails

- **GIVEN** a pull_request to `main` that changes `crates/rules/src/lib.rs`
- **AND** `openspec/changes/add-dice-and-checks/` exists
- **AND** the body line `OpenSpec-Change: add-dice-and-checks` made the `spec-drift` job pass
- **WHEN** the pull request is `edited` and the body has no `OpenSpec-Change` line
- **THEN** the separate spec-drift workflow runs
- **AND** the check fails
- **AND** the log includes `spec-drift: code change missing OpenSpec-Change`

### Requirement: Cargo job remains

The workflow SHALL keep the existing job that runs `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo build --workspace`, and `cargo test --workspace`. `.github/workflows/ci.yml` SHALL keep its current triggers and SHALL NOT skip that job. A body edit SHALL NOT replace a failed Cargo check. A separate workflow SHALL run spec-drift on pull_request `edited`.

#### Scenario: Cargo steps stay

- **GIVEN** `.github/workflows/ci.yml` after this change is applied
- **WHEN** the Cargo job is read
- **THEN** it still runs fmt check, clippy with warnings denied, workspace build, and workspace test

#### Scenario: Editing the body does not replace a failed Cargo check

- **GIVEN** a pull_request to `main` whose Cargo job has failed
- **WHEN** the pull request body is edited
- **THEN** `.github/workflows/ci.yml` does not run for that `edited` event
- **AND** the failed Cargo check stays failed
- **AND** only the separate spec-drift workflow runs

### Requirement: Change reference line

A change reference SHALL be one trimmed body line, after one trailing carriage return is removed, of `OpenSpec-Change:` plus one token. A token of `a-z` and `0-9` groups separated by single hyphens, not starting or ending with a hyphen, SHALL be the id. Any other token SHALL fail, SHALL NOT be skipped or case-folded, and the log SHALL include `spec-drift: invalid change id <token>`. Extra words SHALL NOT be a reference. Fenced lines SHALL count the same. This SHALL be the AGENTS.md line.

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
- **AND** the log includes `spec-drift: code change missing OpenSpec-Change`
- **AND** the log does not include `spec-drift: unknown change`
- **AND** the log does not include `spec-drift: invalid change id`

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

### Requirement: Docs-only and openspec-only pull requests

When the diff has no code path, a missing reference SHALL pass, and a lowercase kebab-case id that names no change SHALL pass. An `OpenSpec-Change:` line whose single token is not that grammar SHALL fail on every pull request, including docs-only and openspec-only. A missing line SHALL NOT fail a docs-only or openspec-only pull request.

#### Scenario: Docs-only pull request passes without a reference

- **GIVEN** a pull_request to `main` whose diff is `docs/architect/decisions.md` and `README.md`
- **AND** the body has no change reference
- **AND** validation exits zero
- **AND** no active change is fully checked
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes

#### Scenario: Openspec-only pull request passes without a reference

- **GIVEN** a pull_request to `main` whose diff is only under `openspec/`
- **AND** the body has no change reference
- **AND** validation exits zero
- **AND** no active change is fully checked
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes

#### Scenario: An exempt pull request ignores an unknown id

- **GIVEN** a pull_request to `main` whose diff is only `README.md`
- **AND** the body line is `OpenSpec-Change: missing-change`
- **AND** validation exits zero
- **AND** no active change is fully checked
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes

#### Scenario: Docs plus a code path still requires a reference

- **GIVEN** a pull_request to `main` whose diff changes `README.md` and `crates/rules/src/lib.rs`
- **AND** the body has no change reference
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: code change missing OpenSpec-Change`

#### Scenario: Openspec plus a workflow file still requires a reference

- **GIVEN** a pull_request to `main` whose diff changes `openspec/changes/add-spec-drift-check/tasks.md` and `.github/workflows/ci.yml`
- **AND** the body has no change reference
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: code change missing OpenSpec-Change`

#### Scenario: Docs plus a code path passes with an existing id

- **GIVEN** a pull_request to `main` whose diff changes `docs/architect/decisions.md` and `Cargo.lock`
- **AND** the body line is `OpenSpec-Change: add-dice-and-checks`
- **AND** that active change exists
- **AND** validation exits zero
- **AND** the completed-change check passes
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes

#### Scenario: docs-only PR with OpenSpec-Change: Add-Foo fails

- **GIVEN** a pull_request to `main` whose diff is `docs/architect/decisions.md`
- **AND** the body line is `OpenSpec-Change: Add-Foo`
- **AND** `openspec/changes/add-foo/` exists
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: invalid change id Add-Foo`
- **AND** the log does not include `spec-drift: code change missing OpenSpec-Change`

## ADDED Requirements

### Requirement: Git locale is C

Every `git` subprocess started by `scripts/spec-drift-check.mjs` SHALL have `LC_ALL=C`, so git error text does not follow the runner locale.

#### Scenario: Git diff runs with LC_ALL=C

- **GIVEN** the parent process has `LC_ALL=fr_FR.UTF-8`
- **AND** a stand-in `git` records the environment of `git diff`
- **WHEN** the `spec-drift` job computes changed paths
- **THEN** that git child is started with `LC_ALL=C`

### Requirement: Invalid change line

An empty line `OpenSpec-Change:` and a spaceless line `OpenSpec-Change:Add` SHALL both be invalid on any pull request. Each SHALL fail. The log SHALL include `spec-drift: invalid change id`. Neither SHALL be ignored or treated only as a missing reference.

#### Scenario: An empty or spaceless change line is invalid

- **GIVEN** a pull_request to `main` whose diff is only `docs/architect/decisions.md`
- **AND** the body contains a line `OpenSpec-Change:` and a line `OpenSpec-Change:Add`
- **WHEN** the `spec-drift` job runs
- **THEN** the job fails
- **AND** the log includes `spec-drift: invalid change id`
- **AND** the log includes `spec-drift: invalid change id Add`
- **AND** the log does not include `spec-drift: code change missing OpenSpec-Change`

### Requirement: Scripts are not a code path

A path under `scripts/` SHALL NOT be a spec-drift code path. A pull request whose changed paths are only under `scripts/` SHALL NOT fail for a missing `OpenSpec-Change` line. That pull request SHALL still need a new Unreleased changelog bullet.

#### Scenario: A scripts-only pull request is not a code path

- **GIVEN** a pull_request to `main` whose only diff is `scripts/note.mjs`
- **AND** the body has no change reference
- **AND** validation exits zero
- **AND** no active change is fully checked
- **WHEN** the `spec-drift` job runs
- **THEN** the job passes
- **AND** the log does not include `spec-drift: code change missing OpenSpec-Change`
