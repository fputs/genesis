# spec-drift-check

## ADDED Requirements

### Requirement: Fixture cleanup retries ENOTEMPTY

Each spec-drift fixture case SHALL remove its temporary git directory when the case ends. When removal returns `ENOTEMPTY`, including `rmdir` of `.git/objects`, it SHALL retry until the directory is gone. That `ENOTEMPTY` SHALL NOT fail the test. The test SHALL fail if the directory remains after the retries.

#### Scenario: A transient ENOTEMPTY is retried

- **GIVEN** a fixture cleanup whose first removal of the temporary git directory returns `ENOTEMPTY` on `.git/objects`
- **AND** the next removal succeeds
- **WHEN** the case ends
- **THEN** the directory is gone
- **AND** the test does not fail
