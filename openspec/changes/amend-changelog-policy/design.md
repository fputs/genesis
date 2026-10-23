# Design

## Context

`CHANGELOG.md` follows Keep a Changelog 1.1.0 and has `## [Unreleased]` only. Two Unreleased bullets have no pull-request link. The spec-drift spec line is one: `add-changelog` task 2.1 required that wording, and pull request 10 merged `openspec/specs/spec-drift-check/spec.md`. See proposal.md for why that contradiction is in scope. The other is the Changed bullet "Genesis v2 restart. v1 is preserved on branch `v1-reference`.", which this change grandfathers. Normative behavior is `specs/changelog/spec.md` and `specs/pr-hygiene/spec.md`.

`scripts/pr-hygiene.mjs` treats any new unfenced list item under `## [Unreleased]` as enough. It does not read `pull_request.number`. Most fixture rows never write an event payload, so that number is absent unless the row already needs a payload. When a payload is written, `scripts/pr-hygiene.test.mjs` sets `pull_request.number` to 7.

Genesis DevOps agent implements the script and the changelog text. This pull request does not.

## Goals / Non-Goals

**Goals:**

- A Deprecated heading in Keep a Changelog order, present only when a deprecation is recorded.
- A bullet added after this change links the pull request that merges the change it records. A re-cut pull request updates that number before merge. The Changed bullet "Genesis v2 restart. v1 is preserved on branch `v1-reference`." is grandfathered and needs no link.
- Only DevOps MAY add a version heading; no other role MAY. DevOps cuts a release by adding the version heading and moving Unreleased bullets into it.
- A mixed pull request with no behavior path needs no bullet. One with a behavior path adds a bullet in each fitting section, in Keep a Changelog order, with no priority ranking.
- Author guidance, not CI-enforced: a pull request that changes only tests under `crates/`, or that has no caller-visible effect, needs no bullet and may add one under Added. The job skips the bullet only when every changed path is under `crates/` and contains `/tests/`, and it does not check that optional bullet's heading. The tests-only skip applies only to the changelog bullet; the own-change-active rule still applies.
- The job rejects a required bullet that lacks one of the six headings or lacks this pull request's URL. N is `pull_request.number` from the event payload.

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

The markdown already used in `CHANGELOG.md` is the required form for a bullet added after this change: `([#N](https://github.com/fputs/genesis/pull/N))`. N is the pull request that merged the change the bullet records. For a pull request that is still open, N is that pull request's own number, because that is the pull request that will merge it. A re-cut is a new pull request for a change whose earlier pull request did not merge. Before the new one merges, N is the new number. A predecessor number does not count.

The Changed bullet "Genesis v2 restart. v1 is preserved on branch `v1-reference`." is grandfathered. The implementation leaves that line unlinked.

The unlinked spec-drift spec bullet records the spec pull request 10 merged. The implementation appends `([#10](https://github.com/fputs/genesis/pull/10))` to that line and does not otherwise reword it. Appending the link does not add a list item, so the edited line does not satisfy the new-bullet check. The implementation pull request still adds its own bullet. The changelog scenario "The spec-drift spec bullet links pull request 10" is that repair.

The job checks only the URL `https://github.com/fputs/genesis/pull/N`. Surrounding markdown is not checked. N is `pull_request.number` from the event payload, and the next character is not a digit. A link to pull request 80 does not satisfy pull request 8. When a bullet is required and the payload has no number, the job fails closed with `implementation PR missing CHANGELOG bullet`.

Alternative considered: accept any `(#N)` without the URL. Rejected. The file's existing links are the URL form, and a bare number is easier to confuse with a version.

### 3. DevOps owns the version cut

`docs/producer/process.md` gains one statement in the Changelog bullet: DevOps adds `## [x.y.z] - YYYY-MM-DD` and moves the bullets then under `## [Unreleased]` into that section. Only DevOps MAY add a version heading; no other role MAY. The sentences that the implementation pull request writes the Unreleased entry, and that there is no separate post-merge changelog pull request, stay. The move copies existing bullets. It does not write new bullet text.

Alternative considered: a GitHub release workflow that rewrites the changelog. Rejected. The backlog assigns the cut to DevOps, and this change does not add a release workflow.

### 4. Mixed pull requests use the behavior sections

A behavior path is any changed path the skip requirement does not exempt. No behavior path means the pull request needs no bullet, including a pull request that touches both `openspec/` and `docs/`. A behavior path means one bullet in each fitting section, listed in Keep a Changelog order, and it needs no bullet for an exempt path. That order is not a priority ranking. The scenarios in `specs/changelog/spec.md` are the fit rules. An addition plus a security tightening is still two bullets, so the scenario "Two kinds of change use two sections" stays.

Alternative considered: one precedence winner and a single bullet. Rejected. An addition and a security tightening are two changes, so they stay two bullets.

### 5. The job checks heading and number only

A new bullet counts for this check only when its unfenced category heading is one of the six names and the bullet text contains the URL for this pull request's number. At least one such bullet is required when a bullet is required. The existing rules stay: the text is absent from the base file, the Unreleased count rose, fenced lines are ignored, and an edit that does not raise the count does not count.

Heading matching is case-sensitive. A `####` heading does not change the category. The category resets when `## [Unreleased]` opens. The number is `pull_request.number` from the event payload. When a bullet is required and the payload has no number, the job fails closed with `implementation PR missing CHANGELOG bullet`. An exempt pull request still passes with no number and no bullet. A tests-only pull request skips the bullet through a separate `isTestsOnly` check inside `missingBullet`. `needsBullet` and `nonExemptImplementation` stay unchanged, so the own-change-active rule still applies. The tests-only skip applies only to the changelog bullet. How the parser tracks the heading is in `tasks.md`.

The scenario "An Added bullet linking pull request 7 passes" names that pass case: pull request 7, a new line under `### Added`, and a link to `https://github.com/fputs/genesis/pull/7`. "A bullet with no link fails" is the old linkless case. Every fixture whose job passes because a bullet was added uses a valid heading and that number's URL. That includes archive and draft rows whose scenario text only says "adds an Unreleased bullet".

Alternative considered: call the GitHub API for the pull request number. Rejected. The event payload already has it, and the workflow stays `contents: read` with no extra permission.

## Risks / Trade-offs

- [Passing fixtures use linkless bullets and often no payload] → Give each passing added bullet a valid heading and `https://github.com/fputs/genesis/pull/` plus the harness number. Write `pull_request.number` on the event payload, and omit it only for "A missing pull request number fails closed".
- [The implementation pull request both repairs the #10 line and adds its own bullet] → The repair does not raise the bullet count. The new bullet does, and it links that pull request's own number. The changelog scenario "The spec-drift spec bullet links pull request 10" is that repair.
- [A change with no caller-visible effect may not sit under `/tests/`] → That case is author guidance in the changelog spec and is not CI-enforced. The job skips the bullet only when every changed path is under `crates/` and contains `/tests/`.
- [Strict validation fails a requirement description past 500 characters] → Each description in this change stays at or under 500. Detail lives in the scenarios.

## Migration Plan

No stored data. The script change and the #10 link land in the implementation pull request. Rollback is reverting that pull request. Version sections do not exist yet, so nothing moves out of Unreleased in this change.

## Open Questions

None. The link form, the #10 repair, the exempt set, and the six headings are decided above.
