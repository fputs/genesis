# Proposal

## Why

Agents currently have only a short pointer at the OpenSpec workflow. Implementation needs one binding constitution so proof, purity, tests, errors, and dependencies stay the same on every change.

## What Changes

- Replace root `AGENTS.md` with the coding constitution. Section 2 lists the proof commands in a fenced code block.
- Add one sentence to the `context` block in `openspec/config.yaml`, and add a `tasks` rule: every implementation task list ends with those proof commands and archiving the change.
- Add the `coding-constitution` capability. No rules code and no new crates.
- Modify the `project-foundation` agent-guide requirement so root `AGENTS.md` is the coding constitution.

## Capabilities

### New Capabilities

- `coding-constitution`: Agents follow `AGENTS.md`; proof commands pass; lint silences use `#[expect(..., reason)]`; `crates/rules` stays pure and dependency-free; tests use scripted dice and cover every scenario; library code does not panic; new dependencies need a spec.

### Modified Capabilities

- `project-foundation`: The agent guide requirement now says root `AGENTS.md` is the coding constitution.

## Impact

Root `AGENTS.md` and `openspec/config.yaml`. This change does not touch `docs/architect/decisions.md`, `CHANGELOG.md`, or `.github/`. Archive runs after the pull request is merged.
