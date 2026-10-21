# add-grid-movement — Rules Input Checklist (Game Designer)

Change: `add-grid-movement` (MVP change 4 in `docs/designer/pf2e-mvp-scope.md`). Depends on project-context (1) and creature-template (3).
Baseline: PF2e **Remaster**, *Player Core* (PC). Every row below was re-fetched live from Archives of Nethys with curl on **2026-10-20 (~21:10 PT)**. The page number is the "Source" line AoN shows. Raw extracts are in `./aon/`.
Rules are paraphrased, not copied: the rule text is ORC, and AoN prose is not to be pasted into specs or fixtures.

> **Shape note:** `add-modifiers-checklist.md`, `pf2e-citations.md` and `rules-deviations.md` were **not on the box** under `/workspace/genesis-design/`. `rules-deviations.md` is also not in `fputs/genesis`. Prior decisions were taken from `fputs/genesis:docs/designer/pf2e-citations.md` (Change 4 table) and `docs/designer/pf2e-mvp-scope.md`. This file uses the house shape: a citation table, then rulings, scope, scenarios, and dependencies. Re-shape it if the modifiers checklist turns up and differs.

## 1. Verified rules

| # | Mechanic | AoN URL | PC page | Rule (key numbers) | Status vs prior |
|---|---|---|---|---|---|
| R1 | Grid / 5-ft squares | https://2e.aonprd.com/Rules.aspx?ID=2356 | 421 | 1 square = 5 ft. A straight (orthogonal) move costs 5 ft per square. Difficult terrain can raise a square's cost. | OK |
| R2 | Diagonal 5/10 alternation | https://2e.aonprd.com/Rules.aspx?ID=2357 | 421 | 1st diagonal square in a **turn** = 5 ft, 2nd = 10 ft, then alternating (4 diagonals = 30 ft). The count is **tracked across all your movement during your turn and resets at end of turn**. | **CONFIRMED: per turn, NOT per move action.** Correction: the cite is pg **421** only. The prior "421/423" pairing mixed it with Terrain (423). |
| R3 | Speed | https://2e.aonprd.com/Rules.aspx?ID=2347 | 420 | Speed = land Speed unless a type is named. Stride moves up to your Speed. | OK |
| R4 | Movement types; Speed floor | https://2e.aonprd.com/Rules.aspx?ID=2346 (also 2345) | 420 | Stride uses land Speed only. You can't Step with a special movement type. Speeds change via item/circumstance/status bonuses and penalties. **Penalties can't reduce a Speed below 5 ft** unless stated otherwise. Switching movement type requires a new action. | OK (floor is deferred, see §3) |
| R5 | Stride | https://2e.aonprd.com/Actions.aspx?ID=2305 | 418 | 1 action, [move]: move up to your Speed. | OK |
| R6 | Step | https://2e.aonprd.com/Actions.aspx?ID=2304 | 418 | 1 action, [move]. **Requires Speed ≥ 10 ft.** Move 5 ft. Triggers no move/enter/leave reactions. **Can't Step into difficult terrain.** Land Speed only. | OK. Nothing else for Speed < 10 beyond the requirement. |
| R7 | Difficult / greater difficult terrain | https://2e.aonprd.com/Rules.aspx?ID=2366 | 423 | Entering a difficult square costs **+5 ft**; entering a greater difficult square costs **+10 ft**. **The extra cost is not increased further on a diagonal.** Can't Step into difficult terrain. Jumping ignores terrain jumped over. | **CONFIRMED** (not doubled on diagonals). The cost is keyed to *entering* the square. |
| R8 | Ignore difficult terrain | https://2e.aonprd.com/Rules.aspx?ID=2367 | 423 | Ignoring difficult terrain lets you treat greater difficult terrain as difficult (+5), not ignore it. | Deferred (no abilities in MVP) |
| R9 | Size, space, reach table | https://2e.aonprd.com/Rules.aspx?ID=2359 | 421 | Tiny: <5 ft, reach 0/0. Small: 5, 5/5. Medium: 5, 5/5. Large: 10, 10 tall/5 long. Huge: 15, 15/10. Gargantuan: ≥20, 20/15. Small+ occupy ≥1 square and don't share squares (mounts aside). **≥4 Tiny creatures can share a square**, and Tiny can share a larger creature's space. A Tiny creature with reach 0 must share a space to attack. | OK. MVP is Medium-only per prior scope. |
| R10 | Moving through a creature's space | https://2e.aonprd.com/Rules.aspx?ID=2360 | 422 | You can move through a **willing** creature's space. For an unwilling one, use Tumble Through. **You can't end your turn in another creature's square.** You *can* end a move action there if you **immediately** use another move action to leave. Accidental overlap: the GM decides who is forced out. | **Wording fix:** RAW is willing/unwilling, not ally/enemy. |
| R11 | Prone/incapacitated sharing | https://2e.aonprd.com/Rules.aspx?ID=2361 | 422 | You may share a space with a prone creature that is willing, unconscious, or dead **and** your size or smaller. | Deferred (prone/unconscious interplay) |
| R12 | Creatures of different sizes | https://2e.aonprd.com/Rules.aspx?ID=2362 | 422 | You can move through the space of a creature **≥3 sizes larger or ≥3 sizes smaller**, but can't end movement there. Tiny creatures can move through and end in others' spaces, and others can end in a Tiny creature's space. | Deferred (Medium-only) |
| R13 | Forced movement | https://2e.aonprd.com/Rules.aspx?ID=2364 | 422 | Distance is set by the effect, not Speed. It doesn't trigger movement reactions. It stops at the last space you can occupy. | Deferred (no Shove/Reposition) |
| R14 | Tumble Through | https://2e.aonprd.com/Actions.aspx?ID=2370 | 233 | 1 action, [move], Acrobatics vs the enemy's Reflex DC when entering its space, during a Stride. Success: pass through, its squares count as difficult terrain; not enough Speed to clear the space counts as a failure. Failure: movement ends; you trigger reactions as if you left your starting square. | **Deferred** (needs skills plus Reflex DC) |
| R15 | Squeeze | https://2e.aonprd.com/Actions.aspx?ID=2372 | 233 | **Exploration** activity measured in minutes. Many tight spaces are just difficult terrain. | Out of scope. The Remaster has no encounter-mode "Large creature squeezing" rule. |
| R16 | Range and reach | https://2e.aonprd.com/Rules.aspx?ID=2379 | 426 | Reach is typically 5 ft, and Melee Strikes use reach. **10-ft reach reaches 2 squares diagonally**, an exception to normal measurement. Reach above 10 ft is measured normally (20 ft → 3 diagonal, 30 ft → 4). Range increment: −2 per increment beyond the first, max −10 after five extra increments. Beyond that, no attack. | OK |
| R17 | Range penalty | https://2e.aonprd.com/Rules.aspx?ID=2290 | 403 | No penalty **at or within** the first increment. −2 per additional increment. Attacks allowed **up to 6 increments**. Crossbow 120: ≤120 → 0; >120 to ≤240 → −2; … >600 to ≤720 → −10. | OK. The boundary is **inclusive** at each increment. |
| R18 | Range penalty is untyped | https://2e.aonprd.com/Rules.aspx?ID=2282 (per pf2e-citations.md, change 2) | 400 | Untyped penalties all add. MAP and range penalty are the named examples. | OK. Stacks with MAP. |
| R19 | Reach weapon trait | https://2e.aonprd.com/Traits.aspx?ID=684 | 283 | The weapon attacks up to 10 ft away "instead of only adjacent enemies". For creatures with reach, it adds +5 ft. | Deferred (weapon traits); defines "adjacent" informally |
| R20 | Area measurement | https://2e.aonprd.com/Rules.aspx?ID=2384 | 428 | On a grid, areas are measured **the same way as movement** (so the diagonal rule applies) but are **never affected by difficult terrain**. | Supports R2-based distance measuring |
| R21 | Immobilized | https://2e.aonprd.com/Conditions.aspx?ID=81 | 444 | You can't use any action with the **move** trait. External forces must beat the holding DC to move you. | Partial (see rulings) |
| R22 | Fly (diagonal-up = difficult) | https://2e.aonprd.com/Actions.aspx?ID=2312 | 419 | Upward movement uses the difficult-terrain rules. | Out of scope (2D only) |

## 2. Designer rulings (recommended calls)

| ID | Ambiguity | Ruling | Basis |
|---|---|---|---|
| D1 | Diagonal parity scope | **One parity counter per creature per turn.** It starts "next diagonal = 5" at the start of that creature's turn and is shared by every Stride/Step in the turn. Orthogonal squares don't flip it. It resets at end of turn. | R2 RAW |
| D2 | Moving on another creature's turn (reactions, readied Stride) | The counter isn't defined off-turn. **Use a fresh counter scoped to that action**, discarded afterwards, which leaves the mover's own turn counter untouched. Ledger it as an interpretation. | R2 is silent |
| D3 | Square cost formula | `cost(square) = base + terrain_extra`, where base is 5 orthogonal or 5/10 by parity on a diagonal, and terrain_extra is 0/+5/+10 for the **destination** square. The terrain extra is never multiplied by the diagonal. A greater-difficult square entered on a 10-ft diagonal = 20 ft. | R2, R7 |
| D4 | Terrain of the square you leave or start in | **Ignored.** Only the entered square's terrain counts. | R7 ("moving into a square") |
| D5 | Exact Speed / insufficient Speed | You may enter a square only if cumulative cost ≤ Speed. Exactly equal is legal. Leftover movement is lost when the action ends; there's no banking across actions, though the diagonal parity does carry (D1). | R5 "up to your Speed" |
| D6 | Step legality | A Step is legal iff Speed ≥ 10 **and** the destination isn't difficult or greater difficult **and** the destination is a legal end square. It costs nothing against Speed and **does advance the diagonal parity counter** (RAW counts "all your movement"). The 5-ft distance is fixed even when the parity says 10. | R2, R6. Ledger the parity point; it's an interpretation |
| D7 | Ally vs enemy spaces | Engine term **willing = same faction** in Phase 1. Same faction may pass through; other faction is **blocked** until Tumble Through lands. | R10. Ledger "faction = willing" |
| D8 | Ending in an occupied square | A move action may end in a willing creature's square **only if the next action is a move action that leaves it**. Phase 1 simplification: **reject ending any move action in an occupied square.** Ledger it as a deviation; it can be lifted later. | R10 |
| D9 | Medium-only | Phase 1 creatures are Small/Medium (1 square, reach 5). The Size enum carries all six sizes with space and reach data, but only 1×1 footprints are accepted. | R9, prior scope |
| D10 | Distance between creatures | Distance = the shortest grid path between the **closest squares** of the two spaces, using the D1 alternation starting at 5, **ignoring terrain and creatures**. The rule is symmetric. Same/overlapping square = 0. | R20 + R16. AoN has no single "measure between creatures" passage; ledger it as the derived rule |
| D11 | Adjacency / reach 5 | Adjacent = the closest squares share an edge or a corner (Chebyshev 1). Reach 5 = adjacent only. | R16, R19 |
| D12 | Reach 10 | A target is in reach if the grid distance (D10) ≤ 10 **or** the target is exactly 2 diagonal squares away (the 10-ft computes as 15 by D1, so it's a special case). Reach > 10 uses D10 unmodified. | R16 |
| D13 | Range increment math | `increments = ceil(distance / increment)`, minimum 1. Penalty = −2 × (increments − 1), **untyped**. Increments > 6 means the attack is illegal. Distance exactly on a boundary falls in the lower increment. | R17, R18 |
| D14 | Zero/negative distances | Distance is ≥ 0 by construction. Negative inputs, or a Speed/increment ≤ 0 in fixtures, are a **validation error**, not a clamp. | Engineering |
| D15 | Off-grid | Any path square outside the map is illegal and the whole action is rejected atomically. No partial move, and no "stop at edge". | Engineering. RAW has no edge-of-world rule |
| D16 | Speed 0 / immobilized | Immobilized means every [move] action is illegal (Stride, Step, and later Stand, Crawl, Drop Prone). Speed 0 from fixtures means Stride moves 0 squares and Step is illegal (Speed < 10). The **5-ft penalty floor isn't modelled** in Phase 1 (no Speed penalties exist yet). | R4, R6, R21 |
| D17 | Path vs destination | Stride takes an explicit ordered path of squares, not just a destination. That's needed for parity, terrain, and later the Reactive Strike leave-square triggers. | Engineering (enables change 8) |

## 3. Phase 1 scope cut

**IN (add-grid-movement):** R1, R2/D1–D2, R3, R5, R6/D6, R7/D3–D5, R9 (data only)/D9, R10 subset (D7, D8), R16 reach 5/10 (D11–D12), R17/R18 range math (D13), R20-derived distance (D10), D14, D15, D16 (immobilized blocks move actions; Speed 0 behaviour), D17.

**DEFERRED:**

| Item | Cite | Owner change |
|---|---|---|
| Minimum Speed 5 ft under penalties | R4, PC 420 | circumstance-modifiers (6), or a later speed-modifiers change |
| Tumble Through | R14, PC 233 | future skills/acrobatics change (needs Reflex DC + Acrobatics) |
| Ending a move action in a willing creature's square (follow-up move rule) | R10, PC 422 | encounter-core (5) follow-up, once action sequencing exists |
| Large+ footprints, size-difference pass-through, Tiny sharing | R9, R12, PC 421–422 | future add-creature-sizes |
| Prone/unconscious space sharing | R11, PC 422 | positioning (9) |
| Forced movement | R13, PC 422 | future shove/reposition |
| Ignore difficult terrain, hazardous terrain, jumping over terrain | R7, R8, PC 423 | positioning (9) for Leap; later for the rest |
| Reach trait weapons, reach > 10 | R19, PC 283 | future weapon-traits |
| Fly/climb/swim/burrow, 3D | R4, R22 | out of MVP |
| Squeeze | R15, PC 233 | out of MVP (exploration) |
| Immobilized as a condition source (grabbed etc.) | R21, PC 444 | conditions change (deferred). Phase 1 tests a fixture flag only |

## 4. Edge-case hunt — proposed scenarios

- **E1 Exact Speed.** Given Medium Speed 25 on open ground, When it Strides 5 orthogonal squares, Then it succeeds with 0 ft left. And When it attempts a 6th square, Then the action is rejected. (R5, D5)
- **E2 Diagonal parity across actions.** Given Speed 25, When it Strides 1 diagonal (5) and then Strides again with 1 diagonal, Then the second diagonal costs 10. (R2 pg 421, D1)
- **E3 Odd diagonal at turn end.** Given the creature ended its turn after 1 diagonal, When its next turn begins and it moves 1 diagonal, Then it costs 5. (R2 reset)
- **E4 Orthogonal doesn't flip parity.** Given diagonal(5), orthogonal(5), diagonal, Then the last diagonal costs 10 (total 20). (R2)
- **E5 Difficult on a diagonal.** Given the 2nd diagonal square of the turn is difficult, Then it costs 10+5 = 15 (not 20). (R7, D3)
- **E6 Greater difficult on a diagonal.** Given the 1st diagonal enters greater difficult terrain, Then it costs 5+10 = 15. Given the 2nd, it costs 20. (R7)
- **E7 Leaving difficult terrain.** Given the start square is difficult and the destination is normal, When it moves 1 orthogonal, Then it costs 5. (D4)
- **E8 Terrain cost exceeds the remainder.** Given 5 ft left and a difficult destination (cost 10), Then the move is rejected. (D5)
- **E9 Step into difficult terrain.** Given an adjacent difficult square, When the creature Steps there, Then it's rejected. (R6, R7)
- **E10 Step with Speed 5.** Given Speed 5, When it Steps, Then it's rejected. (R6)
- **E11 Step advances parity.** Given the creature Stepped diagonally, When it Strides 1 diagonal, Then it costs 10. (D6, ledgered)
- **E12 Through an ally.** Given a same-faction creature in the path, When the creature Strides through to an empty square, Then it succeeds. (R10, D7)
- **E13 Through an enemy.** Given an other-faction creature in the path, When the creature Strides through it, Then it's rejected (Tumble Through deferred). (R10, R14)
- **E14 Ending in an ally's square.** Given an ally in the destination, When the creature ends its Stride there, Then it's rejected (D8, ledgered; RAW PC 422 allows it if immediately followed by another move).
- **E15 Off-grid.** Given a path that leaves the map, Then the whole Stride is rejected and the position is unchanged. (D15)
- **E16 Distance 0 / adjacency.** Given creatures at Chebyshev distance 1 diagonal, Then distance is 5 and adjacent is true. A self-distance query returns 0. (D10, D11)
- **E17 Negative / zero inputs.** Given a fixture with Speed −5 or range increment 0, Then load fails validation. (D14)
- **E18 Range boundary.** Given increment 60 and a target at exactly 60 ft, Then the penalty is 0. At 65 ft the penalty is −2. At 360 ft it's −10. At 365 ft the attack is illegal. (R17, D13)
- **E19 Range + MAP stack.** Given a 2nd attack at 130 ft (increment 60 → 3rd increment), Then the total penalty = −5 MAP + −4 range = −9, untyped and summed. (R17, R18)
- **E20 Reach-10 diagonal exception.** Given reach 10 and a target 2 squares diagonally away (computes as 15 ft by D1), Then it is in reach (special case). Given a (2,1) offset (1 diagonal + 1 orthogonal = 10 ft), Then it is in reach normally. Given a (3,1) offset (1 diagonal + 2 orthogonal = 15 ft), Then it is not in reach. Given reach 20 and a target 3 diagonals away (5+10+5 = 20 ft), Then it is in reach, matching RAW's example. (R16, D12)
- **E21 Immobilized.** Given an immobilized fixture, When it Strides or Steps, Then both are rejected. (R21)
- **E22 Speed 0.** Given Speed 0, When it Strides, Then the action is legal with 0 squares moved (recommend rejecting a non-empty path). Step is rejected. (D16)

**Out of scope (explicit):**
- Large creature squeezing: there's no encounter-mode Remaster rule. Squeeze is exploration-only (PC 233, Actions ID=2372).
- Reach vs Large creature edges: Large footprints are deferred (R9, PC 421). D10's "closest squares" already generalises, so re-test in add-creature-sizes.
- Minimum Speed 5: deferred (R4, PC 420).
- Tumble Through: deferred (R14, PC 233).

## 5. Depends on later changes (keep out of add-grid-movement)

- **Cover** (positioning 9; Rules ID=2372/2373, PC 424): no line or cover computation here. Distance (D10) ignores creatures and terrain by design.
- **Flanking** (positioning 9; ID=2375, PC 425): needs "opposite sides" geometry. Expose adjacency/reach queries only.
- **Prone / Crawl / Stand / Drop Prone** (positioning 9; PC 416–418, 445): Crawl's 5 ft with Speed ≥ 10, Stand, and prone space-sharing (R11) all stay out. The move-trait legality hook (D16) must be reusable by them.
- **Reactive Strike move triggers** (reactive-strike 8; Actions ID=2256, PC 138): grid-movement must **emit per-square leave/enter events along the explicit path (D17)** and mark Step as non-triggering (R6). No reaction resolution here. Tumble Through failure ("trigger as if you moved out of your start square") also lands later.
- **Hidden / undetected** (seek-and-hidden 10; PC 444, 447): no hidden-square guessing or targeting flat checks. Distance/reach queries must take positions, not visibility.
- Also flagged: forced movement (R13) must bypass the move-trigger events when it arrives later.
