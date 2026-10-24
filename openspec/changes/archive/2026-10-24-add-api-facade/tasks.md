# Tasks

The spec commit adds no Rust, no test, and no changelog bullet, and it does not archive. Do not start section 2 until task 1.2 is true. Section 2 lands before section 3. The compile-fail doctests and the two unit tests fail `cargo test` until section 3. That failure is expected. No file on `v1-reference` is named, so none is ported. Do not edit `docs/designer/`.

If a public signature names a type section 3 made `pub(crate)`, stop. Do not add the name to the public list or drop it from the private list in this pull request. That is a spec edit and needs a new `spec-approved: <sha>`.

## 1. Spec

- [x] 1.1 These files are the behavior. The spec commit adds no Rust, no test, and no changelog bullet. Verify `npx @fission-ai/openspec@1.14.3 validate --all --strict` exits 0

- [x] 1.2 Do not start section 2 until this pull request has a review whose body line is `spec-approved: <sha>` and that sha is an ancestor of HEAD. Designer rules on the spec in this pull request

## 2. Tests

- [x] 2.1 Add two doctests on the crate root. One imports every name in the public-list scenario, and not `ScriptedDice`. One builds the 30 HP character from `An external crate starts and ends one turn` and asserts round 1, that order, `start_turn`, and `end_turn`. Verify `cargo test --doc --package rules` passes for those two before any visibility edit

- [x] 2.2 Add one `compile_fail,E0432` doctest per name in `Each removed name is rejected at the root`. The fence is `use rules::<name>;`. Add `compile_fail,E0603` doctests for `use rules::check::degree_of_success;` and `use rules::encounter::Encounter;`. Verify each fence is present. `cargo test --doc --package rules` fails until section 3 because those imports still compile

- [x] 2.3 Add `root_exports_match_the_facade`. It reads the crate root `pub use` names. `ScriptedDice` counts only on the existing `test-support` cfg line. Verify the test fails, and the failure names a private-list export, before section 3

- [x] 2.4 Add `removed_names_are_pub_crate`. It scans `crates/rules/src` for a column-0 `pub fn`, `pub struct`, `pub enum`, `pub trait`, `pub const`, or `pub type` of each private-list name. Indented methods do not count. Verify the test fails before section 3, and that an indented `pub fn stride` on `Encounter` is not a hit

## 3. Visibility

- [x] 3.1 Replace the crate root `pub use` list with the public names. Keep the cfg-gated `ScriptedDice` re-export. Do not add `pub mod`. Verify `root_exports_match_the_facade` passes

- [x] 3.2 Change each column-0 definition of a private-list name from `pub` to `pub(crate)`. Leave `Encounter::stride`, `Encounter::step`, `Encounter::drop_prone`, `Encounter::stand`, and `Encounter::raise_shield` public. Leave public fields and methods of public-list types public. Verify `removed_names_are_pub_crate` passes and `cargo test --doc --package rules` passes, including `use rules::Encounter` and the failure of `use rules::stride`

- [x] 3.3 Point any in-crate path that used a removed root name at its defining module. Add no `#[deprecated]` and no alias. Verify `rg deprecated crates/rules` finds none, and `rg "pub use" crates/rules/src/lib.rs` lists only the facade

- [x] 3.4 Where a public doc link names a now-private item and the build fails on it, replace the link with plain text. Leave the Player Core citation. Verify `cargo test --workspace` still passes and no existing assertion was deleted or weakened

- [x] 3.5 Add one `### Changed` bullet under `## [Unreleased]`, marked **BREAKING**, stating that the rules crate root exports the encounter commands and the types those commands need, that the helper names are crate-private, and that fight results are unchanged. Link this pull request. Verify the bullet is new, under `### Changed`, and links this pull request

## 4. Proof

- [x] 4.1 Run these commands and keep the output. All exit 0

  - `cargo fmt --all --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo build --workspace`
  - `cargo test --workspace`
  - `npx @fission-ai/openspec@1.14.3 validate --all --strict`

## 5. Archive

- [x] 5.1 This is the last commit, right before merge, and only after Architect and QA have both passed that head. Run `openspec archive add-api-facade --yes` with that exact commit subject. Check this box only in that commit. Do not archive the spec draft. Verify the change is under `openspec/changes/archive/` and is no longer active, and that the archive commit does not edit `CHANGELOG.md`
