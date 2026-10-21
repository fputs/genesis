# changelog

## Purpose

The changelog tells a reader what changed. It names where v1 history lives, and it says which pull requests must add an Unreleased bullet before the work is done.

## ADDED Requirements

### Requirement: Header names the format and v1

Root `CHANGELOG.md` MUST follow Keep a Changelog 1.1.0. The text above `## [Unreleased]` MUST say the format is Keep a Changelog 1.1.0 and MUST note that v1 history lives on the `v1-reference` branch.

#### Scenario: A reviewer reads the header

- **GIVEN** root `CHANGELOG.md`
- **WHEN** a reviewer reads the text above `## [Unreleased]`
- **THEN** it says the format is Keep a Changelog 1.1.0
- **AND** it notes that v1 history lives on the `v1-reference` branch

### Requirement: Implementation pull requests add an Unreleased bullet

An implementation pull request changes how the software or its checks behave. Before that pull request is done, it MUST add a bullet under `## [Unreleased]` in the section that fits: `### Added`, `### Changed`, `### Fixed`, `### Removed`, or `### Security`. When more than one of those fits, it MUST add a bullet in each. Each bullet MUST link to the pull request.

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

### Requirement: Spec, docs, and archive pull requests skip the bullet

A spec-only, docs-only, or archive-only pull request MUST NOT need a changelog bullet. Spec-only means every changed path is under `openspec/`. Docs-only means the pull request only edits documentation, including `docs/`, `AGENTS.md`, and `CHANGELOG.md`. Archive-only means the pull request only archives an OpenSpec change. An implementation pull request that archives itself in its last commit still MUST add its bullet.

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
