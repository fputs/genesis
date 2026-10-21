# Design

## Context

`amend-modifier-duplicates` is still unarchived on main. Its spec is commit 01e5ca5. Its implementation, PR #30, is still open. See proposal.md for why this change is written against that post-change fold. The behavior to implement is `specs/modifiers/spec.md`.

On that branch, `total` splits bonuses from penalties, groups conditions by `(ConditionId, Statistic)`, collapses the key `(source, statistic, kind)`, then stacks per statistic. `key_prefers` stops at condition id. `condition_prefers` stops at rank and sequence, and an equal sequence keeps the incumbent. `bonus_prefers` and `penalty_prefers` stop at value and the lowest sequence. `same_duplicate_key` ignores condition id, so after frightened and sickened become status penalties they share a key and the higher level wins.

## Goals / Non-Goals

**Goals:**

- Group condition modifiers by `(condition, statistic)` only.
- Keep unequal condition ids, including a present id against none, out of one source key.
- After that grouping, stack the highest bonus and the worst penalty per type per statistic.
- End every comparison on the lower source id so the total and the reported modifier are independent of input order.
- Leave the caller's conditions in place. The numeric total is the only collapse.

**Non-Goals:**

- A new crate, a new dependency, or a new field on `Modifier`.
- Filling `ConditionSlot` or writing `CombatantState`.
- A row in `docs/designer/rules-deviations.md`. The source-id step is an engineering tiebreak, not a deviation.
- Editing ruling R-01's rules effect. Level, rank, sequence, and the worse equal-sequence penalty stay.
- Changing the multiple-attack-penalty table, Aid, shield bonuses, or the damage floor.

## Decisions

### 1. Fold order

`total` keeps the post-change order:

1. Split bonuses and penalties. A bonus and a penalty never share a condition group.
2. Group condition modifiers by `(ConditionId, Statistic)` only. Source id is not part of that group. The comparison below picks one modifier. Frightened and sickened penalties then become one status penalty of `-rank`. Rank 0 still contributes nothing. A bonus tagged with either id keeps its declared type and value. Other conditions keep the caller's type and value.
3. Collapse the key `(source id, statistic, type, bonus or penalty)` only when the condition ids are equal. `None` matches only `None`. Frightened and sickened from one source both reach stacking.
4. Stack types per statistic. The highest bonus and the worst penalty are chosen across the groups from step 2. Two statistics do not share a bucket. Untyped penalties all stay.
5. `finish` sorts the reported list with a total order, then checked-adds the sum.

The value compared in step 2 for frightened and sickened penalties is the rewritten `-rank`, so two rank-2 penalties are an exact tie even when the caller passed `-1`. Condition 10 is not rewritten, so circumstance `-4` beats circumstance `-1` at the same rank and sequence.

Alternative considered: collapse the source key before condition grouping. Rejected. That is the PR #30 defect. Sickened 1 at level 5 replaces frightened 3 at level 1, and the total becomes `-1`.

Alternative considered: let level win inside a condition group. Rejected. Redundant Conditions (Rules 2455) keeps the highest condition value. Frightened 3 beats frightened 1 even at a lower level. A different condition is a different group, so sickened's level does not enter frightened's comparison.

### 2. Three comparisons, one last step

Each comparison returns the candidate when the first difference says so. The last step of each is the lower source id. Rule: Duplicate Effects and same-type stacking name no further instance once the bonus or penalty is chosen. Why not: they are silent when those match. Substitute: the lower source id wins. Intent: every input order reports the same modifier and the same total. This step does not change a total the rules already chose. It is not a deviation.

Condition group, after the split:

1. Higher rank. A rank beats none.
2. Greater sequence. Newer is the greater sequence. This still beats a milder or worse declared value, so `a_same_rank_condition_keeps_the_newer_sequence` stays.
3. Value. A bonus keeps the greater value. A penalty keeps the more negative value. For frightened and sickened this is `-rank`.
4. Lower source id.

Same-type stacking, per statistic:

1. Higher bonus, or more negative penalty. This stays first so the total follows Rules 2281 and 2282.
2. Lowest sequence. The existing report rule.
3. Lower condition id. A missing condition id loses to any condition id.
4. Lower source id.

Duplicate key, only among equal condition ids:

1. Higher level. A level beats none.
2. Higher rank. A rank beats none.
3. Greater sequence.
4. Value, ruling R-01: higher bonus or more negative penalty.
5. Lower condition id. A missing id loses. Inside one key the ids are equal, so this step does not fire for the new scenarios.
6. Lower source id. Inside one key the source is equal too. The step is still there so the comparator matches the other two.

`the_lower_condition_id_wins_an_equal_value` and `a_three_way_tie_is_the_same_in_all_six_orders` pass through stacking step 3, because those modifiers have different condition ids and no longer share a key. The reported condition id is still 3, and still 1.

Alternative considered: put level before value in stacking. Rejected. A higher-level `+1` would beat a `+4`, and the total would change.

Alternative considered: leave an equal sequence to whichever modifier arrived first. Rejected. `Modifier` includes `source` in `PartialEq`, so the reported modifier would follow the slice order.

### 3. The reported list is a total order

`sort_by_key` is stable, so two applied modifiers that tie on the current key keep their input order. Extend the key until every field that can differ is in it: kind, value, sequence, level, rank, condition id, source, statistic. A missing level or condition id sorts after a present one. The preference functions choose which modifier survives. The sort only orders the survivors. Two fully equal modifiers compare equal, so their order does not change `ModifierTotal`.

The permutation test totals every ordering of the eight-modifier set in the resolution scenario. `8!` is 40320. It uses no random number generator. It asserts `ModifierTotal` equality, including `applied`.

### 4. Conditions stay with the caller

`total` keeps `&[Modifier]` and returns `ModifierTotal`. It does not take `CombatantState`, and it does not clear the slice the caller owns. The frightened-and-sickened test owns a `Vec`, asserts both ranks after the first total, replaces frightened 3 with frightened 2, and asserts sickened 1 is still in that vec. `ConditionSlot` stays an empty slot. No new public type.

Alternative considered: store conditions inside `total` and delete the lesser one. Rejected. Collapsing would then change condition state, which the requirement forbids.

## Risks / Trade-offs

- [Same-source frightened 3 and sickened 1 used to follow the higher level] → The total is the worse status penalty, `-3`, then `-2` after frightened drops to 2. Do not delete `frightened_2_and_sickened_1_equal_minus_2`.
- [Different condition ids no longer share a duplicate key] → Stacking's condition-id step must keep condition 3 and condition 1 in the existing tie tests. Assert through `total`, not through a private key helper.
- [Frightened rank 2 rewrites `-1` to `-2` before the source tie] → `two_frightened_penalties_report_the_lower_source_in_both_orders` expects value `-2` and source 1. It does not expect the declared `-1`.
- [A sort key that ignores condition id] → Two untyped penalties that share a source and differ only by condition would make `applied` follow input order, and the permutation test would fail.

## Migration Plan

No stored data. `total` keeps its signature. Callers who passed same-source frightened and sickened and read the higher level's penalty now receive the worse status penalty. Both conditions remain in the caller's list. Rollback is restoring condition-blind key equality and the old preference tails.

## Open Questions

None. The group key, the condition-id match on the source key, the three ladders, and the caller-owned condition list are decided above.
