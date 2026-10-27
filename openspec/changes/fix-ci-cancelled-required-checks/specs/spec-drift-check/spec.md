# spec-drift-check

## ADDED Requirements

### Requirement: Spec-drift declares no concurrency

The `spec-drift` job in `.github/workflows/ci.yml` and in `.github/workflows/spec-drift-edited.yml` SHALL NOT declare a concurrency block. `ci.yml` SHALL NOT declare workflow-level concurrency. The Cargo `ci` job SHALL NOT gain a concurrency key. A push SHALL NOT cancel an in-progress pull request `spec-drift` run.

#### Scenario: The spec-drift jobs declare no concurrency

- **GIVEN** `.github/workflows/ci.yml` and `.github/workflows/spec-drift-edited.yml`
- **WHEN** each `spec-drift` job is read
- **THEN** neither job contains a `concurrency:` line
- **AND** neither file contains `cancel-in-progress` or `queue:`
- **AND** `ci.yml` has no workflow-level concurrency block

#### Scenario: A push does not cancel a pull request run

- **GIVEN** the `spec-drift` job in `.github/workflows/ci.yml` has no concurrency block
- **WHEN** a push to `main` starts while a pull request `spec-drift` job is in progress
- **THEN** the pull request job is not cancelled

#### Scenario: The Cargo job has no concurrency key

- **GIVEN** `.github/workflows/ci.yml`
- **WHEN** the Cargo `ci` job is read
- **THEN** that job has no concurrency key
- **AND** the workflow has no workflow-level concurrency block

### Requirement: A same-sha burst cancels nothing

A `synchronize`, an `edited`, and a review `submitted` on one head sha SHALL NOT leave a cancelled required check on that sha. The required checks are `ci`, `pr-hygiene`, `spec-drift`, and `spec-approved`. The `ci` job, both `spec-drift` jobs, the `pr-hygiene` workflow, and the `spec-approved` job SHALL NOT declare a concurrency block.

#### Scenario: Three same-sha events leave no cancelled check

- **GIVEN** one head sha
- **AND** `synchronize`, then `edited`, then `pull_request_review` `submitted` arrive while earlier runs are still going
- **AND** `.github/workflows/ci.yml`, `.github/workflows/spec-drift-edited.yml`, `.github/workflows/pr-hygiene.yml`, and `.github/workflows/spec-approved.yml` declare no concurrency block on `ci`, `spec-drift`, `pr-hygiene`, or `spec-approved`
- **WHEN** those runs finish
- **THEN** no required check on that sha has conclusion cancelled

#### Scenario: No required check declares concurrency

- **GIVEN** `.github/workflows/ci.yml`, `.github/workflows/spec-drift-edited.yml`, `.github/workflows/pr-hygiene.yml`, and `.github/workflows/spec-approved.yml`
- **WHEN** each file is read
- **THEN** none of the four files contains a `concurrency:` line
- **AND** none of the four files contains `cancel-in-progress` or `queue:`

## MODIFIED Requirements

### Requirement: Edited spec-drift workflow

`.github/workflows/spec-drift-edited.yml` SHALL run `node --test scripts/spec-drift-check.test.mjs` before `node scripts/spec-drift-check.mjs` on pull_request `edited` targeting `main`. Its only job SHALL be named exactly `spec-drift`. It SHALL set `permissions: contents: read`. That job SHALL NOT declare a concurrency block.

#### Scenario: The edited workflow reuses the spec-drift job name

- **GIVEN** `.github/workflows/spec-drift-edited.yml`
- **WHEN** its lines are checked
- **THEN** it runs on pull_request `edited` targeting `main`
- **AND** its only job is named `spec-drift`
- **AND** that job runs `node --test scripts/spec-drift-check.test.mjs` before `node scripts/spec-drift-check.mjs`
- **AND** permissions are `contents: read`
- **AND** that job does not declare a concurrency block
- **AND** the workflow has no workflow-level concurrency group

## REMOVED Requirements

### Requirement: Shared spec-drift concurrency

**Reason**: The shared pull-request-number group uses `cancel-in-progress: true`, so an `edited` run cancels the in-flight `ci.yml` `spec-drift` job and leaves a cancelled required check on that head sha.
**Migration**: Replaced by "Spec-drift declares no concurrency" and "A same-sha burst cancels nothing".
