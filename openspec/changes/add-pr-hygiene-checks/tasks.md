# Tasks

Genesis DevOps agent implements this change. The changelog bullet is added in an earlier commit. The implementation pull request's last commit archives the change. There is no separate archive pull request. The body includes `OpenSpec-Change: add-pr-hygiene-checks`.

## 1. Hygiene script

- [ ] 1.1 Add `scripts/pr-hygiene.mjs` with no npm dependencies. It parses paths with `git diff -z` split on NUL, including both rename sides, as `scripts/spec-drift-check.mjs` does. Every pull request diff, including classification, the bullet diff, and the base-side Unreleased section, uses `git merge-base` of the event base sha and head sha, not `pull_request.base.sha` directly. Archive-last reads that head commit's message and first-parent diff. It fails closed when the event, a diff, a NUL record, the base, the head, the head's first parent, or the merge-base cannot be parsed. Honor `PR_HYGIENE_EVENT`, `PR_HYGIENE_ACTION`, `PR_HYGIENE_ROOT`, `PR_HYGIENE_BASE_SHA`, `PR_HYGIENE_HEAD_SHA`, and `PR_HYGIENE_BODY`, and otherwise read `GITHUB_EVENT_NAME` and `GITHUB_EVENT_PATH`. Verify `node --check scripts/pr-hygiene.mjs` exits 0.
- [ ] 1.2 Add `scripts/pr-hygiene.test.mjs` with `node:test`. Each case builds a temporary git repository and runs `node scripts/pr-hygiene.mjs`. Verify `node --test scripts/pr-hygiene.test.mjs` exits 0.
- [ ] 1.3 Scenario "Each case is a fixture repository": verify the suite builds one temporary git repository per case and asserts exit status and failure lines.
- [ ] 1.4 Scenario "NUL-split diff includes both rename sides": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.
- [ ] 1.5 Scenario "A rename into crates is an implementation pull request": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.
- [ ] 1.6 Scenario "A diff parse error fails closed": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.
- [ ] 1.7 Scenario "A missing base fails closed": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.
- [ ] 1.8 Scenario "A missing head parent fails closed": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.
- [ ] 1.9 Scenario "The check uses the event head": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.
- [ ] 1.10 Scenario "A crate file is an implementation pull request": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.
- [ ] 1.11 Scenario "A workflow file is an implementation pull request": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.
- [ ] 1.12 Scenario "Root Cargo.toml is an implementation pull request": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.
- [ ] 1.13 Scenario "A scripts-only pull request is not an implementation pull request": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.
- [ ] 1.14 Scenario "A nested Cargo.toml is not a root manifest": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.
- [ ] 1.15 Scenario "An added Unreleased bullet passes": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.
- [ ] 1.16 Scenario "A missing bullet fails": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.
- [ ] 1.17 Scenario "A bullet outside Unreleased fails": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.
- [ ] 1.18 Scenario "Any Unreleased list item passes": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.
- [ ] 1.19 Scenario "An unchanged existing bullet fails": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.
- [ ] 1.20 Scenario "Docs-only passes without a bullet": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.
- [ ] 1.21 Scenario "Openspec-only passes without a bullet": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.
- [ ] 1.22 Scenario "A cursor-only pull request passes without a bullet": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.
- [ ] 1.23 Scenario "A README-only pull request passes without a bullet": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.
- [ ] 1.24 Scenario "A Cargo.lock-only pull request passes without a bullet": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.
- [ ] 1.25 Scenario "A rust-toolchain-only pull request passes without a bullet": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.
- [ ] 1.26 Scenario "The exempt path set together passes without a bullet": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.
- [ ] 1.27 Scenario "Docs plus a crate still needs a bullet": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.
- [ ] 1.28 Scenario "Openspec plus a workflow file still needs a bullet": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.
- [ ] 1.29 Scenario "Cursor plus a crate still needs a bullet": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.
- [ ] 1.30 Scenario "README plus root Cargo.toml still needs a bullet": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.
- [ ] 1.31 Scenario "Cargo.lock plus a crate still needs a bullet": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.
- [ ] 1.32 Scenario "rust-toolchain.toml plus a workflow file still needs a bullet": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.
- [ ] 1.33 Scenario "A label does not skip the bullet": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.
- [ ] 1.34 Scenario "Extra words are not a change reference": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.
- [ ] 1.35 Scenario "A fenced line still counts": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.
- [ ] 1.36 Scenario "The archive commit passes": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.
- [ ] 1.37 Scenario "A path outside archive and specs fails": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.
- [ ] 1.38 Scenario "A missing archive phrase fails": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.
- [ ] 1.39 Scenario "A longer id does not cover a shorter id": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.
- [ ] 1.40 Scenario "A named change that stays active skips archive-last": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.
- [ ] 1.41 Scenario "A removal without a reference skips archive-last": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.
- [ ] 1.42 Scenario "Two archived ids pass": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.
- [ ] 1.43 Scenario "One of two archive phrases is missing": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.
- [ ] 1.44 Scenario "A duplicate reference counts once": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.
- [ ] 1.45 Scenario "Edited re-reads the body": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.
- [ ] 1.46 Scenario "The bullet is added before the archive commit": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.
- [ ] 1.47 Scenario "A bullet inside the archive commit is not last": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.
- [ ] 1.48 Scenario "A body without a head SHA passes": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.
- [ ] 1.49 Scenario "Both checks fail together": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.
- [ ] 1.50 Scenario "The hygiene job enforces the bullet": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.
- [ ] 1.51 Scenario "main advanced after branching": add a test named after this scenario and verify `node --test scripts/pr-hygiene.test.mjs` runs it.

## 2. CI job

- [ ] 2.1 Add `.github/workflows/pr-hygiene.yml` with job `pr-hygiene` on pull_request `opened`, `synchronize`, `reopened`, and `edited` targeting `main`. Permissions are `contents: read` and `pull-requests: read`. Check out with `fetch-depth: 0`, set up Node 22, run `node --test scripts/pr-hygiene.test.mjs`, then run `node scripts/pr-hygiene.mjs`. Do not install an npm package. Do not edit `.github/workflows/ci.yml`. Verify the new file shows those triggers, permissions, Node 22, both commands, and no `npm install`, and that `git diff` for `ci.yml` is empty.
- [ ] 2.2 Scenario "Job runs on the four pull request types": read `.github/workflows/pr-hygiene.yml` and verify the job runs on those four pull_request types targeting `main` and invokes `scripts/pr-hygiene.mjs`.
- [ ] 2.3 Scenario "The job does not run on push": read the workflow and verify it has no push trigger.
- [ ] 2.4 Scenario "Permissions are contents and pull requests read": read the workflow and verify permissions are `contents: read` and `pull-requests: read`.
- [ ] 2.5 Scenario "The job installs no npm package": read the job and verify it installs no npm package, and verify `scripts/pr-hygiene.mjs` imports only Node built-ins.
- [ ] 2.6 Scenario "The job runs the suite first": read the job and verify Node 22 and that `node --test` runs before `scripts/pr-hygiene.mjs`.
- [ ] 2.7 Scenario "Cargo and spec-drift stay": read `.github/workflows/ci.yml` and verify the Cargo job and the `spec-drift` job are unchanged and `pr-hygiene` is not a job in that file.

## 3. Docs and changelog bullet

- [ ] 3.1 Remove backlog item "CI enforcement of the changelog bullet." from `docs/architect/decisions.md` and renumber the remaining backlog items. Leave the text of those remaining items as it is. Verify that sentence is gone and the Deprecated, missing-link, release-cutting, and mixed-category items are still present.
- [ ] 3.2 Update `AGENTS.md` and `docs/producer/process.md` so they do not require a head commit SHA in the pull request body, and so they note that the `pr-hygiene` job enforces the Unreleased bullet and archive-last. Keep the section 2 proof-command paste. Verify neither file requires a head SHA, and both name `pr-hygiene` for the bullet and for archive-last.
- [ ] 3.3 Add a `### Added` bullet under `## [Unreleased]` in `CHANGELOG.md` for this job, with a link to the implementation pull request. Put that edit in a commit before the archive commit. Verify the bullet is under `## [Unreleased]` and that the archive commit does not modify `CHANGELOG.md`.

## 4. Proof

- [ ] 4.1 Run the AGENTS.md proof commands and verify each exits 0: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo build --workspace`, `cargo test --workspace`, and `npx @fission-ai/openspec@1.14.3 validate --all --strict`

## Workflow follow-up

- As the final commit of the implementation pull request, run `openspec archive add-pr-hygiene-checks --yes`. Verify the change is no longer active under `openspec/changes/` and sits under `openspec/changes/archive/`. That commit touches only `openspec/changes/archive/**`, `openspec/specs/**`, and the removed `openspec/changes/add-pr-hygiene-checks/**`, and its message contains `openspec archive add-pr-hygiene-checks`.
