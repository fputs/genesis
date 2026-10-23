# Design

## Context

`CHANGELOG.md` follows Keep a Changelog 1.1.0 and has `## [Unreleased]` only. One Unreleased bullet, the spec-drift spec line, has no pull-request link. `add-changelog` task 2.1 required that wording. Pull request 10 merged `openspec/specs/spec-drift-check/spec.md`. See proposal.md for why that contradiction is in scope. Normative behavior is `specs/changelog/spec.md` and `specs/pr-hygiene/spec.md`.

`scripts/pr-hygiene.mjs` treats any new unfenced list item under `## [Unreleased]` as enough. It does not read `pull_request.number`. Most fixture rows never write an event payload, so that number is absent unless the row already needs a payload. When a payload is written, `scripts/pr-hygiene.test.mjs` sets `pull_request.number` to 7.

Genesis DevOps agent implements the script and the changelog text. This pull request does not.

## Goals / Non-Goals

**Goals:**

- A Deprecated heading in Keep a Changelog order, present only when a deprecation is recorded.
- Every bullet links the pull request that merged the change it records. A re-cut pull request updates that number before merge.
- DevOps is the only role that adds a version heading and moves Unreleased bullets into it.
- A mixed pull request is classified by whether it has a behavior path, and each fitting behavior section gets a bullet.
- The job rejects a required bullet that lacks one of the six headings or lacks this pull request's number.

**Non-Goals:**

- Choosing Added versus Fixed inside the script. The scenarios say which section fits. The job checks that the heading is one of the six.
- A SemVer bump table. The cut names the version DevOps chose and uses `YYYY-MM-DD`.
- Dropping the existing exemptions for `docs/`, `openspec/`, `AGENTS.md`, or `CHANGELOG.md`.
- Editing the Cargo job or spec-drift.
- An empty `### Deprecated` heading on the current changelog.

## Decisions

### 1. Heading order, and no empty Deprecated section

Keep a Changelog 1.1.0 orders the category headings Added, Changed, Deprecated, Removed, Fixed, Security. A missing heading is skipped, so a changelog with no deprecation has no `### Deprecated` line. The implementation does not insert an empty one.

Alternative considered: always emit all six headings. Rejected. The format omits empty sections, and an empty Deprecated heading would be a standing diff with nothing deprecated.

### 2. The link is the merging pull request

The markdown already used in `CHANGELOG.md` is the required form: `([#N](https://github.com/fputs/genesis/pull/N))`. N is the pull request that merged the change the bullet records. For a pull request that is still open, N is that pull request's own number, because that is the pull request that will merge it. A re-cut is a new pull request for a change whose earlier pull request did not merge. Before the new one merges, N is the new number. A predecessor number does not count.

The unlinked spec-drift spec bullet records the spec pull request 10 merged. The implementation appends `([#10](https://github.com/fputs/genesis/pull/10))` to that line and does not otherwise reword it. Appending the link does not add a list item, so the edited line does not satisfy the new-bullet check. The implementation pull request still adds its own bullet.

The job looks for `https://github.com/fputs/genesis/pull/N` in the bullet text, where N is the decimal `pull_request.number` and the next character is not a digit. `/10` does not match pull request 100, and `/80` does not match a link to 8.

Alternative considered: accept any `(#N)` without the URL. Rejected. The file's existing links are the URL form, and a bare number is easier to confuse with a version.

### 3. DevOps owns the version cut

`docs/producer/process.md` gains one statement in the Changelog bullet: DevOps adds `## [x.y.z] - YYYY-MM-DD` and moves the bullets then under `## [Unreleased]` into that section, and no other role does. The sentences that the implementation pull request writes the Unreleased entry, and that there is no separate post-merge changelog pull request, stay. The move copies existing bullets. It does not write new bullet text.

Alternative considered: a GitHub release workflow that rewrites the changelog. Rejected. The backlog assigns the cut to DevOps, and this change does not add a release workflow.

### 4. Mixed pull requests use the behavior sections

A behavior path is any changed path the skip requirement does not exempt. No behavior path means no bullet, including a pull request that touches both `openspec/` and `docs/`. A behavior path means one bullet in each section that fits that behavior, and no bullet for an exempt path. The scenarios in `specs/changelog/spec.md` are the fit rules. The existing scenario "Two kinds of change use two sections" stays: an addition plus a security tightening is still two bullets.

Alternative considered: one precedence winner and a single bullet. Rejected. That would delete the two-section scenario, and a modified requirement cannot drop a scenario the main spec still has.

### 5. The job checks heading and number only

`parseChangelog` tracks the current unfenced `###` heading while it is inside `## [Unreleased]`. A new bullet counts for this check only when that heading is one of the six names and the bullet text contains the URL for this pull request's number. At least one such bullet is required when `needsBullet` is true. The existing rules stay: the text is absent from the base file, the Unreleased count rose, fenced lines are ignored, and an edit that does not raise the count does not count.

The number comes from `pull_request.number` on the event payload. The test harness sets it the same way it sets the shas, including a `PR_HYGIENE_NUMBER` override, so rows that never wrote a payload can still name 7. When a bullet is required and neither the override nor the payload has a number, the job fails closed with `implementation PR missing CHANGELOG bullet`. An exempt pull request still passes with no number and no bullet.

Every fixture whose job passes because a bullet was added uses a valid heading and that number's URL. That includes archive and draft rows whose scenario text only says "adds an Unreleased bullet". The scenario "Any Unreleased list item passes" keeps its name, because dropping it would fail validation, and its body now requires `### Added` and the pull request's own link. "A bullet with no link fails" is the old linkless case.

Alternative considered: call the GitHub API for the pull request number. Rejected. The event payload already has it, and the workflow stays `contents: read` with no extra permission.

## Risks / Trade-offs

- [Passing fixtures use linkless bullets and often no payload] → Give each passing added bullet a valid heading and `https://github.com/fputs/genesis/pull/` plus the harness number. Set the number in the harness, and omit it only for "A missing pull request number fails closed".
- [The kept scenario title "Any Unreleased list item passes" is wider than its new body] → The body is the pass case. The linkless failure is a separate scenario and test. Do not weaken the body to match the old title.
- [The implementation pull request both repairs the #10 line and adds its own bullet] → The repair does not raise the bullet count. The new bullet does, and it links that pull request's own number. The changelog scenario "Repairing the pull request 10 link still needs this pull request's bullet" is that case.
- [Strict validation fails a requirement description past 500 characters] → Each description in this change stays at or under 500. Detail lives in the scenarios.

## Migration Plan

No stored data. The script change and the #10 link land in the implementation pull request. Rollback is reverting that pull request. Version sections do not exist yet, so nothing moves out of Unreleased in this change.

## Open Questions

None. The link form, the #10 repair, the exempt set, and the six headings are decided above.
