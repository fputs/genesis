# Proposal

## Why

A fight on a 5-foot grid cannot resolve Stride, Step, reach, or range until diagonal movement, difficult terrain, and occupied squares are decided. Player Core alternates diagonal cost across the whole turn and adds terrain only when a square is entered. This change records that measurement so a later encounter can spend the action without inventing the geometry.

## What Changes

- Add the `grid-movement` capability: pure 5-foot grid measurement for Stride, Step, distance, reach, and range increments, using Designer rulings D1–D17 and the Phase 1 scope cut in `docs/designer/add-grid-movement-checklist.md`.
- Stride and Step validate a caller-supplied ordered path. They return the total cost and an ordered list of per-square leave and enter events. They do not search for a path.
- Keep one diagonal counter per creature per turn, off the immutable Creature. It resets at end of turn. Movement off-turn uses a fresh counter.
- Accept only Small and Medium creatures (one square). Reject a path over Speed, an occupied end square, an enemy square, a Step into difficult terrain, a square outside the map, and arithmetic overflow. A failed move does not partially apply.
- Own DEV-21 from `docs/designer/rules-deviations.md`: validation rejects a Speed that is not a multiple of 5 feet. Speed 0 stays valid. `add-modifiers` keeps only Speed >= 0.
- Return a range increment count and its untyped penalty as plain numbers. Do not add that penalty to an attack roll.
- Ledger every RAW deviation in `design.md`. Answer Designer QA Q1–Q9 with a scenario or an explicit out-of-scope line, and map each question from `design.md`. No change to modifiers, and no new dependency.

## Capabilities

### New Capabilities

- `grid-movement`: 5-foot squares, diagonal counting, terrain cost, 1×1 occupancy, distance, reach, and range-increment data.

### Modified Capabilities

None. `creature-template` and `dice-and-checks` stay as they are. No modifiers capability is added or edited; `add-modifiers` is a separate change.

## Impact

New grid module in `crates/rules`, re-exported from the crate root. Callers pass Speed, reach, squares, terrain, and occupants in. The diagonal counter is turn state, not a field of `Creature` or `CombatantState`. Action spending, attack rolls, cover, flanking, prone, Crawl, Stand, move-trigger resolution, hidden creatures, and forced movement stay out. `crates/rules` gains no dependency. This change names no `v1-reference` file, so none is ported.
