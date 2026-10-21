# Design

## Context

See proposal.md for why. `scripts/spec-drift-check.mjs` on `d9aef85` already requires one U+0020, sorts completed-change ids with a byte-wise `<` comparison, and runs `node --test scripts/spec-drift-check.test.mjs` before the script in both spec-drift workflows. It still skips some malformed lines, JSON-quotes some whitespace tokens, and treats a dated archive folder name as an id. The two `spec-drift` jobs do not share a concurrency group. Normative behavior is the delta spec. `AGENTS.md` already says `OpenSpec-Change: <id>` with one space, so it needs no amendment.

## Goals / Non-Goals

**Goals:**

- Make the spec match the three behaviors #28 already shipped, then close the three holes the delta names.
- Keep one failure prefix: `spec-drift: invalid change id`.

**Non-Goals:**

- No change to `scripts/pr-hygiene.mjs`, the changelog bullet rules, or archive-last. Those specs stay as synced.
- No new npm dependency, no Cargo trigger change, and no `edited` type or skip `if` on `ci.yml`.
- No case-folding of a token onto a differently cased directory.
- Do not ban an active directory whose name happens to look like a date.

## Decisions

### 1. Ids are active names and archive-stripped names

Both stay. That is what `changeExists` means today, except for one extra match.

An id exists when `openspec/changes/<id>/` is a direct child other than `archive`, or when an archive child contributes that id. A child whose name matches `YYYY-MM-DD-<remainder>` contributes only `<remainder>`. Any other archive child contributes its own name. The full dated folder name is not also an id. `OpenSpec-Change: foo` matches `openspec/changes/archive/2026-10-21-foo/`. `OpenSpec-Change: 2026-10-21-foo` does not, unless an active directory is named `2026-10-21-foo`.

A dated-shaped token that matches no active id and no stripped id is `spec-drift: unknown change <id>` on a code path. It is not an invalid-id token: it still matches the kebab grammar. Docs-only and openspec-only pull requests still ignore an unknown kebab id.

Alternative considered: reject every token that matches `YYYY-MM-DD-*` as an invalid id, including an active directory of that name. Rejected. The hole is the archive folder name being used as the id. An active directory name remains the id.

### 2. A prefix line is a reference or an invalid id

After one trailing CR is removed and surrounding whitespace is trimmed, a line that starts with the exact case-sensitive `OpenSpec-Change:` is a reference. It is valid if and only if it is that prefix, then exactly one U+0020, then one kebab id. Otherwise it is an invalid id and is not skipped. `openspec-change: foo` does not start with that prefix, so it is ignored. Indentation around the whole line still trims away.

The invalid-id line is `spec-drift: invalid change id` plus the whole remainder after `OpenSpec-Change:`, escaped, with nothing stripped. One valid line does not hide an invalid one, and the job does not also log `spec-drift: code change missing OpenSpec-Change`.

Alternative considered: skip two-or-more words, which is why a line of `OpenSpec-Change:`, a space, a tab, and `Add-Foo extra` beside a valid line exits 0. Rejected. A line that starts with the prefix and is not one kebab id is a mistake on every pull request.

### 3. Invalid tokens are escaped per UTF-16 code unit

Replace `JSON.stringify`. The logged text is the whole remainder after `OpenSpec-Change:`, with nothing stripped. For each UTF-16 code unit, write `\u` and four lowercase hex digits when the unit is outside U+0020 through U+007E or is `\`. Copy every other unit. Do not add quotes. A tab is `\u0009`, not `\t`. A typed `\u00a0` becomes `\u005cu00a0`, which differs from the escape of a real NBSP. An empty remainder leaves the line as `spec-drift: invalid change id`.

Alternative considered: keep `JSON.stringify` for tokens that match `\s`. Rejected. It leaves U+200B raw, emits U+00A0 and U+2003 raw, wraps other whitespace in quotes, and is not reversible for a typed backslash.

### 4. One job-level group for both spec-drift jobs

GitHub keeps the newest completed check of a given name. `ci.yml` and `spec-drift-edited.yml` both report `spec-drift`. An in-progress `ci.yml` run can finish after an edit and post a stale success.

Put the same concurrency block on each `spec-drift` job, not on the `ci.yml` workflow:

```yaml
concurrency:
  group: spec-drift-${{ github.event.pull_request.number || github.sha }}
  cancel-in-progress: true
```

On a pull request the number is set, so an edit cancels the in-progress spec-drift job from either workflow. The newest run in the group survives, and older pending or running runs are cancelled. On a push the number is empty, so the group is the sha and does not cancel a pull request job. Remove the workflow-level `spec-drift-edited-<number>` group. The Cargo `ci` job has no concurrency key, so a body edit still cannot cancel it or replace a failed Cargo check. The group key stays `spec-drift-${{ github.event.pull_request.number || github.sha }}`.

Alternative considered: workflow-level concurrency on `ci.yml` with the same group. Rejected. That would cancel the Cargo job when the body is edited.

### 5. Byte order stays the shipped comparison

Completed-change ids sort with `(left < right) - (left > right)` style comparison, not `localeCompare`. For these strings that is UTF-8 byte order, so `Zed` precedes `alpha`. Do not switch to a locale sort. The behavior is already on main; the spec now names it.

### 6. The edited workflow already runs the suite first

`spec-drift-edited.yml` already runs `node --test scripts/spec-drift-check.test.mjs` before `node scripts/spec-drift-check.mjs`, and its steps match the `ci.yml` spec-drift job. Keep that. The spec text that said the workflow runs only the script is what changes. The concurrency move in decision 4 is the workflow edit.

## Risks / Trade-offs

- [A pull request that put a note after the id, such as `OpenSpec-Change: add-foo extra`, now fails as an invalid id] → Fail closed. The line has to be exactly one token.
- [Cancelling an in-progress spec-drift job leaves that check cancelled until the new run finishes] → Accepted. A stale success must not become the newest result. The Cargo job is outside the group.
- [An `edited` event, including a title-only edit, cancels an in-flight `ci.yml` spec-drift job] → The required check must be the job `spec-drift`, not the CI workflow. The Cargo job stays outside the group, so a failed Cargo check is not replaced.
- [A body that copies the archive folder name `2026-10-21-foo` fails on a code path] → Use the stripped id `foo`, or the active directory name when that is the id.
- [Existing whitespace tests expect `JSON.stringify`] → Update them to the `\u` lines in the same implementation. Do not loosen them.

## Migration Plan

Genesis DevOps implements the script, the fixtures, and the two workflow concurrency blocks. Rollback is a revert of those commits before the archive commit. No `AGENTS.md` edit. This proposal pull request does not change the scripts or the workflows.

## Open Questions

None.
