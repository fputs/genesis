# Tasks

Genesis DevOps agent implements this change. Do not edit `crates/rules`. Do not add a dependency. Do not edit the ruleset or `.github/workflows/ci.yml`. Keep the failure line `archive commit is not last`. Permissions stay `contents: read`.

## 1. Draft skips archive-last

- [ ] 1.1 In `scripts/pr-hygiene.mjs`, when `pull_request.draft` is true, run the bullet check and do not apply archive-last. When the field is absent or false, keep today's archive-last rules. Also fail a non-exempt implementation pull request when an `OpenSpec-Change:` body line names an id whose `openspec/changes/<id>/` directory is still on the head. A pull request with no `OpenSpec-Change:` line does not fail because another change is active. Read `draft` from the event payload. Verify `node --test scripts/pr-hygiene.test.mjs` for "A draft unarchived implementation pull request passes", "Converting back to draft relaxes archive-last", "Archive then mark ready passes", "A non-draft unarchived implementation pull request fails", "A non-draft implementation pull request with no change of its own passes", "A push after ready re-runs the full check", "Both checks fail together", "A named change that stays active skips archive-last", and "A removal without a reference fails".

## 2. Workflow types

- [ ] 2.1 In `.github/workflows/pr-hygiene.yml`, add pull_request types `ready_for_review` and `converted_to_draft` next to `opened`, `synchronize`, `reopened`, and `edited`, targeting `main`. Do not add a push trigger or a `merge_group` trigger. Verify `node --test scripts/pr-hygiene.test.mjs` for "Job runs on its pull request types", "The job does not run on push", and "Permissions are contents read only".

## 3. spec-drift fixture cleanup

- [ ] 3.1 In `scripts/spec-drift-check.test.mjs`, remove each fixture git directory through an injectable remover. When a call throws `ENOTEMPTY`, including on `.git/objects`, retry with backoff for at most 5 calls, then fail the test with that error. A remover that throws once and then succeeds passes. Do not use `fs.rm` `maxRetries` as the mechanism. `--test-concurrency=1` alone does not meet this. Verify `node --test scripts/spec-drift-check.test.mjs` for "A transient ENOTEMPTY is retried", "Retries stop after five", and "Each case is a fixture repository".

## 4. Changelog bullet

- [ ] 4.1 Add one `### Changed` bullet under `## [Unreleased]` for the draft archive-last gate, in a commit before the archive commit. The bullet links the implementation pull request. Verify the bullet is under `## [Unreleased]` and that the later archive commit does not modify `CHANGELOG.md`.

## 5. Proof

- [ ] 5.1 Run the AGENTS.md proof commands and verify each exits 0: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo build --workspace`, `cargo test --workspace`, and `npx @fission-ai/openspec@1.14.3 validate --all --strict`.

## Workflow follow-up

- As the final commit of the implementation pull request, run `openspec archive archive-at-merge-hygiene --yes`. The subject is exactly that command. The commit does not modify `CHANGELOG.md`.
