# spec-drift-check

## ADDED Requirements

### Requirement: Fixture cleanup retries ENOTEMPTY

Each spec-drift fixture case SHALL remove its temporary git directory when the case ends, through an injectable remover. When a call throws `ENOTEMPTY`, including on `.git/objects`, it SHALL retry with backoff for at most 5 calls. A throw that a later call clears SHALL NOT fail the test. If all 5 calls throw, the test SHALL fail with that error.

#### Scenario: A transient ENOTEMPTY is retried

- **GIVEN** an injectable remover whose first call throws `ENOTEMPTY` on `.git/objects`
- **AND** its second call removes the temporary git directory
- **WHEN** the case ends
- **THEN** the directory is gone
- **AND** the test does not fail

#### Scenario: Retries stop after five

- **GIVEN** an injectable remover that throws `ENOTEMPTY` on `.git/objects` on every call
- **WHEN** the case ends
- **THEN** the remover was called 5 times
- **AND** the test fails with that error
