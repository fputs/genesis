# add-modifiers — Rules-Lawyer Edge-Case Checklist

Prepared by Designer (rules lawyer), 2026-10-20. Source: Pathfinder 2e Remaster **Player Core** (PC), verified live on Archives of Nethys (AoN). This is not a spec: each item is a rule plus a suggested Given/When/Then for the Architect/QA.

## 1. Types, bonus vs penalty, stacking (Remaster wording)

| # | Rule | AoN | PC pg |
|---|---|---|---|
| T1 | Bonus types: **circumstance, item, status** ("three other types of bonus that frequently appear"), plus **proficiency bonus**. Different types add. Same type: "you can use only the highest bonus on a given roll—they aren't cumulative." | https://2e.aonprd.com/Rules.aspx?ID=2281 | 400 |
| T2 | Penalty types: circumstance, status, "sometimes even item". Same type: "you take only the worst". | https://2e.aonprd.com/Rules.aspx?ID=2282 | 400 |
| T3 | **A bonus and a penalty of the same type both apply** (heroism +1 status with sickened −2 status → both). | https://2e.aonprd.com/Rules.aspx?ID=2282 | 400 |
| T4 | **Untyped penalties always sum** ("you always add all your untyped penalties together"). MAP and range penalty are the named examples. | https://2e.aonprd.com/Rules.aspx?ID=2282 | 400 |
| T5 | **Untyped bonuses do not exist in Remaster RAW.** The text says "Unlike bonuses, penalties can also be untyped". Recommendation: reject an untyped *bonus* at validation, or model it as a ledgered "untyped bonus (sums)" bucket only if a producer needs one. None do in Phase 1. Baked base values (attribute + proficiency already folded in) are **not modifiers**; they're the base. | https://2e.aonprd.com/Rules.aspx?ID=2282 | 400 |
| T6 | **Item type in Phase 1:** baked AC/attack already include armor/weapon item bonuses (AC formula: "10 + Dex + proficiency + armor's item bonus + other bonuses + penalties"). **Recommend:** define `item` in the type enum for completeness and test its stacking generically. Phase 1 has no producers. | https://2e.aonprd.com/Rules.aspx?ID=2295 | 404 |

## 2. Where modifiers apply in Phase 1

| # | Target | Phase 1 producers | Rule / cite |
|---|---|---|---|
| A1 | **Attack roll** (Strike, Reactive Strike, readied Strike) | MAP (untyped), prone attacker −2 circ, Aid ±circ | Prone https://2e.aonprd.com/Conditions.aspx?ID=88 (PC 445); Aid https://2e.aonprd.com/Actions.aspx?ID=2292 (PC 416) |
| A2 | **Perception check** (initiative, Seek) | unconscious −4 status (moot, since the creature can't act) | https://2e.aonprd.com/Conditions.aspx?ID=95 (PC 446) |
| A3 | **Aid check** (aider's `aid_modifier` roll) | none in Phase 1. Generic check path. | PC 416 |
| A4 | **AC (a DC)** | Raise a Shield +N circ, cover +1/+2/+4 circ, off-guard −2 circ, unconscious −4 **status** | Off-guard https://2e.aonprd.com/Conditions.aspx?ID=58 (PC 445); Cover https://2e.aonprd.com/Rules.aspx?ID=2372 (PC 424) |
| A5 | **Stealth DC / Perception DC** (DC = 10 + total modifier) | none yet. The formula holds. | https://2e.aonprd.com/Rules.aspx?ID=2285 (PC 401) |
| A6 | **Damage roll** | none in Phase 1. Rules verified: same stacking as checks ("add only the highest bonus of that type… apply only the greatest penalty of a specific type but apply all untyped penalties together"). **"If the combined penalties on an attack would reduce the damage to 0 or below, you still deal 1 damage."** | https://2e.aonprd.com/Rules.aspx?ID=2301 (PC 406) |
| A7 | **Flat checks: never modified** ("never includes any modifiers, bonuses, or penalties"). The fold must not be applied to them. | https://2e.aonprd.com/Rules.aspx?ID=2278 (PC 400) |
| A8 | Reflex (unconscious −4) | no saves in Phase 1. Out of scope. | PC 446 |

**Roll vs DC:** a penalty to the *defender's* DC (e.g. off-guard −2 to AC) and a penalty to the *attacker's* roll (prone −2 circ) are on different statistics. Both apply, and they **never** compete for "worst circumstance".
*Given* a prone attacker vs an off-guard target, *When* it Strikes, *Then* the attack total includes −2 circ and the AC used is AC −2. Both apply.

## 3. MAP (untyped) interactions
- M1: MAP + range penalty → **sum** (T4). *Given* a 2nd Strike beyond the first range increment (−5 MAP, −2 range), *Then* the total untyped penalty is −7. Range: https://2e.aonprd.com/Rules.aspx?ID=2290 (PC 403, −2 per increment beyond the first). Range is not produced in Phase 1, so the test uses a synthetic untyped penalty.
- M2: MAP + circumstance penalty (prone −2) → **both** apply; different types. *Then* −5 + −2 = −7.
- M3: MAP + Aid circumstance bonus → both apply; the bonus isn't absorbed into the MAP. (The v1 doc called MAP circumstance, which was wrong. MAP: https://2e.aonprd.com/Rules.aspx?ID=2289, PC 402.)
- M4: Agile MAP −4/−8 is still untyped.

## 4. Edge cases (each needs a scenario)

| # | Case | RAW answer | Cite |
|---|---|---|---|
| E1 | Two circumstance bonuses (+2 shield, +1 lesser cover) | Highest only: +2 | 2281 / PC 400 |
| E2 | Shield +2 vs greater cover +4 | +4 | 2281, 2372 |
| E3 | Two circumstance penalties (−2, −1) | Worst only: −2 | 2282 / PC 400 |
| E4 | +2 circ bonus and −2 circ penalty | Both apply: net 0 | 2282 / PC 400 |
| E5 | Two status penalties from **different conditions** (frightened 2, sickened 1) | Both are status penalties, so take the worst: −2 (not −3) | Frightened https://2e.aonprd.com/Conditions.aspx?ID=76 (PC 444); Sickened https://2e.aonprd.com/Conditions.aspx?ID=91 (PC 446); 2282 |
| E6 | **Same condition twice with different values** (frightened 1 then frightened 2) | "Apply only the highest value" (track durations separately). This is condition-level dedupe and happens *before* modifier folding. | Redundant Conditions With Values https://2e.aonprd.com/Rules.aspx?ID=2455 (PC 442) |
| E7 | **Same condition twice, no value** (off-guard from prone and from flanking) | "You can have a given condition only once": a single −2. (Also same-type stacking gives −2 anyway.) | 2455 (PC 442); 2282 |
| E8 | **Same effect from the same source applied twice** (e.g. Raise a Shield twice) | Duplicate Effects: "only one instance applies, using the higher level or rank of the effects, or the newer effect if the two are equal." Model: dedupe by effect identity, keep the higher, tie → newer. | https://2e.aonprd.com/Rules.aspx?ID=2266 (PC 398, Rules Overview › Game Conventions) |
| E9 | Duplicate **untyped** penalty from the same effect | E8 dedupe first, then sum. Without dedupe, an untyped double-count is the one place a bug changes numbers (typed buckets hide duplicates). | 2266 + 2282 |
| E10 | Zero-value modifier (+0 circ, or frightened 0) | A condition at value 0 **ends** ("If a condition value is ever reduced to 0, the condition ends"). A +0 modifier is a no-op. Recommend validation rejects or drops value-0 entries. It must not "win" a bucket over a penalty. | Condition Values https://2e.aonprd.com/Rules.aspx?ID=2456 (PC 442) |
| E11 | Negative total (base +1, penalties −7) | Allowed. The check total = d20 + (−6). The DC from a modifier can go below 10 (10 + −6 = 4). There's no RAW floor on check modifiers. Degrees are still by ±10 and nat 1/20. | 2285 (PC 401); 2286 |
| E12 | Damage total ≤ 0 after penalties | Deal **1**. Note: the RAW sentence is about *penalties*. Resistance reducing damage to 0 is a separate rule (IWR, out of Phase 1, already queued). | 2301 (PC 406) |
| E13 | Speed penalties | "Penalties can't reduce your Speeds below 5 feet unless stated otherwise." Not a Phase 1 producer; ledger it for later. | https://2e.aonprd.com/Rules.aspx?ID=2345 (PC 420) |
| E14 | Applying to a DC vs a roll | The same fold. DC = 10 + total (A5). Off-guard and unconscious modify AC as a DC. | 2285 |
| E15 | Order independence | The fold is commutative: insertion order must not change the result. Exception: the E8 tie-break uses "newer", which needs a timestamp/sequence. | 2266 |
| E16 | Status bonus + status penalty (heroism-style) | Both apply (RAW's own example). No producer yet, but use a synthetic test. | 2282 |

## 5. QA question: validate HP ≥ 1 and Speed ≥ 0 at the fold boundary?
Rules opinion:
- **Max HP ≥ 1: yes, validate at template load** (not in the modifier fold). A creature at 0 max HP is meaningless. RAW: "Creatures can't be reduced to fewer than 0 Hit Points" (https://2e.aonprd.com/Rules.aspx?ID=2324, PC 410). So **current HP ∈ [0, max]** is an invariant of the damage/heal path, not of the template.
- **Speed ≥ 0: yes, validate at template load; allow 0.** Immobilized does **not** set Speed to 0. It forbids actions with the move trait (https://2e.aonprd.com/Conditions.aspx?ID=81, PC 444). So 0 is a legitimate *baked* value (stationary creatures), and negative is invalid. A post-fold Speed floor of 5 (E13) applies only when penalties exist and is deferred.
- **Scope:** both checks belong to creature-template validation (load-time), not add-modifiers. The modifier fold should not clamp checks/DCs (E11). The damage-total minimum of 1 belongs to the damage path (E12).
