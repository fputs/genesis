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

## R-01 clarification, amendment (PR #34 review, 2026-10-21)
- Rules 2455 (Player Core p.442) compares a condition's value. If a condition's modifier isn't derived from its value, we compare the size of the modifier instead (the worse penalty or the bigger bonus). That is our ruling, not the text of Rules 2455, and it is consistent with Player Core p.400 (Rules 2282). It does not count as a deviation.
- Same-type stacking tiebreaks: when values are equal, the lowest sequence goes first, then the lower condition id (an entry with no id loses), then the lower source id. This is an engineering order with no rules effect (Player Core p.400, Rules 2281/2282). It replaces the earlier "lowest source id first" stacking tiebreak (QA #30 ruling to Architect, 2026-10-21).
- Duplicate-key ladder: compare level or rank first, then the newer entry (Player Core p.398, Rules 2266), then the value (R-01 extension). After that, the lower condition id goes first, then the lower source id. These last two steps are engineering tiebreaks with no rules effect.

## R-01 clarification, amendment 2 (PR #35 review, 2026-10-21)
- The condition group key is (condition, statistic, bonus or penalty, modifier type). Penalties of different types from one condition all apply, and only modifiers of the same type compete (Player Core p.400, Rules 2281/2282). Redundant Conditions (Player Core p.442, Rules 2455) keeps the highest value of one condition; it does not merge different types into one. Grouping runs after frightened and sickened are rewritten to status.
- Example: condition 10 applies item -2 and circumstance -2, and source 9 applies circumstance -3. The total is -5: item -2 plus circumstance -3.
- Any later tiebreak inside a group (level, then canonical key) is an engineering choice with no rules effect. Rules 2266 (Player Core p.398) is context only.

## R-03 Frightened and sickened must carry a value (QA, post-#30, 2026-10-21)
- Frightened (Player Core p.444) and sickened (Player Core p.446) always have a value, and each gives a status penalty equal to that value. Valued conditions are described on Player Core p.442 (Rules 2455).
- Ruling: reject at construction any frightened or sickened modifier with no rank, or with a declared type other than status. The value is the rank, so the modifier is always -rank. Rank 0 is allowed and contributes nothing (covered in #34).
- Reason: a frightened penalty that isn't a status penalty can't exist under the rules. Accepting one and stacking it, or guessing a value for it, would produce totals the rules never allow (QA example: -6 instead of -2). Rejecting bad input matches our existing template-load validation. This is not a deviation.
- This replaces the sentence in amendment 2 saying frightened and sickened are "rewritten to status" before grouping. Frightened and sickened are status by construction, and grouping uses their type as declared.
