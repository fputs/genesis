# Proposal

## Why

QA on the open implementation of `amend-modifier-duplicates` (PR #30) shows that condition grouping still loses to the source key, and that several comparisons still follow input order. That change's spec is on main at 01e5ca5 and the change is still unarchived, so this delta is written against its post-change requirements.

## What Changes

- Amend `modifiers` from the post-change requirements in unarchived `amend-modifier-duplicates`. The text in `openspec/specs/modifiers/spec.md` is the pre-change base. The implementation of that change, PR #30, is still open. Archive this change only after that one, because these deltas replace its requirements.
- Condition modifiers group by (condition, statistic) only, never by source key. Within a group the order is the worst penalty or the highest bonus. For a condition whose modifier is derived from its value (frightened, sickened: −value), that is the highest condition value (Player Core p.442, Rules 2455). Otherwise the larger modifier wins (our ruling, see the non-derived requirement). Then the highest rank, then the newest sequence, then the lowest source id. Rank, sequence, and source id are engineering tiebreaks with no rules effect.
- The R-01 source-key dedupe (source, statistic, type, bonus or penalty) leaves entries from different conditions in separate groups. After condition grouping, ordinary type stacking applies across groups: the highest bonus and the worst penalty per type (Player Core pg. 400, Rules 2281 and 2282).
- Every comparison ends in a total order. Same-type stacking keeps the highest bonus or worst penalty, then the lowest sequence, then the lower condition id (a missing id loses), then the lower source id. On a duplicate key, after level, rank, sequence, and value, the lower condition id and the lower source id are both engineering tiebreaks with no rules effect, not a deviation. The resolution, the total, and which modifier is reported are independent of input order.
- Conditions stay separate state. Collapsing affects only the numeric total. One source's frightened 3 and sickened 1 total a status penalty of -3 while both conditions remain. Decrementing frightened to 2 totals -2, and sickened 1 remains. Removing frightened entirely leaves sickened 1 and totals -1.
- When a condition's modifier is not derived from its value, the larger modifier wins. Condition 10 circumstance -4 at rank 1 beats circumstance -1 at rank 2. That is our extension, consistent with Rules 2282 (Player Core pg. 400), not the text of Rules 2455. It is a ruling, not a deviation, and it adds no row to the deviations log.
- A frightened or sickened rank of 0 contributes no modifier. Frightened 0 plus sickened 1 totals -1, and frightened is not applied. Frightened 0 alone totals 0 with nothing applied.
- Removing frightened 2 leaves frightened 1 from another source and totals -1 (Player Core pg. 442). Condition 10 circumstance +2 and circumstance -1, same source and sequence, both apply and total +1 (Rules 2282).
- For any input set, every ordering produces an identical result.
- This pull request is the spec only. It does not change Rust or `CHANGELOG.md`, and it does not archive.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `modifiers`: Condition groups ignore the source key, different conditions stay out of one R-01 key, stacking runs across those groups, and every comparison ends on the lower source id.

## Impact

- The implementation edits the fold in `crates/rules/src/modifiers.rs` on top of `amend-modifier-duplicates`. The duplicate key matches condition id as well as source, statistic, and type. Condition groups keep the worst penalty or the highest bonus. For a condition whose modifier is derived from its value (frightened, sickened: −value), that is the highest condition value (Player Core p.442, Rules 2455). Otherwise the larger modifier wins (our ruling, see the non-derived requirement). Then the highest rank, the newest sequence, and the lowest source id. Same-type stacking and the reported list end on the lower source id, after the lowest sequence and the lower condition id. `total` keeps its signature. It does not write `CombatantState`. `ConditionSlot` stays empty. No new crate and no new dependency. `crates/rules` stays free of Bevy, serde, files, network, clock, and global randomness. Callers still pass sequence numbers. Tests use `ScriptedDice`.
- The source-id step leaves a total unchanged when the rules already chose the bonus or penalty. It names which modifier is reported when that choice ties. Same-source frightened 3 and sickened 1 total -3. The higher level does not replace that with sickened's -1.
- The implementation pull request adds an Unreleased changelog bullet, then archives with `openspec archive amend-modifier-tiebreaks --yes` as its last commit. This pull request does not.
