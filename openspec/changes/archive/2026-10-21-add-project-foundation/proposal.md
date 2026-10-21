# Proposal

## Why

Genesis v2 has no workspace, continuous integration, changelog, or agent guide after the reset. Those have to be specified before any rules procedure is ported from v1-reference. This change records that foundation and the role documentation tree.

## What Changes

- Add a Cargo workspace whose only member is an empty `crates/rules` library with no dependencies, pinned to Rust 1.99.1 (current stable).
- Add GitHub Actions CI on push and pull_request to `main`, running the required Cargo checks and caching Cargo.
- Add root `CHANGELOG.md` in Keep a Changelog format, with an Unreleased section noting the v2 restart and that v1 is preserved on `v1-reference`.
- Add a short root `AGENTS.md` pointing agents at the OpenSpec workflow and the rules in `openspec/config.yaml`.
- Add `docs/` for roles architect, designer, visual, qa, producer, and audio, indexed by `docs/README.md`. No `writer` folder; Writer works in another repo.
- Do not add game or editor crates, and do not implement rules procedures in this change.

## Capabilities

### New Capabilities

- `project-foundation`: Repository workspace, continuous integration, changelog, agent guide, and role documentation tree.

### Modified Capabilities

None.

## Impact

Root workspace files, `rust-toolchain.toml`, `.github/workflows/ci.yml`, `CHANGELOG.md`, `AGENTS.md`, and `docs/`. The docs tree lands with this proposal. Cargo, CI, the changelog, and `AGENTS.md` stay unimplemented until the change is applied.

Acceptance when applied: CI is green on the pull request, and `cargo test --workspace` passes with 0 tests. `openspec validate add-project-foundation --strict` passes on this change.
