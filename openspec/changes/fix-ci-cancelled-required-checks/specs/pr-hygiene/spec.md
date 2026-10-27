# pr-hygiene

## ADDED Requirements

### Requirement: Same-sha pr-hygiene runs queue

A superseded `pr-hygiene` run SHALL NOT leave a cancelled required check on a head sha that a later `pr-hygiene` run on that same sha completes. The workflow SHALL set group `pr-hygiene-${{ github.workflow }}-${{ github.event.pull_request.head.sha }}-${{ github.event_name }}-${{ github.event.action }}` with `cancel-in-progress: false` and `queue: max`.

#### Scenario: A same-sha edit queues pr-hygiene

- **GIVEN** `.github/workflows/pr-hygiene.yml`
- **WHEN** its concurrency block is read
- **THEN** the group is `pr-hygiene-${{ github.workflow }}-${{ github.event.pull_request.head.sha }}-${{ github.event_name }}-${{ github.event.action }}`
- **AND** `cancel-in-progress` is the boolean false
- **AND** `queue` is max
- **AND** the group is not `pr-hygiene-${{ github.event.pull_request.number }}`
- **AND** the workflow has no second concurrency block

#### Scenario: The same head sha stays one group

- **GIVEN** two `pr-hygiene` runs for the same workflow, the same head sha, and the same event name and action
- **WHEN** the group expression is evaluated for each
- **THEN** both groups are equal
- **AND** `cancel-in-progress` is false

#### Scenario: A different head sha is a different group

- **GIVEN** the `pr-hygiene` group expression
- **WHEN** it is evaluated for head sha `aaa` and for head sha `bbb` with the same workflow name, event name, and action
- **THEN** the two group strings differ

## REMOVED Requirements

### Requirement: Cancel the previous run

**Reason**: `cancel-in-progress: true` on the pull request number cancels an in-flight run when the body is edited or a `synchronize` event arrives for the same head sha.
**Migration**: Replaced by "Same-sha pr-hygiene runs queue".
