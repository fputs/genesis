# Design

## Context

`.github/workflows/ci.yml` runs the Cargo job and the `spec-drift` job on push and pull_request to `main`. Workflow permissions there are `contents: read`. See proposal.md for why this gate exists. Normative behavior is `specs/pr-hygiene/spec.md` and the changelog delta.

Spec-drift already parses changed paths with `git diff -z --name-status --find-renames`, split on NUL, and counts both sides of a rename. A code path is exactly `Cargo.toml`, `Cargo.lock`, or `rust-toolchain.toml`, or a path under `crates/` or `.github/workflows/`. An `OpenSpec-Change: <id>` line is one trimmed body line. This job reuses those definitions and does not change that script.

The 2026-10-20 changelog interim ruling treats a pull request that touches only `.cursor/**`, `README.md`, `Cargo.lock`, or `rust-toolchain.toml` as needing no bullet. `docs/architect/decisions.md` still lists "CI enforcement of the changelog bullet" as backlog item 1. `AGENTS.md` section 2 tells an author to paste proof-command output into the pull request. Neither file asks for the head commit SHA. Genesis DevOps agent implements this change.

## Goals / Non-Goals

**Goals:**

- One local script, one new workflow, two checks, deterministic results from the checkout and the event payload.
- Fail an implementation pull request that adds no bullet under `## [Unreleased]`.
- Fail when a named change directory is removed and the head commit is not the archive.
- Keep the Cargo job and spec-drift behavior as they are.

**Non-Goals:**

- A head-SHA check in pull request bodies. CI on the real head is the record.
- Any change to the Cargo job's steps or triggers, or to spec-drift's steps, triggers, or checks.
- An opt-out label.
- Checking which Unreleased subsection a bullet uses, or that the bullet links to the pull request. The changelog spec still requires both of authors. This job checks that an Unreleased bullet was added.
- npm dependencies, a GitHub API call, or running this job on push.
- Requiring a bullet from docs-only, openspec-only, or the exempt path set.

## Decisions

### 1. A separate workflow file

Add `.github/workflows/pr-hygiene.yml` whose only job is `pr-hygiene`. Triggers are pull_request `opened`, `synchronize`, `reopened`, and `edited`, targeting `main`. Permissions are `contents: read` and `pull-requests: read`.

`edited` has to re-run the job because the body can gain or lose `OpenSpec-Change:` lines without a new commit. Putting those types, or `pull-requests: read`, on `.github/workflows/ci.yml` would also change when the Cargo and spec-drift jobs run. Leave that file untouched.

Alternative considered: job-level `if:` on the existing workflow. That still edits the Cargo and spec-drift jobs' trigger block. A second file keeps their behavior identical.

### 2. One Node script, no package

Add `scripts/pr-hygiene.mjs` with no npm dependencies. The job checks out with `fetch-depth: 0`, sets up Node 22, runs `node --test scripts/pr-hygiene.test.mjs`, then runs `node scripts/pr-hygiene.mjs`. It does not install `@fission-ai/openspec` or any other package. Node 22 is the same runtime spec-drift already uses, not a new dependency.

The script prints every failure line and exits 1 if any check failed or a parse failed:

- `implementation PR missing CHANGELOG bullet`
- `archive commit is not last`

Tests may set `PR_HYGIENE_EVENT`, `PR_HYGIENE_ACTION`, `PR_HYGIENE_ROOT`, `PR_HYGIENE_BASE_SHA`, `PR_HYGIENE_HEAD_SHA`, and `PR_HYGIENE_BODY`. When those are unset, the script uses `GITHUB_EVENT_NAME`, `GITHUB_EVENT_PATH`, and the event's `pull_request.head.sha` and `pull_request.base.sha`.

Alternative considered: extend `scripts/spec-drift-check.mjs`. That script also runs on push and owns validation and the change-id reference. Mixing in pull-request-only rules would change its push behavior. A second script keeps spec-drift stable.

### 3. Paths match the spec-drift parse

Run `git diff -z --name-status --find-renames` and split on NUL, including both sides of a rename, the same record walk as `scripts/spec-drift-check.mjs`. The bullet diff is base to the event head. The archive diff is the event head against its first parent.

The default `pull_request` checkout is a merge commit the runner creates. Archive-last must read `pull_request.head.sha`, not that merge. `fetch-depth: 0` makes the base, the head, and the head's first parent available. A missing sha, a missing parent, a non-zero diff, or a broken NUL record fails the job closed. The pull request is not treated as exempt.

The script does not call the GitHub API. `pull-requests: read` is the workflow token scope the job declares. The body still comes from the local event payload, including on `edited`.

### 4. Implementation classification, then exemptions

An implementation pull request is one whose diff includes a spec-drift code path. The bullet check then passes with no new bullet when the diff is docs-only (`docs/**`, `AGENTS.md`, `CHANGELOG.md`), openspec-only (every path under `openspec/`), or every path is under `.cursor/` or is exactly `README.md`, `Cargo.lock`, or `rust-toolchain.toml`.

`Cargo.lock` and `rust-toolchain.toml` are spec-drift code paths. The exemption still wins when the diff is only the exempt set. Adding any other code path, including root `Cargo.toml`, `crates/`, or `.github/workflows/`, removes the exemption. No label skips the check.

A bullet is a trimmed markdown list item with text in the first `## [Unreleased]` section, before the next `## ` heading. Added means the head section has such a line the base section does not. A bullet under a released version, or a line that was already there, does not count. The job does not read labels.

### 5. Archive-last is a conjunction

The rule runs only when a trimmed `OpenSpec-Change: <id>` line names an id and the base has `openspec/changes/<id>/` while the head does not. Fenced lines count, matching spec-drift. Extra words do not. Duplicate lines count once.

The head commit then has to satisfy both:

- Its message contains `openspec archive <id>` with the id ending on whitespace or end of string, so `openspec archive add-foo-extra` does not cover `add-foo`.
- Every path in its first-parent diff is under `openspec/changes/archive/`, under `openspec/specs/`, or is the removed `openspec/changes/<id>/`.

The removal has to be allowed. `openspec archive` deletes or renames that directory, and the NUL parse keeps the pre-image path. Without that allowance the real archive commit would fail the check that requires it. Any other path, including `CHANGELOG.md`, fails with `archive commit is not last`.

Several named removals each need their own phrase. One failure line covers the set. A named id that is still present, or a removal with no reference line, does not fail this rule.

The changelog bullet for an implementation pull request is committed before that archive commit. The archive commit does not edit `CHANGELOG.md`. The pull request diff still shows the new bullet.

### 6. Tests are a table of fixture repositories

`scripts/pr-hygiene.test.mjs` uses `node:test`. Each table row builds a temporary git repository: base commit, head commit, optional parent commit, pull request body, and event action. The test runs the same `node scripts/pr-hygiene.mjs` entry point as CI and asserts the exit status and failure lines.

The table includes a pass fixture and a fail fixture for the bullet rule, for archive-last, and for each exemption (docs-only, openspec-only, `.cursor/**`, `README.md`, `Cargo.lock`, `rust-toolchain.toml`), including the mixed diffs that lose the exemption. It also covers NUL rename sides, fail-closed parse errors, several ids, a fenced line, `edited`, a body with no commit SHA, and both failure lines together. One row reads `.github/workflows/pr-hygiene.yml` and `.github/workflows/ci.yml`.

### 7. Docs land at apply time

When this change is applied, Genesis DevOps agent:

- Removes backlog item "CI enforcement of the changelog bullet." from `docs/architect/decisions.md` and renumbers the remaining items. The other backlog sentences stay.
- Updates `AGENTS.md` and `docs/producer/process.md` so they do not require a head commit SHA in the pull request body, and so they say the `pr-hygiene` job enforces the Unreleased bullet and archive-last. Section 2 of `AGENTS.md` still says to paste the proof-command output.
- Adds one `### Added` bullet under `## [Unreleased]` in a commit before the archive commit.

## Risks / Trade-offs

- [The runner checkout is a merge commit] → Read `pull_request.head.sha` and its first parent. Do not classify `HEAD` after the default checkout.
- [A shallow checkout cannot see the base or the parent] → `fetch-depth: 0` on this job only. `ci.yml` stays as it is.
- [An archive commit deletes `openspec/changes/<id>/`] → That removed directory is an allowed path for each id the rule is enforcing. Every other path must be under `openspec/changes/archive/` or `openspec/specs/`.
- [`Cargo.lock` is a spec-drift code path and still exempt] → Accepted. The exemption matches the 2026-10-20 interim ruling. A second code path drops it.
- [Fenced examples count as references] → Accepted. Matching stays line-based, as in spec-drift.
- [The job does not check the bullet's section or link] → Accepted. Authors still follow the changelog spec. This job only fails a missing Unreleased bullet.
- [A removal with no `OpenSpec-Change:` line skips archive-last] → Accepted. The rule is the conjunction in the spec. Spec-drift still requires the line on its own code paths.
- [Parse errors could be mistaken for an exemption] → Fail closed. Exit non-zero and do not pass the pull request.

## Migration Plan

1. Merge this spec pull request. It touches only `openspec/changes/add-pr-hygiene-checks/`, so it is openspec-only.
2. Genesis DevOps agent applies this change on its own pull request. The body includes `OpenSpec-Change: add-pr-hygiene-checks`. The changelog bullet is added before the last commit. The last commit is `openspec archive add-pr-hygiene-checks --yes`, and that commit touches only the archive tree, `openspec/specs/**`, and the removed change directory.
3. Rollback is deleting `.github/workflows/pr-hygiene.yml` and `scripts/pr-hygiene.mjs`. No data migration. `ci.yml` was not part of the change.
