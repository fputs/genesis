# pr-hygiene

## MODIFIED Requirements

### Requirement: Unreleased changelog bullet

The bullet SHALL use Added, Changed, Deprecated, Removed, Fixed, or Security. N is `pull_request.number` from the event payload. The bullet SHALL contain `https://github.com/fputs/genesis/pull/N` and not a following digit. Only that URL is checked, not surrounding markdown. No number when a bullet is required SHALL fail the job with `implementation PR missing CHANGELOG bullet`. Matching SHALL be case-sensitive, `####` SHALL NOT reset the category, and it SHALL reset when `## [Unreleased]` opens.

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

#### Scenario: Any Unreleased list item passes

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

### Requirement: Changelog bullet shape

A bullet SHALL be an unfenced trimmed markdown list item with text in the first unfenced `## [Unreleased]` section, before the next unfenced `## ` heading. Its text is that line without the list marker, trimmed. It SHALL count as new only when that text is not a bullet anywhere in the base `CHANGELOG.md` and the head section has more bullets than the base. Edited or moved bullets SHALL NOT count. A missing base file or heading SHALL leave the base set empty.

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

## ADDED Requirements

### Requirement: Tests-only pull request

A pull request that changes only tests under `crates/`, or that has no caller-visible effect, needs no bullet. The job SHALL pass when every changed path is under `crates/` and contains `/tests/` and no bullet is added. A bullet it adds MUST be under `### Added`.

#### Scenario: A tests-only pull request passes without a bullet

- **GIVEN** a pull_request whose only diff is `crates/rules/src/encounter/tests/delay_ready_tests.rs`
- **AND** no Unreleased bullet is added
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes

#### Scenario: A tests-only Added bullet passes

- **GIVEN** a pull_request whose only diff is `crates/rules/src/encounter/tests/delay_ready_tests.rs`
- **AND** the head adds one bullet under `### Added` that links that pull request's own number
- **AND** the bullet text is absent from the base changelog
- **WHEN** the `pr-hygiene` job runs
- **THEN** the job passes
