# Proposal

## Why

The changelog spec omits Keep a Changelog's Deprecated section, and it disagrees with the unlinked spec-drift bullet that `add-changelog` task 2.1 required. Release cuts and mixed pull-request categories still sit in a backlog in `docs/architect/decisions.md`. The `pr-hygiene` job still accepts a bullet with no category and no pull-request number.

## What Changes

- Amend `changelog` so `### Deprecated` is a valid section, in Keep a Changelog 1.1.0 order, and so an empty changelog does not invent that heading.
- A bullet added after this change links the pull request that merged the change it records, as `([#N](https://github.com/fputs/genesis/pull/N))`. A re-cut pull request sets N to its own number before merge. The unlinked spec-drift spec bullet records the spec pull request 10 merged, so that bullet links 10. The Changed bullet "Genesis v2 restart. v1 is preserved on branch `v1-reference`." is grandfathered and needs no link.
- DevOps cuts a release by adding one heading `## [x.y.z] - YYYY-MM-DD` and moving the bullets then under `## [Unreleased]` into that version section.
- A pull request with no behavior path needs no bullet. A pull request with a behavior path adds a bullet in each fitting section, in Keep a Changelog order, and needs no bullet for an exempt path. Two fitting sections are still two bullets. That order is not a priority ranking.
- A pull request that changes only tests under `crates/`, or that has no caller-visible effect, needs no bullet. It may add one bullet under `### Added`.
- When a bullet is required, the `pr-hygiene` job fails unless a new bullet is under one of those six headings and contains `https://github.com/fputs/genesis/pull/N`, where N is `pull_request.number` from the event payload and the next character is not a digit. The job checks only that URL. The full skip list stays in `pr-hygiene`, including `docs/`, `openspec/`, `.cursor/`, `AGENTS.md`, `CHANGELOG.md`, `README.md`, `Cargo.lock`, and `rust-toolchain.toml`. DevOps implements that check later. This pull request does not.
- This pull request removes the backlog section in `docs/architect/decisions.md` and marks those items as moved into this spec. It adds no changelog bullet and does not archive.

This change does not add a ruling. The log is [docs/designer/rules-rulings.md](../../../docs/designer/rules-rulings.md).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `changelog`: Deprecated section, the merging pull-request link for bullets added after this change, the grandfathered restart line, DevOps release cuts, the mixed pull-request category, and tests-only pull requests.
- `pr-hygiene`: A required bullet must sit under a valid category and contain this pull request's URL. Link matching, category parsing, and the tests-only skip live here. A linkless list item no longer passes.

## Impact

- The implementation edits `scripts/pr-hygiene.mjs`, `scripts/pr-hygiene.test.mjs`, the unlinked spec-drift line in `CHANGELOG.md`, and the Changelog bullet in `docs/producer/process.md`. No new crate and no new dependency. `crates/rules` is untouched.
- Genesis DevOps agent implements the job change. This pull request is the spec and the decisions note.
- The implementation pull request adds one Unreleased bullet linking its own number, then archives with `openspec archive amend-changelog-policy --yes` as its last commit.
