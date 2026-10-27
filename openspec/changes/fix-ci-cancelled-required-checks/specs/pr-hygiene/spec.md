# pr-hygiene

## ADDED Requirements

### Requirement: Pr-hygiene declares no concurrency

The `pr-hygiene` workflow SHALL NOT declare a concurrency block. A same-sha `edited` or `synchronize` SHALL NOT cancel an in-flight `pr-hygiene` run.

#### Scenario: The pr-hygiene workflow declares no concurrency

- **GIVEN** `.github/workflows/pr-hygiene.yml`
- **WHEN** its lines are read
- **THEN** the file does not contain a `concurrency:` line
- **AND** the file does not contain `cancel-in-progress` or `queue:`

## REMOVED Requirements

### Requirement: Cancel the previous run

**Reason**: `cancel-in-progress: true` on the pull request number cancels an in-flight run when the body is edited or a `synchronize` event arrives for the same head sha. With `cancel-in-progress: false`, a third queued run still cancels the one pending run.
**Migration**: Replaced by "Pr-hygiene declares no concurrency".
