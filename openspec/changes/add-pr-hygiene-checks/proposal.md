# Proposal

## Why

An implementation pull request can merge with no bullet under `## [Unreleased]`, and a later commit can follow `openspec archive` so the archive is not the head. Both have to fail on the pull request. The workflow already runs on the real head, so the body does not also record that SHA.

## What Changes

- Add a pull-request-only job named `pr-hygiene`. It does not run on push. Permissions are `contents: read` only. The body comes from the event payload, so the job does not grant `pull-requests: read`. Triggers are pull_request `opened`, `synchronize`, `reopened`, and `edited`. Concurrency group is `pr-hygiene-<pull request number>` and an in-progress run is cancelled.
- The job parses changed paths with `git diff -z` split on NUL, the same way `scripts/spec-drift-check.mjs` does. Every pull request diff uses `git merge-base` of the event base sha and head sha, not `pull_request.base.sha` directly. The script has no npm dependencies and fails closed when a parse fails or no merge-base exists.
- Changelog bullet: an implementation pull request adds at least one bullet under `## [Unreleased]` in `CHANGELOG.md`, or the job fails with `implementation PR missing CHANGELOG bullet`. Implementation uses the spec-drift code-path classification. Exempt: docs-only, openspec-only, and a diff whose every path is under `.cursor/`, or is exactly `README.md`, `Cargo.lock`, or `rust-toolchain.toml`. No label opts out.
- Archive last: when the body has `OpenSpec-Change: <id>` and the diff removes `openspec/changes/<id>`, the head commit touches only `openspec/changes/archive/**` and `openspec/specs/**` (plus that removal) and its message contains `openspec archive <id>`. Otherwise the job fails with `archive commit is not last`. Several body lines are several archived changes, and each must pass.
- No head-SHA check in pull request bodies. The Cargo job and spec-drift behavior stay as they are.
- When applied, remove "CI enforcement" from the backlog in `docs/architect/decisions.md`, and update `AGENTS.md` and `docs/producer/process.md` so they stop requiring head SHAs in pull request bodies and note that `pr-hygiene` enforces the bullet and archive-last. Genesis DevOps agent implements the job and the script.

This pull request is the spec only. It does not add the script or the job.

## Capabilities

### New Capabilities

- `pr-hygiene`: pull-request CI gate for an Unreleased changelog bullet on implementation pull requests, and for an archive commit that is last when a named change is removed.

### Modified Capabilities

- `changelog`: the Unreleased bullet rule is now CI-enforced by the `pr-hygiene` job.

## Impact

- New workflow `.github/workflows/pr-hygiene.yml` and Node script `scripts/pr-hygiene.mjs` (no npm dependencies) with `node --test` fixtures for each rule and each exemption. `.github/workflows/ci.yml` is unchanged, so the Cargo job and spec-drift keep their current behavior.
- Node 22 on the runner. No new package and no network call. The script reads the local checkout and the workflow event payload only.
- `docs/architect/decisions.md`, `AGENTS.md`, `docs/producer/process.md`, and `CHANGELOG.md` change on the implementation pull request, whose last commit is `openspec archive add-pr-hygiene-checks --yes`.
- Genesis DevOps agent implements the job and the script. This pull request stays spec only.
