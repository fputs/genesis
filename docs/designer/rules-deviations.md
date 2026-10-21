# Genesis — Rules Deviations Log (intentional departures from RAW)

Owner: Designer (rules lawyer). Baseline: PF2e Remaster Player Core (PC), verified on AoN 2026-10-20. Designer rulings of 2026-10-20 are applied. **FINAL.**
Changes: 2 dice-and-checks, 3 creature-template, 4 grid-movement, 5 encounter-core, 6 add-modifiers, 7 shields, 8 reactive-strike, 9 positioning, 10 seek-and-hidden, 11 delay, 12 ready, 13 aid.

## Deviations

| ID | Rule | RAW citation | Our behavior | Reason | Owning change |
|---|---|---|---|---|---|
| DEV-01 | Derived statistics | https://2e.aonprd.com/Rules.aspx?ID=2295 (PC 404); https://2e.aonprd.com/Rules.aspx?ID=2281 (PC 400) | AC, attack, Perception, and DCs are **baked** numbers. There are no attributes or proficiency. | Prove the encounter loop before derivation. | 3 |
| DEV-02 | Aid check type | https://2e.aonprd.com/Actions.aspx?ID=2292 (PC 416): check "of a type decided by the GM" | Baked `aid_modifier` vs DC 15. | No skills and no GM in Phase 1. | 3, 13 |
| DEV-03 | Aid critical-success bonus | Same page: +2, +3 at master, +4 at legendary | Always +2. | The baked modifier has no proficiency rank. | 13 |
| DEV-04 | Aid DC | Same page: "typical DC is 15, but the GM might adjust" | Always 15. | No GM adjudication. | 13 |
| DEV-05 | PC–PC initiative ties | https://2e.aonprd.com/Rules.aspx?ID=2423 (PC 435): tied PCs decide between themselves | Fixed combatant-ID order. PC–enemy ties follow RAW (enemy first), so they are not a deviation. | Determinism and replay. | 5 |
| DEV-06 | Reactions before your first turn | https://2e.aonprd.com/Rules.aspx?ID=2432 (PC 436): the GM decides | Available from encounter start. | Fixes a GM call. | 5 |
| DEV-07 | Who uses dying rules | https://2e.aonprd.com/Rules.aspx?ID=2324 (PC 410): most creatures die at 0 HP; the GM may grant dying to significant foes | Per-creature `uses_dying_rules`, default false. PCs are true. The default matches RAW; an enemy set to true models the GM option. | Explicit data instead of a GM call. | 3, 5 |
| DEV-08 | Seek secrecy | https://2e.aonprd.com/Actions.aspx?ID=2301 (PC 417): "The GM attempts a single secret Perception check" | Open roll. The area limit ("almost always 30 feet or less in any dimension") **is** enforced. | No hidden-information UI in Phase 1. | 10 |
| DEV-09 | Damage types | https://2e.aonprd.com/Rules.aspx?ID=2301 (PC 406); https://2e.aonprd.com/Rules.aspx?ID=2312 | All damage is physical and untyped. No resistance, weakness, or immunity. | Deferred subsystem. | 5 |
| DEV-10 | Stealth DC source | https://2e.aonprd.com/Rules.aspx?ID=2285 (PC 401); Hide https://2e.aonprd.com/Actions.aspx?ID=2404 (PC 244) | `stealth_dc` is a baked optional field. Absent means the creature can never be hidden or undetected. No Hide action. | No skills in Phase 1. | 3, 10 |
| DEV-11 | Untyped bonuses | https://2e.aonprd.com/Rules.aspx?ID=2282 (PC 400): only penalties can be untyped | Rejected at validation. | Prevents invented stacking. This is a strict reading, logged so QA knows it's intentional. | 6 |
| DEV-12 | Unconscious drops held items | https://2e.aonprd.com/Conditions.aspx?ID=95 (PC 446): "you fall prone and drop items you're wielding or holding" | **Not modeled in Phase 1.** Prone is applied; held items stay held. | No item drop/pickup flow tied to knockout yet. Ruling 2026-10-20. | 5 |
| DEV-13 | Temporary HP from a new source | https://2e.aonprd.com/Rules.aspx?ID=2321 (PC 410): "choose whether to keep the amount you already have … or to gain the new" | Auto-keep the **higher** amount, with its duration. Damage reduces temp HP first; lost temp HP can't be healed. | Deterministic engine; the choice may be exposed later. Ruling 2026-10-20. | 5 |
| DEV-14 | Minimum damage vs critical doubling | https://2e.aonprd.com/Rules.aspx?ID=2301 (PC 406): min 1 after penalties; https://2e.aonprd.com/Rules.aspx?ID=2307 (PC 407): doubling. RAW is silent on the order. | The min-1 floor applies **before** doubling, so the smallest crit deals 2. | Fills a RAW gap with a fixed order. Ruling 2026-10-20. | 5 |
| DEV-15 | Diagonal count for off-turn movement | https://2e.aonprd.com/Rules.aspx?ID=2357 (PC 421): count tracked across your turn; silent off-turn | Movement on another creature's turn (reaction, readied Stride) uses a **fresh counter** for that action, discarded afterwards. The mover's own-turn counter is untouched. | RAW gap; deterministic. (grid checklist D2) | 4 |
| DEV-16 | Step on a 10-ft diagonal | https://2e.aonprd.com/Actions.aspx?ID=2304 (PC 418): Step moves 5 ft; https://2e.aonprd.com/Rules.aspx?ID=2357 (PC 421): count "all your movement" | A Step always moves exactly one square (5 ft) even when parity says 10, **and** it advances the diagonal counter. | Interpretation; ledgered. (D6) | 4 |
| DEV-17 | Willing creatures | https://2e.aonprd.com/Rules.aspx?ID=2360 (PC 422): you may move through a **willing** creature's space | Willing = **same faction**. Other factions block (Tumble Through deferred). | No consent model in Phase 1. (D7) | 4 |
| DEV-18 | Ending in an occupied square | https://2e.aonprd.com/Rules.aspx?ID=2360 (PC 422): you can end a move action in a willing creature's square if you immediately use another move action to leave; you can't end your turn there | **No move action may end in an occupied square** in Phase 1. | Needs action sequencing; lift later. (D8) | 4 |
| DEV-19 | Creature sizes | https://2e.aonprd.com/Rules.aspx?ID=2359 (PC 421); https://2e.aonprd.com/Rules.aspx?ID=2362 (PC 422) | **Small/Medium only** (1×1 footprint, reach 5). The enum carries all sizes, but others are rejected. | MVP scope. (D9) | 3, 4 |
| DEV-20 | Distance between creatures | No single RAW passage. Derived from https://2e.aonprd.com/Rules.aspx?ID=2384 (PC 428: areas measured like movement, ignoring difficult terrain) and https://2e.aonprd.com/Rules.aspx?ID=2379 (PC 426) | Distance = shortest diagonal-count path (5/10 alternation from 5) between the **nearest squares**, ignoring terrain and creatures. The reach-10 two-diagonal exception is special-cased. | A rule had to be derived. (D10, D12) | 4 |
| DEV-21 | Non-multiple-of-5 Speeds | https://2e.aonprd.com/Rules.aspx?ID=2347 (PC 420), Speed. RAW Speeds are always multiples of 5 ft; there's no rule for others. | **Validation rejects** a Speed that isn't a multiple of 5 (and negatives). | Grid squares are 5 ft. This is a validation choice, not a rules change. | 3, 4 |
| DEV-22 | Corner-cutting diagonals (**future**) | No Remaster corner rule found. Grid movement https://2e.aonprd.com/Rules.aspx?ID=2356 (PC 421) is silent | When walls arrive: a diagonal step is blocked **only if both** orthogonal neighbours block. One blocked orthogonal still allows the diagonal. | RAW gap; house rule for later walls/obstacles. | 4 (future) / 9 |

## v1 errors corrected (NOT deviations — v2 follows RAW)

| ID | v1 behavior | RAW | Cite | Change |
|---|---|---|---|---|
| FIX-01 | MAP treated as a circumstance penalty | **MAP is untyped.** It sums with other untyped penalties and applies alongside circumstance penalties. | https://2e.aonprd.com/Rules.aspx?ID=2282 (PC 400); https://2e.aonprd.com/Rules.aspx?ID=2289 (PC 402) | 2, 6 |
| FIX-02 | Wounded +1 on the knockout hit | Wounded +1 when you **lose** dying | https://2e.aonprd.com/Rules.aspx?ID=2328 (PC 411); https://2e.aonprd.com/Conditions.aspx?ID=99 (PC 447) | 5 |
| FIX-03 | No initiative move on knockout | Move initiative to directly before the turn in which you were reduced to 0 HP | https://2e.aonprd.com/Rules.aspx?ID=2324 (PC 410) | 5 |
| FIX-04 | Delay: resuming spends the reaction; the reaction stays usable while delayed | Resume is a **free action**; **no reactions** while delayed | https://2e.aonprd.com/Actions.aspx?ID=2294 (PC 416) | 11 |
| FIX-05 | Take Cover: none → +1, lesser → +2 | Otherwise gain standard cover (+2); standard → greater (+4); prone gives greater cover vs ranged only | https://2e.aonprd.com/Actions.aspx?ID=2307 (PC 418) | 9 |
| FIX-06 | Drop Prone and Stand don't provoke | Both have the move trait, so they trigger Reactive Strike | https://2e.aonprd.com/Actions.aspx?ID=2295 ; https://2e.aonprd.com/Actions.aspx?ID=2303 (PC 416, 418) | 8, 9 |
| FIX-07 | Point Out doesn't provoke | It has the manipulate trait, so it triggers Reactive Strike | https://2e.aonprd.com/Actions.aspx?ID=2315 (PC 419) | 8, 10 |
| FIX-08 | Reactive Strike triggers on move/manipulate only | It also triggers on making a ranged attack within reach | https://2e.aonprd.com/Actions.aspx?ID=2256 (PC 138) | 8 |
| FIX-09 | A readied Strike increments the strike count | MAP is frozen at Ready time; no increment; the turn ends after Ready | https://2e.aonprd.com/Actions.aspx?ID=2299 (PC 417) | 12 |
| FIX-10 | Aid with no preparation | Requires a prepare-to-help action on your turn | https://2e.aonprd.com/Actions.aspx?ID=2292 (PC 416) | 13 |
| FIX-11 | Shield Block on Strikes only | Triggers on physical damage from any attack | https://2e.aonprd.com/Feats.aspx?ID=5212 (PC 262) | 7 |
| FIX-12 | A single +2 shield bonus | Each shield has its own listed bonus (buckler +1) | https://2e.aonprd.com/Actions.aspx?ID=2316 (PC 419); https://2e.aonprd.com/Rules.aspx?ID=2281 (PC 400) | 3, 7 |
| FIX-13 | Flanking off-guard to everything | Off-guard only to the flankers' **melee** attacks | https://2e.aonprd.com/Rules.aspx?ID=2375 (PC 425) | 9 |
| FIX-14 | A hidden creature doesn't make the observer off-guard | A creature you're hidden from is off-guard to you | https://2e.aonprd.com/Conditions.aspx?ID=79 (PC 444) | 9, 10 |
| FIX-15 | Massive damage only from a blow taken while above 0 HP | Damage ≥ 2× max HP "in one blow" kills, with **no HP precondition**. Reverted to RAW per the 2026-10-20 ruling. | https://2e.aonprd.com/Rules.aspx?ID=2332 (PC 412) | 5 |
| FIX-16 | A reaction attack never takes MAP | Exception: **a readied attack uses the MAP from when it was Readied**. The merged dice-and-checks requirement must be amended. | https://2e.aonprd.com/Actions.aspx?ID=2299 (PC 417) | 2 (amend), 12 |
