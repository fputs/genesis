# Tasks

Genesis DevOps agent implements this change. Do not edit `crates/rules`. Do not add a dependency. Keep the failure line `archive commit is not last`. Do not grant `pull-requests: read`. Do not read the pull request body in archive-last.

## 1. Archive-last moves to the merge-time run

- [ ] 1.1 In `scripts/pr-hygiene.mjs`, keep the bullet check on `pull_request` and do not apply archive-last on that event. The log SHALL NOT include `archive commit is not last`. Accept event `merge_group` as the merge-time run. On that run, the base sha is `merge_group.base_sha` (tests may still set `PR_HYGIENE_BASE_SHA`) and the pull request head is the second parent of the group head sha (`PR_HYGIENE_HEAD_SHA` is the group head). Run the bullet diff and archive-last on that second parent and its first parent, not on the group head and not on a runner merge. A group head without exactly two parents fails closed. A non-exempt implementation pull request (an implementation pull request that is not bullet-exempt) fails the merge-time run when no active change id was removed. When ids were removed, keep the current archive-last rules. Verify `node --test scripts/pr-hygiene.test.mjs` for scenarios "An unarchived implementation pull request passes review", "A fix after archive passes review", "An unarchived implementation pull request fails at merge", "A lockfile-only pull request passes at merge", "A scripts-only pull request passes at merge without an archive", "A one-parent merge group fails closed", "The merge group uses the pull request head", and "The check uses the event head".
- [ ] 1.2 Point every existing archive-last fixture at the merge-time run. Build a group head whose first parent is the merge-base and whose second parent is the commit that fixture already used as the head, and pass the group head as the event head. Keep each scenario name. Verify `node --test scripts/pr-hygiene.test.mjs` for "Two archived ids pass", "Two archived ids fail", "One of two archive phrases is missing", "A duplicate reference counts once", "The bullet is added before the archive commit", "A bullet inside the archive commit is not last", "Both checks fail together", "Extra words are not a change reference", "A fenced line does not exempt a removal", "The archive commit passes", "An earlier archive add is not last", "An empty head commit fails", "A specs-only head commit fails", "A phrase below the subject does not count", "A path outside archive and specs fails", "A later fix-typo commit is not last", "A missing archive phrase fails", "A longer id does not cover a shorter id", "A body edit does not excuse a bad archive", "An edited body drops the change reference", "A pre-existing archive allows the removal commit", "A named change that stays active skips archive-last", and "A removal without a reference fails".

## 2. Workflow triggers

- [ ] 2.1 In `.github/workflows/pr-hygiene.yml`, also run on `merge_group`. Keep pull_request types `opened`, `synchronize`, `reopened`, and `edited` targeting `main`, concurrency `pr-hygiene-<number>` with `cancel-in-progress: true`, permissions `contents: read` only, Node 22, and `node --test` before the script. Do not add a push trigger. Verify `node --test scripts/pr-hygiene.test.mjs` for "Job runs on the four pull request types", "The job runs on the merge group", "The job does not run on push", and "Permissions are contents read only".
- [ ] 2.2 In `.github/workflows/ci.yml`, add `merge_group` to the existing push and pull_request triggers. Keep the Cargo steps and the `spec-drift` steps. Do not add an `edited` type or a skip `if`. Do not put `pr-hygiene` in that file. Leave `.github/workflows/spec-drift-edited.yml` as it is. Verify `node --test scripts/pr-hygiene.test.mjs` for "Cargo and spec-drift stay" and `node --test scripts/spec-drift-check.test.mjs` for "Job triggers match the Cargo workflow", "Cargo steps stay", and "Editing the body does not replace a failed Cargo check".

## 3. spec-drift on the queue, and fixture cleanup

- [ ] 3.1 Keep the reference check gated on event `pull_request`, so `merge_group` skips it and still runs validation and the completed-change check on the checkout. Verify `node --test scripts/spec-drift-check.test.mjs` for "A code push has no pull request body", "A merge group skips the reference check", "A pull request that leaves every box checked fails", and "A merge group backstop fails a completed change".
- [ ] 3.2 In `scripts/spec-drift-check.test.mjs`, retry removal of each fixture git directory when `rmdir` returns `ENOTEMPTY`, including on `.git/objects`, until the directory is gone. Fail the test if it is still there after the retries. `maxRetries` on `fs.rm` meets this. `--test-concurrency=1` alone does not. Verify `node --test scripts/spec-drift-check.test.mjs` for "A transient ENOTEMPTY is retried" and "Each case is a fixture repository".

## 4. Main ruleset merge queue

- [ ] 4.1 GET `repos/fputs/genesis/rulesets/25563418`, then PUT it back with the existing deletion rule, non-fast-forward rule, and required checks `ci`, `pr-hygiene`, and `spec-drift`, plus a `merge_queue` rule: `merge_method` `MERGE`, `max_entries_to_build` 1, `max_entries_to_merge` 1, `min_entries_to_merge` 1, `min_entries_to_merge_wait_minutes` 0, `grouping_strategy` `ALLGREEN`, `check_response_timeout_minutes` 60. Do not drop a rule that is already there. Verify a follow-up GET shows those fields, and paste that JSON into the implementation pull request.

## 5. Changelog bullet

- [ ] 5.1 Add one `### Changed` bullet under `## [Unreleased]` in `CHANGELOG.md` for the merge-queue archive gate, in a commit before the archive commit. The bullet links the implementation pull request. Verify the bullet is under `## [Unreleased]` and that the later archive commit does not modify `CHANGELOG.md`.

## 6. Proof

- [ ] 6.1 Run the AGENTS.md proof commands and verify each exits 0: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo build --workspace`, `cargo test --workspace`, and `npx @fission-ai/openspec@1.14.3 validate --all --strict`.

## Workflow follow-up

- As the final commit of the implementation pull request, run `openspec archive archive-at-merge-hygiene --yes`. The subject is exactly that command. The commit deletes `openspec/changes/archive-at-merge-hygiene/` and adds `openspec/changes/archive/<YYYY>-<MM>-<DD>-archive-at-merge-hygiene/`, and it does not modify `CHANGELOG.md`.
