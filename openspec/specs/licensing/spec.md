# licensing Specification

## Purpose
Record that source code is MIT, that rules mechanics are under the ORC License, and that contributors follow `docs/architect/licensing.md`.

## Requirements

### Requirement: Licensing document

The repository SHALL contain `docs/architect/licensing.md`. That document SHALL state numbered rules whose identifiers start at `LIC-1`. It SHALL mark unverified research with `**UNVERIFIED**`. It SHALL include an "Open questions for Evan and an IP lawyer" section. This change SHALL NOT add a CI job that enforces those rules.

#### Scenario: The licensing document is present

- **GIVEN** a checkout of the repository
- **WHEN** `docs/architect/licensing.md` is read
- **THEN** it contains `LIC-1`
- **AND** it contains `**UNVERIFIED**`
- **AND** it contains `Open questions for Evan and an IP lawyer`

### Requirement: Contributors follow the licensing document

Root `AGENTS.md` SHALL say that all contributors, human or bot, follow `docs/architect/licensing.md`, and that every rules spec or doc cites its source book and page. The changelog sentences already required in `AGENTS.md` section 2 and section 10 SHALL stay in place.

#### Scenario: The constitution names the licensing document

- **GIVEN** root `AGENTS.md`
- **WHEN** section 10 is read
- **THEN** it says all contributors, human or bot, follow `docs/architect/licensing.md`
- **AND** it says every rules spec or doc cites its source book and page
- **AND** section 2 still says an implementation pull request adds a changelog entry under `## [Unreleased]` before the work is done, and that spec-only, docs-only, and archive-only pull requests do not

### Requirement: Rules crate license is MIT

The `crates/rules` package manifest SHALL set `license` to `MIT`. Root `LICENSE` SHALL be the MIT license and SHALL name Copyright (c) 2026 Evan Burkey.

#### Scenario: The rules manifest names MIT

- **GIVEN** `crates/rules/Cargo.toml`
- **WHEN** its package license is read
- **THEN** the license is `MIT`

### Requirement: ORC notice file

Root `NOTICE-ORC.md` SHALL contain the headings `ORC Notice`, `Attribution`, `Reserved Material`, and `Expressly Designated Licensed Material`. The author-line TODO SHALL remain. The generic Reserved Material list SHALL remain. Both SHALL be marked `**UNVERIFIED**` pending the legal pages of Player Core, GM Core, and Monster Core.

#### Scenario: The notice keeps the pending lines

- **GIVEN** `NOTICE-ORC.md`
- **WHEN** the notice is read
- **THEN** it contains the four headings
- **AND** it contains the author-line TODO
- **AND** it contains `**UNVERIFIED**` pending the legal pages of Player Core, GM Core, and Monster Core
