# spec-approved

## ADDED Requirements

### Requirement: Same-sha spec-approved runs queue

A superseded `spec-approved` run SHALL NOT leave a cancelled required check on a head sha that a later `spec-approved` run on that same sha completes. The job SHALL set group `spec-approved-${{ github.workflow }}-${{ github.event.pull_request.head.sha }}-${{ github.event_name }}-${{ github.event.action }}` with `cancel-in-progress: false` and `queue: max`.

#### Scenario: A same-sha review queues spec-approved

- **GIVEN** `.github/workflows/spec-approved.yml`
- **WHEN** the `spec-approved` job concurrency block is read
- **THEN** the group is `spec-approved-${{ github.workflow }}-${{ github.event.pull_request.head.sha }}-${{ github.event_name }}-${{ github.event.action }}`
- **AND** `cancel-in-progress` is the boolean false
- **AND** `queue` is max
- **AND** the group is not `spec-approved-${{ github.event.pull_request.number }}`
- **AND** the workflow has no workflow-level concurrency block

#### Scenario: The same head sha stays one group

- **GIVEN** two `spec-approved` runs for the same workflow, the same head sha, and the same event name and action
- **WHEN** the group expression is evaluated for each
- **THEN** both groups are equal
- **AND** `cancel-in-progress` is false

#### Scenario: A different head sha is a different group

- **GIVEN** the `spec-approved` group expression
- **WHEN** it is evaluated for head sha `aaa` and for head sha `bbb` with the same workflow name, event name, and action
- **THEN** the two group strings differ

## REMOVED Requirements

### Requirement: Concurrency

**Reason**: `cancel-in-progress: true` on the pull request number cancels an in-flight run when a review, an edit, or a same-sha `synchronize` starts another run.
**Migration**: Replaced by "Same-sha spec-approved runs queue".
