# Proposal

## Why

`harden-pr-hygiene` merged as `d9aef85` (PR #28) and its specs were synced, but three shipped behaviors never made it into the requirement text, and three more holes remain. A stale `spec-drift` success can still overwrite a newer failure, an invisible character can hide in an invalid-id log line, and a dated archive folder name is accepted as a change id.

## What Changes

- **BREAKING** for a pull request whose `OpenSpec-Change` line is not exactly one token, or whose id is a dated archive folder name such as `2026-10-21-foo`. Those lines fail closed. No new package and no new failure-line prefix.
- Align the edited-PR workflow with what #28 shipped: `node --test scripts/spec-drift-check.test.mjs` runs before `node scripts/spec-drift-check.mjs`. This replaces "SHALL run only `spec-drift-check.mjs`".
- Align the reference line with what #28 shipped: exactly one U+0020 after `OpenSpec-Change:`. A tab, or any other leading whitespace before the id, is an invalid id. Line-end trim still allows indentation around the whole line.
- Align the completed-unarchived log with what #28 shipped: ids sort by UTF-8 byte order, not by locale.
- The invalid-id line is `spec-drift: invalid change id` plus the whole remainder after `OpenSpec-Change:`, with nothing stripped. Every UTF-16 code unit outside U+0020 through U+007E, and `\`, is `\u` plus four lowercase hex digits, so a zero-width space, NBSP, em space, or typed backslash is visible. The line is not JSON-quoted.
- A trimmed line that starts with the exact case-sensitive `OpenSpec-Change:` is a reference. It is valid if and only if it is that prefix, then exactly one U+0020, then one kebab id. Otherwise it is an invalid id and is not skipped. `openspec-change: foo` is ignored. Today `OpenSpec-Change:` plus a space, a tab, and `Add-Foo extra` beside a valid line exits 0.
- The `spec-drift` jobs in `ci.yml` and `spec-drift-edited.yml` share one job-level concurrency group keyed on the pull request number, with `cancel-in-progress`. The newest run survives, and older pending or running runs are cancelled. The Cargo job is not in that group. A push uses the commit sha so it does not join a pull request group.
- An id exists only as an active change id or an archive-stripped id. Both stay, which is what the lookup already means. The dated folder name is not itself an id. `OpenSpec-Change: foo` matches `openspec/changes/archive/2026-10-21-foo/`. `OpenSpec-Change: 2026-10-21-foo` does not, unless an active directory has that name.

This pull request is the spec only. It does not change the scripts or the workflows. `pr-hygiene` and `changelog` stay as synced: archive-last still ignores the body, and the bullet rules do not change. `AGENTS.md` already requires `OpenSpec-Change: <id>` with one space.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `spec-drift-check`: edited-workflow test step, one U+0020 before the id, byte-order failure list, visible invalid-id escapes, malformed lines fail as invalid ids, shared spec-drift concurrency, and dated archive folder names are not ids.

## Impact

- DevOps edits `scripts/spec-drift-check.mjs`, `scripts/spec-drift-check.test.mjs`, `.github/workflows/spec-drift-edited.yml`, and the `spec-drift` job in `.github/workflows/ci.yml`.
- No new dependency. `crates/rules` is untouched. `scripts/pr-hygiene.mjs` is untouched.
- The implementation pull request adds a `### Changed` bullet under `## [Unreleased]` before the archive commit, then runs `openspec archive amend-pr-hygiene-followups --yes`.
