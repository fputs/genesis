# Proposal

## Why

The order dependence comes from cross-type grouping, not from a missing tiebreak. A condition group keyed only on (condition, statistic) puts a circumstance penalty and a status penalty in one group, so input order chooses which type survives. Different types stack (Player Core p.400, Rules 2281 and 2282). Redundant Conditions (Player Core p.442, Rules 2455) keeps the highest value of one condition and does not merge types.

## What Changes

- Key the condition group on (condition, statistic, bonus or penalty, type), after frightened and sickened are rewritten to status. Different types stack. There is no type tiebreak. Type is part of the key.
- Within a group, after the source id, the higher level wins and a level beats none. That level step is an engineering tiebreak with no rules effect. Rules 2266 (Player Core p.398) is context only for it. The canonical duplicate-key comparison decides last. That comparison is a total order over distinct modifiers. Every ordering keeps the same applied modifier and the same total.
- Restate the repro. Circumstance -2 applies alone in its type, and status -3 beats status -2. The total stays -5. The circumstance-and-item case totals -5: item -2 from condition 10, source 1, plus circumstance -3 from source 9, and circumstance -2 is not applied. A same-type level tie still keeps the higher level. Frightened level 3 is the applied modifier and the total is -2.
- Two identical condition-10 status -2 entries, same source, sequence, rank, and level, give exactly one applied entry and a total of -2 in both orders. The property generator emits one exact-duplicate set and one set with both levels absent, so the comparison falls through to the canonical key. It shuffles every ordering and asserts the totals and the kept modifiers match. No new crate.
- This pull request is the spec only. It does not change Rust or `CHANGELOG.md`, and it does not archive. Designer review of the type order is pending, so `docs/designer/rules-rulings.md` is not edited here.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `modifiers`: The condition group key includes bonus or penalty and type, after the status rewrite. Different types stack. A level tie, then the canonical duplicate-key comparison, finishes the group. A property test guards every ordering.

## Impact

- The implementation edits the condition-group comparison in `crates/rules/src/modifiers.rs` on the open PR #30, on top of `amend-modifier-tiebreaks`. No new field, no new crate, and no new dependency. `crates/rules` stays free of Bevy, serde, files, network, clock, and global randomness. The property test uses a fixed seed in the test module. Callers still pass sequence numbers. Tests use `ScriptedDice`.
- The type order has no RAW basis. Designer review is pending. This pull request does not record it in `docs/designer/rules-rulings.md`.
- The implementation pull request adds an Unreleased changelog bullet, then archives this change last, after `amend-modifier-duplicates` and `amend-modifier-tiebreaks`. This pull request does not.
