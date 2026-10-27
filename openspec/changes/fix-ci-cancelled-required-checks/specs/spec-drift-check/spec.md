# spec-drift-check

## ADDED Requirements

### Requirement: Same-sha spec-drift runs queue

A superseded `spec-drift` run SHALL NOT leave a cancelled required check on a head sha that a later `spec-drift` run on that same sha completes. Each `spec-drift` job SHALL set job-level group `spec-drift-${{ github.workflow }}-${{ github.event.pull_request.head.sha || github.sha }}-${{ github.event_name }}-${{ github.event.action }}` with `cancel-in-progress: false` and `queue: max`.

#### Scenario: Same-sha spec-drift runs queue

- **GIVEN** `.github/workflows/ci.yml` and `.github/workflows/spec-drift-edited.yml`
- **WHEN** the `spec-drift` job concurrency block in each file is read
- **THEN** each group is `spec-drift-${{ github.workflow }}-${{ github.event.pull_request.head.sha || github.sha }}-${{ github.event_name }}-${{ github.event.action }}`
- **AND** each sets `cancel-in-progress` to the boolean false
- **AND** each sets `queue` to max
- **AND** neither group is `spec-drift-${{ github.event.pull_request.number || github.sha }}`
- **AND** neither file sets concurrency at workflow level

#### Scenario: A different head sha is a different group

- **GIVEN** the `spec-drift` group expression
- **WHEN** it is evaluated for head sha `aaa` and for head sha `bbb` with the same workflow name, event name, and action
- **THEN** the two group strings differ
- **AND** `cancel-in-progress` is false

### Requirement: Spec-drift workflows do not share a group

The `spec-drift` job in `.github/workflows/ci.yml` and the `spec-drift` job in `.github/workflows/spec-drift-edited.yml` SHALL NOT share a concurrency group. The Cargo `ci` job SHALL NOT set a concurrency key. `ci.yml` SHALL NOT set workflow-level concurrency. A push SHALL use `github.sha` in the `spec-drift` group and SHALL NOT join a pull request group.

#### Scenario: An edited workflow does not join the ci.yml group

- **GIVEN** `.github/workflows/ci.yml` is named `CI`
- **AND** `.github/workflows/spec-drift-edited.yml` is named `Spec drift edited`
- **AND** both `spec-drift` groups include `github.workflow`
- **WHEN** both groups are evaluated for the same head sha, event name, and action
- **THEN** the groups differ
- **AND** an `edited` run does not cancel the `ci.yml` `spec-drift` job

#### Scenario: A push does not join a pull request group

- **GIVEN** a push event has no pull request head sha
- **WHEN** the `ci.yml` `spec-drift` group is evaluated with `github.sha` `abc123def`
- **THEN** the group uses `abc123def`
- **AND** it is not equal to the group for a `pull_request` event whose head sha is `abc123def`

#### Scenario: The Cargo job has no concurrency key

- **GIVEN** `.github/workflows/ci.yml`
- **WHEN** the Cargo `ci` job is read
- **THEN** that job has no concurrency key
- **AND** the workflow has no workflow-level concurrency block

## MODIFIED Requirements

### Requirement: Edited spec-drift workflow

`.github/workflows/spec-drift-edited.yml` SHALL run `node --test scripts/spec-drift-check.test.mjs` before `node scripts/spec-drift-check.mjs` on pull_request `edited` targeting `main`. Its only job SHALL be named exactly `spec-drift`. It SHALL set `permissions: contents: read`. Its job-level concurrency SHALL be the same-sha spec-drift group, with `cancel-in-progress: false` and `queue: max`.

#### Scenario: The edited workflow reuses the spec-drift job name

- **GIVEN** `.github/workflows/spec-drift-edited.yml`
- **WHEN** its lines are checked
- **THEN** it runs on pull_request `edited` targeting `main`
- **AND** its only job is named `spec-drift`
- **AND** that job runs `node --test scripts/spec-drift-check.test.mjs` before `node scripts/spec-drift-check.mjs`
- **AND** permissions are `contents: read`
- **AND** that job's concurrency group is `spec-drift-${{ github.workflow }}-${{ github.event.pull_request.head.sha || github.sha }}-${{ github.event_name }}-${{ github.event.action }}`
- **AND** `cancel-in-progress` is the boolean false
- **AND** `queue` is max
- **AND** the workflow has no workflow-level concurrency group

## REMOVED Requirements

### Requirement: Shared spec-drift concurrency

**Reason**: The shared pull-request-number group uses `cancel-in-progress: true`, so an `edited` run cancels the in-flight `ci.yml` `spec-drift` job and leaves a cancelled required check on that head sha.
**Migration**: Replaced by "Same-sha spec-drift runs queue" and "Spec-drift workflows do not share a group".
