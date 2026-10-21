# Tasks

Genesis DevOps agent implements this change. The implementation pull request's last commit is `openspec archive add-spec-drift-check --yes`. There is no separate archive pull request. `main` stays green.

## 1. Drift script

- [x] 1.1 Add `scripts/spec-drift-check.mjs` with no npm dependencies. It runs `openspec validate --all --strict`, the completed-change check, and the reference check, forwards validation output, prints every failure line from `specs/spec-drift-check/spec.md`, and exits 1 if any check fails. The completed-change check reads only the tree under test and does not compare `tasks.md` with the base. Honor `SPEC_DRIFT_EVENT`, `SPEC_DRIFT_ROOT`, `SPEC_DRIFT_BASE_SHA` for the code-path diff, and `SPEC_DRIFT_BODY`, and otherwise read `GITHUB_EVENT_NAME` and `GITHUB_EVENT_PATH`. Verify `node --check scripts/spec-drift-check.mjs` exits 0.
- [x] 1.2 Add `scripts/spec-drift-check.test.mjs` with `node:test`. For the validation scenarios, build a temporary git repository and a stub `openspec` on `PATH`, run `node scripts/spec-drift-check.mjs`, and assert the exit status, `spec-drift: validate failed`, and the stub output. Verify `node --test scripts/spec-drift-check.test.mjs` exits 0.
- [x] 1.3 Extend the table for every completed-change scenario: checkbox forms (including `- [ x]`, `- [~]`, and `- []`), a pull request whose result is fully checked, the same tree on push as a backstop, a copy that leaves the active directory, and an implementation head whose last commit is `openspec archive <id> --yes` with the following push staying green. Verify `node --test scripts/spec-drift-check.test.mjs` exits 0.
- [x] 1.4 Extend the table for every reference scenario: each code path (including delete and both rename directions), docs-only, openspec-only, a bad id ignored when exempt, several ids, duplicates, a trimmed line, extra words, unknown ids, dated and undated archive ids, a partial suffix, and a code push with no body. Verify `node --test scripts/spec-drift-check.test.mjs` exits 0.

## 2. CI job

- [x] 2.1 Add a `spec-drift` job to `.github/workflows/ci.yml` on the existing push and pull_request triggers for `main`. Check out with `fetch-depth: 0`, set up Node 22, install `@fission-ai/openspec@1.14.3` on `PATH`, run `node --test scripts/spec-drift-check.test.mjs`, then run `node scripts/spec-drift-check.mjs`. Leave the Cargo job steps unchanged. Verify the workflow file shows those triggers, `fetch-depth: 0`, Node 22, the exact package version, both commands, and the Cargo fmt, clippy, build, and test steps.
- [x] 2.2 Add a test that reads `.github/workflows/ci.yml` and asserts the `@fission-ai/openspec@1.14.3` pin, Node 22, and the absence of `@latest` and a version range for that package. Verify `node --test scripts/spec-drift-check.test.mjs` exits 0.

## 3. Phase 1 order

- [x] 3.1 Append a dated decision to `docs/architect/decisions.md` stating that `spec-drift-check` slots immediately after `add-dice-and-checks`, and do not rewrite the 2026-10-20 order lists. Verify the new sentence is present and the existing modifiers-after-dice-and-checks sentence is still present.
- [x] 3.2 Append a dated decision that an implementation pull request's last commit is `openspec archive <id> --yes`, that there is no separate archive pull request, and that this gate leaves `main` green. Verify those sentences are present.

## 4. Scenario coverage

- [x] 4.1 Scenario "Job triggers match the Cargo workflow": read `.github/workflows/ci.yml` and verify the `spec-drift` job runs on push and pull_request to `main` and invokes `scripts/spec-drift-check.mjs`.
- [x] 4.2 Scenario "Cargo steps stay": read the Cargo job and verify it still runs fmt check, clippy with warnings denied, workspace build, and workspace test.
- [x] 4.3 Scenario "Workflow pins the CLI": read the `spec-drift` job and verify it installs `@fission-ai/openspec@1.14.3` and does not use `@latest` or a version range.
- [x] 4.4 Scenario "A missing reference needs no network lookup": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.5 Scenario "Invalid tree fails validation": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.6 Scenario "Valid tree passes validation": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.7 Scenario "Archive children are not active": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.8 Scenario "A sibling of archive is active": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.9 Scenario "A titled box counts": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.10 Scenario "Space around the x still counts as checked": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.11 Scenario "An open box is not fully checked": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.12 Scenario "No checkboxes is not fully checked": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.13 Scenario "A missing tasks file is not fully checked": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.14 Scenario "Other list markers are not checkboxes": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.15 Scenario "A checkbox needs whitespace before the title": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.16 Scenario "A non-x marker is unchecked": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.17 Scenario "An empty box is unchecked": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.18 Scenario "A pull request that leaves every box checked fails": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.19 Scenario "Push backstop fails that same tree": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.20 Scenario "An open task passes the push": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.21 Scenario "An archived completed change passes the push": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.22 Scenario "One of two active changes is complete": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.23 Scenario "Uppercase checked boxes fail the push": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.24 Scenario "Indented boxes fail the push": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.25 Scenario "A later pull request still has the completed change": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.26 Scenario "Copying to archive without removing the active change fails": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.27 Scenario "Checked tasks still active fails the pull request": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.28 Scenario "A new fully checked active change fails": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.29 Scenario "The last commit is the archive": verify this implementation pull request's final commit is `openspec archive add-spec-drift-check --yes`.
- [x] 4.30 Scenario "The archived result passes the pull request": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.31 Scenario "The following push stays green": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.32 Scenario "A crate file is a code path": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.33 Scenario "Root Cargo.toml is a code path": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.34 Scenario "Cargo.lock is a code path": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.35 Scenario "rust-toolchain.toml is a code path": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.36 Scenario "A workflow file is a code path": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.37 Scenario "Deleting a crate file is a code path": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.38 Scenario "Renaming into crates is a code path": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.39 Scenario "Renaming out of crates is a code path": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.40 Scenario "A nested Cargo.toml is not a root manifest": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.41 Scenario "A crates prefix without a slash is not a code path": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.42 Scenario "Files outside the diff do not count": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.43 Scenario "A trimmed line is a reference": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.44 Scenario "Extra words are not a reference": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.45 Scenario "An active id exists": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.46 Scenario "A dated archive id exists": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.47 Scenario "An undated archive directory exists": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.48 Scenario "An unknown id fails": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.49 Scenario "A partial archive suffix does not match": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.50 Scenario "Two existing ids pass": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.51 Scenario "One of two ids is unknown": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.52 Scenario "A duplicate id counts once": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.53 Scenario "Docs-only pull request passes without a reference": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.54 Scenario "Openspec-only pull request passes without a reference": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.55 Scenario "An exempt pull request ignores an unknown id": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.56 Scenario "Docs plus a code path still requires a reference": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.57 Scenario "Openspec plus a workflow file still requires a reference": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.58 Scenario "Docs plus a code path passes with an existing id": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.59 Scenario "A code push has no pull request body": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.60 Scenario "Archived implementation pull request passes": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.61 Scenario "Push of that merge stays green": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.62 Scenario "Validation and a completed change both fail": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.63 Scenario "A clean tree passes": add a test named after this scenario and verify `node --test scripts/spec-drift-check.test.mjs` runs it.
- [x] 4.64 Scenario "The job runs the suite first": read the `spec-drift` job and verify `node --test` runs before `node scripts/spec-drift-check.mjs`.
- [x] 4.65 Scenario "Each case is a fixture repository": verify the suite builds one temporary git repository per case and asserts exit status and failure lines.

## 5. Proof and archive

- [ ] 5.1 Run the AGENTS.md proof commands and verify each exits 0: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo build --workspace`, `cargo test --workspace`, and `npx @fission-ai/openspec@1.14.3 validate --all --strict`
- [ ] 5.2 As the final commit of this implementation pull request, run `openspec archive add-spec-drift-check --yes` and verify the change is no longer active under `openspec/changes/` and sits under `openspec/changes/archive/`
