# Tasks

Genesis DevOps agent implements this change. Do not edit `crates/rules`. Do not add a dependency. Do not edit `scripts/pr-hygiene.mjs`. Keep failure prefixes `spec-drift: invalid change id`, `spec-drift: unknown change`, and `spec-drift: code change missing OpenSpec-Change`.

## 1. One space and a visible invalid id

- [ ] 1.1 In `scripts/spec-drift-check.mjs`, keep a reference as one trimmed body line of `OpenSpec-Change:` plus one U+0020 plus one kebab token. A tab, or any other whitespace before that token, stays an invalid id and is not case-folded. Replace `JSON.stringify` with a per-UTF-16-code-unit escape: copy U+0020 through U+007E, and write every other unit as `\u` plus four lowercase hex digits, with no quotes. An empty token stays `spec-drift: invalid change id` with no trailing token. Verify `node --test scripts/spec-drift-check.test.mjs` for scenarios "A trimmed line is a reference", "An uppercase id is not skipped", "A valid id does not hide an invalid token", "A tab after the colon is invalid", "Two spaces before the id are invalid", "A zero-width space is visible", "An NBSP is visible", "An em space is visible", and "An empty or spaceless change line is invalid".

## 2. Malformed lines fail as invalid ids

- [ ] 2.1 Stop skipping a line that starts with `OpenSpec-Change:` and is not exactly one U+0020 plus one non-whitespace token. Whitespace is ASCII whitespace plus Unicode space separators. Log the remainder after that one U+0020 when it is present, otherwise the remainder after the colon, escaped as in task 1.1. Do not also log `spec-drift: code change missing OpenSpec-Change`. Verify `node --test scripts/spec-drift-check.test.mjs` for scenarios "Extra words are not a reference", "A tabbed extra word beside a valid line fails", and "A docs-only malformed line fails". The tabbed-extra fixture is `OpenSpec-Change:` plus one U+0020 plus a tab plus `Add-Foo extra` beside a valid `OpenSpec-Change: add-dice-and-checks` line, and it must exit non-zero.

## 3. Active ids and archive-stripped ids

- [ ] 3.1 In the archive lookup, a child whose name matches `<YYYY>-<MM>-<DD>-<id>` contributes only the full remainder. Any other archive child contributes its own name. Do not also accept the dated folder name. An active direct child other than `archive` is still an id, including a name that looks like a date. An unknown kebab id on a code path stays `spec-drift: unknown change <id>`. Verify `node --test scripts/spec-drift-check.test.mjs` for scenarios "An active id exists", "A dated archive id exists", "An undated archive directory exists", "An unknown id fails", "A partial archive suffix does not match", "A dated archive folder name is not an id", "The stripped id of a dated folder exists", and "An active dated directory name is still an id".

## 4. Byte order of completed changes

- [ ] 4.1 Keep the completed-unarchived id sort as UTF-8 byte order. Do not use `localeCompare`. Verify `node --test scripts/spec-drift-check.test.mjs` for scenario "Completed changes are listed in byte order", where `Zed` is logged before `alpha`.

## 5. Shared spec-drift concurrency

Workflow scenarios in this section are verified by a line or regex check of the workflow file, with no YAML dependency.

- [ ] 5.1 On the `spec-drift` job in `.github/workflows/ci.yml` and the `spec-drift` job in `.github/workflows/spec-drift-edited.yml`, set job-level concurrency group `spec-drift-${{ github.event.pull_request.number || github.sha }}` and `cancel-in-progress: true`. Remove the workflow-level `spec-drift-edited-` group. Do not add a concurrency key to the Cargo `ci` job. Do not add an `edited` type or a skip `if` to `ci.yml`. Keep `node --test scripts/spec-drift-check.test.mjs` before `node scripts/spec-drift-check.mjs` in the edited workflow. Verify scenarios "Job triggers match the Cargo workflow", "The edited workflow reuses the spec-drift job name", "Both spec-drift jobs share a pull request group", "A push does not join a pull request group", "Editing the body does not replace a failed Cargo check", and "Cargo and spec-drift stay".
- [ ] 5.2 Add the `### Fixed` bullet under `## [Unreleased]` for this change in a commit before the archive commit. The bullet links the implementation pull request. Verify the bullet is present and that the later archive commit does not modify `CHANGELOG.md`.

## 6. Proof

- [ ] 6.1 Run the AGENTS.md proof commands and verify each exits 0: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo build --workspace`, `cargo test --workspace`, and `npx @fission-ai/openspec@1.14.3 validate --all --strict`.

## Workflow follow-up

- As the final commit of the implementation pull request, run `openspec archive amend-pr-hygiene-followups --yes`. The subject is exactly that command. The commit deletes `openspec/changes/amend-pr-hygiene-followups/` and adds `openspec/changes/archive/<YYYY>-<MM>-<DD>-amend-pr-hygiene-followups/`, and it does not modify `CHANGELOG.md`.
