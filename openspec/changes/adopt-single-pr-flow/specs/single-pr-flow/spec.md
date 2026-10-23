# single-pr-flow

## Purpose

One OpenSpec change travels on one draft pull request, from the spec commit through tests, code, archive, and merge.

## ADDED Requirements

### Requirement: One draft pull request

A change SHALL be one draft pull request. The body SHALL include `OpenSpec-Change: <id>`. The same branch SHALL carry the spec, the tests, and the code. The change SHALL NOT open a second pull request for its implementation.

#### Scenario: The change stays on one draft pull request

- **GIVEN** change `adopt-single-pr-flow`
- **WHEN** the work is opened
- **THEN** it is one draft pull request whose body includes `OpenSpec-Change: adopt-single-pr-flow`
- **AND** tests and code are later commits on that branch
- **AND** there is no second pull request for that change

### Requirement: Spec-only first commit

The first commit SHALL add, modify, or delete paths only under `openspec/changes/<id>/`. A later commit SHALL add the tests, and a commit after those tests SHALL add the code. Review SHALL check that order. No CI job SHALL enforce it.

#### Scenario: The first commit is only the change directory

- **GIVEN** a new change branch
- **WHEN** its first commit is read
- **THEN** every path is under `openspec/changes/<id>/`

#### Scenario: Tests land before the code they cover

- **GIVEN** the spec-only commit is already on the branch
- **WHEN** the commits that follow are read
- **THEN** the tests for the new behavior are committed before the code for that behavior

### Requirement: Designer rules before approval

Designer SHALL post the spec ruling on the pull request before an allowed user submits a review whose body line is `spec-approved: <sha>`. Designer and QA SHALL settle edge cases on that pull request before tests or code are committed. The pull request SHALL link `docs/designer/rules-rulings.md` and SHALL NOT copy that log.

#### Scenario: The ruling is on the pull request

- **GIVEN** a draft pull request whose only commit is the change directory
- **WHEN** Designer rules
- **THEN** the ruling is a comment on that pull request
- **AND** the pull request links `docs/designer/rules-rulings.md`
- **AND** the log text is not copied into the pull request

#### Scenario: Coding waits for the ruling and the approval

- **GIVEN** Designer has not ruled, or no allowed user has submitted `spec-approved: <sha>`
- **WHEN** a commit would add tests or code
- **THEN** that commit is not made

### Requirement: Authorized approval review

Evan SHALL approve the spec by a pull request review whose body line is `spec-approved: <sha>` and whose `user.id` is in the base authors file. The sha SHALL name the commit whose spec digest is approved. An issue comment SHALL NOT approve. A behavior or scope change SHALL go to Chief of Staff for Evan, not into this review.

#### Scenario: An allowed user id approves the spec sha

- **GIVEN** Designer has ruled on the pull request
- **WHEN** user id `239839957` submits a review whose body line is `spec-approved: <sha>` for the spec commit
- **THEN** the spec is approved at that sha

#### Scenario: A scope change is not an approval review

- **GIVEN** a behavior or scope change
- **WHEN** it needs a decision
- **THEN** it goes to Chief of Staff for Evan
- **AND** it is not recorded only as a `spec-approved` review

### Requirement: Clarification re-triggers approval

A small spec clarification SHALL be fixed in this pull request and SHALL cite Designer. Any edit to `proposal.md`, `design.md`, or `specs/**` of the change SHALL require a new `spec-approved: <sha>` review before `spec-approved` passes again.

#### Scenario: A clarification cites Designer and needs a new review

- **GIVEN** an approved spec digest
- **WHEN** the pull request edits `proposal.md`, `design.md`, or a file under `specs/` and cites Designer
- **THEN** the previous `spec-approved` review no longer matches
- **AND** a new `spec-approved: <sha>` review is required

### Requirement: Parallel review, archive, and merge

After tests and code land, Architect and QA SHALL review that head in parallel and post one combined numbered fix list. The merger SHALL archive once, with `openspec archive <id> --yes`, as the last commit, right before merge, and only after both have passed that head. Architect SHALL merge when CI is green. The change SHALL NOT be un-archived.

#### Scenario: The merger archives right before merge

- **GIVEN** Architect and QA have both passed the current head
- **AND** CI is green
- **WHEN** Architect merges
- **THEN** the last commit is `openspec archive <id> --yes`
- **AND** that commit is on this pull request
- **AND** the change is not un-archived afterward

#### Scenario: A red check sends the pull request back

- **GIVEN** CI is red on the head
- **WHEN** Architect and QA review
- **THEN** the red check is the send-back
- **AND** Architect does not merge

### Requirement: AGENTS.md section 1

`AGENTS.md` section 1 SHALL state this single-pull-request flow and SHALL drop the separate spec pull request. It SHALL say that a small spec clarification is fixed in the pull request, cites Designer, and re-triggers `spec-approved`. It SHALL keep the review, escalation, git, and port rules already in that section.

#### Scenario: Section 1 states the single pull request

- **GIVEN** `AGENTS.md` after this change
- **WHEN** section 1 is read
- **THEN** it describes one draft pull request, the spec-only first commit, and Designer ruling on it
- **AND** it describes a `spec-approved: <sha>` review, tests then code, archive by the merger, and merge on green CI
- **AND** it says a small spec clarification cites Designer and re-triggers `spec-approved`

#### Scenario: Section 1 keeps the standing rules

- **GIVEN** `AGENTS.md` after this change
- **WHEN** section 1 is read
- **THEN** it keeps one procedure family, exact tasks, and the `OpenSpec-Change` line
- **AND** it keeps parallel review, the fix list, escalation, the rulings link, and never un-archive
- **AND** it keeps the git rules and the `v1-reference` port rule
- **AND** it does not say Architect launches implementation on its own pull request

### Requirement: Config tasks rule

The `tasks` rule in `openspec/config.yaml` SHALL end with archiving the change right before merge in the same pull request. The proof commands SHALL stay in that rule.

#### Scenario: The tasks rule names the same pull request

- **GIVEN** `openspec/config.yaml` after this change
- **WHEN** the `tasks` rule is read
- **THEN** it is `Every implementation task list ends with the AGENTS.md proof commands and archiving the change right before merge in the same pull request.`

### Requirement: Workflow docs

`docs/architect/decisions.md`, `docs/producer/process.md`, and `docs/qa/test-plan.md` SHALL describe this flow. The decisions note SHALL record a 2026-10-23 revision. It SHALL list two admin steps for Evan: enable code-owner review, and make the `spec-approved` check required. The test plan SHALL NOT say that only specs merged to `main` count. Evan SHALL still not review implementation code.

#### Scenario: The docs drop the separate spec pull request

- **GIVEN** those three docs after this change
- **WHEN** they are read
- **THEN** approval is the `spec-approved: <sha>` review on this pull request
- **AND** QA reads the change on the pull request head
- **AND** `docs/qa/test-plan.md` does not say that only specs merged to `main` count
- **AND** `docs/architect/decisions.md` lists code-owner review and the required `spec-approved` check as Evan's admin steps

### Requirement: CODEOWNERS

`.github/CODEOWNERS` SHALL give `/openspec/` and `/AGENTS.md` to `@fputs`. No other owner SHALL be set on those paths. CODEOWNERS SHALL take effect only once Evan enables code-owner review and makes the `spec-approved` check required. Both are admin steps. This change SHALL NOT edit GitHub branch protection or rulesets.

#### Scenario: The two paths are owned by fputs

- **GIVEN** `.github/CODEOWNERS` after this change
- **WHEN** the owner rules are read
- **THEN** `/openspec/` is owned by `@fputs`
- **AND** `/AGENTS.md` is owned by `@fputs`
- **AND** neither path has another owner

#### Scenario: Branch protection is not in the repository change

- **GIVEN** this change
- **WHEN** its repository diff is read
- **THEN** it does not change branch protection or a ruleset
- **AND** enabling code-owner review remains Evan's admin step
- **AND** making `spec-approved` a required check remains Evan's admin step

#### Scenario: CODEOWNERS waits for those admin steps

- **GIVEN** `.github/CODEOWNERS` gives `/openspec/` and `/AGENTS.md` to `@fputs`
- **AND** code-owner review is off, or `spec-approved` is not a required check
- **WHEN** a pull request is opened
- **THEN** those owner rules are not in force
- **AND** they take effect only after both admin steps

### Requirement: Refactor behavior stays a human gate

Architect SHALL NOT merge a refactor pull request until Architect and QA have both confirmed, in the combined review, that behavior is unchanged. No CI job SHALL make that confirmation.

#### Scenario: CI green is not the behavior confirmation

- **GIVEN** a pull request whose refactor exemption holds
- **AND** CI is green
- **AND** the combined review has not confirmed that behavior is unchanged
- **WHEN** Architect considers merging
- **THEN** Architect does not merge
