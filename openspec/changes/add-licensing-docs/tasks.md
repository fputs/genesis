# Tasks

## 1. Documents

- [x] 1.1 Add `docs/architect/licensing.md` with numbered rules `LIC-1` onward from the 2026-10-21 ORC research, including `**UNVERIFIED**` marks and the open-questions section. Add no CI job.
- [x] 1.2 Keep the `NOTICE-ORC.md` author-line TODO and the generic Reserved Material list, and mark both `**UNVERIFIED**` pending the books' legal pages.
- [x] 1.3 Link `docs/architect/licensing.md` from the README license section and from `docs/README.md`. Add the contributor rule to `AGENTS.md` section 10 without changing the changelog sentences in sections 2 and 10.

## 2. Manifest and changelog

- [x] 2.1 Set `license = "MIT"` in `crates/rules/Cargo.toml` and add root `LICENSE` with `Copyright (c) 2026 Evan Burkey`.
- [x] 2.2 Add one bullet under `### Added` in `## [Unreleased]` of `CHANGELOG.md` that links to the pull request.

## 3. Proof and archive

- [x] 3.1 Run the AGENTS.md proof commands and verify each exits 0: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo build --workspace`, `cargo test --workspace`, and `npx @fission-ai/openspec@1.14.3 validate --all --strict`. Confirm `node --test scripts/pr-hygiene.test.mjs` and `node --test scripts/spec-drift-check.test.mjs` still pass.
- [x] 3.2 As the final commit, run `openspec archive add-licensing-docs --yes`.
