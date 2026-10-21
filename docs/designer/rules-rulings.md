# Designer rulings log

## R-01 (2026-10-20): duplicates key on the effect instance, not the source alone
One effect that grants several modifiers (for example a bonus and a penalty) is ONE instance of that effect. Duplicate Effects (Rules 2266, PC 398) collapses only repeated instances of the same effect. A bonus and a penalty of the same type both apply (Rules 2282, PC 400). The dedupe key is (source id, statistic, type, bonus or penalty). Among matching keys, keep the higher level or rank, then the newer one. Extension (our ruling, consistent with RAW): if instances share a sequence, keep the higher bonus or the worse penalty. That gives the same total as same-type stacking (Rules 2282, PC 400); Duplicate Effects (Rules 2266) has no value step. Condition grouping (Rules 2455) is keyed on (condition, statistic).

## R-02 (2026-10-20): the damage floor applies only when penalties cause it (#25)
Rules 2302, PC 406: "If the combined penalties on an attack would reduce the damage to 0 or below, you still deal 1 damage." The floor of 1 applies only when the pre-penalty total is at least 1 and the post-penalty total is at most 0. A negative baked damage modifier (for example the -5 in 1d4-5) is a modifier, not a penalty. Totals below 0 clamp to 0. The order is: dice plus modifiers, then penalties, then the floor or the clamp, then crit doubling (Rules 2307, PC 407). The ordering is our ruling DEV-14, and the clamp at 0 is an engineering call because RAW is silent. Examples: 1d4-5 rolling 1 with an untyped -1 deals 0. A pre-penalty total of 2 with a -2 penalty deals 1, or 2 on a crit. The halving minimum of 1 is deferred until basic saves exist.

## R-01 clarification (QA #30, 2026-10-21)
- Condition modifiers are grouped by (condition, statistic) only, never by source key across different conditions. Source: Redundant Conditions, Player Core p.442 (AoN Rules 2455).
- Within a group the highest value applies (for penalties, the worst one). Value is compared first. Rank, sequence, and source id only break exact ties, and those tiebreaks are engineering choices with no rules effect. Source: Player Core p.442.
- Across groups, normal stacking applies: only the largest bonus and the worst penalty of each type count. Source: Player Core p.400 (AoN Rules 2281/2282).
- Different conditions are not Duplicate Effects, even when they come from the same source. Source: Player Core p.398 (AoN Rules 2266).
- Example: frightened 3 plus sickened 1 gives a -3 status penalty, and both conditions stay active. When frightened drops to 2, the penalty becomes -2 and sickened 1 is still active.
