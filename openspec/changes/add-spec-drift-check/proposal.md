# Proposal

## Why

A change can reach `main` with an invalid OpenSpec tree, with every task already checked and still sitting under `openspec/changes/`, or as engine code that names no change at all. The completed-but-unarchived case appears when an implementation pull request merges, so the gate has to run on push to `main` as well as on pull requests, while the follow-up archive pull request still passes. This change slots immediately after `add-dice-and-checks` in the Phase 1 order (`add-project-foundation` is already archived). It is a process gate, not a rules procedure, and it does not reorder the rules changes that follow `add-dice-and-checks`.

## What Changes

- Add a `spec-drift` job to `.github/workflows/ci.yml` that runs a small local script. The existing Cargo job is unchanged.
- The job fails when `openspec validate --all --strict` fails.
- The job fails when the tree under test has an active change whose `tasks.md` is fully checked. On push to `main` that catches a just-merged implementation pull request. On pull_request to `main` it fails only when that completed change was already fully checked on the base branch, so the implementation pull request that checks the last boxes still passes, and the archiving pull request passes because the change is no longer active.
- The job fails when a pull request changes `crates/**`, root `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, or `.github/workflows/**` without a pull request body line `OpenSpec-Change: <change-id>` for an id that exists under `openspec/changes/` or `openspec/changes/archive/`. Docs-only and openspec-only pull requests are exempt. Several ids are several such lines, and every line must resolve.
- Pin `@fission-ai/openspec` at `1.14.3`. No external services.
- Record the Phase 1 insertion in `docs/architect/decisions.md` when this change is applied.

This pull request is the spec only. It does not add the script or the job.

## Capabilities

### New Capabilities

- `spec-drift-check`: CI gate for strict OpenSpec validation, completed-but-unarchived changes, and change-id references on code pull requests.

### Modified Capabilities

None. `project-foundation` still requires the Cargo job on push and pull_request to `main`. This change adds a second job in the same workflow file and does not change those requirements.

## Impact

- `.github/workflows/ci.yml`: new `spec-drift` job on the existing push and pull_request triggers for `main`.
- New Node script `scripts/spec-drift-check.mjs` (no npm dependencies) and `node --test` coverage that builds fixture repositories from a table of pass/fail cases.
- Node 22 on the runner, OpenSpec CLI `1.14.3` on `PATH`. The script reads the local checkout and the workflow event payload only.
- Authors of code-path pull requests add one or more `OpenSpec-Change: <change-id>` lines to the pull request body. That is the only reference mechanism.
- `docs/architect/decisions.md`: when applied, state that `spec-drift-check` slots immediately after `add-dice-and-checks` in the Phase 1 order.
