# spec-approved

## ADDED Requirements

### Requirement: Spec-approved declares no concurrency

The `spec-approved` job SHALL NOT declare a concurrency block. The workflow SHALL NOT declare workflow-level concurrency. A same-sha review SHALL NOT cancel an in-flight `spec-approved` run.

#### Scenario: The spec-approved job declares no concurrency

- **GIVEN** `.github/workflows/spec-approved.yml`
- **WHEN** the `spec-approved` job and the workflow header are read
- **THEN** the job does not contain a `concurrency:` line
- **AND** the workflow has no workflow-level concurrency block
- **AND** the file does not contain `cancel-in-progress` or `queue:`

## REMOVED Requirements

### Requirement: Concurrency

**Reason**: `cancel-in-progress: true` on the pull request number cancels an in-flight run when a review, an edit, or a same-sha `synchronize` starts another run. With `cancel-in-progress: false`, a third queued run still cancels the one pending run.
**Migration**: Replaced by "Spec-approved declares no concurrency".
