# Proposal

## Why

A change can reach `main` with an invalid OpenSpec tree, with every task already checked and still sitting under `openspec/changes/`, or as engine code that names no change at all. The completed-but-unarchived case has to fail on the pull request, before merge, so `main` stays green. Push to `main` runs that same check only as a backstop. This change slots immediately after `add-dice-and-checks` in the Phase 1 order (`add-project-foundation` is already archived). It is a process gate and does not reorder the rules changes that follow `add-dice-and-checks`.

## What Changes

- Add a `spec-drift` job to `.github/workflows/ci.yml` that runs a small local script. The existing Cargo job is unchanged.
- The job fails when `openspec validate --all --strict` fails.
- The job fails when, after a pull request's changes, any active change has a fully checked `tasks.md`. The implementation pull request archives its own change in its last commit, `openspec archive <id> --yes`. There is no separate archive pull request. Push to `main` runs that same check only as a backstop. The designed merge already passed, so `main` does not go red.
- The job fails when a pull request changes `crates/**`, root `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, or `.github/workflows/**` without a pull request body line `OpenSpec-Change: <change-id>` for an id that exists under `openspec/changes/` or `openspec/changes/archive/`. Docs-only and openspec-only pull requests are exempt. Several ids are several such lines, and every line must resolve.
- Pin `@fission-ai/openspec` at `1.14.3`. No external services.
- When applied, record in `docs/architect/decisions.md` that this change slots immediately after `add-dice-and-checks`, and that an implementation pull request's last commit is `openspec archive <id> --yes`. Genesis DevOps agent implements the job and the script.

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
- `docs/architect/decisions.md`: when applied, state that `spec-drift-check` slots immediately after `add-dice-and-checks` in the Phase 1 order, and that an implementation pull request's last commit is `openspec archive <id> --yes`.
- Genesis DevOps agent implements the job and the script. This pull request stays spec only.
