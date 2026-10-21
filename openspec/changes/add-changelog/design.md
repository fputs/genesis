# Design

## Context

Root `CHANGELOG.md` is already Keep a Changelog 1.1.0. Its header cites that format and Semantic Versioning. `## [Unreleased]` has one `### Changed` bullet: Genesis v2 restarted, and v1 is preserved on branch `v1-reference`. `project-foundation` still requires that Unreleased note. See proposal.md for why a written rule is needed. The behavior to check is `specs/changelog/spec.md`.

`AGENTS.md` section 2 is the definition of done (the proof commands). Section 10 says to update `CHANGELOG.md` only through `docs/producer/process.md`. That doc says Producer drafts an entry after a pull request merges, and Architect lands it in a later pull request.

## Goals / Non-Goals

**Goals:**

- Say where the v1 note sits, which section a bullet uses, and which pull requests must add one.
- Make the bullet part of done for an implementation pull request, in that same pull request, with a link to it.
- Leave a reviewer a file they can read. No new program.

**Non-Goals:**

- A CI check that fails a pull request with a missing bullet. A later change can add that to the spec-drift job. This change does not.
- Editing `docs/producer/process.md` or `AGENTS.md` section 10.
- Rewriting the `project-foundation` changelog requirement, or deleting the existing Unreleased restart bullet.
- Release headings, version numbers, or a `### Deprecated` section. Keep a Changelog 1.1.0 has Deprecated. This change does not require that heading, and it does not add an empty one. A later change can add it when something is deprecated.
- A test harness. A reviewer checks these scenarios by reading the diff.

## Decisions

### 1. A new `changelog` capability, not a constitution edit

The coding constitution records how agents write code: proof commands, purity, tests, panics, dependencies. Changelog format, section names, and which pull requests need a bullet are a documentation rule. A new `changelog` capability owns that rule.

`project-foundation` already says the file exists and that Unreleased notes the v2 restart. This change does not modify that requirement. It adds the ongoing entry rule beside it.

Alternative considered: a MODIFIED delta on `coding-constitution`. Rejected. The constitution spec would then mix a markdown file contract into the coding rules, and the format details would not have a home of their own.

### 2. The header gains one sentence. The restart bullet stays

Add one sentence above `## [Unreleased]`: v1 history lives on the `v1-reference` branch. Leave the Keep a Changelog 1.1.0 lines and the Semantic Versioning line as they are.

Leave the existing Unreleased `### Changed` restart bullet in place. `project-foundation` still says a reader of Unreleased finds the v2 restart and that v1 is preserved on `v1-reference`. Deleting that bullet would make that requirement false. This change does not modify `project-foundation`.

Alternative considered: replace `## [Unreleased]` with only the pull request #10 text, and move the v1 note out of Unreleased. Rejected, because that needs a MODIFIED delta on `project-foundation`, and the request allowed a new capability instead of rewriting an existing requirement.

### 3. The implementation pull request carries the bullet

"Done" means the bullet is already in the pull request that changes behavior, under `## [Unreleased]`, in Added, Changed, Fixed, Removed, or Security, with a link to that pull request. It is not a follow-up pull request after merge.

A pull request that changes how the software or its checks behave is an implementation pull request (code, scripts, or workflow files). Spec-only (only `openspec/`), docs-only (`docs/`, `AGENTS.md`, or `CHANGELOG.md`), and archive-only (only an archive move) do not add a bullet for themselves. An implementation pull request that archives its own change in the last commit still adds its bullet.

The five section names are the ones an entry may use. One pull request uses every section that fits. Pull request #10 is the example: Added and Security.

### 4. One new line in `AGENTS.md` section 2

The implementation adds this single line to section 2 and changes no other line in that file:

`An implementation pull request adds a changelog entry under ## [Unreleased] before the work is done. Spec-only, docs-only, and archive-only pull requests do not.`

That line is the constitution amendment for definition of done. Section 10 stays. It still points at `docs/producer/process.md`.

### 5. The first bullets are pull request #10, verbatim

The implementation inserts the Added and Security text from the task, including the links to `https://github.com/fputs/genesis/pull/10`. Section order follows Keep a Changelog: Added, then the existing Changed restart bullet, then Security. Those #10 bullets are the record of work that merged before this rule existed. The changelog pull request itself only edits `AGENTS.md` and `CHANGELOG.md`, so it does not add a bullet about itself.

## Risks / Trade-offs

- [Section 10 and the producer doc still say Producer drafts the entry after merge] → This change does not edit them. The new section 2 line is the done rule for implementation pull requests. A reviewer who wants those two docs rewritten should say so; the tasks do not authorize that edit. An agent who treats the leftover sentences as a conflict stops and reports it, per `AGENTS.md`, instead of editing them anyway.
- [The restart bullet and the new header both mention v1] → Both stay. The header is the short pointer. The Unreleased bullet still satisfies `project-foundation`.
- [Nothing fails the build when a bullet is missing] → Accepted. Enforcement is a follow-up, not this change.
- [These scenarios have no automated test] → A reviewer reads `CHANGELOG.md` and `AGENTS.md`. The tasks do not add a test.

## Migration Plan

This pull request adds only `openspec/changes/add-changelog/`. The later implementation edits `CHANGELOG.md` and adds the one `AGENTS.md` line, runs the proof commands, and archives with `openspec archive add-changelog --yes` in its last commit. Rollback of that later pull request is reverting it. The file already on `main` is the starting point, not a new empty file.

## Open Questions

None.
