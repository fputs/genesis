# Design

## Context

See proposal.md for why. `Inline test block move` already lets review treat a uniform leading-indentation change as no change, and it says any other whitespace change is a change. The spec-drift job does not parse Rust and does not compare a removed block to a new test file. Normative behavior is the delta spec. The requirement description stays the current paragraph: it is already 491 characters, and strict validation fails a description over 500.

## Goals / Non-Goals

**Goals:**

- Take a rustfmt-only reflow out of "any other whitespace change" when a whitespace-insensitive token check shows the code is unchanged and the pull request body carries that output.
- Leave that check with review.

**Non-Goals:**

- No token checker, Rust parser, or pull-request-body reader in `scripts/spec-drift-check.mjs` or `scripts/pr-hygiene.mjs`.
- No second copy of the rule in `AGENTS.md` section 3 or section 8.
- No changelog bullet and no new dependency.

## Decisions

### 1. Subtract the reflow from the existing sentence

The constitution sentence is `Any other whitespace change is.` Implementation replaces that one sentence, in `AGENTS.md` section 1's refactor bullet and in the refactor paragraph of `docs/architect/decisions.md`, with:

```
A rustfmt-only reflow of a moved block is not a change, as long as a whitespace-insensitive token check shows the code is unchanged. The pull request body carries that token-check output. Pull request 88 is the precedent. Any other whitespace change is.
```

The uniform-indentation sentence stays. `Any other whitespace change is.` stays, and it no longer covers the reflow the new sentence names. No new bullet, and no new requirement. Section 3 is the rustfmt settings list. `amend-refactor-test-moves` did not put this rule there.

Alternative considered: add the reflow sentence and leave `Any other whitespace change is.` unqualified. Rejected. The two sentences would disagree, and the change would only add.

### 2. Review reads the body. The job does not

Pull request 88 is the precedent for carrying the output: its body pastes the whitespace-insensitive token check and the output. This change does not add that script to the repo. A token that differs is still a change, including a trailing comma that rustfmt deletes. Pull request 88's `standard_pair` comma is not waived by this sentence. The precedent is the body output, not that comma.

Alternative considered: teach the job to tokenize the moved block. Rejected. The requirement still says the job does not distinguish a verbatim move from an edit, and does not parse either file.

### 3. The new scenario is review-enforced

`Review allows a rustfmt-only reflow` and the narrowed reject scenario are review checks. No fixture is added to `scripts/spec-drift-check.test.mjs`. The job cases from `amend-refactor-test-moves` stay as they are.

## Risks / Trade-offs

- [A pasted token check can disagree with the diff] → Review compares them. The job must not grow a parser to close that hole.
- [rustfmt deletes a token, such as a trailing comma] → The check shows the code changed, so the reflow is still a change. The wording does not say "format the old source and compare."
- [Hand-edited whitespace that is not rustfmt] → `Any other whitespace change is.` still rejects it.

## Migration Plan

No data migration. Rollback is reverting this change. There is no Unreleased bullet: the implementation paths are `AGENTS.md` and `docs/architect/decisions.md`, and the archive commit stays under `openspec/`.
