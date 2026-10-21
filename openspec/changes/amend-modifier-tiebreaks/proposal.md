# Proposal

## Why

QA on the open implementation of `amend-modifier-duplicates` (PR #30) shows that condition grouping still loses to the source key, and that several comparisons still follow input order. That change's spec is on main at 01e5ca5 and the change is still unarchived, so this delta is written against its post-change requirements.

## What Changes

- Amend `modifiers` from the post-change requirements in unarchived `amend-modifier-duplicates`. The text in `openspec/specs/modifiers/spec.md` is the pre-change base. The implementation of that change, PR #30, is still open. Archive this change only after that one, because these deltas replace its requirements.
- Condition modifiers group by (condition, statistic) only, never by source key. Within a group the highest value applies, which for a penalty is the worse penalty (Redundant Conditions, Player Core pg. 442, Rules 2455). An exact tie breaks on the lower source id.
- The R-01 source-key dedupe (source, statistic, type, bonus or penalty) leaves entries from different conditions in separate groups. After condition grouping, ordinary type stacking applies across groups: the highest bonus and the worst penalty per type (Player Core pg. 400, Rules 2281 and 2282).
- Every comparison ends in a total order. After the existing ladder (level, rank, sequence, value, condition id), the last tiebreak is the lower source id, so the resolution, the total, and which modifier is reported as applied are independent of input order. That source-id step is an engineering tiebreak with no rules effect, not a deviation.
- Conditions stay separate state. Collapsing affects only the numeric total. One source's frightened 3 and sickened 1 total a status penalty of -3 while both conditions remain. Decrementing frightened to 2 totals -2, and sickened 1 remains.
- For any input set, every ordering produces an identical result.
- This pull request is the spec only. It does not change Rust or `CHANGELOG.md`, and it does not archive.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `modifiers`: Condition groups ignore the source key, different conditions stay out of one R-01 key, stacking runs across those groups, and every comparison ends on the lower source id.

## Impact

- The implementation edits the fold in `crates/rules/src/modifiers.rs` on top of `amend-modifier-duplicates`. The duplicate key matches condition id as well as source, statistic, and type. Condition groups keep the worse penalty, then the lower source id. Same-type stacking and the reported list end on that source id. `total` keeps its signature. It does not write `CombatantState`. `ConditionSlot` stays empty. No new crate and no new dependency. `crates/rules` stays free of Bevy, serde, files, network, clock, and global randomness. Callers still pass sequence numbers. Tests use `ScriptedDice`.
- The source-id step leaves a total unchanged when the rules already chose the bonus or penalty. It names which modifier is reported when that choice ties. Same-source frightened 3 and sickened 1 total -3. The higher level does not replace that with sickened's -1.
- The implementation pull request adds an Unreleased changelog bullet, then archives with `openspec archive amend-modifier-tiebreaks --yes` as its last commit. This pull request does not.
