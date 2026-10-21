# Proposal

## Why

PR #30 still has a second ordering gap. A condition group that ties on value, rank, sequence, and source id follows input order when the two entries differ in type or level. A circumstance -2 and a status -2 on condition 10, source 1, sequence 5, rank 1, plus an unconditioned status -3, total -5 in one order and -3 in the other. That breaks "every ordering SHALL produce an identical result". Continuing the comparison is a structural guard: the order becomes a total order over distinct modifiers.

## What Changes

- Modify the condition-group tie requirement. After a tied source id, the earlier type wins, in the fixed order circumstance, item, status, untyped. That step is an engineering tiebreak with no RAW basis. The higher level wins next, and a level beats none. The canonical duplicate-key comparison decides last. That comparison is a total order, and so is the whole condition-group comparison, over distinct modifiers.
- Modify the wording of "Comparisons end on the lower source id". Stacking and the duplicate-key comparison still end on the lower source id. A condition-group comparison continues after that id. Every ordering keeps the same applied modifier and the same total.
- Add the repro scenario. Circumstance -2 is kept, the unconditioned status -3 stacks, and every order totals -5. Add a same-type, different-level scenario. The higher level is kept. Add the circumstance-and-item case: the group keeps circumstance -2, stacking replaces it with circumstance -3, and every order applies that modifier and totals -3. Add the frightened level case: level 3 is the applied modifier and the total is -2.
- Add a requirement for a seeded property test. It generates modifier sets that include same-source, same-value collisions across type and across level, shuffles every ordering, and asserts the totals and the kept modifiers match. No new crate.
- This pull request is the spec only. It does not change Rust or `CHANGELOG.md`, and it does not archive. Designer review of the type order is pending, so `docs/designer/rules-rulings.md` is not edited here.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `modifiers`: Condition-group ties continue after the source id by type, then level, then the canonical duplicate-key comparison. The "ends on the lower source id" wording no longer says a condition group stops there. A property test guards every ordering.

## Impact

- The implementation edits the condition-group comparison in `crates/rules/src/modifiers.rs` on the open PR #30, on top of `amend-modifier-tiebreaks`. No new field, no new crate, and no new dependency. `crates/rules` stays free of Bevy, serde, files, network, clock, and global randomness. The property test uses a fixed seed in the test module. Callers still pass sequence numbers. Tests use `ScriptedDice`.
- The type order has no RAW basis. Designer review is pending. This pull request does not record it in `docs/designer/rules-rulings.md`.
- The implementation pull request adds an Unreleased changelog bullet, then archives this change last, after `amend-modifier-duplicates` and `amend-modifier-tiebreaks`. This pull request does not.
