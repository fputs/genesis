# changelog

## MODIFIED Requirements

### Requirement: Implementation pull requests add an Unreleased bullet

An implementation pull request changes how the software or its checks behave. Before that pull request is done, it MUST add a bullet under `## [Unreleased]` in the section that fits: `### Added`, `### Changed`, `### Fixed`, `### Removed`, or `### Security`. When more than one of those fits, it MUST add a bullet in each. Each bullet MUST link to the pull request. The `pr-hygiene` job MUST fail an implementation pull request that adds no bullet under `## [Unreleased]`.

#### Scenario: A behavior change has a linked bullet

- **GIVEN** an implementation pull request that is otherwise ready
- **WHEN** a reviewer reads the `CHANGELOG.md` diff
- **THEN** `## [Unreleased]` has a new bullet under Added, Changed, Fixed, Removed, or Security
- **AND** the bullet links to that pull request

#### Scenario: A fix is filed under Fixed

- **GIVEN** an implementation pull request whose change is a fix
- **WHEN** a reviewer reads the new bullet
- **THEN** the bullet is under `### Fixed`
- **AND** the bullet links to that pull request

#### Scenario: Two kinds of change use two sections

- **GIVEN** an implementation pull request that adds behavior and also tightens security
- **WHEN** a reviewer reads `## [Unreleased]`
- **THEN** one bullet is under `### Added` and another is under `### Security`
- **AND** both bullets link to that pull request

#### Scenario: The hygiene job enforces the bullet

- **GIVEN** an implementation pull request that adds no bullet under `## [Unreleased]`
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`
