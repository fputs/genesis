# Tasks

Genesis DevOps agent implements this change. Do not edit `crates/rules`. Do not add a dependency. Do not edit the ruleset or `.github/workflows/ci.yml`. Keep the failure line `archive commit is not last`. Permissions stay `contents: read`.

## 1. Draft skips archive-last

- [ ] 1.1 In `scripts/pr-hygiene.mjs`, when `pull_request.draft` is true, run the bullet check and do not apply archive-last. When the field is absent or false, keep today's archive-last rules, and fail a non-exempt implementation pull request whose merge-base has an active change id and whose head removed none. Read `draft` from the event payload. Verify `node --test scripts/pr-hygiene.test.mjs` for "A draft unarchived implementation pull request passes", "Converting back to draft relaxes archive-last", "Archive then mark ready passes", "A non-draft unarchived implementation pull request fails", "A push after ready re-runs the full check", "Both checks fail together", "A named change that stays active skips archive-last", and "A removal without a reference fails".

## 2. Workflow types

- [ ] 2.1 In `.github/workflows/pr-hygiene.yml`, add pull_request types `ready_for_review` and `converted_to_draft` next to `opened`, `synchronize`, `reopened`, and `edited`, targeting `main`. Do not add a push trigger or a `merge_group` trigger. Verify `node --test scripts/pr-hygiene.test.mjs` for "Job runs on the four pull request types", "Ready and draft conversion re-run the job", "The job does not run on push", and "Permissions are contents read only".

## 3. spec-drift fixture cleanup

- [ ] 3.1 In `scripts/spec-drift-check.test.mjs`, retry removal of each fixture git directory when `rmdir` returns `ENOTEMPTY`, including on `.git/objects`, until the directory is gone. Fail the test only if it remains. `maxRetries` on `fs.rm` meets this. `--test-concurrency=1` alone does not. Verify `node --test scripts/spec-drift-check.test.mjs` for "A transient ENOTEMPTY is retried" and "Each case is a fixture repository".

## 4. Changelog bullet

- [ ] 4.1 Add one `### Changed` bullet under `## [Unreleased]` for the draft archive-last gate, in a commit before the archive commit. The bullet links the implementation pull request. Verify the bullet is under `## [Unreleased]` and that the later archive commit does not modify `CHANGELOG.md`.

## 5. Proof

- [ ] 5.1 Run the AGENTS.md proof commands and verify each exits 0: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo build --workspace`, `cargo test --workspace`, and `npx @fission-ai/openspec@1.14.3 validate --all --strict`.

## Workflow follow-up

- As the final commit of the implementation pull request, run `openspec archive archive-at-merge-hygiene --yes`. The subject is exactly that command. The commit does not modify `CHANGELOG.md`.
