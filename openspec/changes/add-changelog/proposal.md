# Proposal

## Why

`CHANGELOG.md` already exists, but two flows describe who writes an entry. The producer doc says Producer drafts one after a pull request merges, and Architect lands it in a later pull request. That post-merge step conflicts with writing the entry in the implementation pull request before the work is done. This change keeps one flow, the implementation pull request, and retires the old one. It does not add a CI check.

## What Changes

- Keep root `CHANGELOG.md` in Keep a Changelog 1.1.0. Add a short header sentence that v1 history lives on the `v1-reference` branch.
- Every merged implementation pull request adds a bullet under `## [Unreleased]`, in Added, Changed, Fixed, Removed, or Security, and links the pull request. That bullet is part of the definition of done.
- Spec-only, docs-only, and archive-only pull requests do not need a bullet.
- The later implementation adds one line to `AGENTS.md` section 2, updates section 10, and updates `docs/producer/process.md` so they describe that same flow.
- Retire the old post-merge changelog step wherever a living doc still describes it. On main today that is two sentences in `docs/producer/process.md`: Producer drafts an entry after merge, and Architect lands it in a later pull request.
- The later implementation adds the first real entry for pull request #10.
- No CI enforcement in this change.

## Capabilities

### New Capabilities

- `changelog`: The changelog file's header, the Unreleased entry an implementation pull request must add, the pull requests that skip that entry, and the retirement of the separate post-merge changelog step.

### Modified Capabilities

None. `project-foundation` still requires the file and the Unreleased note about the v2 restart. `coding-constitution` still records the coding rules. This change does not rewrite either requirement.

## Impact

- Later implementation edits root `CHANGELOG.md`, `AGENTS.md` sections 2 and 10, and `docs/producer/process.md`. It does not edit CI or Rust code, and it does not rewrite archived OpenSpec changes.
- This pull request is the spec only. It adds `openspec/changes/add-changelog/` and does not create the changelog entry yet.
- Reviewers check the scenarios by reading the pull request diff, `CHANGELOG.md`, `AGENTS.md`, and `docs/`. There is no new test and no new CI job.
