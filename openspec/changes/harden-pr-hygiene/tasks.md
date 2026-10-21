# Tasks

Genesis DevOps agent implements this change. Do not edit `crates/rules`. Do not add a dependency. Keep failure lines `archive commit is not last` and `implementation PR missing CHANGELOG bullet`. Add only `spec-drift: invalid change id <token>`.

## 1. spec-drift reference and locale

- [ ] 1.1 In `scripts/spec-drift-check.mjs`, treat `OpenSpec-Change:` plus one token as a reference. A token matching `^[a-z0-9]+(?:-[a-z0-9]+)*$` stays the id. Any other token fails with `spec-drift: invalid change id <token>`, is not case-folded, and does not also log `spec-drift: code change missing OpenSpec-Change`. Extra words stay a missing reference. Verify with `node --test scripts/spec-drift-check.test.mjs` for scenarios "An uppercase id is not skipped", "A valid id does not hide an invalid token", "A trimmed line is a reference", and "Extra words are not a reference".
- [ ] 1.2 Start every `git` child in `scripts/spec-drift-check.mjs` with `LC_ALL=C`. Verify scenario "Git diff runs with LC_ALL=C" by a stand-in `git` that records the child environment while the parent has `LC_ALL=fr_FR.UTF-8`.

## 2. spec-drift on edited

- [ ] 2.1 In `.github/workflows/ci.yml`, run pull_request types `opened`, `synchronize`, `reopened`, and `edited`. Give the Cargo job `if: github.event_name != 'pull_request' || github.event.action != 'edited'`. Leave `spec-drift` without that `if`, and do not change its steps or the Cargo steps. Verify scenarios "Job triggers include edited", "Cargo steps stay", and "Cargo does not run on edited" by reading the workflow.
- [ ] 2.2 Add a fixture for scenario "Dropping the change line on edit fails": a code-path pull request that passed with `OpenSpec-Change: add-dice-and-checks` fails on `edited` after the body drops that line, and the log includes `spec-drift: code change missing OpenSpec-Change`. Verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [ ] 2.3 Add the `### Fixed` bullet under `## [Unreleased]` for this job in a commit before the archive commit. The bullet links the implementation pull request. Verify the bullet is present and that the later archive commit does not modify `CHANGELOG.md`.

## 3. Archive-last

- [ ] 3.1 In `scripts/pr-hygiene.mjs`, stop reading the pull request body for archive-last. For each direct child of `openspec/changes/` other than `archive` removed between the merge-base and the head, require the dated archive directory, the head commit's delete, the head commit's add when that directory is new, and the path allowlist. The trimmed `git log -1 --format=%s` subject SHALL be `openspec archive <id> --yes` for each removed id, joined by ` && ` in ascending id order. Do not read the commit body. Verify `node --test scripts/pr-hygiene.test.mjs`.
- [ ] 3.2 Update the archive fixtures to the modified scenarios and verify each runs: "The archive commit passes", "An earlier archive add is not last", "An empty head commit fails", "A specs-only head commit fails", "A phrase below the subject does not count", "A path outside archive and specs fails", "A later fix-typo commit is not last", "A missing archive phrase fails", "A longer id does not cover a shorter id", "Edited re-reads the body", "An edited body drops the change reference", and "A pre-existing archive allows the removal commit". The edited-body and empty-commit fixtures must fail.
- [ ] 3.3 Keep scenario names "A removal without a reference skips archive-last" and "Two archived ids pass", and change their fixtures to the new outcomes: the skip scenario now fails when no dated archive directory exists, and two ids pass only when the subject is `openspec archive add-bar --yes && openspec archive add-foo --yes`. Add "Two archived ids fail" for a second command that appears only on a later line. Keep "One of two archive phrases is missing" failing and "A duplicate reference counts once" passing. Verify those fixtures with `node --test scripts/pr-hygiene.test.mjs`.
- [ ] 3.4 Update "Extra words are not a change reference" and "A fenced line still counts" so neither body shape exempts a removal that has no dated archive directory. Verify both fixtures fail with `archive commit is not last`.

## 4. Bullets, fences, and exemptions

- [ ] 4.1 Count a bullet as new only when its marker-stripped text is not a bullet in any unfenced section of the base `CHANGELOG.md` and the head Unreleased section has more unfenced bullets than the base. Ignore fenced regions: a trimmed line of three or more backticks toggles. Verify scenarios "A moved bullet is not new", "An edited bullet is not new", "A fenced bullet is not new", "A fenced heading does not open the section", "A fenced heading does not close the section", "An added Unreleased bullet passes", and "Any Unreleased list item passes".
- [ ] 4.2 Treat a diff as exempt when every path is under `docs/`, `openspec/`, or `.cursor/`, or is exactly `AGENTS.md`, `CHANGELOG.md`, `README.md`, `Cargo.lock`, or `rust-toolchain.toml`. Verify scenario "Exempt categories together pass without a bullet", and verify the existing single-category and mixed-with-crate fixtures still match their scenarios.
- [ ] 4.3 Start every `git` child in `scripts/pr-hygiene.mjs` with `LC_ALL=C`. Verify scenario "A missing path uses C-locale git text" with a stand-in `git` that prints a non-English missing line unless `LC_ALL=C`.

## 5. Proof

- [ ] 5.1 Run the AGENTS.md proof commands and verify each exits 0: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo build --workspace`, `cargo test --workspace`, and `npx @fission-ai/openspec@1.14.3 validate --all --strict`.

## Workflow follow-up

- As the final commit of the implementation pull request, run `openspec archive harden-pr-hygiene --yes`. The subject is exactly that command. The commit deletes `openspec/changes/harden-pr-hygiene/` and adds `openspec/changes/archive/<YYYY>-<MM>-<DD>-harden-pr-hygiene/`, and it does not modify `CHANGELOG.md`.
