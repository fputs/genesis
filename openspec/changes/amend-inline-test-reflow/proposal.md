# Proposal

## Why

`amend-refactor-test-moves` (archived, pull request 86) says a uniform leading-indentation change of a moved `#[cfg(test)]` block is not a change, and that any other whitespace change is. `cargo fmt` reflows a moved block by more than a uniform indent. Pull request 88 had to show a whitespace-insensitive token check in the body because that reflow was still a change under the current sentence.

## What Changes

- Subtract a rustfmt-only reflow from the sentence `Any other whitespace change is.` In that place the rule says: "A rustfmt-only reflow of a moved block is not a change, as long as a whitespace-insensitive token check shows the code is unchanged." The pull request body carries that token-check output. Pull request 88 is the precedent. `Any other whitespace change is.` stays for every other whitespace change.
- That replacement is the only edit to the rule text. It lands in the refactor bullet of `AGENTS.md` section 1 and in the refactor paragraph of `docs/architect/decisions.md`. Section 3 states rustfmt settings and does not carry this rule, so it is not given a second copy. Section 8's pointer to the inline test block move rule stays as it is.
- The `Inline test block move` scenarios that state the whitespace rule gain the same sentences. No new requirement. The spec-drift job still does not parse Rust, compare tokens, or read the pull request body.

This pull request's first commit is the spec only. The constitution sentences wait for `spec-approved`.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `spec-drift-check`: `Inline test block move` keeps the verbatim move. It stops treating a rustfmt-only reflow as a change when a whitespace-insensitive token check shows the code is unchanged and the pull request body carries that output. Any other whitespace change stays a change.

## Impact

- Implementation replaces one sentence in `AGENTS.md` section 1 and the same sentence in `docs/architect/decisions.md`. No other sentence in those files changes. `crates/rules` is untouched. `scripts/pr-hygiene.mjs` and `scripts/spec-drift-check.mjs` are untouched.
- No changelog bullet. The implementation diff is `AGENTS.md` and `docs/architect/decisions.md`, and the archive commit stays under `openspec/`. The changelog spec does not require an Unreleased bullet for those paths. Spec-only, docs-only, and archive-only pull requests do not add one.
- No dependency is added. Review enforces the token check. The job's existing-tests log line is unchanged.
