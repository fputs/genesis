# Design

## Context

See proposal.md for why. `.github/workflows/pr-hygiene.yml` runs `pr-hygiene` on pull_request `opened`, `synchronize`, `reopened`, and `edited` targeting `main`. It does not run on push. Permissions are `contents: read`. The script treats any event other than `pull_request` as a hard error. Archive-last compares active change ids on the merge-base with the head. Zero removals pass. A removal fails unless the head commit subject is `openspec archive <id> --yes` for each removed id, joined by ` && ` in ascending id order, and the first-parent diff stays on archive paths.

The Main ruleset `25563418` requires status checks `ci`, `pr-hygiene`, and `spec-drift`. It also blocks deletion and non-fast-forward pushes. It has no merge queue. The repository is public, so GitHub merge queue is available. Pull requests may merge, squash, or rebase today.

`scripts/spec-drift-check.mjs` runs the reference check only when the event is `pull_request`. Validation and the completed-change check run on every event. Run `39705218352` failed `node --test scripts/spec-drift-check.test.mjs` in `Copying to archive without removing the active change fails` with `ENOTEMPTY: directory not empty, rmdir '/tmp/spec-drift-4nhpSw/.git/objects'`. The throw is `fs.rm` in fixture teardown while `node:test` runs cases in parallel. Normative behavior is the delta specs.

## Goals / Non-Goals

**Goals:**

- A review push of an unarchived implementation pull request stays green on `pr-hygiene`.
- A review push whose head is a fix after an archive commit stays green on archive-last.
- The merge itself is blocked until archive-last holds, including when an archive commit exists and must be last.
- The failure line stays `archive commit is not last`.
- Spec-drift fixture teardown survives `ENOTEMPTY` on `.git/objects`.

**Non-Goals:**

- Relaxing the spec-drift failure for a fully checked active change on pull_request. Leave the archive task unchecked until the one archive commit.
- Requiring a spec-only, docs-only, or bullet-exempt pull request to archive at merge. Spec pull requests on main leave the change active. The implementation pull request archives it.
- Reading the pull request body inside archive-last, or granting `pull-requests: read`.
- Changing `scripts/pr-hygiene.test.mjs` cleanup. The observed flake is the spec-drift suite.
- Editing `AGENTS.md` or `docs/producer/process.md`. They already say to archive once, as the last commit right before merge.
- A new npm dependency, or any edit under `crates/rules`.

## Decisions

### 1. The gate is `merge_group` on the existing `pr-hygiene` check

GitHub re-runs required checks at merge only through a merge queue. The Main ruleset already requires `pr-hygiene`. Adding a merge queue makes that check run again on `merge_group` after the review run is green. A failing queue run ejects the pull request, so the merge does not land.

Rejected: treat "Ready for review" as the gate. Review on this repository happens on non-draft pull requests. Failing archive-last whenever `draft` is false is red for the whole review. That is the cycle in implementation PRs #25 and #28.

Rejected: a label. The ruleset blocks merge when a required check fails. A missing label does not. If the pull_request run passes without the label, GitHub allows the merge. A check that fails until the label is present is red during review.

The merge group head is a merge commit when the queue method is `MERGE`: first parent is the queue base, second parent is the pull request head. Archive-last and the bullet diff use that second parent and its first parent. They do not use the group head (its subject is a merge subject such as `Merge pull request`) and they do not use the runner's checkout merge. A group head that does not have exactly two parents fails closed, so a squash or rebase queue cannot pass by skipping the parent check.

DevOps adds one `merge_queue` rule to ruleset `25563418` and leaves deletion, non-fast-forward, and the required checks `ci`, `pr-hygiene`, and `spec-drift` in place:

- `merge_method`: `MERGE`
- `max_entries_to_build`: `1`
- `max_entries_to_merge`: `1`
- `min_entries_to_merge`: `1`
- `min_entries_to_merge_wait_minutes`: `0`
- `grouping_strategy`: `ALLGREEN`
- `check_response_timeout_minutes`: `60`

One pull request per group keeps the second parent unambiguous. `MERGE` keeps the archive commit as that parent.

`.github/workflows/ci.yml` also triggers on `merge_group`. The queue will not finish while required checks `ci` and `spec-drift` are unreported. Cargo steps stay. The file gains no `edited` type and no skip `if`.

### 2. The pull_request run does not apply archive-last

On `pull_request` the script still runs the bullet check. It does not log `archive commit is not last`. Scenarios that already describe a bad or missing archive move to the merge-time run: the fixture sets the event to `merge_group` and builds a two-parent group head whose second parent is the commit the scenario already built.

### 3. Which pull request must archive

A non-exempt implementation pull request is an implementation pull request that is not bullet-exempt. That is a diff that includes root `Cargo.toml`, a path under `crates/`, or a path under `.github/workflows/`, together with any path outside the exempt set. `Cargo.lock` and `rust-toolchain.toml` are code paths and bullet-exempt, so a lockfile-only pull request passes the merge run with no removal. A `scripts/` diff is not an implementation pull request.

When that non-exempt diff removed no active id, the merge-time run fails with `archive commit is not last`. When it did remove ids, the existing archive-last rules apply: subject, dated directory, allowed paths, and the head commit performing the delete. The pull request body stays unread.

### 4. spec-drift on the queue

`merge_group` is not `pull_request`, so the reference check stays skipped, the same as push. Validation and the completed-change check still run on the group checkout. A fully checked active change fails the queue. The job does not fetch a pull request body and does not gain permissions.

### 5. Retry `ENOTEMPTY` in spec-drift fixture teardown

Node's `fs.rm({ recursive: true })` throws `ENOTEMPTY` on `.git/objects` when parallel cleanups overlap. `maxRetries` retries that code. The spec-drift helper uses that option, or a loop with the same effect, and fails the test only if the directory is still present afterward. One test calls the helper with a first attempt that returns `ENOTEMPTY` and a second that succeeds. Setting `--test-concurrency=1` alone does not meet the requirement: a single recursive removal can still throw `ENOTEMPTY`.

## Risks / Trade-offs

- [The ruleset is never updated] → The implementation task updates ruleset `25563418` in the same change. Until that rule exists, `merge_group` does not run and review CI is the only gate.
- [spec-drift on `merge_group` starts requiring a body] → It must not. The event is not `pull_request`. The scenario is "A merge group skips the reference check".
- [Every task box is checked before the archive commit] → spec-drift stays red on pull_request until that one archive commit. That is one archive, not a re-archive per fix.
- [The queue lands a merge commit instead of a squash] → `MERGE` is required so the group head has two parents and the archive commit remains the pull request head. A one-parent group head fails closed.
- [GitHub batches several pull requests into one group head] → `max_entries_to_merge` is 1, so the second parent is that pull request's head.

## Migration Plan

DevOps ships the script, the tests, both workflow triggers, and the ruleset rule together. Rollback is a revert of that implementation plus deletion of the `merge_queue` rule from ruleset `25563418`. After the rule is active, an in-flight implementation pull request stays green through review without an archive commit. The queue enforces archive-last when someone merges.
