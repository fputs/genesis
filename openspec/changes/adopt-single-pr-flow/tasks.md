# Tasks

DevOps does sections 1 and 2 on this branch after user id `239839957` submits a review whose body line is `spec-approved: <sha>`. Architect does section 3 after that. Tests are committed before the code they cover. The archive checkbox stays open until the last commit, so the change is not fully checked while it is still active.

## 1. Spec-drift refactor exemption

- [ ] 1.1 In `scripts/spec-drift-check.test.mjs`, add a fixture test for each refactor scenario in `specs/spec-drift-check/spec.md`, before changing the check. Verify `node --test scripts/spec-drift-check.test.mjs` lists those test names.

- [ ] 1.2 Implement those rules in `scripts/spec-drift-check.mjs` and export `isCodePath`. Diff from `git merge-base` of the base sha and the head. Use the kebab item, the `/tests/` path rule, and the failure lines in that spec. Do not edit the spec-drift workflows or the synced changelog specs. Keep `changeReferences` as the `OpenSpec-Change` parser. A status `T` fixture replaces the test file with a symlink. Verify `node --test scripts/spec-drift-check.test.mjs` exits 0.

## 2. spec-approved job

- [ ] 2.1 Add `scripts/spec-approved.test.mjs` with one injected-review fixture per scenario in `specs/spec-approved/spec.md`, and no network call, before adding the production script. Verify the file names each of those scenarios.

- [ ] 2.2 Guard `main()` in `scripts/pr-hygiene.mjs` with `runningAsScript()`, as `scripts/spec-drift-check.mjs` does, and export `unfencedBody`. Add a test that importing the module does not run the check. Add `scripts/spec-approved.mjs` with no npm dependency. It imports `changeReferences`, `isCodePath`, and `unfencedBody`, hashes as the spec requires, checks `git merge-base --is-ancestor`, and fails closed on the log lines in that spec. Verify `node --test` for both test files exits 0.

- [ ] 2.3 Add `.github/workflows/spec-approved.yml` whose job id is `spec-approved`, with no `name:` key and no matrix. Add `openspec/spec-approved-authors.txt` whose only id is `239839957`. Check out the head, fetch `refs/pull/<n>/head`, and run `git worktree add $RUNNER_TEMP/base` for `pull_request.base.sha`. Run tests and the script from that worktree. The head checkout is the repo root. Concurrency group is `spec-approved-${{ github.event.pull_request.number }}`.

- [ ] 2.4 Triggers include pull_request_review `dismissed`, and ignore `DISMISSED` and `PENDING` when choosing the latest review. Permissions are `contents: read` and `pull-requests: read`, with no secrets. Bootstrap only when the base lacks both the script and the workflow file. Add `.github/CODEOWNERS` giving `/openspec/`, `/AGENTS.md`, `/.github/`, and `/scripts/` to `@fputs`. Verify those settings.

- [ ] 2.5 In a commit before archive, add an Unreleased bullet for this change under `### Added`, linking this pull request's number. Verify the new bullet contains `https://github.com/fputs/genesis/pull/` plus that number.

## 3. Constitution and workflow docs

- [ ] 3.1 Update `AGENTS.md` section 1 to the single-pull-request flow in `specs/single-pr-flow/spec.md`, including the Designer citation and the `spec-approved` re-trigger. Verify section 1 no longer says Architect opens an implementation pull request, and that the review, git, and port rules remain.

- [ ] 3.2 Set the `tasks` rule in `openspec/config.yaml` to `Every implementation task list ends with the AGENTS.md proof commands and archiving the change right before merge in the same pull request.` Verify that sentence is the rule.

- [ ] 3.3 Update `docs/architect/decisions.md`, `docs/producer/process.md`, and `docs/qa/test-plan.md` as `specs/single-pr-flow/spec.md` requires, including the 2026-10-23 revision. List required code-owner review through Chief of Staff, the required `spec-approved` check, and that older-base pull requests keep bootstrap until an event refreshes their base. Verify the test plan no longer says that only specs merged to `main` count. Do not edit GitHub branch protection.

## 4. Proof and archive

- [ ] 4.1 Run the AGENTS.md proof commands and verify each exits 0. Paste the output into the pull request.
  - `cargo fmt --all --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo build --workspace`
  - `cargo test --workspace`
  - `npx @fission-ai/openspec@1.14.3 validate --all --strict`

- [ ] 4.2 As the last commit, right before merge, the merger runs `openspec archive adopt-single-pr-flow --yes` and checks this box in that commit. Verify the change is absent under `openspec/changes/` and present under `openspec/changes/archive/`.
