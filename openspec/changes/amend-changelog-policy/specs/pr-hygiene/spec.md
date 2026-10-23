# pr-hygiene

## MODIFIED Requirements

### Requirement: Changelog bullet shape

A bullet SHALL be an unfenced trimmed list item in the first unfenced `## [Unreleased]` section, before the next unfenced `## ` heading. Its text is that line without the list marker, trimmed. It SHALL count as new only when that text is absent from the base and the head has more bullets. Edited or moved bullets SHALL NOT count. A missing base file or heading SHALL leave the base set empty. Matching SHALL be case-sensitive. `####` SHALL NOT reset the category. `## [Unreleased]` SHALL reset it.

#### Scenario: An unchanged existing bullet fails

- **GIVEN** a pull_request whose diff changes `Cargo.toml`
- **AND** the only Unreleased list item is already on the base
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

#### Scenario: A missing base changelog leaves no bullets

- **GIVEN** an implementation pull request whose base has no `CHANGELOG.md`, or whose base `CHANGELOG.md` has no `## [Unreleased]` heading
- **AND** the head `## [Unreleased]` section has a list item under `### Added` that links the pull request's own number
- **WHEN** the `pr-hygiene` job runs
- **THEN** the base bullet set is empty
- **AND** the job passes

#### Scenario: A moved bullet is not new

- **GIVEN** a pull_request whose diff changes `crates/rules/src/lib.rs`
- **AND** the base `CHANGELOG.md` has the bullet text `Record the gate` under `## [0.1.0]`
- **AND** the head places that same text under `## [Unreleased]`
- **AND** the head adds no bullet text absent from the base file
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

#### Scenario: An edited bullet is not new

- **GIVEN** a pull_request whose diff changes `Cargo.toml`
- **AND** the base `## [Unreleased]` section's only bullet is `- Record the gate`
- **AND** the head changes that bullet to `- Record the gate today`
- **AND** the head section has the same number of bullets as the base section
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

#### Scenario: A lowercase category heading fails

- **GIVEN** a pull_request whose diff changes `crates/rules/src/lib.rs`
- **AND** the only new bullet sits under `### added` inside `## [Unreleased]`
- **AND** that bullet links the pull request's own number
- **AND** the head section has more bullets than the base section
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

#### Scenario: A level-four heading does not set the category

- **GIVEN** a pull_request whose diff changes `crates/rules/src/lib.rs`
- **AND** `## [Unreleased]` opens with no `###` heading
- **AND** the only heading before the new bullet is `#### Added`
- **AND** that bullet links the pull request's own number
- **AND** the head section has more bullets than the base section
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

#### Scenario: A level-four heading does not reset the category

- **GIVEN** a pull_request whose diff changes `crates/rules/src/lib.rs`
- **AND** `### Added` is open inside `## [Unreleased]`
- **AND** a `#### Removed` heading follows that `### Added` and precedes the new bullet
- **AND** that bullet links the pull request's own number
- **AND** the bullet text is absent from the base changelog
- **AND** the head section has more unfenced bullets than the base section
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes

#### Scenario: The category resets when Unreleased opens

- **GIVEN** a pull_request whose diff changes `crates/rules/src/lib.rs`
- **AND** `### Added` appears before `## [Unreleased]`
- **AND** the only new bullet sits inside `## [Unreleased]` with no `###` heading after that open
- **AND** that bullet links the pull request's own number
- **AND** the head section has more bullets than the base section
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

### Requirement: Fenced changelog blocks

Fenced lines SHALL be ignored when finding `## [Unreleased]` and when collecting bullets in either changelog. A trimmed line of three or more backticks SHALL toggle the fence. A fenced heading SHALL NOT open or close the section. A fenced list item SHALL NOT be a bullet.

#### Scenario: A fenced bullet is not new

- **GIVEN** a pull_request whose diff changes `crates/rules/src/lib.rs`
- **AND** the only list item added under `## [Unreleased]` sits inside a fenced code block
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

#### Scenario: A fenced heading does not open the section

- **GIVEN** an implementation pull request
- **AND** the only `## [Unreleased]` line sits inside a fenced code block
- **AND** no unfenced `## [Unreleased]` heading exists
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

#### Scenario: A fenced heading does not close the section

- **GIVEN** an implementation pull request
- **AND** a fenced `## [0.1.0]` sits inside the unfenced `## [Unreleased]` section
- **AND** after that fence the section has a new bullet under `### Added` that links the pull request's own number and whose text is absent from the base changelog
- **AND** the head section has more unfenced bullets than the base section
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes

### Requirement: Bullet exemptions

The bullet check SHALL pass with no new bullet when every changed path is exempt. Exempt paths are those under `docs/`, `openspec/`, or `.cursor/`, plus exactly `AGENTS.md`, `CHANGELOG.md`, `README.md`, `Cargo.lock`, and `rust-toolchain.toml`. Categories SHALL combine. Any other path SHALL remove the exemption, except the tests-only skip in Tests-only pull request.

#### Scenario: Docs-only passes without a bullet

- **GIVEN** a pull_request whose diff is `docs/architect/decisions.md`, `AGENTS.md`, and `CHANGELOG.md`
- **AND** no Unreleased bullet is added
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes

#### Scenario: Openspec-only passes without a bullet

- **GIVEN** a pull_request whose diff is only under `openspec/`
- **AND** no Unreleased bullet is added
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes

#### Scenario: A cursor-only pull request passes without a bullet

- **GIVEN** a pull_request whose only diff is `.cursor/rules/example.md`
- **AND** no Unreleased bullet is added
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes

#### Scenario: A README-only pull request passes without a bullet

- **GIVEN** a pull_request whose only diff is `README.md`
- **AND** no Unreleased bullet is added
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes

#### Scenario: A Cargo.lock-only pull request passes without a bullet

- **GIVEN** a pull_request whose only diff is `Cargo.lock`
- **AND** no Unreleased bullet is added
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes

#### Scenario: A rust-toolchain-only pull request passes without a bullet

- **GIVEN** a pull_request whose only diff is `rust-toolchain.toml`
- **AND** no Unreleased bullet is added
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes

#### Scenario: The exempt path set together passes without a bullet

- **GIVEN** a pull_request whose diff is `.cursor/rules/example.md`, `README.md`, `Cargo.lock`, and `rust-toolchain.toml`
- **AND** no Unreleased bullet is added
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes

#### Scenario: Exempt categories together pass without a bullet

- **GIVEN** a pull_request whose diff is `docs/architect/decisions.md`, `openspec/specs/changelog/spec.md`, `.cursor/rules/example.md`, `README.md`, `Cargo.lock`, and `rust-toolchain.toml`
- **AND** no Unreleased bullet is added
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes

#### Scenario: Docs plus a crate still needs a bullet

- **GIVEN** a pull_request whose diff changes `docs/architect/decisions.md` and `crates/rules/src/lib.rs`
- **AND** no Unreleased bullet is added
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

#### Scenario: Openspec plus a workflow file still needs a bullet

- **GIVEN** a pull_request whose diff changes `openspec/specs/changelog/spec.md` and `.github/workflows/ci.yml`
- **AND** no Unreleased bullet is added
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

#### Scenario: Cursor plus a crate still needs a bullet

- **GIVEN** a pull_request whose diff changes `.cursor/rules/example.md` and `crates/rules/src/lib.rs`
- **AND** no Unreleased bullet is added
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

#### Scenario: README plus root Cargo.toml still needs a bullet

- **GIVEN** a pull_request whose diff changes `README.md` and `Cargo.toml`
- **AND** no Unreleased bullet is added
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

#### Scenario: Cargo.lock plus a crate still needs a bullet

- **GIVEN** a pull_request whose diff changes `Cargo.lock` and `crates/rules/src/lib.rs`
- **AND** no Unreleased bullet is added
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

#### Scenario: rust-toolchain.toml plus a workflow file still needs a bullet

- **GIVEN** a pull_request whose diff changes `rust-toolchain.toml` and `.github/workflows/ci.yml`
- **AND** no Unreleased bullet is added
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

## RENAMED Requirements

- FROM: `### Requirement: Unreleased changelog bullet`
- TO: `### Requirement: Unreleased changelog bullet retired`

## REMOVED Requirements

### Requirement: Unreleased changelog bullet retired

## ADDED Requirements

### Requirement: Unreleased changelog bullet

A non-exempt implementation PR or any `scripts/` path SHALL add one new bullet, else the job SHALL fail with `implementation PR missing CHANGELOG bullet`. Missing head `CHANGELOG.md` or `## [Unreleased]` SHALL fail the same way. It SHALL use Added, Changed, Deprecated, Removed, Fixed, or Security, contain `https://github.com/fputs/genesis/pull/N`, not a following digit. N is `pull_request.number` from the event payload. Only the URL counts, not surrounding markdown. No N SHALL fail the same way.

#### Scenario: An added Unreleased bullet passes

- **GIVEN** a pull_request whose diff changes `crates/rules/src/lib.rs`
- **AND** the head `## [Unreleased]` section has a bullet under `### Added` whose text appears nowhere in the base `CHANGELOG.md`
- **AND** that bullet links that pull request's own number
- **AND** that section has more bullets than the base section
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes

#### Scenario: A missing bullet fails

- **GIVEN** a pull_request whose diff changes `crates/rules/src/lib.rs`
- **AND** the Unreleased list items on the head match the base
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

#### Scenario: A bullet outside Unreleased fails

- **GIVEN** a pull_request whose diff changes `crates/rules/src/lib.rs`
- **AND** the only new list item is under `## [0.1.0]`
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

#### Scenario: An Added bullet linking pull request 7 passes

- **GIVEN** a pull_request whose diff changes `.github/workflows/pr-hygiene.yml`
- **AND** the pull request number is 7
- **AND** the base `CHANGELOG.md` has no bullet text `Record the gate`
- **AND** the new line under `### Added` in `## [Unreleased]` links `https://github.com/fputs/genesis/pull/7`
- **AND** the head section has more bullets than the base section
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes

#### Scenario: A bullet with no link fails

- **GIVEN** a pull_request whose diff changes `.github/workflows/pr-hygiene.yml`
- **AND** the pull request number is 7
- **AND** the base `CHANGELOG.md` has no bullet text `Record the gate`
- **AND** the new line under `### Added` in `## [Unreleased]` is `- Record the gate` with no pull request link
- **AND** the head section has more bullets than the base section
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

#### Scenario: A bullet under an unknown heading fails

- **GIVEN** a pull_request whose diff changes `crates/rules/src/lib.rs`
- **AND** the only new bullet sits under `### Notes` inside `## [Unreleased]`
- **AND** that bullet links the pull request's own number
- **AND** the head section has more bullets than the base section
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

#### Scenario: A bullet that links another pull request fails

- **GIVEN** a pull_request numbered 80 whose diff changes `crates/rules/src/lib.rs`
- **AND** the only new bullet is under `### Added` and contains `https://github.com/fputs/genesis/pull/10`
- **AND** the head section has more bullets than the base section
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

#### Scenario: Pull request 8 linking pull 80 fails

- **GIVEN** a pull_request numbered 8 whose diff changes `crates/rules/src/lib.rs`
- **AND** the only new bullet is under `### Added` and contains `https://github.com/fputs/genesis/pull/80`
- **AND** the head section has more bullets than the base section
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

#### Scenario: A bullet with no category fails

- **GIVEN** a pull_request whose diff changes `crates/rules/src/lib.rs`
- **AND** the only new list item sits directly under `## [Unreleased]` before any `###` heading
- **AND** that bullet links the pull request's own number
- **AND** the head section has more bullets than the base section
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

#### Scenario: A Deprecated bullet linking this pull request passes

- **GIVEN** a pull_request whose diff changes `crates/rules/src/lib.rs`
- **AND** the new bullet is under `### Deprecated` and links that pull request's own number
- **AND** the bullet text is absent from the base changelog
- **AND** the head section has more unfenced bullets than the base section
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes

#### Scenario: A missing pull request number fails closed

- **GIVEN** an implementation pull request whose event has no pull request number
- **AND** the head adds a bullet under `### Added` that contains `https://github.com/fputs/genesis/pull/7`
- **AND** the head section has more bullets than the base section
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

#### Scenario: A missing head changelog fails

- **GIVEN** an implementation pull request whose head has no `CHANGELOG.md`, or whose `CHANGELOG.md` has no `## [Unreleased]` heading
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

#### Scenario: A docs-only pull request with no changelog passes

- **GIVEN** a pull_request whose only diff is `docs/architect/decisions.md`
- **AND** the head has no `CHANGELOG.md`
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes

#### Scenario: Edited still checks the bullet

- **GIVEN** a pull_request `edited` event
- **AND** the diff changes `crates/rules/src/lib.rs`
- **AND** no Unreleased bullet is added
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

#### Scenario: A scripts-only pull request needs a changelog bullet

- **GIVEN** a pull_request whose only diff is `scripts/note.mjs`
- **AND** no Unreleased bullet is added
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `implementation PR missing CHANGELOG bullet`

### Requirement: Tests-only pull request

When every changed path is under `crates/` and contains `/tests/`, the job SHALL pass with no bullet added. The job SHALL NOT check the heading of an optional tests-only bullet. The tests-only skip applies only to the changelog bullet; the own-change-active rule still applies.

#### Scenario: A tests-only pull request passes without a bullet

- **GIVEN** a pull_request whose only diff is `crates/rules/src/encounter/tests/delay_ready_tests.rs`
- **AND** no Unreleased bullet is added
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes

#### Scenario: The job ignores a tests-only bullet heading

- **GIVEN** a pull_request whose only diff is `crates/rules/src/encounter/tests/delay_ready_tests.rs`
- **AND** the head adds one bullet under `### Fixed`
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes

#### Scenario: A ready tests-only pull request naming an active own change fails archive-last

- **GIVEN** a pull_request whose `pull_request.draft` is false
- **AND** the only diff is `crates/rules/src/encounter/tests/delay_ready_tests.rs`
- **AND** the body line is `OpenSpec-Change: add-foo`
- **AND** the merge-base and the head both have `openspec/changes/add-foo/`
- **AND** no Unreleased bullet is added
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job fails
- **AND** the log includes `archive commit is not last`
- **AND** the log does not include `implementation PR missing CHANGELOG bullet`
