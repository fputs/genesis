# Tasks

## 1. Drift script

- [ ] 1.1 Add `scripts/spec-drift-check.mjs` with no npm dependencies. It runs `openspec validate --all --strict`, the completed-change check, and the reference check, forwards validation output, prints every failure line from `specs/spec-drift-check/spec.md`, and exits 1 if any check fails. Honor `SPEC_DRIFT_EVENT`, `SPEC_DRIFT_ROOT`, `SPEC_DRIFT_BASE_SHA`, and `SPEC_DRIFT_BODY`, and otherwise read `GITHUB_EVENT_NAME` and `GITHUB_EVENT_PATH`. Verify `node --check scripts/spec-drift-check.mjs` exits 0.
- [ ] 1.2 Add `scripts/spec-drift-check.test.mjs` with `node:test`. For the validation scenarios, build a temporary git repository and a stub `openspec` on `PATH`, run `node scripts/spec-drift-check.mjs`, and assert the exit status, `spec-drift: validate failed`, and the stub output. Verify `node --test scripts/spec-drift-check.test.mjs` exits 0.
- [ ] 1.3 Extend the table for every completed-change scenario: checkbox forms (including `- [ x]`, `- [~]`, and `- []`), push to `main`, the implementation pull request, an inherited completed change, a copy that leaves the active directory, and the archive pull request that moves the change under `openspec/changes/archive/`. Verify `node --test scripts/spec-drift-check.test.mjs` exits 0.
- [ ] 1.4 Extend the table for every reference scenario: each code path (including delete and both rename directions), docs-only, openspec-only, a bad id ignored when exempt, several ids, duplicates, a trimmed line, extra words, unknown ids, dated and undated archive ids, a partial suffix, and a code push with no body. Verify `node --test scripts/spec-drift-check.test.mjs` exits 0.

## 2. CI job

- [ ] 2.1 Add a `spec-drift` job to `.github/workflows/ci.yml` on the existing push and pull_request triggers for `main`. Check out with `fetch-depth: 0`, set up Node 22, install `@fission-ai/openspec@1.14.3` on `PATH`, run `node --test scripts/spec-drift-check.test.mjs`, then run `node scripts/spec-drift-check.mjs`. Leave the Cargo job steps unchanged. Verify the workflow file shows those triggers, `fetch-depth: 0`, Node 22, the exact package version, both commands, and the Cargo fmt, clippy, build, and test steps.
- [ ] 2.2 Add a test that reads `.github/workflows/ci.yml` and asserts the `@fission-ai/openspec@1.14.3` pin, Node 22, and the absence of `@latest` and a version range for that package. Verify `node --test scripts/spec-drift-check.test.mjs` exits 0.

## 3. Phase 1 order

- [ ] 3.1 Append a dated decision to `docs/architect/decisions.md` stating that `spec-drift-check` slots immediately after `add-dice-and-checks`, and do not rewrite the 2026-10-20 order lists. Verify the new sentence is present and the existing modifiers-after-dice-and-checks sentence is still present.

## 4. Integration

- [ ] 4.1 From the repo root, run `node --test scripts/spec-drift-check.test.mjs` and `npx --yes @fission-ai/openspec@1.14.3 validate --all --strict`. Verify both exit 0.
- [ ] 4.2 Run `node scripts/spec-drift-check.mjs` as a pull_request whose base commit does not have `add-spec-drift-check` fully checked, whose diff includes `.github/workflows/ci.yml`, and whose `SPEC_DRIFT_BODY` is `OpenSpec-Change: add-spec-drift-check`. Verify exit 0.

## Workflow follow-up

- After the implementation pull request merges, the push to `main` fails with `spec-drift: completed unarchived change add-spec-drift-check` until an archive pull request moves this change under `openspec/changes/archive/`.
- Archive only after CI on the implementation pull request is green, tasks are checked, and the review rules in `docs/architect/decisions.md` are met. The archive pull request must pass this job.
