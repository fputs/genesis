# Proposal

## Why

The changelog spec omits Keep a Changelog's Deprecated section, and it disagrees with the unlinked spec-drift bullet that `add-changelog` task 2.1 required. Release cuts and mixed pull-request categories still sit in a backlog in `docs/architect/decisions.md`. The `pr-hygiene` job still accepts a bullet with no category and no pull-request number.

## What Changes

- Amend `changelog` so `### Deprecated` is a valid section, in Keep a Changelog 1.1.0 order, and so an empty changelog does not invent that heading.
- Each bullet links the pull request that merged the change it records, as `([#N](https://github.com/fputs/genesis/pull/N))`. A re-cut pull request sets N to its own number before merge. The unlinked spec-drift spec bullet records the spec pull request 10 merged, so that bullet links 10. That removes the contradiction with `add-changelog` task 2.1.
- DevOps alone cuts a version: it adds `## [x.y.z] - YYYY-MM-DD` and moves the Unreleased bullets into that section. No other role does.
- A mixed pull request with no behavior path adds no bullet. One with a behavior path adds a bullet in each fitting section for that behavior (Security, Removed, Deprecated, Added, Fixed, Changed) and adds none for an exempt path. Two fitting sections are still two bullets.
- When a bullet is required, the `pr-hygiene` job fails unless a new bullet is under one of those six headings and links that pull request's own number. A pull request whose paths are only under `.cursor/`, or are exactly `README.md`, `Cargo.lock`, or `rust-toolchain.toml`, stays exempt. DevOps implements that check later. This pull request does not.
- This pull request removes the backlog section in `docs/architect/decisions.md` and marks those items as moved into this spec. It adds no changelog bullet and does not archive.

This change does not add a ruling. The log is [docs/designer/rules-rulings.md](../../../docs/designer/rules-rulings.md).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `changelog`: Deprecated section, the merging pull-request link, DevOps release cuts, the mixed pull-request category, and the CI bullet shape.
- `pr-hygiene`: A required bullet must sit under a valid category and link the pull request's own number. A linkless list item no longer passes.

## Impact

- The implementation edits `scripts/pr-hygiene.mjs`, `scripts/pr-hygiene.test.mjs`, the unlinked spec-drift line in `CHANGELOG.md`, and the Changelog bullet in `docs/producer/process.md`. No new crate and no new dependency. `crates/rules` is untouched.
- Genesis DevOps agent implements the job change. This pull request is the spec and the decisions note.
- The implementation pull request adds one Unreleased bullet linking its own number, then archives with `openspec archive amend-changelog-policy --yes` as its last commit.
