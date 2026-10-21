# Genesis coding constitution

Read this before every change. It binds every agent that writes code in this repo. If a rule here conflicts with a spec, stop and report it; do not pick one yourself.

## 1. How work happens
- Work only from an approved OpenSpec change in `openspec/changes/<id>/`. Implement exactly its tasks and scenarios. No extra features, refactors, or "while I'm here" fixes.
- If the spec is wrong, unclear, or impossible, stop and report it. Do not improvise. (Why: deviations go back to Evan as spec amendments.)
- One change = one procedure family = one PR. The PR body includes `OpenSpec-Change: <id>`. The final commit archives the change (`openspec archive <id> --yes`).
- Git: never force-push, rebase pushed commits, delete branches, or merge to main. Fix mistakes with new commits.
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
- Every pure function gets unit tests in the same file (`#[cfg(test)] mod tests`).
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
