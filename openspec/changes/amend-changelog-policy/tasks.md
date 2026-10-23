# Tasks

Genesis DevOps agent implements this change. The spec pull request already removed the changelog backlog from `docs/architect/decisions.md` and marked those items as moved into this spec. Do not put that backlog back. The changelog bullet is added in an earlier commit than the archive. The implementation pull request's last commit archives the change. There is no separate archive pull request. The body includes `OpenSpec-Change: amend-changelog-policy`.

## 1. Changelog text and the release owner

- [ ] 1.1 On the Unreleased bullet whose text is `` `openspec/specs/spec-drift-check` spec, archived from the add-spec-drift-check change. ``, append ` ([#10](https://github.com/fputs/genesis/pull/10))` and do not otherwise reword it. Leave every other bullet as it is. Do not add `### Deprecated`. Verify that line contains `https://github.com/fputs/genesis/pull/10` and that `CHANGELOG.md` has no `### Deprecated` heading
- [ ] 1.2 In the Changelog bullet of `docs/producer/process.md`, state that DevOps adds `## [x.y.z] - YYYY-MM-DD` and moves the bullets then under `## [Unreleased]` into that section, and that no other role does. Keep the sentences that the implementation pull request writes the Unreleased entry and that there is no separate post-merge changelog pull request. Verify those three statements are all present in that bullet

## 2. Category and pull request number

- [ ] 2.1 In `scripts/pr-hygiene.mjs`, require at least one new Unreleased bullet whose nearest unfenced `###` heading is `Added`, `Changed`, `Deprecated`, `Removed`, `Fixed`, or `Security`, and whose text contains `https://github.com/fputs/genesis/pull/N` where N is `pull_request.number` and the next character is not a digit. Read N from `PR_HYGIENE_NUMBER` when that override is set, otherwise from the event payload. When a bullet is required and N is missing, fail closed with `implementation PR missing CHANGELOG bullet`. Keep the exempt paths, the fence rules, and the rule that an edit which does not raise the Unreleased count is not new. Add no npm dependency. Verify `node --check scripts/pr-hygiene.mjs` exits 0
- [ ] 2.2 Set the harness number to 7 unless a row overrides it. Every fixture whose job passes because a bullet was added, including archive and draft rows, uses a valid category heading and `https://github.com/fputs/genesis/pull/7` (or that row's number). Update the fixture for "Any Unreleased list item passes" so its new line is under `### Added` and links pull request 7. Verify `node --test scripts/pr-hygiene.test.mjs` exits 0
- [ ] 2.3 Add a test named "A bullet with no link fails" for the scenario of that name, and verify `node --test scripts/pr-hygiene.test.mjs` runs it and the job logs `implementation PR missing CHANGELOG bullet`
- [ ] 2.4 Add a test named "A bullet under an unknown heading fails" for the scenario of that name, and verify `node --test scripts/pr-hygiene.test.mjs` runs it and the job logs `implementation PR missing CHANGELOG bullet`
- [ ] 2.5 Add a test named "A bullet that links another pull request fails" for the scenario of that name, with pull request number 80 and a link to pull request 10, and verify `node --test scripts/pr-hygiene.test.mjs` runs it and the job logs `implementation PR missing CHANGELOG bullet`
- [ ] 2.6 Add a test named "A Deprecated bullet linking this pull request passes" for the scenario of that name, and verify `node --test scripts/pr-hygiene.test.mjs` runs it and the job exits 0
- [ ] 2.7 Add a test named "A missing pull request number fails closed" for the scenario of that name, with no `PR_HYGIENE_NUMBER` and no `pull_request.number`, and verify `node --test scripts/pr-hygiene.test.mjs` runs it and the job logs `implementation PR missing CHANGELOG bullet`
- [ ] 2.8 Add the implementation pull request's own bullet under `## [Unreleased]` in the section that fits, linking this pull request's own number, in a commit before the archive commit. Verify the new bullet contains `https://github.com/fputs/genesis/pull/` plus that pull request's number, and that the #10 repair from task 1.1 is still present

## 3. Proof and archive

- [ ] 3.1 Run the AGENTS.md proof commands and verify each exits 0:
  - `cargo fmt --all --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo build --workspace`
  - `cargo test --workspace`
  - `npx @fission-ai/openspec@1.14.3 validate --all --strict`
- [ ] 3.2 As the final commit of the implementation pull request, run `openspec archive amend-changelog-policy --yes` and verify the change is no longer active under `openspec/changes/` and sits under `openspec/changes/archive/`
