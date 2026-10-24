# Proposal

## Why

`crates/rules` re-exports every procedure module's public items from the crate root. Phase 2 game crates are not here yet. If they arrive against that flat list, they will depend on helpers the encounter already calls. The root should be the encounter commands, the values those commands return, and the types a caller must name to build those arguments.

## What Changes

- Add `api-facade`. The crate root exports a closed set: `begin_encounter`, the existing `Encounter` methods, and the types those signatures and the character-build path require.
- Names on that set stay `pub`. Every other name that is `pub` today and re-exported from the root becomes `pub(crate)`.
- No `#[deprecated]` alias and no compatibility re-export. In-crate callers that named a removed root item are updated in the same pull request.
- **BREAKING** for an external user of the root: free functions and helper types leave the public API. No such user exists in this workspace. Fight results do not change.
- No new command, no new event enum, and no new workspace crate. `ScriptedDice` stays behind `test-support` and this crate's tests.

## Capabilities

### New Capabilities

- `api-facade`: Which names an external crate can use, which names become `pub(crate)`, and the doctests that check both.

### Modified Capabilities

None. Procedure specs still describe the same rolls, costs, and degrees. They do not require a crate-root re-export. This change does not edit their requirements.

## Impact

The crate root of `rules`, and the visibility keyword on items that leave that root. Modules stay private. A parallel change may split the encounter's files; this change names items, not files. No new dependency. The workspace member list stays `crates/rules`, so the acceptance check is a doctest, not a new package. This pull request's first commit is the spec only: no Rust, no test, no changelog bullet, and no archive. Designer rules on this pull request. Code waits for a review line `spec-approved: <sha>`. The archive commit is last, right before merge, and only after Architect and QA have both passed that head. The ruling log is [docs/designer/rules-rulings.md](../../../docs/designer/rules-rulings.md).
