# Design

## Context

See proposal.md for why. The `Main` ruleset requires check contexts `ci`, `pr-hygiene`, `spec-drift`, and `spec-approved`. `strict_required_status_checks_policy` is false. A cancelled check run on the head sha is enough for the merge API to return 405, including when a later run of the same name on that sha succeeded. Bots cannot rerun Actions.

Inventory of `.github/workflows/*.yml` on this base. Required contexts are the ruleset's. No other workflow file sets `concurrency`.

| Workflow file | `name` | Triggers | Concurrency | Job id | Required context |
| --- | --- | --- | --- | --- | --- |
| `ci.yml` | CI | `push` to `main`; `pull_request` to `main` (default `opened`, `synchronize`, `reopened`) | none on the workflow; job `spec-drift` group `spec-drift-${{ github.event.pull_request.number \|\| github.sha }}`, `cancel-in-progress: true` | `ci` has no concurrency key. `spec-drift` has the group above | `ci` and `spec-drift` |
| `spec-drift-edited.yml` | Spec drift edited | `pull_request` `edited` to `main` | job `spec-drift` uses that same group and `cancel-in-progress: true` | `spec-drift` | `spec-drift` (same context as `ci.yml`) |
| `pr-hygiene.yml` | PR hygiene | `pull_request` `opened`, `synchronize`, `reopened`, `edited`, `ready_for_review`, `converted_to_draft` to `main` | workflow group `pr-hygiene-${{ github.event.pull_request.number }}`, `cancel-in-progress: true` | `pr-hygiene` | `pr-hygiene` |
| `spec-approved.yml` | Spec approved | `pull_request` `opened`, `synchronize`, `reopened`, `edited`, `ready_for_review` to `main`; `pull_request_review` `submitted`, `edited`, `dismissed` | job group `spec-approved-${{ github.event.pull_request.number }}`, `cancel-in-progress: true` | `spec-approved` | `spec-approved` |

On pull request 87, head `ac160bce06cc8fe1b03c5f7edd7ff7d0f6da8ca2`, `pr-hygiene` run 40144650988 and `spec-approved` run 40144650995 concluded cancelled beside later successes. CI run 40144650992 later succeeded for both `ci` and `spec-drift`. On pull request 89, head `6306905cb06432ceb90718dfccbfdefe67193e07`, the `ci.yml` `spec-drift` job in run 40385934723 concluded cancelled beside a successful `spec-drift` from `spec-drift-edited` run 40385968118, and `pr-hygiene` run 40385934722 and `spec-approved` run 40385934772 concluded cancelled beside later successes. `mergeStateStatus` was `BLOCKED`.

GitHub documents concurrency at [jobs.<job_id>.concurrency](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#jobsjob_idconcurrency) and in [Control the concurrency of workflows and jobs](https://docs.github.com/en/actions/how-tos/writing-workflows/choosing-when-your-workflow-runs/control-the-concurrency-of-workflows-and-jobs). `queue` is a supported key. `queue: max` keeps up to 100 pending runs. The default, `queue: single`, keeps one pending run. With `cancel-in-progress: false` there is still at most one running run and one pending run per group, and a newer queued run cancels the pending one. That cancellation is `conclusion: cancelled` on the head sha. `queue: max` combined with `cancel-in-progress: true` is a workflow validation error.

Normative behavior is the delta specs. Tests read the workflow files as text. Do not add a YAML parser.

## Goals / Non-Goals

**Goals:**

- No required check declares a concurrency block, so a same-sha burst cannot cancel a pending or running run.
- `synchronize`, then `edited`, then review `submitted` on one head sha leaves no cancelled `ci`, `pr-hygiene`, `spec-drift`, or `spec-approved` check.
- The Cargo `ci` job stays without a concurrency key. Nothing adds one.

**Non-Goals:**

- No edit to branch protection, the ruleset, or `strict_required_status_checks_policy`.
- No `merge_group` trigger and no merge queue.
- No skip `if`, no new `edited` type on `ci.yml`, and no change to permissions or to the script each job runs.
- No rerun permission for bots. The fix does not depend on clicking Re-run.
- No `queue` key. Support for `queue: max` is not why it is omitted.

## Decisions

### 1. Delete the concurrency blocks

Owner review of pull request 90 at `4087a9c`. Architect's decision for this revision: remove the concurrency block from the `spec-drift` job in `ci.yml`, the `spec-drift` job in `spec-drift-edited.yml`, the `pr-hygiene` workflow, and the `spec-approved` job. Do not replace them. The Cargo `ci` job is required and already has no concurrency block. Leave that job as it is.

GitHub's default without a `concurrency` key is to let workflow runs proceed together. No group means a newer event cannot cancel a pending run or an in-progress run. A burst of three or more events on one sha (`synchronize`, `edited`, review `submitted`) finishes every duplicate. The two `spec-drift` jobs no longer share a group, so an `edited` run cannot cancel the `ci.yml` job.

`queue: max` is a supported key on the pages cited above. It is dropped with the blocks. It would still cancel once 100 pending runs are waiting, and any shared group with the default queue cancels the previous pending run. That is the same-sha burst of three: one running, one pending, and the third cancels the pending one.

A group of workflow, sha, event name, action, and `github.run_id` would also keep runs from sharing a group. Each run would be independent. That still leaves a `concurrency` key. A later edit can drop `run_id` and restore a shared group. The workflow tests can reject the word `concurrency:` with a line check. They cannot prove a group expression stays unique. Absence is the stricter check.

The redundant minutes are accepted. These checks are Node 22 jobs: `node --test` and one script, and `spec-drift` also installs the pinned OpenSpec CLI. The repository is public, so GitHub-hosted minutes are free. Overlapping runs on one sha do not spend a paid budget. The Cargo job can already overlap, because its concurrency was never set.

Option (b), skipping `spec-drift-edited` on a body-only edit, still leaves `pr-hygiene` and `spec-approved` cancelling themselves, and the `OpenSpec-Change` line lives in the body. Option (c), exiting success from a superseded job, cannot rewrite a concurrency cancellation. Deleting the blocks means there is no cancellation to rewrite.

### 2. Where the lines come out

Delete the job-level block under `spec-drift` in both spec-drift workflows. Delete the workflow-level block in `pr-hygiene.yml`. Delete the job-level block under `spec-approved`. Do not add a workflow-level block to `ci.yml`. Triggers, permissions, and steps stay. `ci.yml` after this change has no `concurrency:` line at all.

### 3. Tests read the files

Replace the tests that require `cancel-in-progress: true`. Do not add a YAML parser. Assert that `ci.yml`, `spec-drift-edited.yml`, `pr-hygiene.yml`, and `spec-approved.yml` contain no `concurrency:` line. The scenario "Three same-sha events leave no cancelled check" is that file check: with no block, those three events cannot cancel a required check.

## Risks / Trade-offs

- [A `synchronize` run and an `edited` run of `spec-drift` both finish, and the script reads the body from the event payload] → Accepted. Cancelling the in-flight run is what leaves the cancelled check. Both results land. The ruleset bug is the cancelled conclusion.
- [Same-sha bursts spend extra runner time] → The Node checks are cheap and public-repo minutes are free. Recorded in decision 1.
- [The ruleset still treats some other non-success conclusion as blocking] → This change only stops these workflows from writing `cancelled` onto the current head. It does not edit the ruleset.

## Migration Plan

DevOps updates the three test files, then deletes the four concurrency blocks, on this pull request after a new `spec-approved: <sha>` review. The implementation commit adds the `### Fixed` bullet. Rollback is reverting those commits before the archive commit. No ruleset change. This proposal pull request does not change workflows or scripts.

## Backlog

This change covers the earlier QA item about the cross-workflow race between `ci.yml` `spec-drift` and `spec-drift-edited`. That race is the shared group: an `edited` run cancels the in-flight `ci.yml` job and leaves `conclusion: cancelled` on the head sha. Decision 1 deletes both blocks. It is not a separate follow-up.
