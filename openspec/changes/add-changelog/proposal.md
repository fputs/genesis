# Proposal

## Why

`CHANGELOG.md` already exists, but nothing says which pull requests must update it, which section the bullet goes in, or that the bullet is part of done. An implementation pull request can merge with no record of what changed. This change writes that rule so a reviewer can check the file. It does not add a CI check.

## What Changes

- Keep root `CHANGELOG.md` in Keep a Changelog 1.1.0. Add a short header sentence that v1 history lives on the `v1-reference` branch.
- Every merged implementation pull request adds a bullet under `## [Unreleased]`, in Added, Changed, Fixed, Removed, or Security, and links the pull request. That bullet is part of the definition of done.
- Spec-only, docs-only, and archive-only pull requests do not need a bullet.
- The later implementation adds one line to `AGENTS.md` section 2, and adds the first real entry for pull request #10.
- No CI enforcement in this change.

## Capabilities

### New Capabilities

- `changelog`: The changelog file's header, the Unreleased entry an implementation pull request must add, and the pull requests that skip that entry.

### Modified Capabilities

None. `project-foundation` still requires the file and the Unreleased note about the v2 restart. `coding-constitution` still records the coding rules. This change does not rewrite either requirement.

## Impact

- Later implementation edits root `CHANGELOG.md` and adds one line to `AGENTS.md` section 2. It does not edit `docs/producer/process.md`, CI, or Rust code.
- This pull request is the spec only. It adds `openspec/changes/add-changelog/` and does not create the changelog entry yet.
- Reviewers check the scenarios by reading the pull request diff and `CHANGELOG.md`. There is no new test and no new CI job.
