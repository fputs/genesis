# Design

## Context

See proposal.md for why. Normative behavior is `specs/spec-drift-check/spec.md`. `scripts/spec-drift-check.mjs` already parses `OpenSpec-Change:` after one trailing CR and a trim, requires one U+0020 plus one kebab id, and logs a bad remainder with the `\u` plus four lowercase hex escape. It collects changed paths from `git diff -z --name-status --find-renames` and then drops the status. `scripts/pr-hygiene.mjs` imports `changeReferences` and treats only unfenced `OpenSpec-Change` lines as own-change ids. Both spec-drift workflows already run that script in the shared group `spec-drift-${{ github.event.pull_request.number || github.sha }}`.

Phase 1 names in `docs/architect/decisions.md` are kebab tokens (`dice-and-checks`, `grid-movement`, `circumstance-modifiers`). There is no separate roadmap file.

## Goals / Non-Goals

**Goals:**

- One new body prefix, parsed with the same trim and escape rules as `OpenSpec-Change:`.
- Path rules decided from name-status records the script already asks git for.
- Leave the workflow files, the Cargo job, and `changeReferences` behavior alone.

**Non-Goals:**

- No edit to `scripts/pr-hygiene.mjs`, `openspec/specs/changelog/spec.md`, or `openspec/specs/pr-hygiene/spec.md`.
- No lookup of the roadmap item in `docs/architect/decisions.md`.
- No Rust parse, and no check that behavior is unchanged.
- No `--find-copies`, no new event name, and no new concurrency group.
- No exemption from the validation or completed-change checks.

## Decisions

### 1. The roadmap item is a kebab token, not a looked-up heading

Malformed means the line is not exactly `OpenSpec-Refactor:`, one U+0020, and one token matching `^[a-z0-9]+(?:-[a-z0-9]+)*$`. That is the change-id grammar. Empty, a missing space, two spaces, a tab, extra words, uppercase, a slash (`positioning/flanking`), and a double hyphen are all malformed. The log is `spec-drift: invalid refactor item` plus the unstripped remainder, escaped like `spec-drift: invalid change id`.

The decision log names items in that shape and also uses prose ("positioning/flanking"). A lookup would parse prose. The job checks the token shape only. `not-a-real-item` passes when the paths qualify.

Alternative considered: any printable ASCII after the space. Rejected. Extra words would be a valid "item", which is the hole the change-id rule already closed.

### 2. `CHANGELOG.md` is the only path outside `crates/`

Chief of Staff required every changed path under `crates/`. The current changelog and pr-hygiene specs also require a new Unreleased bullet on a pull request that touches `crates/`. Those two rules cannot both hold unless `CHANGELOG.md` may sit in the same diff. The qualifying set is therefore: every changed path starts with `crates/` or is exactly `CHANGELOG.md`. `changelog.md`, `README.md`, `Cargo.toml`, `scripts/`, `docs/`, and `.github/workflows/` fail with `spec-drift: refactor is not crates-only`.

A path under `openspec/` fails that line and also logs `spec-drift: refactor changes openspec`. Both sides of a rename count. The diff command stays `git diff -z --name-status --find-renames` with no `--find-copies`.

Alternative considered: keep crates-only and let pr-hygiene fail every real refactor. Rejected. The exemption would never go green together with the bullet.

### 3. A test file is a `crates/` path containing `/tests/`

`crates/rules/src/encounter/tests/delay_ready_tests.rs` and `crates/rules/tests/foo.rs` are test files. `crates/rules/src/lib.rs`, `crates/rules/src/tests.rs`, and `crates/rules/src/mytests/foo.rs` are not. The slash on both sides of `tests` is the directory segment.

Statuses on those records:

- `A` on a test file is an add and is allowed.
- `M`, `D`, or `T` on a test file fails with `spec-drift: refactor changes existing tests`.
- A status starting with `R` fails only when the old path is a test file. Renaming `crates/rules/src/lib.rs` to a path under `tests/` is allowed.
- The status token includes a score suffix such as `R100`.

`#[cfg(test)]` inside a non-test source is not visible from the path list. The script does not read file text. Architect and QA cover that on review. The job does not fail a `lib.rs` edit for inline tests.

Alternative considered: treat any path ending in `_tests.rs` or `tests.rs` as a test file. Rejected. `src/tests.rs` is an ordinary module path, and the pending changelog delta already uses the `/tests/` substring for its tests-only skip.

### 4. Structural body failures outrank path failures

Evaluate path rules only when the body has exactly one valid refactor line, no `OpenSpec-Change:` line, and the diff has a code path. Otherwise:

- Each malformed refactor remainder logs `spec-drift: invalid refactor item` plus the escaped remainder, once per distinct remainder.
- Two or more refactor lines, valid or not, log `spec-drift: multiple OpenSpec-Refactor lines`. Duplicates do not collapse. This differs from change ids, which count once.
- Any refactor line plus any `OpenSpec-Change:` line logs `spec-drift: refactor line conflicts with OpenSpec-Change`.
- Do not also log the missing-reference line or the path lines.

Order when several of those fire: invalid change ids, invalid refactor items, the multiple line, the conflict line. Path lines, when they apply, follow as crates-only, then openspec, then existing tests. Unknown change ids and the missing-reference line stay last and only under the rules already in the spec.

A valid refactor line on a diff with no code path is ignored, the same way an unknown kebab change id is ignored. Malformed lines, multiple lines, and conflicts still fail on docs-only and openspec-only pull requests.

Push still skips the whole reference check, so a push does not read refactor lines.

`changeReferences` keeps returning only `OpenSpec-Change` ids. `pr-hygiene` therefore does not see `OpenSpec-Refactor` as an own-change line and does not require an archive commit for it.

### 5. The bullet stays with pr-hygiene

This job does not read `CHANGELOG.md`. A qualifying refactor with no bullet still passes spec-drift.

The bullet judgment uses the synced specs, not the open change `amend-changelog-policy`. Today `changelog` and `pr-hygiene` treat a `crates/` path as an implementation pull request that must add a new Unreleased bullet. They do not exempt tests-only diffs, and they do not exempt "no caller-visible effect". A refactor needs the bullet, including when every path contains `/tests/`.

That open change, if archived later, would skip the bullet only when every changed path is under `crates/` and contains `/tests/`, and would leave "no caller-visible effect" as author guidance rather than a CI skip. A production refactor would still need a bullet under that reading. This change does not adopt the tests-only skip and does not edit those specs. If that change archives first, the "tests-only still needs a bullet" sentence in this delta is the conflict to send back; do not silently widen this exemption.

### 6. Behavior unchanged is a human gate

Architect and QA confirm behavior is unchanged before merge. The script does not read reviews, approvals, or Rust. Neither workflow gains an `if` for that confirmation. The shared concurrency group is unchanged, so an edit still cancels the in-flight spec-drift run from either workflow and the newest run survives. Dropping the refactor line on edit is the existing missing-reference failure, because the edited workflow runs this same script.

## Risks / Trade-offs

- [A refactor that also fixes behavior can pass CI] → Accepted. The path rules do not judge behavior. Architect and QA are the gate.
- [Inline `#[cfg(test)]` edits are invisible] → Accepted. The path list cannot see them. Review does.
- [`CHANGELOG.md` widens the crates-only rule] → It is the only extra path, and it is what makes the bullet possible. Any other root file still fails.
- [A docs-only pull request may carry a valid refactor line and pass] → Same as an unknown change id on a docs-only pull request. A malformed line still fails.
- [`amend-changelog-policy` may later waive a tests-only bullet] → This delta does not depend on it. The conflict note is in decision 5.
- [Fenced examples count as refactor lines] → Accepted. Matching stays line-based, as it is for `OpenSpec-Change`.

## Migration Plan

1. Merge this spec pull request. It touches only `openspec/changes/add-refactor-exemption/`.
2. DevOps applies the script and the fixture tests. The implementation pull request's body is `OpenSpec-Change: add-refactor-exemption`. It adds one Unreleased bullet before the archive commit. The last commit is `openspec archive add-refactor-exemption --yes`.
3. Rollback is reverting that script change. No workflow data migration.

## Open Questions

None. The token grammar, the test-file path, the `CHANGELOG.md` exception, and the bullet reading are decided above.
