# Designer rulings log

## R-01 (2026-10-20): duplicates key on the effect instance, not the source alone
One effect that grants several modifiers (for example a bonus and a penalty) is ONE instance of that effect. Duplicate Effects (Rules 2266, PC 398) collapses only repeated instances of the same effect. A bonus and a penalty of the same type both apply (Rules 2282, PC 400). The dedupe key is (source id, statistic, type, bonus or penalty). Among matching keys, keep the higher level or rank, then the newer one. Extension (our ruling, consistent with RAW): if instances share a sequence, keep the higher bonus or the worse penalty. That gives the same total as same-type stacking (Rules 2282, PC 400); Duplicate Effects (Rules 2266) has no value step. Condition grouping (Rules 2455) is keyed on (condition, statistic).

## R-02 (2026-10-20): the damage floor applies only when penalties cause it (#25)
Rules 2302, PC 406: "If the combined penalties on an attack would reduce the damage to 0 or below, you still deal 1 damage." The floor of 1 applies only when the pre-penalty total is at least 1 and the post-penalty total is at most 0. A negative baked damage modifier (for example the -5 in 1d4-5) is a modifier, not a penalty. Totals below 0 clamp to 0. The order is: dice plus modifiers, then penalties, then the floor or the clamp, then crit doubling (Rules 2307, PC 407). The ordering is our ruling DEV-14, and the clamp at 0 is an engineering call because RAW is silent. Examples: 1d4-5 rolling 1 with an untyped -1 deals 0. A pre-penalty total of 2 with a -2 penalty deals 1, or 2 on a crit. The halving minimum of 1 is deferred until basic saves exist.

R-02 closed by #81 (add-saving-throws). The halving minimum is now implemented for basic saves. Halving rounds down, but 1 damage halved stays 1 (PC 407, https://2e.aonprd.com/Rules.aspx?ID=2307). A critical success on a basic save deals 0 because it is "no damage", not a halving (PC 404, https://2e.aonprd.com/Rules.aspx?ID=2297). A 0 result skips the blow entirely: no HP or temp HP change, no dying increase, no knockout, because dying increases only when you take damage (PC 411, https://2e.aonprd.com/Rules.aspx?ID=2327). The damage-after-penalties floor of 1 (PC 406, https://2e.aonprd.com/Rules.aspx?ID=2302) stays on Strike under DEV-14. A critical failure doubles the listed, already-floored amount once (R-07).

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

## R-04 Frightened and sickened never give a bonus (QA #37, 2026-10-21)
- Frightened (Player Core p.444) and sickened (Player Core p.446) only give a status penalty equal to their value. Neither one ever gives a bonus.
- Ruling: reject at construction any bonus tagged frightened or sickened, at any rank including 0 (InvalidValuedCondition). This extends R-03 and is not a deviation.
- This replaces the sentence "a frightened status bonus keeps its declared value" from the #36 review. Scenarios that tested bonus-versus-penalty with a made-up frightened bonus should use a condition whose modifiers are not derived from its value instead (for example condition 10). A bonus and a penalty of the same type from that condition both apply (Player Core p.400, Rules 2282).

## R-05 A readied attack keeps the multiple attack penalty it had at Ready time (FIX-16, 2026-10-22)
- Ready (Player Core p.417, AoN Actions 2299): if the readied action is an attack, it takes the multiple attack penalty the creature had when it used Ready. Multiple Attack Penalty (Player Core p.402, Rules 2289) applies only on your own turn.
- Ruling for the dice-and-checks amendment: an attack-trait check made as a readied action SHALL use the penalty frozen when Ready resolved (0, -5 or -10; with an agile current weapon, 0, -4 or -8). The weapon used for the readied attack sets agile, and the snapshot stores the attack count, not the number. Any other reaction attack keeps a penalty of 0. A readied attack does not add to any own-turn count.
- Scenarios: (1) a Strike, then Ready, then the readied Strike with a non-agile weapon fires: -5. (2) Ready as the first action, then the readied Strike: 0. (3) Strike twice, then Ready (the turn ends), then the readied agile Strike: -8. (4) after a -5 readied Strike, the reactor's next own turn begins with a penalty of 0. (5) a Reactive Strike on the same round has a penalty of 0.
- This is RAW, not a deviation. It replaces "a reaction always has a penalty of 0" for readied attacks only.

## R-06 Temporary HP (DEV-13, confirmed 2026-10-22)
- Player Core p.410 (Rules 2321): temp HP from a new source doesn't stack, and the creature chooses which amount to keep. We always keep the higher amount and its duration (DEV-13, intentional).
- Edge cases: if the amounts are equal, keep the existing amount and its duration. Damage removes temp HP first and the rest goes to HP. Healing never restores temp HP. Temp HP doesn't count toward max HP or toward the massive damage threshold (2x max HP, Player Core p.412, Rules 2332, which compares the blow's damage, not HP lost). When temp HP expires, unused points vanish. Temp HP of 0 means no temp HP.
- Example: a creature with 5 temp HP gains 3 and keeps 5. Then 7 damage takes 5 temp HP and 2 HP.

## R-07 Minimum damage happens before critical doubling (DEV-14, confirmed 2026-10-22)
- The order stays as in R-02: dice plus modifiers, then penalties, then the floor of 1 (only if penalties caused a total of 0 or less, Player Core p.406, Rules 2302) or a clamp at 0, then doubling on a crit (Player Core p.407, Rules 2307).
- So the smallest crit when the floor applies deals 2. A total clamped at 0 (because a negative modifier, not a penalty, brought it there) deals 0 even on a crit. This is intentional (DEV-14). RAW gives no order.

## R-08 Damage at 0 HP while unconscious and not dying is a knockout (PR #46 review, 2026-10-22)
- RAW doesn't cover this case: Getting Knocked Out (Player Core p.410, Rules 2324) triggers on being reduced to 0 HP, and Unconscious (Player Core p.446, Conditions 95) covers damage only above 0 HP.
- Ruling: damage above 0 to a creature that is unconscious at 0 HP and not dying applies Getting Knocked Out. It gains dying 1 (dying 2 on a critical), plus its wounded value, and moves its initiative (FIX-03). This is a ruling, not a deviation.
- Example: unconscious at 0 HP with wounded 1. A non-critical 1 damage gives dying 2.

## R-09 The unconscious -4 under R-01 to R-04 (PR #46 review, 2026-10-22)
- Unconscious (Player Core p.446, Conditions 95) gives a -4 status penalty to AC, Perception and Reflex. It is a condition modifier with a reserved condition id. Its group key is (unconscious, statistic, penalty, status).
- It is not a valued condition, so penalties are compared by size (R-01 amendment). Reject it at construction if it has a rank, a type other than status, an amount other than -4, or is a bonus.
- It competes with frightened, sickened and other status penalties, and only the worst one applies. It adds to circumstance penalties such as off-guard -2 and to status bonuses (Player Core p.400, Rules 2282; Player Core p.442, Rules 2455).
- Examples: frightened 2 and unconscious on AC give -4 status and -2 off-guard, -6 total. Frightened 5 and unconscious on AC give -7. Frightened 3, sickened 1 and unconscious on Reflex give -4. Unconscious with a +1 status bonus on Reflex gives -3. In a tie between frightened 4 and unconscious, the total is -4; the tiebreak has no rules effect.

## R-10 Temp HP and dying (PR #46 review, 2026-10-22)
- A blow fully absorbed by temporary HP still counts as taking damage, so dying still rises (Player Core p.411, Rules 2327; Player Core p.410, Rules 2321). This is a ruling, not a deviation. Example: dying 1 with 5 temp HP takes 3 damage, giving dying 2 and 2 temp HP.
- A knockout blow hits temp HP first. Example: 3 HP with 5 temp HP takes 7 damage, leaving 0 temp HP, 1 HP and no dying.
- Every death clears temp HP, from massive damage and from reaching dying 4 alike. RAW doesn't say (Player Core p.411, Rules 2329); we do it for consistency.

## R-11 Take Cover after Stand (PR #45 review, 2026-10-22)
- Stand does not end Take Cover. The end triggers are moving from your space, an attack action, becoming unconscious, or a free action (Player Core p.418, Actions 2307). But the greater cover against ranged attacks comes from being prone ("If you're prone, you gain greater cover against ranged attacks"; Prone, Player Core p.445, Conditions 88), so it lasts only while the creature is prone.
- Ruling: on Stand, drop only the part that came from prone. StandardPlusGreaterVsRanged becomes Standard (+2 against melee and ranged, with no prone off-guard). GreaterVsRanged alone (prone, with no feature and no cover) leaves nothing, so the Take Cover ends. Greater from standard cover stays Greater (+4). This extends DEV-24.
- Unconscious and prone: unconscious ends Take Cover. Off-guard from prone and off-guard from unconscious are the same condition and apply once, as -2 circumstance (Player Core p.445, Conditions 58; Player Core p.400, Rules 2282). With the unconscious -4 status penalty, AC totals -6, never -8.

## R-12 A failed targeting flat check still uses the action and counts for MAP (PR #60 QA question, 2026-10-22)
- Hidden says that on a failed flat check "you don't affect the creature, though the actions you used are still expended" (Player Core p.434, Rules 2416). Concealed (Player Core p.442, Conditions 62) says only "you aren't affected" and is silent on cost. But in both cases the flat check comes during targeting, after the attack action has already been used.
- MAP counts attack actions used during your turn (Player Core p.402, Rules 2289), whether or not they resolve.
- Ruling: a failed targeting flat check at any DC (5 concealed, 11 hidden, or one roll at the highest DC when both apply, Player Core p.400, Rules 2278) spends the action and counts toward MAP, and the target is not affected. This is a ruling, not a deviation.
- Example: a Strike against a concealed target rolls a natural 4 on the DC 5 flat check. The target isn't affected, the action is spent, and the next Strike this turn takes -5 (-4 agile).

## R-14 Delay runs some end-of-turn effects immediately (Designer, PR #94, 2026-10-27)

- Delay (Player Core p.416, https://2e.aonprd.com/Actions.aspx?ID=2294): negative effects that would happen at the end of your turn happen immediately when you Delay. A beneficial effect that lasts through the end of your turn also ends. You do not run those effects a second time when you later act.
- Ruling: On Delay, (a) negative end-of-turn effects run immediately, which is the persistent-damage hook and its flat check; (b) a beneficial effect lasting through the end of the turn ends; (c) frightened does not decrement, and a negative condition that lasts until the end of your turn, or until the end of your next turn, does not expire. Those run at the end of the turn the creature returns to, once per actual turn.
- Reason: a creature cannot Delay to dodge a negative or to extend a benefit.
- Scenarios: off-guard until the end of the creature's own next turn stays through Delay and is gone when the returned turn ends. Frightened 2 stays 2 on Delay and drops to 1 at the end of the returned turn. Persistent fire 2 ticks at Delay. A creature that never returns follows the existing Delay spec.
- This is a ruling, not a deviation. The core set has no beneficial condition. (b) is stated so a later beneficial duration follows it.
