# Tasks

## 1. Constitution file

- [x] 1.1 Replace root `AGENTS.md` with the coding constitution and verify section 2's proof commands are in a fenced code block

## 2. OpenSpec config

- [x] 2.1 Append the coding-constitution sentence inside the `context: |` block of `openspec/config.yaml`, after the existing paragraph and at the same indentation, and verify that line is present
- [x] 2.2 Add `tasks:` under `rules:` at the same level as `specs:`, with the single item that every implementation task list ends with the AGENTS.md proof commands and archiving the change, and verify no other line changed

## 3. Proof and archive

- [x] 3.1 Run the AGENTS.md proof commands and verify each exits 0: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo build --workspace`, `cargo test --workspace`, and `npx @fission-ai/openspec@1.14.3 validate --all --strict`
- [ ] 3.2 After Evan merges, archive the change with `openspec archive add-coding-constitution --yes` and verify it moves under `openspec/changes/archive/`
