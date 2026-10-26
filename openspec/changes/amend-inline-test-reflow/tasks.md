# Tasks

Genesis DevOps agent implements this change after `spec-approved`. Do not edit `crates/rules`. Do not edit `scripts/pr-hygiene.mjs`, `scripts/spec-drift-check.mjs`, or `scripts/spec-drift-check.test.mjs`. Do not add a dependency. Do not add a Rust parser, a token checker, or a pull-request-body reader. Do not edit `CHANGELOG.md`. The body includes `OpenSpec-Change: amend-inline-test-reflow`.

## 1. Constitution text

- [ ] 1.1 In `AGENTS.md` section 1, in the refactor bullet, replace the sentence `Any other whitespace change is.` with the block below. In `docs/architect/decisions.md`, in the refactor paragraph, replace that same sentence with the same block. Change no other sentence. Do not add the block to section 3 or section 8. Verify both files contain `A rustfmt-only reflow of a moved block is not a change, as long as a whitespace-insensitive token check shows the code is unchanged.` and `The pull request body carries that token-check output. Pull request 88 is the precedent.` and still contain `Any other whitespace change is.` Verify section 3 does not contain `rustfmt-only reflow`. Verify this commit does not modify `CHANGELOG.md`.

```
A rustfmt-only reflow of a moved block is not a change, as long as a whitespace-insensitive token check shows the code is unchanged. The pull request body carries that token-check output. Pull request 88 is the precedent. Any other whitespace change is.
```

## 2. Proof and archive

- [ ] 2.1 Run the AGENTS.md proof commands and verify each exits 0: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo build --workspace`, `cargo test --workspace`, and `npx @fission-ai/openspec@1.14.3 validate --all --strict`. `Review allows a rustfmt-only reflow` is review-enforced. Do not add a job fixture for it.
- [ ] 2.2 As the final commit of this pull request, after Architect and QA have both passed the previous head, run `openspec archive amend-inline-test-reflow --yes` and check this box in that same commit. The commit deletes `openspec/changes/amend-inline-test-reflow/` and adds `openspec/changes/archive/<YYYY>-<MM>-<DD>-amend-inline-test-reflow/`. It does not modify `CHANGELOG.md`. Leave this box open until that commit so the change is not a completed unarchived change. Verify the change is no longer an active directory under `openspec/changes/`.

## Workflow follow-up

- Task 2.2 is the archive commit, right before merge, and only after Architect and QA have passed the head that was archived.
