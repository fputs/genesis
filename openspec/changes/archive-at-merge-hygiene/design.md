# Design

## Context

See proposal.md for why. `pr-hygiene` runs on pull_request `opened`, `synchronize`, `reopened`, and `edited`. Archive-last passes when no active change id was removed, and fails when a removal is not the head commit `openspec archive <id> --yes`. The event payload includes `pull_request.draft`. `gh pr ready` sends `ready_for_review`. Converting back sends `converted_to_draft`. Both are `pull_request` actions, so the script's event name stays `pull_request`.

`gh api repos/fputs/genesis --jq .owner.type` returns `User`.

## Goals / Non-Goals

**Goals:**

- A draft implementation pull request stays green without an archive commit.
- `gh pr ready` runs today's archive-last rules, plus the non-exempt failure below.
- A later push or a return to draft re-runs that same choice.
- Spec-drift teardown survives `ENOTEMPTY` on `.git/objects`.

**Non-Goals:**

- A merge queue, a ruleset edit, a merge-method change, or a `merge_group` trigger on `ci.yml` or `pr-hygiene.yml`.
- Changing spec-drift's reference check or its completed-change check.
- `pr-hygiene` fixture cleanup. The flake is the spec-drift suite.
- Editing `AGENTS.md`. It already says to archive once, right before merge.
- A new dependency, or any edit under `crates/rules`.

## Decisions

### 1. Draft skips archive-last. Ready runs it.

The agent implements in a draft. Archive-last does not run, so fixes do not restore and re-archive the change. The bullet check still runs. The final commit is `openspec archive <id> --yes`. The agent then runs `gh pr ready`. That `ready_for_review` run sees `draft: false` and applies archive-last. Architect merges that head.

A missing `pull_request.draft` field is not true, so existing fixtures keep today's rules.

When the pull request is not a draft, today's archive-last rules run. A non-exempt implementation pull request also fails if an `OpenSpec-Change:` body line names an id whose `openspec/changes/<id>/` directory is still on the head. Non-exempt means an implementation pull request that is not bullet-exempt. A pull request with no `OpenSpec-Change:` line does not fail because another change is active on the merge-base.

A removed id still has to satisfy the archive-last requirements already in the spec: subject, dated directory, and the head commit performing the delete. Those scenarios stay as they are. They describe a run whose draft flag is not true. Archive-last still does not read the body. The body line is only how this extra failure finds the pull request's own change.

`synchronize` on a non-draft pull request runs the bullet check and archive-last again. A fix commit after the archive fails until a new archive commit is last and the agent marks ready again.

### 2. `converted_to_draft` is a trigger

`ready_for_review` is the edge that turns the check on. GitHub does not send `edited` when someone converts a pull request back to a draft, so the red ready check would stick. `converted_to_draft` re-runs the job with `draft: true` and archive-last does not fail.

### 3. Merge queue is rejected

GitHub merge queues are available for organization-owned repositories. `fputs/genesis` is owned by the user account `fputs` (`owner.type` is `User`), so this repository cannot enable a queue. A queue would also replace Architect's API merge, which checks the head sha before merging. The draft flag is already on the pull_request payload, needs no new permission, and does not change that merge.

### 4. Retry `ENOTEMPTY` in spec-drift fixture teardown

Run 39705218352 threw `ENOTEMPTY` from `rmdir` of `.git/objects` while `node:test` cleaned fixture git directories in parallel. Teardown calls an injectable remover so the test can throw `ENOTEMPTY` once without racing a real `rmdir`. On that error it retries with backoff, at most 5 calls, then fails the test with that error. `--test-concurrency=1` is not the fix, and `fs.rm` `maxRetries` is not the mechanism.

## Risks / Trade-offs

- [Ready is clicked before the archive commit] → The `ready_for_review` run fails when the body names that change and it is still active. Convert back to draft, archive as the last commit, then mark ready again.
- [Architect merges a draft] → GitHub does not offer merge on a draft. The enforced run is the non-draft head.

## Migration Plan

DevOps ships the script, its fixtures, and the two new pull_request types together. Rollback is a revert of that implementation. In-flight pull requests that are already ready keep today's archive-last behavior.
