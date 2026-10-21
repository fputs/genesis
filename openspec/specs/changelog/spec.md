# changelog Specification

## Purpose
The changelog tells a reader what changed. It names where v1 history lives, and it says which pull requests must add an Unreleased bullet before the work is done.

## Requirements

### Requirement: Header names the format and v1

Root `CHANGELOG.md` MUST follow Keep a Changelog 1.1.0. The text above `## [Unreleased]` MUST say the format is Keep a Changelog 1.1.0 and MUST note that v1 history lives on the `v1-reference` branch.

#### Scenario: A reviewer reads the header

- **GIVEN** root `CHANGELOG.md`
- **WHEN** a reviewer reads the text above `## [Unreleased]`
- **THEN** it says the format is Keep a Changelog 1.1.0
- **AND** it notes that v1 history lives on the `v1-reference` branch

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

#### Scenario: A scripts-only pull request needs a bullet

- **GIVEN** a pull request whose only diff is `scripts/note.mjs`
- **AND** no Unreleased bullet is added
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

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

### Requirement: The done section names the changelog entry

`AGENTS.md` section 2 MUST include one line that an implementation pull request adds its changelog entry before the work is done. That line MUST leave spec-only, docs-only, and archive-only pull requests out of the requirement.

#### Scenario: Section 2 has one changelog line

- **GIVEN** `AGENTS.md` after this change is implemented
- **WHEN** a reviewer reads section 2
- **THEN** it contains one line about adding the changelog entry before done
- **AND** that line does not require an entry from spec-only, docs-only, or archive-only pull requests

### Requirement: No post-merge changelog step

The changelog entry MUST be written in the implementation pull request before that work is done. `docs/producer/process.md` and `AGENTS.md` section 10 MUST say so. No file under `docs/` and no part of `AGENTS.md` MUST describe a separate post-merge changelog step.

#### Scenario: No doc describes a post-merge changelog step

- **GIVEN** `docs/` and `AGENTS.md` after this change is implemented
- **WHEN** a reviewer reads them for how a changelog entry is written
- **THEN** no document describes a separate post-merge changelog step
- **AND** `docs/producer/process.md` and `AGENTS.md` section 10 say the implementation pull request writes the entry before the work is done
