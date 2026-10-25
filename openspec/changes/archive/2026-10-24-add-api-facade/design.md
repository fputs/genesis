# Design

## Context

See proposal.md for why the crate root is too wide. Today `crates/rules/src/lib.rs` `pub use`s 198 names from private modules. `Encounter` already wraps Strike, Aid, shields, saves, Delay, Ready, Reactive Strike, Stride, Step, Drop Prone, and Stand. Callers inside this crate import `crate::<module>::<item>`, not the root re-export. There is no other workspace crate. `openspec/specs/project-foundation` requires the only member to be `crates/rules`.

A parallel change may split the encounter source into submodules. This design names items and crate-root paths. It does not name an encounter file layout.

## Goals / Non-Goals

**Goals:**

- The root `pub use` set is the public list in `specs/api-facade/spec.md`.
- Each removed name is `pub(crate)` at its definition, so a later `pub use` of it does not compile.
- Doctests are the external crate: one that uses the facade, one `compile_fail,E0432` per removed name, and two `compile_fail,E0603` module paths.
- A unit test reads the crate root's `pub use` names and compares them to that list.

**Non-Goals:**

- A new command, including Seek, Hide, Take Cover, healing, or victory. Those free functions stay implemented and become `pub(crate)`. `Detections::new` and `Detections::store` stay public, so a caller can still pass a detection map into Strike.
- An event enum. `StrikeDelivery`, `AidOutcome`, `ReactiveOutcome`, `SaveRoll`, `MoveOutcome`, `PostureChange`, and `ResolvedReaction` stay the results.
- A public `encounter` module. `Encounter` is exported at the root.
- A new package, `trybuild`, or any other dependency.
- Editing a function body, a match arm, or an existing assertion.
- A `#[deprecated]` alias.

## Decisions

### Root re-exports, not a public module

The facade is the `pub use` list in the crate root. Modules stay `mod`, not `pub mod`. A file split can move `Encounter` without becoming the path callers use. `use rules::encounter::Encounter` is E0603. `use rules::Encounter` stays.

Alternative: `pub mod encounter` and drop the flat list. That makes the file split part of the API, which this change refuses.

### What stays `pub`

A name stays `pub` when an external caller must write it to build a participant or to call a public `Encounter` method and read the result. That includes `CharacterBuild`, `derive`, `new_combatant_state`, `Dice`, `SeededDice`, `Modifier` and the types `Modifier::bonus` takes, `Detections` and the types `Detections::store` takes, and every type in a public signature or in a public field of such a type (`HpError` in `EncounterError`, `SavingThrows` on `Creature`, `InitiativeMove` on `CombatantState`).

Public fields and methods on those types stay public. `CombatantState.hit_points` stays readable. Narrowing it would make `Encounter::state` useless.

`ScriptedDice` stays `#[cfg(any(test, feature = "test-support"))] pub use`. Facade doctests use `SeededDice` so they compile with the feature off.

### What becomes `pub(crate)`

Every other name in today's root `pub use` becomes `pub(crate)` at the definition. Free functions the encounter calls (`degree_of_success`, `total`, `seek`, `fold_contributions`, `apply_healing`, and the rest of the private list) are in that set.

Five free functions share a name with an `Encounter` method: `stride`, `step`, `drop_prone`, `stand`, and `raise_shield`. The free function becomes `pub(crate)`. The method stays `pub`. A column-0 `pub fn stride` is the free function. The method is indented inside `impl Encounter` and is not demoted.

`positioning`'s intra-crate `pub use` of `Side` stays. The module is private, so it is not a second root export.

Alternative: leave helpers `pub` inside private modules and only delete the root `pub use`. External crates still could not name them, but a one-line `pub use` would publish them again. `pub(crate)` makes that line a compile error.

### Proof without a new crate

Rustdoc compiles doctests as external crates. `compile_fail,E0432` on `use rules::<name>;` passes only when that import fails with unresolved import. One fence per private-list name, so one successful import cannot hide the others. `compile_fail,E0603` covers `rules::check::degree_of_success` and `rules::encounter::Encounter`.

A unit test `include_str!`s the crate root and parses `pub use` identifiers. `ScriptedDice` is allowed only on the cfg-gated line. This catches a new root name that the compile-fail list does not mention.

A second unit test scans `crates/rules/src` for a column-0 `pub` definition of each private-list name (`pub fn`, `pub struct`, `pub enum`, `pub trait`, `pub const`, `pub type`). Indented methods are ignored, which keeps `Encounter::stride` legal.

Alternative: a compile-only workspace crate, or `trybuild`. Both are out. The first violates the one-member workspace. The second adds a dependency `crates/rules` does not have.

### Callers

No `#[deprecated]`. Delete the root re-export in the same change that narrows the definition. In-crate paths already go through the defining module. If a path used the root name, point it at the module in that same change. Do not add a shim.

### Docs

Player Core comments stay. A public doc link that would point at a now-private item becomes plain text when it would fail the build. That edit does not change the cited rule.

## Risks / Trade-offs

- [A public signature still names a private-list type, so the crate does not compile] → Stop. Do not add the name to the public list and do not drop it from the private list in this change. The lists were checked against current signatures; a parallel edit may invalidate that.
- [The encounter file split lands first] → Apply `pub(crate)` to the item wherever it moved. Do not recreate a deleted file and do not specify the new paths here.
- [Demoting `Encounter::stride` because a free function shares the name] → The column-0 rule above. The scenario `The encounter Stride method stays callable` fails if the method is gone.
- [`compile_fail` passes on any E0432, including a typo in `rules`] → The fence is exactly `use rules::<name>;` with the name from the private list. The public-list doctest imports `rules` successfully, so the crate name is real.
- [Seek, Hide, healing, and victory are not callable from outside] → Accepted. This change subtracts. A later change can add a command. `Vital` on `CombatantState` stays readable.

## Migration Plan

This pull request's first commit is only `openspec/changes/add-api-facade/`. After `spec-approved: <sha>`, the next commit adds the doctests and the two unit tests. Those compile-fail doctests fail `cargo test` until the following commit narrows visibility and replaces the root `pub use` list. That visibility commit adds one `## [Unreleased]` `### Changed` bullet, marked **BREAKING**, linking this pull request. No `docs/designer/` edit. No file on `v1-reference` is named, so none is ported. `openspec archive add-api-facade --yes` is the last commit, only after Architect and QA have both passed that head.

Rollback is reverting the visibility commit. No saved data uses the old root paths.
