# Backfill Findings — merged specs on main @ 21a45fb (PR #7)

Reviewer: Designer (rules lawyer), 2026-10-20. Specs read via the GitHub connector: `openspec/specs/dice-and-checks/spec.md` and `openspec/specs/creature-template/spec.md`. Rules verified live on AoN (PC = Player Core). Designer rulings of 2026-10-20 are applied. **FINAL.**
Legend: **PROPOSE** = add a scenario. **OUT OF SCOPE** = not this spec (owner named). **QUEUED** = already queued. **RULED** = Designer decision recorded.

## Overall verdict
Both specs are correct against RAW for what they state:
- natural 20/1 handling, including the shift applied first
- degree boundaries (DC met exactly, beat by 9, beat by 10)
- flat-check automatic results at DC 0, 1, 20 and 21
- roll once at the highest DC
- untyped MAP
- per-shield bonus
- HP > 0 and Speed ≥ 0 validation
- `uses_dying_rules`

The gaps are negative totals, dice bounds, HP clamping, uniqueness, and the damage/temp-HP rules.

## dice-and-checks

| ID | Gap | RAW (AoN) | Proposal |
|---|---|---|---|
| B-01 | Negative check totals | Degree comes from the DC bands only. https://2e.aonprd.com/Rules.aspx?ID=2286 (PC 401) | **PROPOSE:** GIVEN a natural 1, modifier −5, and DC 5, THEN the total is −4 and the degree is critical failure (it stays there). |
| B-02 | DC from a modifier below 0 | DC = 10 + modifier. https://2e.aonprd.com/Rules.aspx?ID=2285 (PC 401) | **PROPOSE:** GIVEN a modifier of −12, THEN the DC is −2 (no clamp). |
| B-03 | Same-thing flat checks whose highest DC is ≥ 21 or ≤ 1 | https://2e.aonprd.com/Rules.aspx?ID=2278 (PC 400) | **PROPOSE:** DCs 5 and 21 → failure with no roll consumed. DCs 0 and 1 → success with no roll consumed. |
| B-04 | Natural 20 on a mid-range flat check | https://2e.aonprd.com/Rules.aspx?ID=2287 (PC 401) | **PROPOSE:** DC 11, natural 20 → critical success. Consumers without a critical entry read it as success. |
| B-05 | MAP resets at the start of each own turn | https://2e.aonprd.com/Rules.aspx?ID=2289 (PC 402) | **PROPOSE:** after three attacks on turn N, the first attack on turn N+1 has penalty 0. |
| B-06 | Readied attack keeps the MAP from Ready time | https://2e.aonprd.com/Actions.aspx?ID=2299 (PC 417) | **RULED: amend dice-and-checks.** The MAP requirement must read: "A reaction on another creature's turn SHALL take no penalty and SHALL NOT add to this creature's own-turn count, **except a readied attack, which uses the multiple attack penalty from when it was Readied**." Scenario (owned by the ready change): GIVEN two own-turn attacks and then Ready Strike, WHEN the trigger fires, THEN the penalty is −10 (non-agile). Logged as FIX-16. |
| B-07 | Dice-expression bounds | https://2e.aonprd.com/Rules.aspx?ID=2301 (PC 406) | **PROPOSE:** GIVEN 1d4−3 and a scripted 1, THEN the raw total is −2 and is passed to the damage step (B-08 applies). Validation must reject a dice count of 0 and invalid faces separately. The current FoldError "out-of-range or zero damage die" doesn't say whether it means count or faces. |
| B-08 | Minimum 1 damage after penalties | "you still deal 1 damage." https://2e.aonprd.com/Rules.aspx?ID=2301 (PC 406) | **QUEUED.** Note: after **resistance** the floor is **0**, not 1. https://2e.aonprd.com/Rules.aspx?ID=2318 (PC 408) |
| B-09 | Critical doubling of a floored roll | https://2e.aonprd.com/Rules.aspx?ID=2301 ; https://2e.aonprd.com/Rules.aspx?ID=2307 (PC 406–407) | **RULED (deviation DEV-14):** the floor applies before doubling. **PROPOSE (encounter-core):** a crit with 1d4−3 rolling 1 → raw −2 → floored to 1 → doubled to 2. |
| B-10 | Ordering of multiple degree adjusters | Paizo clarification on https://2e.aonprd.com/Rules.aspx?ID=2287 | **OUT OF SCOPE:** no Phase 1 producers. |

## creature-template

| ID | Gap | RAW (AoN) | Proposal |
|---|---|---|---|
| B-11 | Overkill clamps current HP at 0 | https://2e.aonprd.com/Rules.aspx?ID=2324 (PC 410) | **PROPOSE:** 9 HP takes 15 damage → current HP 0, for both values of `uses_dying_rules`. |
| B-12 | Healing capped at max HP | https://2e.aonprd.com/Rules.aspx?ID=2320 (PC 410) | **OUT OF SCOPE:** no heal producer in Phase 1. Invariant: 0 ≤ current ≤ max. |
| B-13 | Fresh state starts at max HP | PC 410 | Already covered. |
| B-14 | Temporary HP | https://2e.aonprd.com/Rules.aspx?ID=2321 (PC 410) | **QUEUED. RULED (deviation DEV-13):** auto-keep the higher amount, with its duration. Damage reduces temp HP first, then carries over to HP. Lost temp HP can't be healed. |
| B-15 | Validation of strike dice, reach, and shield | https://2e.aonprd.com/Rules.aspx?ID=2181 (PC 274); https://2e.aonprd.com/Rules.aspx?ID=2359 (PC 421) | **PROPOSE:** reject dice count < 1, faces ∉ {4,6,8,10,12}, reach < 0, shield HP < 1, hardness < 0, and BT < 0 or BT > HP. Reach 0 is legal. |
| B-16 | Extreme AC, Perception, or Stealth DC | ID 2285 | **OUT OF SCOPE:** state that negative values are legal so a validator doesn't reject them. |
| B-17 | Duplicate CombatantIds | — (engine invariant) | **PROPOSE:** a roster containing two CombatantId(3) entries is rejected. |
| B-18 | Prepared-to-help state has no home | https://2e.aonprd.com/Actions.aspx?ID=2292 (PC 416) | **OUT OF SCOPE:** the aid change must add it to CombatantState. The "No prepared flag is required" scenario covers derive only; it must not be read as "no preparation needed". |
| B-19 | Aid deviations shown inline | ID 2292 | The crit +2 is documented in the spec. Also add the fixed DC 15 (DEV-04). |
| B-20 | `uses_dying_rules` = false vs nonlethal and massive damage | https://2e.aonprd.com/Rules.aspx?ID=2324 (PC 410); https://2e.aonprd.com/Rules.aspx?ID=2332 (PC 412) | **OUT OF SCOPE:** nonlethal damage. Massive damage follows RAW with no HP precondition (FIX-15). |
| B-21 | Death at 0 HP is immediate | PC 410 | **PROPOSE (encounter-core):** an enemy reduced to 0 is dead before any later reaction resolves. |
| B-22 | Citation quality | — | Advisory: the "derive is pure" and "fold ordered contributions" requirements cite ID 2319. They should be marked "engineering, no rule". |

## Summary
- PROPOSE: 11 (B-01–05, B-07, B-09, B-11, B-15, B-17, B-21)
- OUT OF SCOPE: 5 (B-10, B-12, B-16, B-18, B-20), plus 1 advisory (B-22)
- QUEUED: 2 (B-08, B-14)
- RULED: 3 (B-06 amend, B-09, B-14)
- B-06 is resolved by amending dice-and-checks (FIX-16).
