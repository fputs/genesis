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
- When a modifier is not derived from the condition value, keep the larger modifier. That is a ruling.

**Non-Goals:**

- A new crate, a new dependency, or a new field on `Modifier`.
- Filling `ConditionSlot` or writing `CombatantState`.
- A row in `docs/designer/rules-deviations.md`. The non-derived larger-modifier choice is a ruling, not a deviation. Rank, sequence, and source id inside a condition group, and the condition-id and source-id steps on a duplicate key, are engineering tiebreaks with no rules effect.
- Rewriting ruling R-01. This change copies `docs/designer/rules-rulings.md` from the designer's file as written, including the PR #34 amendment. Level, rank, and sequence inside a duplicate key stay.
- Changing the multiple-attack-penalty table, Aid, shield bonuses, or the damage floor.

## Decisions

### 1. Fold order

`total` keeps the post-change order:

1. Split bonuses and penalties. A bonus and a penalty never share a condition group.
2. Group condition modifiers by `(ConditionId, Statistic)` only. Source id is not part of that group. The comparison below picks one modifier. Frightened and sickened penalties then become one status penalty of `-rank`. Rank 0 still contributes nothing. A bonus tagged with either id keeps its declared type and value. Other conditions keep the caller's type and value.
3. Collapse the key `(source id, statistic, type, bonus or penalty)` only when the condition ids are equal. `None` matches only `None`. Frightened and sickened from one source both reach stacking.
4. Stack types per statistic. The highest bonus and the worst penalty are chosen across the groups from step 2. Two statistics do not share a bucket. Untyped penalties all stay.
5. `finish` sorts the reported list with a total order, then checked-adds the sum.

Step 2 compares the worst penalty or the highest bonus. For a condition with a value, that is the highest condition value. Frightened and sickened penalties use the rewritten `-rank`, so frightened 3 is `-3` and two rank-2 penalties are an exact tie even when the caller passed `-1`. When the modifier is not derived from the condition value, the larger modifier wins. That choice is our extension, consistent with Rules 2282 (Player Core pg. 400), not the text of Rules 2455, and it is a ruling, not a deviation. Condition 10 is not rewritten, so circumstance `-4` at rank 1 beats circumstance `-1` at rank 2. A bonus and a penalty split before this step, so condition 10 circumstance `+2` and circumstance `-1` from the same source and sequence both apply and total `+1`. A frightened or sickened rank of 0 contributes no modifier and is absent from `applied`.

Alternative considered: collapse the source key before condition grouping. Rejected. That is the PR #30 defect. Sickened 1 at level 5 replaces frightened 3 at level 1, and the total becomes `-1`.

Alternative considered: let level win inside a condition group. Rejected. Redundant Conditions (Rules 2455) keeps the highest condition value. A different condition is a different group, so sickened's level does not enter frightened's comparison.

### 2. Three comparisons, one last step

Each comparison returns the candidate when the first difference says so. The last step of each is the lower source id. Rule: Duplicate Effects and same-type stacking name no further instance once the bonus or penalty is chosen. Why not: they are silent when those match. Substitute: the lower source id wins. Intent: every input order reports the same modifier and the same total. This step does not change a total the rules already chose. It is not a deviation.

Condition group, after the split. For a condition with a value, step 1 is the rules comparison (Redundant Conditions, Rules 2455, Player Core pg. 442). Steps 2, 3, and 4 are engineering tiebreaks with no rules effect:

1. Worst penalty or highest bonus. For a condition with a value, that is the highest condition value. A penalty keeps the more negative value. A bonus keeps the greater value. For frightened and sickened this is `-rank`. Frightened 3 at sequence 1 beats frightened 1 at sequence 2, and the total is `-3`. Frightened 1 at sequence 1, frightened 3 at sequence 2, and frightened 2 at sequence 3 total `-3` in all 6 orders. Removing the frightened 2 entry leaves frightened 1, and the total is `-1`.
2. Highest rank. A rank beats none. This fires only when the penalties or bonuses tie.
3. Newest sequence. Newer is the greater sequence. `a_same_rank_condition_keeps_the_newer_sequence` stays because both ranks rewrite to `-2`, so the values tie and sequence 9 wins. It does not compare the declared `-9`.
4. Lowest source id.

When the penalty or bonus is not derived from the condition value, step 1 keeps the larger modifier. Condition 10 circumstance `-4` at rank 1 beats circumstance `-1` at rank 2, and the total is `-4`. That is our extension, consistent with Rules 2282 (Player Core pg. 400), not the text of Rules 2455. It is a ruling, not a deviation. There is no row in `docs/designer/rules-deviations.md`.

Alternative considered: highest rank, then newest sequence, then the condition value. Rejected by the designer review of #34. That order lets a newer frightened 1 beat an older frightened 3.

Alternative considered: treat the non-derived larger modifier as the text of Rules 2455. Rejected. Rules 2455 compares a condition's value. Comparing the modifier's size is our ruling.

Same-type stacking, per statistic:

1. Higher bonus, or more negative penalty. This stays first so the total follows Rules 2281 and 2282.
2. Lowest sequence. The existing report rule. It wins before source id: circumstance `+2` from source 2 at sequence 1 is reported over circumstance `+2` from source 1 at sequence 4.
3. Lower condition id. A missing condition id loses to any condition id.
4. Lower source id.

Duplicate key, only among equal condition ids:

1. Higher level. A level beats none.
2. Higher rank. A rank beats none.
3. Greater sequence.
4. Value, ruling R-01: higher bonus or more negative penalty.
5. Lower condition id. A missing id loses. This step is an engineering tiebreak with no rules effect, not a deviation. Inside one key the ids are equal, so it does not fire for the new scenarios.
6. Lower source id. This step is an engineering tiebreak with no rules effect, not a deviation. Inside one key the source is equal too. The step is still there so the comparator matches the other two.

`the_lower_condition_id_wins_an_equal_value` and `a_three_way_tie_is_the_same_in_all_six_orders` pass through stacking step 3, because those modifiers have different condition ids and no longer share a key. The reported condition id is still 3, and still 1.

Alternative considered: put level before value in stacking. Rejected. A higher-level `+1` would beat a `+4`, and the total would change.

Alternative considered: leave an equal sequence to whichever modifier arrived first. Rejected. `Modifier` includes `source` in `PartialEq`, so the reported modifier would follow the slice order.

### 3. The reported list is a total order

`sort_by_key` is stable, so two applied modifiers that tie on the current key keep their input order. Extend the key until every field that can differ is in it: kind, value, sequence, level, rank, condition id, source, statistic. A missing level or condition id sorts after a present one. The preference functions choose which modifier survives. The sort only orders the survivors. Two fully equal modifiers compare equal, so their order does not change `ModifierTotal`.

The permutation test totals every ordering of the eight-modifier set in the resolution scenario. `8!` is 40320. It uses no random number generator. It asserts `ModifierTotal` equality, including `applied`.

### 4. Conditions stay with the caller

`total` keeps `&[Modifier]` and returns `ModifierTotal`. It does not take `CombatantState`, and it does not clear the slice the caller owns. The frightened-and-sickened test owns a `Vec`, asserts both ranks after the first total, replaces frightened 3 with frightened 2, and asserts sickened 1 is still in that vec. A second test removes frightened entirely and asserts sickened 1 is still present and the total is `-1`. Removing the frightened 2 entry from frightened 2 and frightened 1 on different sources leaves frightened 1, and the total is `-1` (Player Core pg. 442). `ConditionSlot` stays an empty slot. No new public type.

Alternative considered: store conditions inside `total` and delete the lesser one. Rejected. Collapsing would then change condition state, which the requirement forbids.

## Risks / Trade-offs

- [Same-source frightened 3 and sickened 1 used to follow the higher level] → The total is the worse status penalty, `-3`, then `-2` after frightened drops to 2. Do not delete `frightened_2_and_sickened_1_equal_minus_2`.
- [Different condition ids no longer share a duplicate key] → Stacking's condition-id step must keep condition 3 and condition 1 in the existing tie tests. Assert through `total`, not through a private key helper.
- [Frightened rank 2 rewrites `-1` to `-2` before the source tie] → `two_frightened_penalties_report_the_lower_source_in_both_orders` expects value `-2` and source 1. It does not expect the declared `-1`.
- [Condition 9 circumstance `-4` at rank 1 and circumstance `-1` at rank 2] → The larger modifier wins, so the kept penalty is `-4` and the rank is 1. The type stays circumstance. In `a_ranked_penalty_keeps_its_declared_type`, change the value, rank, and total assertions. Do not delete the test. This is the same ruling as condition 10: our extension, consistent with Rules 2282, not the text of Rules 2455, and not a deviation. Do not add a DEV row.
- [Rank 0] → Frightened 0 is absent from `applied`. With sickened 1 the total is `-1`. Alone the total is 0 and `applied` is empty.
- [Removing a condition] → Frightened removed from frightened 3 and sickened 1 leaves sickened 1 and totals `-1`. Removing frightened 2 leaves frightened 1 and totals `-1`.
- [Same condition, same source, bonus and penalty] → Both apply. Condition 10 circumstance `+2` and `-1` at one sequence total `+1` in either order.
- [A sort key that ignores condition id] → Two untyped penalties that share a source and differ only by condition would make `applied` follow input order, and the permutation test would fail.

## Migration Plan

No stored data. `total` keeps its signature. Callers who passed same-source frightened and sickened and read the higher level's penalty now receive the worse status penalty. Both conditions remain in the caller's list. Rollback is restoring condition-blind key equality and the old preference tails.

## Open Questions

None. The group key, the condition-id match on the source key, the three ladders, and the caller-owned condition list are decided above.
