# changelog

## MODIFIED Requirements

### Requirement: Implementation pull requests add an Unreleased bullet

An implementation pull request changes how the software or its checks behave. Before it is done, it MUST add a bullet under `## [Unreleased]` in each fitting section: `### Added`, `### Changed`, `### Fixed`, `### Removed`, or `### Security`. Each bullet MUST link to the pull request. Text already in the base changelog, an edited or moved bullet, or a fenced list item MUST NOT count. The `pr-hygiene` job MUST fail when no such bullet was added.

#### Scenario: A behavior change has a linked bullet

- **GIVEN** an implementation pull request that is otherwise ready
- **WHEN** a reviewer reads the `CHANGELOG.md` diff
- **THEN** `## [Unreleased]` has a new bullet under Added, Changed, Fixed, Removed, or Security
- **AND** the bullet links to that pull request
- **AND** that bullet's text is not already in the base changelog

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

#### Scenario: A moved bullet does not satisfy the rule

- **GIVEN** an implementation pull request whose only Unreleased bullet was already under a released heading in the base changelog
- **WHEN** a reviewer decides whether a new bullet was added
- **THEN** that moved bullet does not count
- **AND** the `pr-hygiene` job fails

#### Scenario: An edited bullet does not satisfy the rule

- **GIVEN** an implementation pull request that only rewords a bullet already under `## [Unreleased]`
- **WHEN** a reviewer decides whether a new bullet was added
- **THEN** that edited bullet does not count
- **AND** the `pr-hygiene` job fails

#### Scenario: A fenced bullet does not count

- **GIVEN** an implementation pull request whose only new list item is inside a fenced code block
- **WHEN** a reviewer reads `## [Unreleased]`
- **THEN** that fenced item does not count
- **AND** the `pr-hygiene` job fails

### Requirement: Spec, docs, and archive pull requests skip the bullet

A pull request MUST NOT need a changelog bullet when every changed path is under `openspec/`, `docs/`, or `.cursor/`, or is exactly `AGENTS.md`, `CHANGELOG.md`, `README.md`, `Cargo.lock`, or `rust-toolchain.toml`. Those categories combine. Archive-only means only moving a change into `openspec/changes/archive/`. An implementation pull request that archives itself in its last commit still MUST add its bullet.

#### Scenario: A spec-only pull request skips the bullet

- **GIVEN** a pull request whose changed paths are all under `openspec/`
- **WHEN** a reviewer decides whether `CHANGELOG.md` must change
- **THEN** no new bullet is required

#### Scenario: A docs-only pull request skips the bullet

- **GIVEN** a pull request that only edits `docs/`, `AGENTS.md`, or `CHANGELOG.md`
- **WHEN** a reviewer decides whether a new bullet is required
- **THEN** no new bullet is required for that pull request

#### Scenario: An archive-only pull request skips the bullet

- **GIVEN** a pull request that only moves a change into `openspec/changes/archive/`
- **WHEN** a reviewer decides whether a new bullet is required
- **THEN** no new bullet is required

#### Scenario: The archive commit does not drop the bullet

- **GIVEN** an implementation pull request whose last commit archives its OpenSpec change
- **WHEN** a reviewer reads `## [Unreleased]`
- **THEN** the pull request still has its linked bullet

#### Scenario: Mixed exempt paths skip the bullet

- **GIVEN** a pull request whose paths are `docs/architect/decisions.md`, `openspec/specs/changelog/spec.md`, `.cursor/rules/example.md`, `README.md`, `Cargo.lock`, and `rust-toolchain.toml`
- **WHEN** a reviewer decides whether a new bullet is required
- **THEN** no new bullet is required
