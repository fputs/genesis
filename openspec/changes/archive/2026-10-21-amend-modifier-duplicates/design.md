# Design

## Context

`total` in `crates/rules/src/modifiers.rs` groups every modifier that shares a `ConditionId`, then collapses by `SourceId` alone, then stacks types. `source_prefers` keeps the higher `i32` when the sequence is equal, so a penalty tie keeps the milder penalty. See proposal.md for why that is wrong. The behavior to implement is `specs/modifiers/spec.md`.

## Goals / Non-Goals

**Goals:**

- Collapse duplicates only on (source id, statistic, type, bonus or penalty).
- Compare that key with a total order: level, rank, sequence, then the higher bonus or the more negative penalty, then the lower condition id.
- Group frightened and sickened on `(condition, statistic)` after the bonus and penalty split, and before that key collapse. Stack each statistic on its own.
- Keep `total`'s signature, the checked sum, and same-type stacking across different sources.

**Non-Goals:**

- A new crate, a new dependency, or a change to `attack_check_total`'s signature.
- Editing the damage floor. Ruling R-02 is recorded in `docs/designer/rules-rulings.md` and does not change `damage_dealt`.
- Changing cross-source stacking: the highest bonus, the worst penalty, and the lowest sequence when those values tie.
- Changing the multiple-attack-penalty table, Aid, or shield bonuses.

## Decisions

### 1. Fold order

`total` does this:

1. Split the slice into bonuses and penalties. A bonus and a penalty never share a condition group.
2. Within each side, group frightened and sickened by `(ConditionId, Statistic)`, not by `ConditionId` alone. Other conditions also group per statistic. The highest rank wins. The same rank, or no rank, keeps the greater sequence. Value does not decide this step. Frightened and sickened penalties then become one status penalty of `-rank`. A bonus tagged with either id keeps its declared type and value. Rank 0 still contributes nothing, as it does today.
3. Collapse by the key (source id, statistic, type, bonus or penalty). The type is the type after step 2, so a frightened circumstance penalty joins the status-penalty key. The comparison below is the whole order.
4. Stack types per statistic. Two statistics do not share a highest bonus or a worst penalty. `finish` keeps its canonical sort so the reported list does not follow input order. A test that puts two statistics in one list asserts each statistic's total from the applied modifiers. It does not assert one sum of both.

Alternative considered: collapse by source before the frightened rewrite. Rejected. A status penalty and a frightened penalty from one source would then keep the pre-rewrite type and could both survive as status penalties.

Alternative considered: let the penalty value rule run inside condition grouping. Rejected. `a_same_rank_condition_keeps_the_newer_sequence` keeps frightened sequence 9 and rewrites it to -2, even though the older value is -9.

### 2. Total order inside one key

Replace the equal-sequence arm of `source_prefers`. The candidate replaces the incumbent when the first difference says so:

1. Higher level. A level beats no level.
2. Higher rank. A rank beats no rank.
3. Greater sequence. Newer is the greater sequence.
4. Value, which is ruling R-01: an extension consistent with RAW via Rules 2282, not a deviation. A bonus keeps the greater value; a penalty keeps the more negative value. Duplicate Effects (Rules 2266) has no value step. Same-type stacking reaches the same total.
5. Lower condition id. A missing condition id loses to any condition id.
6. Otherwise the modifiers are equal. Keeping the incumbent does not change the result.

Step 4 is the ruling, not a deviation. Today `candidate.value > current.value` keeps -1 over -8.

Tie-break substitute for step 5, recorded in the test comment: rule, Duplicate Effects stops at level or rank and then the newer effect; why not, it does not say which instance remains when those and the value also match; substitute, the lower condition id wins and a missing id loses; intent, every input order reports the same modifier.

Alternative considered: leave equal values to input order. Rejected. `Modifier` includes `condition` in `PartialEq`, so the reported modifier would change with the slice order.

### 3. Statistic is on the modifier

```rust
pub enum Statistic {
    Attack,
    ArmorClass,
    Perception,
    Damage,
}

impl Modifier {
    pub fn bonus(ty: BonusType, value: i32, source: SourceId, sequence: u32, level: Option<u32>, statistic: Statistic) -> Result<Self, ModifierError>;
    pub fn penalty(ty: PenaltyType, value: i32, source: SourceId, sequence: u32, level: Option<u32>, statistic: Statistic) -> Result<Self, ModifierError>;
    pub fn statistic(self) -> Statistic;
    pub fn condition_id(self) -> Option<ConditionId>;
}
```

`bonus` and `penalty` take a `Statistic`. There is no default, and `Attack` is not implied for a modifier that forgot to choose. `attack_modifiers` passes `Attack`. `ac_modifiers` passes `ArmorClass`. `perception_modifiers` passes `Perception`. Every damage modifier passes `Damage`, including the modifiers `damage_dealt` folds and the bonuses in `damage_keeps_the_highest_circumstance_bonus`. `damage_dealt` keeps its signature. The caller tags the slice before the call.

`condition_id` and `statistic` are public so the new scenarios assert through the public API. Tests stay in `modifiers.rs`. No `unwrap` in library code. No new dependency.

Alternative considered: leave an unset statistic as `Attack`. Rejected. A damage modifier would then merge with an attack modifier from the same source.

## Risks / Trade-offs

- [Equal-sequence penalty totals change from the milder value to the worse value] → Update `an_equal_sequence_keeps_the_higher_value` so its penalty half expects -8. Do not delete that test. The bonus half still expects +4.
- [Condition grouping after the split changes who frightened rank 3 can hide] → The frightened-bonus scenario is a synthetic robustness check: frightened never grants a bonus. The single-modifier frightened bonus test stays.
- [Call sites that built a modifier without a statistic] → Pass `Attack`, `ArmorClass`, `Perception`, or `Damage` at the construction site. Do not invent an `Attack` default.

## Migration Plan

No stored data. `total` keeps its signature. Callers that depended on the milder equal-sequence penalty now receive the worse penalty. Rollback is reverting the fold order and `source_prefers`.

## Open Questions

None. The comparison order, the required statistic, and the condition-id substitute are decided above.
