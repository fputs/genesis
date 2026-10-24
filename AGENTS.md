# Genesis coding constitution

Read this before every change. It binds every agent that writes code in this repo. If a rule here conflicts with a spec, stop and report it; do not pick one yourself.

## 1. How work happens
- Work only from an approved OpenSpec change in `openspec/changes/<id>/`. Implement exactly its tasks and scenarios. No extra features, refactors, or "while I'm here" fixes.
- One change = one procedure family = one draft pull request. The body includes `OpenSpec-Change: <id>`. The first commit only adds, modifies, or deletes paths under `openspec/changes/<id>/`. Designer rules on that pull request before tests or code. The pull request links `docs/designer/rules-rulings.md` and does not copy the log. Evan approves the spec only by a pull request review whose body line is `spec-approved: <sha>` for that spec commit. An issue comment does not approve. Tests are committed next, then the code they cover, on the same branch. There is no second pull request.
- A small spec clarification is fixed in this pull request, cites Designer, and needs a new `spec-approved: <sha>` review. A behavior or scope change goes to Chief of Staff for Evan.
- Designer and QA settle edge cases and tie rulings on the spec before coding is dispatched.
- After tests and code land, the router wakes Architect once per head SHA, after CI finishes. Architect and QA review that head in parallel and post one combined numbered fix list. A red CI check is the send-back. Reviewers do not re-check CI-gated items: format, the changelog bullet, archive order, spec drift.
- Apply the whole fix list in one push. After two send-back rounds, escalate to Chief of Staff.
- The merger archives once, as the last commit, right before merge, only after Architect and QA have both passed that head (`openspec archive <id> --yes`). Never un-archive. If a fix is needed after the archive, Architect escalates to Chief of Staff.
- Architect merges when CI is green after that archive commit. Architect and QA have both passed the head that was archived. No per-PR Chief of Staff acceptance. Chief of Staff spot-checks merged PRs daily.
- A refactor with no spec edit uses one body line `OpenSpec-Refactor: <item>` and no `OpenSpec-Change` line. `<item>` is one kebab token, `^[a-z0-9]+(?:-[a-z0-9]+)*$`, and is not looked up. Every changed path starts with `crates/` or is exactly `CHANGELOG.md`. Do not modify a `crates/` path that contains `/tests/`. An in-file #[cfg(test)] block may be moved verbatim into a new test file. A use line or a path line may change, as the inline test block move requirement defines them. A uniform change in the moved block's leading indentation is not a change. Any other whitespace change is. The dropped `#[cfg(test)] mod tests {` and `}` lines, and an added `#[cfg(test)] mod …;` declaration, are not changes. A move into `crates/<crate>/tests/` is allowed only if the block compiles unchanged apart from use and path lines. Asserting logic may not change, in place or in the move. That includes an assertion, an expected value, control flow, or a comment. A dropped block that no new test file contains is a change. A new test that was not in the removed block is not a change to the moved block. Review enforces the verbatim move and the unchanged asserting logic. The spec-drift job does not distinguish a verbatim move from an edit and does not reject a non-test crates/ file that loses a #[cfg(test)] block when a new test file is added. The Unreleased bullet is still required. Architect does not merge until Architect and QA have both confirmed, in the combined review, that behavior is unchanged. No CI job makes that confirmation.
- spec-drift does not require OpenSpec-Change for script and approval paths; spec-approved enforces it. Those paths are `scripts/spec-approved.mjs`, `scripts/spec-approved.test.mjs`, `scripts/spec-drift-check.mjs`, `scripts/pr-hygiene.mjs`, and `openspec/spec-approved-authors.txt`.
- Git: never force-push, rebase pushed commits, or delete branches. Fix mistakes with new commits. Coding agents do not merge to main.
- Port code from branch `v1-reference` only when the change names the file. Port it to these rules, not as-is.

## 2. Proof it works
Before saying "done", run and paste output into the PR:
```
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace
cargo test --workspace
npx @fission-ai/openspec@1.14.3 validate --all --strict
```
All must pass. CI runs the same checks. (Why: "it should work" isn't evidence.)
An implementation pull request adds a changelog entry under ## [Unreleased] before the work is done. Spec-only, docs-only, and archive-only pull requests do not.
- The `pr-hygiene` job enforces the Unreleased bullet and archive-last.

## 3. Formatting and linting
- Default `rustfmt` settings. Don't hand-format, don't add a `rustfmt.toml` without a spec.
- Clippy warnings are errors. Fix the code rather than silencing the lint.
- A lint may be silenced only with `#[expect(clippy::name, reason = "...")]` on the smallest item possible, with a real reason. Never `#[allow]` at crate level. (Why: `expect` complains once the reason goes away, so silences don't rot.)
- Toolchain is pinned in `rust-toolchain.toml`; edition 2024. Don't change either without a spec.

## 4. Crates and modules
- `crates/rules` is the Pathfinder 2e rules engine. It has no Bevy, no serde, no file, network, clock, or global randomness. Callers pass dice in. (Why: the rules must run the same everywhere, in tests and later in the editor.)
- No new crates until their phase (game/editor crates are Phase 2).
- One module per rules concept (`dice`, `check`, `creature`, ...). Modules talk through their public types, not each other's internals.
- Keep the public surface small: `pub` only what another module or a test needs; default to private or `pub(crate)`.
- Identify combatants by `CombatantId`, never by list position or reference.

## 5. Immutability and purity (rules engine)
- Characters are built by composition: immutable `CharacterBuild` input, a pure `derive()` function, a `Creature` output. Same inputs, same character, every time.
- Anything that changes during a fight (current HP, conditions, dying, actions left) lives in `CombatantState`, never in `CharacterBuild` or `Creature`.
- Rules functions take inputs and return results. No hidden state: no `static mut`, globals, interior mutability (`Cell`, `RefCell`, `Mutex`), or reading the clock.
- Prefer returning a new value over mutating one. Where mutation is needed, take `&mut` explicitly so it's visible at the call site.
- All randomness goes through an injected `Dice`. Same seed or script, same result.

## 6. Types and naming
- Standard Rust naming: `snake_case` functions and modules, `CamelCase` types, `SCREAMING_SNAKE_CASE` constants.
- Use Pathfinder's own words: `DegreeOfSuccess`, `Strike`, `Stride`, `flat_check`. A player should recognise the names.
- Make illegal states unrepresentable: enums over booleans and magic numbers (`DegreeOfSuccess::CriticalSuccess`, not `3`). Wrap meaningful numbers in small types where a mix-up is plausible (e.g. `Dc`, `Hp`).
- Derive `Debug, Clone, PartialEq, Eq` where sensible so values are comparable in tests.
- Doc comments (`///`) on every public item, citing the Player Core section for rules.

## 7. Errors
- Library code never panics on bad input: no `unwrap()`, `expect()`, `panic!`, or unchecked indexing outside tests. Return `Result` with a specific error enum for that module.
- Use the type system first; return errors only for things types can't rule out.
- No catch-all error strings. Each error variant says what went wrong.
- `unsafe` is forbidden: `#![forbid(unsafe_code)]` in every crate.

## 8. Tests
- Every pure function gets unit tests in the same file (`#[cfg(test)] mod tests`). An OpenSpec-Refactor pull request may move that block verbatim into a new test file under the inline test block move rule.
- Every spec scenario (Given/When/Then) maps to at least one test, named after it, citing the Player Core section.
- Tests use scripted dice so results are exact. No real randomness, no sleeps, no order dependence.
- Test behaviour through public APIs, not internals.
- No test-only shortcuts in production code: no `#[cfg(test)]` branches inside logic, no hidden "test mode" flags, no weakening a check to make a test pass. If code is hard to test, fix the design.
- Never delete or loosen an existing test to get green. Ported v1 tests must keep passing.
- Include a determinism test for anything that derives or resolves: same inputs twice, identical output.

## 9. Dependencies
- Default answer is no. Use the standard library first.
- Adding a crate needs a spec that names it and says why. Prefer small, widely used, maintained crates; no git or path dependencies outside the workspace.
- `crates/rules` stays dependency-free unless a spec explicitly allows one.
- Commit `Cargo.lock`. Don't bump versions incidentally.

## 10. Comments and docs
- Code should explain itself; comment only *why*, never *what*.
- Record a tabletop substitute in the spec as: rule, why not, substitute, intent.
- An implementation pull request writes its CHANGELOG.md entry under ## [Unreleased] before the work is done. There is no separate changelog pull request after merge.
- All contributors, human or bot, follow `docs/architect/licensing.md`. Every rules spec or doc cites its source book and page.
