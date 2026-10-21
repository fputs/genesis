# Proposal

## Why

A frightened or sickened modifier with no rank, or with a type other than status, is rewritten to status or stacked as written. Designer ruling R-03 says that input is invalid. The property-test oracle still calls production helpers, and the attack check total labels its internal terms with caller sources 1 through 3.

## What Changes

- **BREAKING**: A frightened (Player Core p.444) or sickened (Player Core p.446) modifier with no rank, or with a declared type other than status, is rejected at construction with `InvalidValuedCondition`. It is not rewritten to status, and it does not panic. A sickened item penalty of rank 1 and a frightened untyped penalty of rank 1 each fail with that error. A penalty whose rank is above 0 applies as status `-rank` (Player Core p.442, Rules 2455). Rank 0 stays legal and contributes nothing. A status bonus tagged with either condition still keeps its declared value. Any other condition with no rank stays legal.
- The property-test oracle stacks in three stages and does not call production helpers. (a) Group by condition, keyed on (condition, statistic, bonus or penalty, type). (b) Collapse same-source duplicates (rule R-01), keyed on (source, statistic, type, bonus or penalty), among entries with the same condition id. (c) For each statistic, keep the best bonus per type, the worst penalty per type, and every untyped penalty. Stacking ties go to value, then lowest sequence, then lower condition id (an entry with no condition id loses), then lower source id. A condition-10 circumstance penalty of -2 plus an unconditioned circumstance penalty of -3 totals -3, not -5 (Player Core p.400, Rules 2282). A fixed seed draws at least 1,000 sets. Each draw is a bonus or a penalty; a type of circumstance, item, or status, with untyped allowed for a penalty only; and a statistic of attack roll or Armor Class. Frightened is always status, with rank Some(0..2), ranks 0, 1, and 2. The draws include at least one reserved source. Every generated modifier builds without error. Sets of up to 4 are exhaustively permuted, including a conditioned exact duplicate.
- The multiple attack penalty applies once per check (Player Core p.402, Rules 2289) and reuses MultipleAttackPenalty. The attack check does not add a separate internal MAP source. A second attack with no caller MAP gets -5, once. The other untyped internal penalty is the range penalty (Player Core p.403, Rules 2290). A reserved source does not exempt a term from same-type stacking: an internal circumstance penalty of -2 plus a caller circumstance penalty of -3 from source 1 totals -3.
- Type is part of the condition-group key, so a type-order tiebreak (circumstance, item, status, untyped) is not a step. A search of the live specs, design notes, and docs found no such step. This change says type is not a tiebreak.
- This pull request copies `docs/designer/rules-rulings.md` exactly from the designer log. R-03 replaces the amendment 2 sentence that said frightened and sickened are rewritten to status. Designer sign-off is pending. It does not change Rust or `CHANGELOG.md`, and it does not archive.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `modifiers`: Construction rejects an unranked or non-status frightened or sickened modifier. The property-test oracle stacks in three stages and does not call production helpers. The multiple attack penalty is applied once through MultipleAttackPenalty. The other untyped internal penalty is the range penalty, and a reserved source does not skip same-type stacking. Type is not a tiebreak.

## Impact

- Implementation edits construction in `crates/rules/src/modifiers.rs` and the internal terms of the attack check total in `crates/rules/src/check.rs`. No new crate and no new dependency. `crates/rules` stays free of Bevy, serde, files, network, clock, and global randomness.
- Condition construction must be able to fail. Callers still cannot construct a reserved source.
- The implementation pull request adds an Unreleased changelog bullet, then archives with `openspec archive harden-modifier-validation --yes` as its last commit. This pull request does not.
- Designer sign-off on R-03 is pending.
