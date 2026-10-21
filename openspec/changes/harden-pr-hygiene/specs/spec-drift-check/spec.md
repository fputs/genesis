# spec-drift-check

## MODIFIED Requirements

### Requirement: Spec-drift job

The repository SHALL run a `spec-drift` job in `.github/workflows/ci.yml` on push to `main` and on pull_request `opened`, `synchronize`, `reopened`, and `edited` targeting `main`. The job SHALL invoke `scripts/spec-drift-check.mjs`. That script SHALL have no npm dependencies. A code-path pull request that then drops every `OpenSpec-Change` line on `edited` SHALL fail that re-run.

#### Scenario: Job triggers match the Cargo workflow

- **GIVEN** `.github/workflows/ci.yml`
- **WHEN** the workflow triggers are read
- **THEN** the `spec-drift` job runs on push and pull_request to `main`
- **AND** it invokes `scripts/spec-drift-check.mjs`

#### Scenario: Job triggers include edited

- **GIVEN** `.github/workflows/ci.yml`
- **WHEN** the workflow triggers are read
- **THEN** the `spec-drift` job runs on push to `main`
- **AND** it runs on pull_request `opened`, `synchronize`, `reopened`, and `edited` targeting `main`
- **AND** it invokes `scripts/spec-drift-check.mjs`

#### Scenario: Dropping the change line on edit fails

- **GIVEN** a pull_request to `main` that changes `crates/rules/src/lib.rs`
- **AND** `openspec/changes/add-dice-and-checks/` exists
- **AND** the body line `OpenSpec-Change: add-dice-and-checks` made the `spec-drift` job pass
- **WHEN** the pull request is `edited` and the body has no `OpenSpec-Change` line
- **THEN** the `spec-drift` job runs again
- **AND** the job fails
- **AND** the log includes `spec-drift: code change missing OpenSpec-Change`

### Requirement: Cargo job remains

The workflow SHALL keep the existing job that runs `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo build --workspace`, and `cargo test --workspace`. That job SHALL NOT run on pull_request `edited`.

#### Scenario: Cargo steps stay

- **GIVEN** `.github/workflows/ci.yml` after this change is applied
- **WHEN** the Cargo job is read
- **THEN** it still runs fmt check, clippy with warnings denied, workspace build, and workspace test

#### Scenario: Cargo does not run on edited

- **GIVEN** a pull_request `edited` event targeting `main`
- **WHEN** `.github/workflows/ci.yml` is read
- **THEN** the Cargo job does not run
- **AND** the `spec-drift` job does run

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

## ADDED Requirements

### Requirement: Git locale is C

Every `git` subprocess started by `scripts/spec-drift-check.mjs` SHALL have `LC_ALL=C`, so git error text does not follow the runner locale.

#### Scenario: Git diff runs with LC_ALL=C

- **GIVEN** the parent process has `LC_ALL=fr_FR.UTF-8`
- **AND** a stand-in `git` records the environment of `git diff`
- **WHEN** the `spec-drift` job computes changed paths
- **THEN** that git child is started with `LC_ALL=C`
