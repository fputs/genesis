# Tasks

Genesis DevOps agent implements this change. Do not edit `crates/rules`. Do not add a dependency. Do not edit `scripts/pr-hygiene.mjs`. Keep failure prefixes `spec-drift: invalid change id`, `spec-drift: unknown change`, and `spec-drift: code change missing OpenSpec-Change`.

## 1. One space and a visible invalid id

- [ ] 1.1 In `scripts/spec-drift-check.mjs`, treat a trimmed line that starts with the exact case-sensitive `OpenSpec-Change:` as a reference. It is valid only when the line is that prefix, exactly one U+0020, and one kebab id. Log an invalid id as `spec-drift: invalid change id` plus the whole remainder after `OpenSpec-Change:`, with nothing stripped. Escape each UTF-16 code unit outside U+0020 through U+007E, and `\`, as `\u` plus four lowercase hex digits, with no JSON quotes. Verify `node --test scripts/spec-drift-check.test.mjs` for scenarios "A trimmed line is a reference", "A lowercase prefix is ignored", "An uppercase id is not skipped", "A valid id does not hide an invalid token", "A tab after the colon is invalid", "Two spaces before the id are invalid", "A zero-width space is visible", "An NBSP is visible", "An em space is visible", "A typed backslash escape differs from a real NBSP", and "An empty or spaceless change line is invalid".

## 2. Malformed lines fail as invalid ids

- [ ] 2.1 Stop skipping a line that starts with the exact case-sensitive `OpenSpec-Change:` and is not the prefix, exactly one U+0020, and one kebab id. Log `spec-drift: invalid change id` plus the whole remainder after `OpenSpec-Change:`, escaped as in task 1.1, with nothing stripped. Do not also log `spec-drift: code change missing OpenSpec-Change`. Verify `node --test scripts/spec-drift-check.test.mjs` for scenarios "Extra words are not a reference", "A tabbed extra word beside a valid line fails", and "A docs-only malformed line fails". The tabbed-extra fixture is `OpenSpec-Change:` plus one U+0020 plus a tab plus `Add-Foo extra` beside a valid `OpenSpec-Change: add-dice-and-checks` line, and it must exit non-zero.

## 3. Active ids and archive-stripped ids

- [ ] 3.1 In the archive lookup, a child whose name matches `<YYYY>-<MM>-<DD>-<id>` contributes only the full remainder. Any other archive child contributes its own name. Do not also accept the dated folder name. An active direct child other than `archive` is still an id, including a name that looks like a date. An unknown kebab id on a code path stays `spec-drift: unknown change <id>`. Verify `node --test scripts/spec-drift-check.test.mjs` for scenarios "An active id exists", "A dated archive id exists", "An undated archive directory exists", "An unknown id fails", "A partial archive suffix does not match", "A dated archive folder name is not an id", "The stripped id of a dated folder exists", and "An active dated directory name is still an id".

## 4. Byte order of completed changes

- [ ] 4.1 Keep the completed-unarchived id sort as UTF-8 byte order. Do not use `localeCompare`. Verify `node --test scripts/spec-drift-check.test.mjs` for scenario "Completed changes are listed in byte order", where `Zed` is logged before `alpha`.

## 5. Shared spec-drift concurrency

- [ ] 5.1 On the `spec-drift` job in `.github/workflows/ci.yml` and the `spec-drift` job in `.github/workflows/spec-drift-edited.yml`, set job-level concurrency group `spec-drift-${{ github.event.pull_request.number || github.sha }}` and `cancel-in-progress: true`, so the newest run in the group survives and older pending or running runs are cancelled. Remove the workflow-level `spec-drift-edited-` group. Do not add a concurrency key to the Cargo `ci` job. Do not add an `edited` type or a skip `if` to `ci.yml`. Keep `node --test scripts/spec-drift-check.test.mjs` before `node scripts/spec-drift-check.mjs` in the edited workflow. Add test cases in `scripts/spec-drift-check.test.mjs` that read both workflow files, named "Job triggers match the Cargo workflow", "The edited workflow reuses the spec-drift job name", "Both spec-drift jobs share a pull request group", "A push does not join a pull request group", and "Editing the body does not replace a failed Cargo check". Verify those cases with `node --test scripts/spec-drift-check.test.mjs`. The existing test "Cargo and spec-drift stay" in `scripts/pr-hygiene.test.mjs` still passes.
- [ ] 5.2 Add the `### Changed` bullet under `## [Unreleased]` for this change in a commit before the archive commit. Previously passing pull request bodies can now fail, so the section is `### Changed`, not `### Fixed`. The bullet links the implementation pull request. Verify the bullet is present and that the later archive commit does not modify `CHANGELOG.md`.

## 6. Proof

- [ ] 6.1 Run the AGENTS.md proof commands and verify each exits 0: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo build --workspace`, `cargo test --workspace`, and `npx @fission-ai/openspec@1.14.3 validate --all --strict`.

## Workflow follow-up

- As the final commit of the implementation pull request, run `openspec archive amend-pr-hygiene-followups --yes`. The subject is exactly that command. The commit deletes `openspec/changes/amend-pr-hygiene-followups/` and adds `openspec/changes/archive/<YYYY>-<MM>-<DD>-amend-pr-hygiene-followups/`, and it does not modify `CHANGELOG.md`.
