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

On pull request 87, head `ac160bce06cc8fe1b03c5f7edd7ff7d0f6da8ca2`, `pr-hygiene` run 40144650988 and `spec-approved` run 40144650995 concluded cancelled beside later successes. CI run 40144650992, the run named in the report, later succeeded for both `ci` and `spec-drift`. On pull request 89, head `6306905cb06432ceb90718dfccbfdefe67193e07`, the `ci.yml` `spec-drift` job in run 40385934723 concluded cancelled beside a successful `spec-drift` from `spec-drift-edited` run 40385968118, and `pr-hygiene` run 40385934722 and `spec-approved` run 40385934772 concluded cancelled beside later successes. `mergeStateStatus` was `BLOCKED`.

Normative behavior is the delta specs. Tests read the workflow files as text. Do not add a YAML parser.

## Goals / Non-Goals

**Goals:**

- A same-sha rerun of `spec-drift`, `pr-hygiene`, or `spec-approved` does not cancel the in-flight run, and a pending rerun in that group is not cancelled either.
- A new head sha is a different group, so its run does not cancel the previous sha.
- `ci.yml` `spec-drift` and `spec-drift-edited.yml` `spec-drift` no longer share a group.
- The Cargo `ci` job stays free of concurrency, and `ci.yml` gains no workflow-level group.

**Non-Goals:**

- No edit to branch protection, the ruleset, or `strict_required_status_checks_policy`.
- No `merge_group` trigger and no merge queue.
- No skip `if`, no new `edited` type on `ci.yml`, and no change to permissions or to the script each job runs.
- No rerun permission for bots. The fix does not depend on clicking Re-run.

## Decisions

### 1. Option (a): key the group on workflow, head sha, and event type

Each required check that can be cancelled today gets one concurrency block:

```yaml
concurrency:
  group: <job>-${{ github.workflow }}-${{ github.event.pull_request.head.sha || github.sha }}-${{ github.event_name }}-${{ github.event.action }}
  cancel-in-progress: false
  queue: max
```

`pr-hygiene` and `spec-approved` do not run on `push`, so their group uses `github.event.pull_request.head.sha` and not `github.sha`. On `pull_request`, `github.sha` is the merge commit, not the head the ruleset reads. `spec-drift` keeps `head.sha || github.sha` so a push to `main` still keys on the pushed commit.

`cancel-in-progress` is the boolean `false`, not an expression. `queue: max` keeps up to 100 pending runs. The default queue keeps one pending run and cancels the previous pending run, which would still leave `conclusion: cancelled` on that head sha. `queue: max` cannot be combined with `cancel-in-progress: true`.

`github.workflow` is the workflow `name`. `CI` and `Spec drift edited` therefore do not share a group, so an `edited` run cannot cancel the in-flight `ci.yml` `spec-drift` job. `github.event_name` and `github.event.action` keep a `synchronize` run and an `edited` run from cancelling each other. A second event of the same type on the same sha queues. A new head sha is a new group, so the new head's run starts without cancelling the previous sha. The previous sha is not the head, and its check is not the one the ruleset reads.

This is the "key the group on the head sha" form of option (a). The expression form, `cancel-in-progress` true only when `before != after`, needs one group that spans shas. `edited` and `pull_request_review` payloads do not carry `before` and `after`. A true value on a same-sha `synchronize` cancels the in-flight run on the sha the ruleset reads, which is the 405.

Option (b), splitting the shared `spec-drift` group and skipping `spec-drift-edited` when only the body changed, does not cover `pr-hygiene` or `spec-approved`. Those workflows cancel themselves on `edited` and on a same-sha `synchronize`. The `OpenSpec-Change` line lives in the body, so a body-only skip would drop the re-run `spec-drift-edited` exists to perform.

Option (c), making a superseded job exit success, does not work. Concurrency cancellation sets `conclusion: cancelled` without running the job, so a job `if` cannot choose success. A synthetic success is the stale result the shared group was added to stop.

### 2. Where the block sits

`spec-drift` stays job-level in both files, with the same source text. `ci.yml` stays without a workflow-level block, so the Cargo job cannot be cancelled by a body edit. `pr-hygiene` keeps a single workflow-level block, replaced in place. `spec-approved` keeps a single job-level block, replaced in place. Triggers, permissions, and steps stay.

### 3. Tests read the files

Update the existing line checks. Do not add a YAML parser. Evaluate the group expression in the test for two head shas, and for a push versus a pull request, the way "A push does not join a pull request group" already does. Assert `github.workflow` against each file's `name` so the two `spec-drift` groups differ even though the source line matches.

## Risks / Trade-offs

- [A `synchronize` run and an `edited` run of `spec-drift` can both finish, and the script reads the body from the event payload] → Accepted. Cancelling the in-flight run is what leaves the cancelled check. The edit run still posts its own result. GitHub does not promise order inside a concurrency group either.
- [An old sha keeps running after a new commit] → Its check is not on the new head. Runner time is the cost. Cancelling it would require the expression form rejected in decision 1.
- [`queue: max` holds at most 100 pending runs, then cancels] → A burst past 100 same-sha events can still cancel. That is outside the edits that blocked 87 and 89.
- [The ruleset still treats some other non-success conclusion as blocking] → This change only removes the cancelled conclusion these groups write onto the current head. It does not edit the ruleset.

## Migration Plan

DevOps updates the three test files, then the four workflow concurrency blocks, on this pull request after `spec-approved`. The implementation commit adds the `### Fixed` bullet. Rollback is reverting those commits before the archive commit. No ruleset change. This proposal pull request does not change workflows or scripts.

## Backlog

This change covers the earlier QA item about the cross-workflow race between `ci.yml` `spec-drift` and `spec-drift-edited`. That race is the shared group: an `edited` run cancels the in-flight `ci.yml` job and leaves `conclusion: cancelled` on the head sha. Decision 1 splits those groups. It is not a separate follow-up.
