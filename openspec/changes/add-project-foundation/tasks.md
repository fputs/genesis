# Tasks

## 1. Workspace

- [x] 1.1 Add a root Cargo workspace whose only member is `crates/rules`, and verify `cargo metadata` lists that member
- [x] 1.2 Add `crates/rules` as a library with no dependencies and a source file that is only a doc comment, and verify the manifest has no dependencies
- [x] 1.3 Add `rust-toolchain.toml` pinning channel `1.99.1` and verify the channel field

## 2. Continuous integration

- [x] 2.1 Add `.github/workflows/ci.yml` triggered on push and pull_request to `main`, and verify those triggers
- [x] 2.2 Run `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo build --workspace`, and `cargo test --workspace` in that workflow, with a Cargo cache, and verify each step is present

## 3. Changelog and agent guide

- [x] 3.1 Add root `CHANGELOG.md` in Keep a Changelog format with an Unreleased section noting the v2 restart and that v1 is preserved on `v1-reference`, and verify that text
- [x] 3.2 Add a short root `AGENTS.md` pointing at the OpenSpec workflow and the rules in `openspec/config.yaml`, and verify those pointers

## 4. Role docs

- [x] 4.1 Create `docs/` with `architect`, `designer`, `visual`, `qa`, `producer`, and `audio`, and verify there is no `writer` folder. Empty folders have `.gitkeep`. `docs/architect/decisions.md` and `docs/producer/process.md` are the documents for those roles
- [x] 4.2 Write `docs/README.md` indexing those six folders and including "Design docs and decisions, one folder per role. OpenSpec specs live in openspec/.", and verify `writer` is not listed

## 5. Acceptance

- [x] 5.1 Run `cargo test --workspace` after the workspace exists and verify it passes with 0 tests
- [ ] 5.2 Confirm GitHub Actions is green on the implementation pull request
