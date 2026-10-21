# Genesis v2 — PF2e Remaster Rule Citations (verified against Archives of Nethys)

Verified 2026-10-20 by fetching each page live (curl). "Page" is the Source line AoN shows.
Status: **all changes (2–13) verified**. FINAL.

## Change 2 — dice-and-checks

| Mechanic | AoN URL | Source | Rule (key numbers) | Doc flag |
|---|---|---|---|---|
| Check procedure | https://2e.aonprd.com/Rules.aspx?ID=2278 | Player Core pg. 400 | Roll d20, add modifiers/bonuses/penalties, compare to the DC; result ≥ DC succeeds. | OK (doc said "Playing the Game › Checks"; AoN: Chapter 8: Playing the Game › Checks) |
| Flat checks | https://2e.aonprd.com/Rules.aspx?ID=2278 (Flat Checks section) | Player Core pg. 400 | Bare d20 vs DC, no modifiers ever. If several flat checks govern the same thing, roll once at the highest DC. DC ≤ 1 auto-succeeds; DC ≥ 21 auto-fails. | **Gap:** doc omitted the auto-success/auto-fail and highest-DC rules. Specify them. |
| Degree of success | https://2e.aonprd.com/Rules.aspx?ID=2286 | Player Core pg. 401 (Step 4: Degree of Success) | Beat the DC by 10+ = critical success; miss it by 10+ = critical failure. With no crit entry, crit success = success and crit failure = failure. A Strike crit deals double damage. | Section name corrected: "Step 4: Degree of Success" |
| Natural 1 / natural 20 | https://2e.aonprd.com/Rules.aspx?ID=2287 | Player Core pg. 401 | Nat 20 = one degree better; nat 1 = one degree worse. Apply this before any other degree-changing ability. | OK. Add "apply first" ordering. |
| DC from a modifier | https://2e.aonprd.com/Rules.aspx?ID=2285 | Player Core pg. 401 | DC = 10 + total modifier (e.g. AC-style DCs, Perception DC). | New. Useful for baked DCs. |
| Multiple attack penalty (MAP) | https://2e.aonprd.com/Rules.aspx?ID=2289 | Player Core pg. 402 | 2nd attack-trait check in your turn −5, 3rd+ −10; agile −4/−8. Use the weapon of the *current* attack. Applies only during your turn (so reactions on others' turns take no MAP). Every attack-trait check counts. | OK: −5/−10 and −4/−8 confirmed. Lives in encounter-core (5), referenced here. |
| MAP is **untyped** | https://2e.aonprd.com/Rules.aspx?ID=2282 | Player Core pg. 400 (Penalties) | Untyped penalties always **add** together. MAP and range penalty are the named examples. | **Confirms the decision**: MAP is untyped. v1 docs calling it "circumstance" were wrong. |

## Change 3 — creature-template (baked)

| Mechanic | AoN URL | Source | Rule (key numbers) | Doc flag |
|---|---|---|---|---|
| Stealth DC (baked `stealth_dc`, optional) | https://2e.aonprd.com/Actions.aspx?ID=2301 (Seek) | Player Core pg. 417 | Seek: a secret Perception check vs the **Stealth DC** of undetected/hidden creatures. Crit success → observed; success → undetected becomes hidden, hidden becomes observed. | OK: an optional baked field is enough. Absent = the creature cannot be Hidden/Undetected (deliberate deviation). |
| Hide (where Stealth DC comes from) | https://2e.aonprd.com/Actions.aspx?ID=2404 | Player Core pg. 244 | Stealth check vs the observer's Perception DC; +2 circumstance with standard cover, +4 with greater cover. | Context only; Hide is not in the MVP. In RAW the Stealth DC is 10 + Stealth modifier (Rules ID=2285). |
| Hidden | https://2e.aonprd.com/Conditions.aspx?ID=79 | Player Core pg. 444 | The observer knows your space; DC 11 flat check to target you. **You are off-guard to the creature you're hidden from.** | **Gap:** off-guard from hidden was not listed. Add it to positioning/seek. |
| Concealed | https://2e.aonprd.com/Conditions.aspx?ID=62 | Player Core pg. 442 | DC 5 flat check to target you; area effects are exempt. | OK |
| Aid (baked `aid_modifier`) | https://2e.aonprd.com/Actions.aspx?ID=2292 | Player Core pg. 416 | Reaction. Trigger: an ally is about to use an action that requires a skill check or attack roll. Requirement: you **prepared to help** (usually an action on your turn). GM picks the check; **typical DC 15**. Crit success +2 circumstance (**+3 if master, +4 if legendary**); success +1; failure 0; crit failure −1 circumstance. | **Corrections:** (a) RAW requires a preparatory action on your turn, which v1/doc omitted. (b) The crit bonus scales with proficiency. The baked modifier has no rank, so fix it at +2 and ledger that. DC 15 confirmed. |
| Raise a Shield value (template shield) | https://2e.aonprd.com/Rules.aspx?ID=2281 (Bonuses example) | Player Core pg. 400 | Example: a buckler grants a **+1** circumstance bonus to AC when raised; cover +2. | **Flag:** "+2 from Raise a Shield" is the standard-shield value only. The template should bake the shield's AC bonus per shield. Full verification in change 6. |
| Proficiency / typed bonus stacking (ref for change 6) | https://2e.aonprd.com/Rules.aspx?ID=2281 | Player Core pg. 400 | Same type: highest bonus only. Different types add. Proficiency 0 / lvl+2 / +4 / +6 / +8. | Deferred, apart from the circumstance rule (change 6). |
| Penalties stacking (ref for change 6) | https://2e.aonprd.com/Rules.aspx?ID=2282 | Player Core pg. 400 | Same type: worst penalty only. A bonus and a penalty of the same type both apply. Untyped penalties all add. | Matches change 6 as designed. |

## Change 4 — grid-movement

| Mechanic | AoN URL | Source | Rule (key numbers) | Flag |
|---|---|---|---|---|
| Grid movement | https://2e.aonprd.com/Rules.aspx?ID=2356 | Player Core pg. 421 | 1 square = 5 ft; a straight move costs 5 ft per square. | OK |
| Diagonals | https://2e.aonprd.com/Rules.aspx?ID=2357 | Player Core pg. 421 | Diagonal squares cost 5, then 10, alternating. The count is tracked **across all movement in the turn** and resets at end of turn. | **Flag:** the count is per turn, not per action. |
| Size, space, reach | https://2e.aonprd.com/Rules.aspx?ID=2359 | Player Core pg. 421 | Small/Medium: 5-ft space, 5-ft reach. Large: 10-ft space, reach 10 (tall) or 5 (long). | OK. MVP is Medium only. |
| Difficult terrain | https://2e.aonprd.com/Rules.aspx?ID=2366 | Player Core pg. 423 | +5 ft per difficult square, +10 ft per greater difficult square. The extra cost is **not** increased on diagonals. You can't Step into difficult terrain. Jumping ignores the terrain jumped over. | **Flag:** spec that the extra cost is not doubled on a diagonal. |
| Movement types / Speed floor | https://2e.aonprd.com/Rules.aspx?ID=2345 | Player Core pg. 420 | Stride uses land Speed only. Penalties can't reduce a Speed below 5 ft. | Minor addition. |

## Change 5 — encounter-core

| Mechanic | AoN URL | Source | Rule (key numbers) | Flag |
|---|---|---|---|---|
| Initiative | https://2e.aonprd.com/Rules.aspx?ID=2423 | Player Core pg. 435 | Usually a Perception check, ranked highest first. **Ties: the enemy goes before the PC**; tied PCs choose their order. | **WRONG in doc:** the doc said "PCs first". RAW is enemies first. The PC-vs-PC tie can be deterministic by ID (deviation). |
| Start of turn | https://2e.aonprd.com/Rules.aspx?ID=2428 | Player Core pg. 435 | Reduce durations, roll the recovery check if dying, then **regain 3 actions and 1 reaction** (an unused reaction is lost). | OK |
| End of turn | https://2e.aonprd.com/Rules.aspx?ID=2430 | Player Core pg. 436 | End effects, persistent damage, afflictions, then conditions like frightened decrease. | OK (order matters for deferred conditions). |
| Reactions in encounters | https://2e.aonprd.com/Rules.aspx?ID=2432 | Player Core pg. 436 | 1 reaction per round, usable on any turn. **The GM decides whether reactions are usable before your first turn.** | **Flag:** v1 grants reactions at encounter start. Ledger it as a GM choice. |
| End the encounter | https://2e.aonprd.com/Rules.aspx?ID=2426 | Player Core pg. 435 | Ends when foes are defeated, a truce is reached, etc. | OK. Victory/Defeat is our formalisation. |
| Stride | https://2e.aonprd.com/Actions.aspx?ID=2305 | Player Core pg. 418 | 1 action [move]: move up to Speed. | OK |
| Step | https://2e.aonprd.com/Actions.aspx?ID=2304 | Player Core pg. 418 | 1 action [move], requires Speed ≥ 10: move 5 ft; triggers no move-based reactions; not into difficult terrain. | OK |
| Strike | https://2e.aonprd.com/Actions.aspx?ID=2306 | Player Core pg. 418 | 1 action [attack] vs AC. Crit success deals double damage; success deals damage. | OK |
| MAP | https://2e.aonprd.com/Rules.aspx?ID=2289 | Player Core pg. 402 | −5/−10, agile −4/−8, untyped. See change 2. | OK |
| Damage rolls / minimum | https://2e.aonprd.com/Rules.aspx?ID=2301 | Player Core pg. 406 | Dice + modifiers. If penalties would reduce damage to 0 or less, deal 1. | OK (doc said min 1). |
| Doubling (crit) | https://2e.aonprd.com/Rules.aspx?ID=2307 | Player Core pg. 407 | Roll normally, then double the total. Rolling dice twice is a GM option. Fatal/deadly extras are not doubled. | OK. Use double-the-total. |
| Knocked out | https://2e.aonprd.com/Rules.aspx?ID=2324 | Player Core pg. 410 | **Most creatures die at 0 HP**; PCs and significant NPCs are knocked out instead. **Move initiative to directly before the turn in which you hit 0 HP.** Dying 1 (2 if crit), + wounded value. | **Flags:** (a) the initiative move is missing from the doc/v1; (b) decided: optional `uses_dying_rules`, default false (enemies die at 0 HP). |
| Dying | https://2e.aonprd.com/Rules.aspx?ID=2325 ; https://2e.aonprd.com/Conditions.aspx?ID=69 | Player Core pg. 411 (condition: pg. 443) | You are unconscious while dying. Dying 4 = death. | OK |
| Recovery checks | https://2e.aonprd.com/Rules.aspx?ID=2326 | Player Core pg. 411 | Flat check at **DC 10 + dying** at the start of turn. Crit success −2, success −1, failure +1, crit failure +2. | OK |
| Damage while dying | https://2e.aonprd.com/Rules.aspx?ID=2327 | Player Core pg. 411 | Dying +1, or +2 from a critical hit. | OK |
| Losing dying / wounded | https://2e.aonprd.com/Rules.aspx?ID=2328 ; https://2e.aonprd.com/Conditions.aspx?ID=99 | Player Core pg. 411; pg. 447 | **Wounded +1 when you LOSE dying** (not when you gain it). Gaining dying adds the wounded value. | **WRONG in v1:** v1 increments wounded on the knockout hit. |
| Unconscious | https://2e.aonprd.com/Conditions.aspx?ID=95 (text quoted on Rules.aspx?ID=2325) | Player Core pg. 446 | Can't act; **−4 status penalty to AC, Perception, Reflex**; blinded and off-guard; falls prone and drops held items. | **Flag:** needs a −4 *status* penalty, i.e. a status bucket. A minimal typed-modifier rule is needed by change 5/6. |
| Massive damage | https://2e.aonprd.com/Rules.aspx?ID=2332 | Player Core pg. 412 | Damage ≥ 2× max HP in one blow = instant death. | **Flag:** RAW doesn't require positive HP. v1 "while it had >0 HP" is an interpretation, so ledger it. |

## Change 6 — circumstance-modifiers

| Mechanic | AoN URL | Source | Rule | Flag |
|---|---|---|---|---|
| Bonuses | https://2e.aonprd.com/Rules.aspx?ID=2281 | Player Core pg. 400 | Same type: highest only. Types add. | OK |
| Penalties | https://2e.aonprd.com/Rules.aspx?ID=2282 | Player Core pg. 400 | Same type: worst only. A bonus and a penalty of the same type both apply. Untyped penalties sum. | OK. **Recommend the bucket model also cover `status`** (the unconscious −4 needs it). |

## Change 7 — shields

| Mechanic | AoN URL | Source | Rule | Flag |
|---|---|---|---|---|
| Raise a Shield | https://2e.aonprd.com/Actions.aspx?ID=2316 | Player Core pg. 419 | 1 action, requires wielding a shield; the shield's **listed** circumstance AC bonus lasts until the start of your next turn. | Per-shield bonus (decided). |
| Shield statistics | https://2e.aonprd.com/Rules.aspx?ID=2181 | Player Core pg. 274 | AC bonus (only when raised), Hardness, HP, BT. | OK |
| Shield Block | https://2e.aonprd.com/Feats.aspx?ID=5212 | Player Core pg. 262 (General feat 1) | Reaction. Trigger: shield raised and you would take **physical** damage from **an attack**. Prevents damage up to Hardness; you and the shield each take the rest. | **Flag:** the trigger is any attack's physical damage, not only Strikes. |

## Change 8 — reactive-strike

| Mechanic | AoN URL | Source | Rule | Flag |
|---|---|---|---|---|
| Reactive Strike | https://2e.aonprd.com/Actions.aspx?ID=2256 | Player Core pg. 138 (Fighter) | Trigger: a creature in reach uses a manipulate or move action, **makes a ranged attack**, or leaves a square during a move action. Melee Strike; a crit disrupts a manipulate action. Doesn't count toward or take MAP. | **Flag:** ranged-attack trigger missing; also the move-action trigger (e.g. Stand, Drop Prone are [move]). |
| Disrupting actions | https://2e.aonprd.com/Rules.aspx?ID=2342 | Player Core pg. 415 | A disrupted action still spends its actions and costs, but its effects don't occur. | OK |

## Change 9 — positioning

| Mechanic | AoN URL | Source | Rule | Flag |
|---|---|---|---|---|
| Drop Prone | https://2e.aonprd.com/Actions.aspx?ID=2295 | Player Core pg. 416 | 1 action **[move]**: fall prone. | **Flag:** it's a move action, so it triggers Reactive Strike. v1 said it doesn't provoke. |
| Stand | https://2e.aonprd.com/Actions.aspx?ID=2303 | Player Core pg. 418 | 1 action **[move]**. | **Same flag as Drop Prone.** |
| Crawl | https://2e.aonprd.com/Actions.aspx?ID=2293 | Player Core pg. 416 | Requires prone and Speed ≥ 10; move 5 ft, stay prone. | Add the Speed ≥ 10 requirement. |
| Leap | https://2e.aonprd.com/Actions.aspx?ID=2298 | Player Core pg. 417 | Horizontal 10 ft (Speed ≥ 15) or 15 ft (Speed ≥ 30); none if Speed < 15. Vertical 3 ft up + 5 ft across. | OK (vertical out of scope). |
| Take Cover | https://2e.aonprd.com/Actions.aspx?ID=2307 | Player Core pg. 418 | Requires standard cover, a nearby feature, or prone. Standard → greater (+4); otherwise you gain **standard (+2)**. Prone: greater cover vs **ranged** only. Ends on moving, an attack action, unconsciousness, or dismissal. | **WRONG in v1:** the "none → +1, lesser → +2" ladder isn't RAW. Prone +4 applies vs ranged only. |
| Cover | https://2e.aonprd.com/Rules.aspx?ID=2372 | Player Core pg. 424 | Lesser +1 (typically from a creature), standard +2, greater +4; circumstance. Center-to-center line. | OK |
| Flanking | https://2e.aonprd.com/Rules.aspx?ID=2375 | Player Core pg. 425 | Off-guard to **melee** attacks from the flankers. Opposite sides/corners; both able to act, wielding melee, with the foe in reach. | **Flag:** off-guard from flanking applies only to the flankers' melee attacks. |
| Off-guard | https://2e.aonprd.com/Conditions.aspx?ID=58 | Player Core pg. 445 | −2 circumstance to AC. | OK |
| Prone | https://2e.aonprd.com/Conditions.aspx?ID=88 | Player Core pg. 445 | Off-guard; −2 circumstance to attack rolls; only Crawl/Stand as move actions. | OK |
| **Hidden ⇒ off-guard** | https://2e.aonprd.com/Conditions.aspx?ID=79 | Player Core pg. 444 | A creature you're hidden from is off-guard to you. | **Added** to positioning and seek-and-hidden. |

## Change 10 — seek-and-hidden

| Mechanic | AoN URL | Source | Rule | Flag |
|---|---|---|---|---|
| Seek | https://2e.aonprd.com/Actions.aspx?ID=2301 | Player Core pg. 417 | Exact area wording: "Choose an area to scan. The GM determines the area you can scan with one Seek action—almost always 30 feet or less in any dimension." The GM rolls a secret Perception check vs Stealth DC. Crit: observed. Success: undetected → hidden, hidden → observed. | **Flag:** the Remaster has **no** "30-ft cone or 15-ft burst" wording (that phrasing is presumably legacy Core Rulebook, not verified). Decision: open roll (ledgered); area ≤ 30 ft enforced. |
| Hidden | https://2e.aonprd.com/Conditions.aspx?ID=79 | Player Core pg. 444 | DC 11 flat check to target; you are off-guard to it. | See change 9. |
| Concealed | https://2e.aonprd.com/Conditions.aspx?ID=62 | Player Core pg. 442 | DC 5 flat check to target. | OK |
| Undetected | https://2e.aonprd.com/Conditions.aspx?ID=96 | Player Core pg. 447 | The observer must guess your square; a DC 11 flat check applies if it guesses right. | OK |
| Point Out | https://2e.aonprd.com/Actions.aspx?ID=2315 | Player Core pg. 419 | 1 action [auditory, **manipulate**, visual]: undetected → hidden for allies who can see you. | **Flag:** it's manipulate, so it triggers Reactive Strike. v1 lists it as unreadiable only. |

## Change 11 — delay

| Mechanic | AoN URL | Source | Rule | Flag |
|---|---|---|---|---|
| Delay | https://2e.aonprd.com/Actions.aspx?ID=2294 | Player Core pg. 416 | Free action, trigger: your turn begins. You leave the order; **return as a free action** at the end of any other creature's turn (permanent new position). **No reactions while delayed.** Negative start/end-of-turn effects happen immediately. Delaying a full round loses the turn; your position is unchanged. | **WRONG in v1:** v1 spends the reaction to resume and keeps it available while delayed. |

## Change 12 — ready

| Mechanic | AoN URL | Source | Rule | Flag |
|---|---|---|---|---|
| Ready | https://2e.aonprd.com/Actions.aspx?ID=2299 | Player Core pg. 417 | 2 actions [concentrate]: choose an action and a trigger; **your turn then ends**. If triggered before your next turn, use it as a reaction. A readied attack uses **the MAP you had when you Readied**. | **Flags:** the turn ends (remaining action lost); MAP is frozen at Ready time. Decided: no strike-count increment (v1's is dropped). |

## Change 13 — aid

| Mechanic | AoN URL | Source | Rule | Flag |
|---|---|---|---|---|
| Aid | https://2e.aonprd.com/Actions.aspx?ID=2292 | Player Core pg. 416 | See change 3. Prepare action required; DC 15; +2/+1/0/−1 (+3/+4 at master/legendary). | Decided: RAW prep kept; crit fixed at +2 (ledger). |
