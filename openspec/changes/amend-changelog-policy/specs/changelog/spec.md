# changelog

## MODIFIED Requirements

### Requirement: Implementation pull requests add an Unreleased bullet

An implementation pull request changes how the software or its checks behave. Except a tests-only pull request, it MUST add a bullet under `## [Unreleased]` in each fitting section: `### Added`, `### Changed`, `### Deprecated`, `### Removed`, `### Fixed`, or `### Security`. Each bullet MUST link the merging pull request's own number. Text already in the base changelog, an edited or moved bullet, or a fenced list item MUST NOT count. The `pr-hygiene` job MUST fail when no such bullet was added.

#### Scenario: A behavior change has a linked bullet

- **GIVEN** an implementation pull request that is otherwise ready
- **WHEN** a reviewer reads the `CHANGELOG.md` diff
- **THEN** `## [Unreleased]` has a new bullet under Added, Changed, Deprecated, Removed, Fixed, or Security
- **AND** the bullet links that pull request's own number
- **AND** that bullet's text is not already in the base changelog

#### Scenario: A fix is filed under Fixed

- **GIVEN** an implementation pull request whose change is a fix
- **AND** that change does not restrict a token, permission, credential, or vulnerability
- **AND** it does not remove, deprecate, or add caller-visible behavior
- **WHEN** a reviewer reads the new bullet
- **THEN** the bullet is under `### Fixed`
- **AND** the bullet links that pull request's own number

#### Scenario: Two kinds of change use two sections

- **GIVEN** an implementation pull request that adds behavior and also tightens security
- **WHEN** a reviewer reads `## [Unreleased]`
- **THEN** one bullet is under `### Added` and another is under `### Security`
- **AND** both bullets link that pull request's own number

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

#### Scenario: Cursor, README, and toolchain files skip the bullet

- **GIVEN** a pull request whose paths are `.cursor/rules/example.md`, `README.md`, `Cargo.lock`, and `rust-toolchain.toml`
- **WHEN** a reviewer decides whether a new bullet is required
- **THEN** no new bullet is required

## ADDED Requirements

### Requirement: Deprecated section

When `CHANGELOG.md` records behavior that still works and is marked for later removal, that record MUST use the heading `### Deprecated`. Present category headings under `## [Unreleased]` or a version MUST stay in this order, skipping any that are absent: Added, Changed, Deprecated, Removed, Fixed, Security. An empty changelog MUST NOT add `### Deprecated`.

#### Scenario: Deprecated follows Changed and precedes Removed

- **GIVEN** an Unreleased section that has `### Changed`, `### Deprecated`, and `### Removed`
- **WHEN** a reviewer reads those headings in order
- **THEN** `### Deprecated` follows `### Changed`
- **AND** `### Deprecated` precedes `### Removed`

#### Scenario: A lone Deprecated heading is valid

- **GIVEN** `## [Unreleased]` has `### Deprecated` and no other category heading
- **WHEN** a reviewer checks the heading order
- **THEN** that heading is allowed

#### Scenario: No deprecation means no Deprecated heading

- **GIVEN** a changelog that marks nothing for later removal
- **WHEN** a reviewer reads the headings
- **THEN** there is no `### Deprecated` heading

### Requirement: A bullet links the merging pull request

A bullet added after this change MUST link the pull request that merges the change it records, as `([#N](https://github.com/fputs/genesis/pull/N))`, where N is that pull request's number. A re-cut pull request MUST set N to its own number before merge. A link that still names a predecessor MUST NOT satisfy the re-cut pull request. The Changed bullet "Genesis v2 restart. v1 is preserved on branch `v1-reference`." is grandfathered and needs no link.

#### Scenario: The spec-drift spec bullet links pull request 10

- **GIVEN** the Unreleased bullet `` `openspec/specs/spec-drift-check` spec, archived from the add-spec-drift-check change. ``
- **AND** that bullet has no pull request link
- **AND** pull request 10 merged that spec
- **WHEN** the changelog is brought in line with this requirement
- **THEN** the bullet contains `([#10](https://github.com/fputs/genesis/pull/10))`

#### Scenario: A re-cut pull request updates its link

- **GIVEN** a bullet that links pull request 10
- **AND** that change was not merged
- **AND** the same change is re-cut as pull request 80
- **WHEN** pull request 80 is ready to merge
- **THEN** the bullet links pull request 80
- **AND** the bullet does not link pull request 10

#### Scenario: The v2 restart bullet is grandfathered

- **GIVEN** the Unreleased bullet under `### Changed` whose text is "Genesis v2 restart. v1 is preserved on branch `v1-reference`."
- **AND** that bullet has no pull request link
- **WHEN** the changelog is brought in line with this requirement
- **THEN** that bullet is unchanged
- **AND** it still has no pull request link

### Requirement: DevOps cuts releases

DevOps cuts a release by adding one heading `## [x.y.z] - YYYY-MM-DD` and moving the bullets then under `## [Unreleased]` into that version section without adding bullet text, and `docs/producer/process.md` MUST name DevOps as the owner of that cut.

#### Scenario: An implementation pull request leaves version headings alone

- **GIVEN** an implementation pull request
- **WHEN** a reviewer reads the `CHANGELOG.md` diff
- **THEN** no version heading is added
- **AND** no bullet moves out of `## [Unreleased]`

#### Scenario: DevOps moves Unreleased into a version

- **GIVEN** `## [Unreleased]` has bullets
- **AND** DevOps cuts version 0.2.0 on 2026-10-23
- **WHEN** the release change is read
- **THEN** those bullets sit under `## [0.2.0] - 2026-10-23`
- **AND** they are no longer under `## [Unreleased]`
- **AND** the release change adds no new bullet text

#### Scenario: The producer process names DevOps

- **GIVEN** `docs/producer/process.md` after this change is implemented
- **WHEN** a reviewer reads the Changelog bullet
- **THEN** it says DevOps cuts the version and moves Unreleased bullets into the version section
- **AND** it still says the implementation pull request writes the Unreleased entry
- **AND** it does not describe a separate post-merge changelog pull request

### Requirement: Mixed pull request category

A pull request with no behavior path needs no bullet. A behavior path is any changed path the skip requirement does not exempt. A pull request with a behavior path MUST add a bullet in each fitting section: `### Added`, `### Changed`, `### Deprecated`, `### Removed`, `### Fixed`, or `### Security`. It needs no bullet for an exempt path.

#### Scenario: A mixed spec and docs pull request skips the bullet

- **GIVEN** a pull request whose paths are only `openspec/specs/changelog/spec.md` and `docs/producer/process.md`
- **WHEN** a reviewer decides whether a new bullet is required
- **THEN** no new bullet is required

#### Scenario: A security change beside docs is filed under Security

- **GIVEN** a pull request that restricts a workflow token to read-only and also edits `docs/producer/process.md`
- **AND** it does not add, remove, or deprecate caller-visible behavior
- **WHEN** a reviewer reads `## [Unreleased]`
- **THEN** the only new bullet is under `### Security`
- **AND** no bullet is required for the docs path

#### Scenario: A removal and a deprecation use two sections

- **GIVEN** a pull request that deletes one caller-visible behavior and marks a different one for later removal while leaving it working
- **AND** it does not restrict a token, permission, credential, or vulnerability
- **WHEN** a reviewer reads `## [Unreleased]`
- **THEN** one bullet is under `### Removed` and another is under `### Deprecated`

#### Scenario: A deprecation stays under Deprecated

- **GIVEN** a pull request that marks caller-visible behavior for later removal and leaves that behavior working
- **AND** it does not restrict a token, permission, credential, or vulnerability, and it does not delete or add behavior
- **WHEN** a reviewer reads the new bullet
- **THEN** the only new bullet is under `### Deprecated`

#### Scenario: An addition and a fix use two sections

- **GIVEN** a pull request that adds caller-visible behavior and also corrects other behavior to match its already accepted spec
- **AND** it does not restrict a token, permission, credential, or vulnerability, and it does not remove or deprecate behavior
- **WHEN** a reviewer reads `## [Unreleased]`
- **THEN** one bullet is under `### Added` and another is under `### Fixed`

#### Scenario: A fix is not filed as Changed

- **GIVEN** a pull request whose only behavior change corrects behavior so it matches its already accepted spec
- **WHEN** a reviewer reads the new bullet
- **THEN** the only new bullet is under `### Fixed`

#### Scenario: Any other behavior change is Changed

- **GIVEN** a pull request that changes existing caller-visible behavior
- **AND** it does not add, remove, or deprecate behavior, it is not a fix, and it does not restrict a token, permission, credential, or vulnerability
- **WHEN** a reviewer reads the new bullet
- **THEN** the only new bullet is under `### Changed`

#### Scenario: Spec plus behavior files one Added bullet

- **GIVEN** a pull request whose paths include `openspec/specs/changelog/spec.md`, `docs/producer/process.md`, and `crates/rules/src/lib.rs`
- **AND** the crate change only adds caller-visible behavior
- **WHEN** a reviewer reads `## [Unreleased]`
- **THEN** one new bullet is under `### Added`
- **AND** no second bullet is required for the spec or docs paths

### Requirement: Tests-only pull request

A pull request that changes only tests under `crates/`, or that has no caller-visible effect, needs no bullet. It MAY add one bullet, and that bullet MUST be under `### Added`.

#### Scenario: A tests-only pull request needs no bullet

- **GIVEN** a pull request whose only diff is `crates/rules/src/encounter/tests/delay_ready_tests.rs`
- **WHEN** a reviewer decides whether a new bullet is required
- **THEN** no new bullet is required
- **AND** a bullet it adds is under `### Added`
