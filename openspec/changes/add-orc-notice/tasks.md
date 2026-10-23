# Tasks

## 1. Notice file

- [x] 1.1 Add `LICENSES/ORC-NOTICE` with the ORC Notice, Attribution, Reserved Material, and Licensed Material parts in `specs/orc-notice/spec.md`, including the three attribution lines and `Project Genesis © 2026 Evan Burkey.` Add the player-campaign note in that file. Verify each scenario that names `LICENSES/ORC-NOTICE` by reading the file.

## 2. README

- [x] 2.1 Add a short `README.md` section that links to `LICENSES/ORC-NOTICE` and repeats the player-campaign note. Verify the README scenario by reading that section.

## 3. Changelog

- [x] 3.1 Add one bullet under `## [Unreleased]` and `### Added` that links this pull request. `LICENSES/ORC-NOTICE` is not an exempt docs path. Verify the changelog scenario by reading the new bullet.

## 4. Proof and archive

- [x] 4.1 Run the AGENTS.md proof commands and verify each exits 0. Paste the output into the pull request.
  - `cargo fmt --all --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo build --workspace`
  - `cargo test --workspace`
  - `npx @fission-ai/openspec@1.14.3 validate --all --strict`

- [ ] 4.2 As the last commit, right before merge, the merger runs `openspec archive add-orc-notice --yes` and checks this box in that commit. Verify the change is absent under `openspec/changes/` and present under `openspec/changes/archive/`.
