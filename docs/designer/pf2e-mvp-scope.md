# Genesis v2 — PF2e Rules Engine MVP Scope (Phase 1)

Status: **FINAL draft** (Game Designer), 2026-10-20. Aligned to the post-v1-audit Phase 1 order. This is not an OpenSpec artifact. It feeds proposals in `openspec/changes/`.
Rules baseline: Pathfinder 2e **Remaster**, *Player Core* (PC). Every rule was verified on Archives of Nethys. URLs and quotes-in-summary are in `pf2e-citations.md`. Page numbers below are PC pages as AoN shows them.

## Goal
Port v1's encounter behaviour into a spec-first, deterministic, seeded, headless core: four PCs against fixture enemies on a 5-ft grid, using **baked** stat blocks. Derivation (attributes, proficiency, the full modifier engine) is deferred.

## MVP subsystems (build order = numbering)

| # | Change | Scope | Source (PC page) |
|---|---|---|---|
| 1 | project-context | Stack, test lanes, seeded/scripted dice, Remaster naming. | — |
| 2 | dice-and-checks | Dice expressions; d20 + modifier vs DC; four degrees (±10); nat 20/1 shifts one degree, applied first; DC = 10 + modifier. **Flat checks**: no modifiers, roll once at the highest DC, DC ≤ 1 auto-succeeds, DC ≥ 21 auto-fails. | Checks 400; Flat Checks 400; Step 4: Degree of Success 401; Natural 1 and Natural 20 401; Getting a DC from a Modifier 401 |
| 3 | creature-template (baked) | HP, AC, Perception, Speed, one Strike (attack bonus, damage dice, traits incl. agile), optional shield (**its own** AC bonus, Hardness, HP, BT), optional **`stealth_dc`**, optional **`aid_modifier`**, optional bool **`uses_dying_rules`** (default false: the creature dies at 0 HP; PC fixtures set true). | Hit Points 410; Getting Knocked Out 410; Shield Statistics 274; Seek 417; Aid 416 |
| 4 | grid-movement | 5-ft squares. Diagonals cost 5/10 alternating, counted across the whole turn. Difficult +5 ft, greater difficult +10 ft (not doubled on diagonals). Medium space/reach 5 ft. Speed floor 5 ft. | Grid Movement 421; Diagonal Movement 421; Size, Space, and Reach 421; Difficult Terrain 423; Movement 420 |
| 5 | encounter-core | Perception initiative (**tie: enemy first**). Start of turn: recovery check, then regain 3 actions + 1 reaction. End-of-turn hook. Stride, Step (Speed ≥ 10, not into difficult terrain, no move reactions), Strike vs AC. MAP −5/−10, agile −4/−8, **untyped**. Damage minimum 1; crit doubles the total. Creatures with `uses_dying_rules` false die at 0 HP. Knocked out (uses_dying_rules): **move initiative to just before the turn that dropped you**; dying 1/2 + wounded. Recovery DC 10 + dying (±1/±2). Damage while dying +1/+2. Dying 4 = dead. **Wounded +1 when dying is lost.** Unconscious applies the −4 **status** penalty and off-guard; prone and blinded from unconscious are deferred (DEV-31). Massive damage ≥ 2× max HP. Victory/defeat. | Step 1: Roll Initiative 435; Start/End Your Turn 435–436; Reactions in Encounters 436; Stride/Step/Strike 418; Multiple Attack Penalty 402; Penalties 400; Damage Rolls 406; Doubling and Halving 407; Getting Knocked Out 410; Dying, Recovery Checks, Taking Damage, Losing the Dying Condition 411; Massive Damage 412; Unconscious 446; Wounded 447 |
| 6 | circumstance-modifiers | Typed buckets: same type keeps the highest bonus and the worst penalty; a bonus and a penalty both apply; untyped penalties sum. Covers **circumstance**, plus **status** (needed for unconscious −4), plus untyped. Item is deferred. | Bonuses 400; Penalties 400 |
| 7 | shields | Raise a Shield (its listed bonus until your next turn starts). Shield Block: reaction when the raised shield and you would take **physical damage from an attack**; Hardness absorbs, the rest goes to both; broken/destroyed. | Raise a Shield 419; Shield Statistics 274; Shield Block (General feat 1) 262 |
| 8 | reactive-strike | Trigger: a creature in reach uses a **move** or **manipulate** action, **makes a ranged attack**, or leaves a square mid-move. Melee Strike, no MAP; a crit disrupts manipulate (actions are still spent). | Reactive Strike 138; Disrupting Actions 415 |
| 9 | positioning | Drop Prone / Stand (both **[move]**, so they can provoke), Crawl (prone, Speed ≥ 10), Leap (10/15 ft), Take Cover (standard → greater +4, otherwise standard +2; prone gives greater vs **ranged** only; ends on move/attack/unconscious). Cover: lesser +1, standard +2, greater +4. Flanking: off-guard to the flankers' **melee** attacks. Off-guard −2 circumstance AC. Prone: off-guard, −2 circumstance to attacks. **A creature you are hidden from is off-guard to you.** | Basic Actions 416–419; Cover 424; Flanking 425; Off-Guard 445; Prone 445; Hidden 444 |
| 10 | seek-and-hidden | Per-observer Observed/Concealed/Hidden/Undetected. Seek is an **open** Perception roll vs `stealth_dc` over one chosen area **no more than 30 ft in any dimension** (crit → observed; success: undetected → hidden, hidden → observed). Flat checks to target: concealed DC 5, hidden DC 11, undetected guess + DC 11. Point Out (**manipulate**, so it provokes). **The observer is off-guard to a hidden creature.** | Seek 417; Concealed 442; Hidden 444; Undetected 447; Point Out 419 |
| 11 | delay | Free action at the start of your turn; leave the order; **return as a free action** after any other creature's turn (new position); **no reactions while delayed**; start/end negatives happen immediately; a full skipped round keeps the old slot. | Delay 416 |
| 12 | ready | 2 actions [concentrate]: pick an action + trigger; **your turn ends**; fires as a reaction before your next turn; a readied attack uses the MAP **frozen at Ready time** and does not increment the strike count (v1's increment dropped). | Ready 417 |
| 13 | aid | Requires a **prepare-to-help action** on your turn (prepared state). Reaction before an ally's check, `aid_modifier` vs DC 15: crit +2, success +1, crit fail −1 circumstance. | Aid 416 |

## Deferred
| Subsystem | Why | Source when revisited |
|---|---|---|
| Item-bonus bucket, full modifier engine | Baked stats; change 6 covers circumstance/status/untyped. | PC 400 |
| Attributes, proficiency, derived stats, saves, skills | Fixed stat blocks prove the loop first. | Character Creation; Bonuses 400 (proficiency table) |
| Broader conditions (frightened, sickened, clumsy, stunned, slowed, persistent damage…) | Need a condition model with durations. | Conditions Appendix; End Your Turn 436 |
| Damage types, resistance/weakness/immunity, nonlethal | All MVP damage is physical. | Immunity, Weakness, and Resistance (AoN Rules ID 2312; page not verified) |
| Spells, ancestries/classes/feats, items/crafting, exploration/downtime, afflictions, Hero Points, content import | Out of encounter-loop scope / licensing risk. | Respective chapters |

## Deliberate deviations from RAW (ledger each)
1. **Baked stats**: AC, attack, Perception, `stealth_dc` and `aid_modifier` are entered, not derived.
2. **Aid crit bonus fixed at +2.** RAW gives +3 at master and +4 at legendary; a baked modifier has no rank. Aid uses `aid_modifier` instead of a GM-chosen check.
3. **PC–PC initiative ties** are broken by fixed ID. RAW lets the players choose. PC–enemy ties follow RAW (enemy first).
4. **Reactions before your first turn** are available. RAW makes this a GM call.
5. **Dying rules are per creature via `uses_dying_rules`.** PCs use them; enemies default to death at 0 HP, which is the RAW norm. Setting true on a significant foe models the GM option. This is not strictly a deviation, but it is ledgered because v1 differs.
6. ~~Massive damage HP precondition~~: **reverted to RAW** 2026-10-20 (any blow ≥ 2× max HP kills; see rules-deviations.md FIX-15).
7. **Seek is an open roll**, not a secret check: no hidden-information UI in Phase 1. The RAW area limit (≤ 30 ft in any dimension) **is** enforced.
8. **Physical-only damage**; no damage types yet.
9. **Unconscious does not drop held items** in Phase 1 (not modeled; ruling 2026-10-20).
10. **Temp HP**: auto-keep the higher amount; RAW is player choice (ruling 2026-10-20).
11. **Min-1 damage applies before crit doubling** (smallest crit = 2); RAW is silent (ruling 2026-10-20).
12. **A readied attack uses the MAP from Ready time** (RAW); dice-and-checks needs an amendment (FIX-16).

The canonical ledger is `rules-deviations.md`.

## Risks
- **Licensing:** Remaster rules are under the **ORC License**, which allows reuse with attribution. Paizo trademarks, art and setting are Reserved Material. Avoid OGL-only legacy terms. Add an ORC notice. Don't copy AoN prose into fixtures or specs; paraphrase and cite. Owner sign-off is needed before release.
- **v1 has RAW errors.** See the corrections list in `pf2e-citations.md`: wounded timing, Delay reactions, Take Cover ladder, Drop Prone/Stand provoking, initiative ties, missing knockout initiative move. Port by re-spec, not by copying tests.
- **Status penalties arrive with unconscious (change 5)** before change 6. Either change 5 hardcodes a ledgered −4 or change 6 lands first (see Q1).

## Recommended build order (dependencies)
1 project-context: —
2 dice-and-checks: 1
3 creature-template: 1
4 grid-movement: 1, 3
5 encounter-core: 2, 3, 4, 6 (unconscious status penalty)
6 circumstance-modifiers: 2. **Proposed, pending Architect:** build it right after 2, i.e. before 3–5.
7 shields: 5, 6
8 reactive-strike: 4, 5
9 positioning: 4, 5, 6, 8 (prone/stand provoke)
10 seek-and-hidden: 2, 3, 4, 5, 6 (off-guard)
11 delay: 5
12 ready: 5, 8
13 aid: 5, 6, 8

## Resolved decisions log
| Date | Decision | By |
|---|---|---|
| 2026-10-20 | circumstance-modifiers is its own change at step 6. | Architect |
| 2026-10-20 | `stealth_dc` **and** `aid_modifier` are baked into creature-template now (optional fields). | CoS (overruled deferral) |
| 2026-10-20 | MAP is **untyped** (PC 400 Penalties; PC 402). | Architect |
| 2026-10-20 | Flat checks live in dice-and-checks (recovery checks, concealed/hidden). | Architect |
| 2026-10-20 | Aid keeps RAW's prepare-to-help action; the prepared state lands in change 13. | Design |
| 2026-10-20 | Aid crit bonus fixed at +2 (ledgered deviation). | Design |
| 2026-10-20 | Shield AC bonus is baked per shield (a buckler gives +1, PC 400 example). | Design |
| 2026-10-20 | Hidden ⇒ the observer is off-guard to that creature: in changes 9 and 10. | Design (RAW, PC 444) |
| 2026-10-20 | Q1: circumstance-modifiers moves to right after dice-and-checks and also covers status penalties minimally (unconscious −4). **Proposed, pending Architect.** | Designer |
| 2026-10-20 | Q2: PCs use dying/wounded; enemies die at 0 HP by default. creature-template gains optional bool `uses_dying_rules` (default false). | Designer |
| 2026-10-20 | Q3: a readied Strike uses the MAP frozen at Ready time (RAW, PC 417); v1's strike-count increment is dropped. | Designer |
| 2026-10-20 | Q4: Phase 1 keeps an open Seek roll (ledgered) but enforces the RAW area: "almost always 30 feet or less in any dimension" (PC 417). The cone/burst wording is not in the Remaster text. | Designer |

No open questions remain. Q1's ordering awaits the Architect's call.

### Decision 2026-10-20 (Designer): flat checks use full degrees of success
Flat checks are checks, so the normal Step 4 degree rules and the natural 20/1 shift both apply. Player Core's Flat Checks section exempts flat checks only from modifiers, and no rule limits them to success or failure. A DC 20 flat check that rolls a natural 20 is a CRITICAL SUCCESS: the numbers give a success, then the natural 20 moves it up one degree. Effects with no critical entry treat a critical success as a success and a critical failure as a failure. Recovery checks do list critical outcomes, so the engine must always return all four degrees. Sources: https://2e.aonprd.com/Rules.aspx?ID=2278 (Flat Checks; Step 4) and https://2e.aonprd.com/Rules.aspx?ID=2287.
