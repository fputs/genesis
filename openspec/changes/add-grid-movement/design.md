# Design

## Context

`crates/rules` has dice, checks, and creature templates. It has no grid. `Creature` and `CombatantState` are already specified; this change does not add fields to either. `add-modifiers` is a separate draft and is not imported here. See proposal.md for why the measurement exists. Designer sign-off on the checklist rulings is pending; the rulings below are the ones this change specifies.

`crates/rules` stays dependency-free. Every fallible grid function returns `Result` with `GridError`. Library code does not panic: no `unwrap`, `expect`, `panic!`, or unchecked indexing. Differences, sums, and products use checked arithmetic. This change names no file on `v1-reference`, so none is ported.

## Goals / Non-Goals

**Goals:**

- One new module, `crates/rules/src/grid.rs`, re-exported from `lib.rs`.
- Stride and Step validate an explicit path and return cost, end square, updated diagonal counter, and leave/enter events.
- Distance, adjacency, reach, and range increments are pure functions of squares and numbers.
- The immobilized check is a public function Step, Stride, and later move-trait actions can call.

**Non-Goals:**

- Path search. No function takes a destination and returns a route.
- Action spending, turn order, and reactions. `triggers` is only a mark.
- Modifier stacking, attack totals, and edits to `check.rs`.
- Cover, flanking, prone, Crawl, Stand, Drop Prone, hidden squares, forced movement, Tumble Through, and footprints other than 1×1. Crawl will share the turn's diagonal counter when positioning adds it.
- The 5-foot Speed floor. No Speed penalty exists yet.
- Walls and corner cutting. Sharing a square with a prone, dead, willing, or Tiny creature.

## Decisions

### Module and purity

```text
crates/rules/src/lib.rs     mod grid; pub use the items below
crates/rules/src/grid.rs    squares, movement, distance, reach, range
```

`grid.rs` does not read a clock, a file, the network, or a global, and it does not use `Cell`, `RefCell`, `Mutex`, or `unsafe`. It does not take `Dice`. It does not take `&Creature` or `&mut CombatantState`. Callers copy `speed_feet` and reach in. `Cargo.toml` gains no dependency. `lib.rs` keeps `#![forbid(unsafe_code)]`.

Alternative considered: store the diagonal counter on `CombatantState`. Rejected. That amends `creature-template` and mixes turn geometry into Hit Point state.

### No pathfinding API

Callers supply an explicit ordered path of squares. Stride and Step validate that path. They do not search. No function takes a destination and returns a route.

### Public shape

```rust
pub struct Square { pub x: i32, pub y: i32 }

pub enum Terrain { Open, Difficult, GreaterDifficult }

pub enum Size { Tiny, Small, Medium, Large, Huge, Gargantuan }

pub enum Space { UnderFive, Exact(u32), AtLeast(u32) }

pub struct SizeData {
    pub space: Space,
    pub reach_tall_feet: u32,
    pub reach_long_feet: u32,
}

pub struct FactionId(pub u32);

pub struct Occupant { pub square: Square, pub faction: FactionId }

pub struct MapBounds { pub width: u32, pub height: u32 }

pub struct DiagonalCounter { /* private diagonals: u32 */ }

pub enum SquareEvent { Left { square: Square }, Entered { square: Square } }

pub struct MoveOutcome {
    pub cost_feet: u32,
    pub end: Square,
    pub events: Vec<SquareEvent>,
    pub counter: DiagonalCounter,
    pub triggers: bool,
}

pub struct MoveInput<'a> {
    pub start: Square,
    pub path: &'a [Square],
    pub speed_feet: i32,
    pub size: Size,
    pub immobilized: bool,
    pub faction: FactionId,
    pub bounds: MapBounds,
    pub terrain: &'a [(Square, Terrain)],
    pub occupants: &'a [Occupant],
    pub counter: DiagonalCounter,
}

pub struct RangeBand { pub increments: u32, pub penalty: i32 }

pub enum GridError {
    ExceedsSpeed,
    SpeedNotMultipleOfFive,
    NegativeSpeed,
    OccupiedEnd,
    EnemySquare,
    StepIntoDifficultTerrain,
    StepRequiresTenFeet,
    StepNotOneSquare,
    PathNotAdjacent,
    OffGrid,
    Overflow,
    Immobilized,
    UnsupportedSize,
    InvalidRangeIncrement,
    NegativeDistance,
    BeyondRange,
}
```

`size_data(Size) -> SizeData` returns Tiny `UnderFive` reach 0/0; Small and Medium `Exact(5)` reach 5/5; Large `Exact(10)` reach 10/5; Huge `Exact(15)` reach 15/10; Gargantuan `AtLeast(20)` reach 20/15.

`DiagonalCounter::fresh` is 0 diagonals, so the next diagonal costs 5. `end_turn(self) -> Self` returns `fresh`. `diagonals(self) -> u32` is how a test reads the count. The counter is not a field of `Creature` or `CombatantState`.

`stride(MoveInput) -> Result<MoveOutcome, GridError>` and `step(MoveInput) -> Result<MoveOutcome, GridError>` take the counter by value and return the updated one. They do not mutate a hidden turn. Off-turn movement passes `DiagonalCounter::fresh()` and does not write the returned counter over the stored turn counter. That is the whole off-turn rule: the function has no global turn to touch.

`move_trait_allowed(immobilized: bool) -> Result<(), GridError>` returns `Immobilized` or `Ok(())`. Stride and Step call it. Crawl, Stand, and Drop Prone are not implemented.

`distance`, `adjacent`, and `in_reach` take two squares and two sizes. `in_reach` also takes `reach_feet: u32`. `range_band(distance_feet: i32, increment_feet: i32) -> Result<RangeBand, GridError>` returns the count and the penalty. `penalty` is a plain `i32`. It is not a modifier bucket and it is not added to `multiple_attack_penalty`.

Unlisted terrain is `Open`. The first listing of a square wins. An occupant whose `FactionId` differs from the mover makes that square an enemy square. Otherwise a matching `FactionId` is same faction. The mover is not in the occupant list.

### Cost, path, and events

A step from one square to the next is adjacent when the Chebyshev distance is 1, using checked subtraction. Orthogonal base is 5. Diagonal base is 5 when `diagonals` is even and 10 when odd, then the counter increments by 1 only for a diagonal. Difficult adds 5 and greater difficult adds 10, added once, not multiplied. The square being left adds nothing. A square is inside the map when `x` and `y` are at least 0 and strictly below `width` and `height`.

Stride accepts the path when the running cost, checked, is at most Speed. Equal to Speed succeeds. The outcome cost is that sum. Step's path must be exactly one adjacent square, Speed must be at least 10, and the destination must not be difficult or greater difficult. The outcome cost is 5 even when the diagonal base would be 10, and that 5 is not compared with Speed. A diagonal Step still increments the counter.

Events for a path `[b, c]` from start `a` are `Left a`, `Entered b`, `Left b`, `Entered c`. Stride sets `triggers` true. Step sets `triggers` false. An empty Stride has cost 0, no events, `end == start`, and the same counter. Nothing in the module searches toward a square the path omitted.

Stride checks, first match wins: unsupported size, immobilized, negative Speed, Speed not a multiple of 5, off-grid, overflow while measuring a step, path not adjacent, enemy square on any entered square, occupied end (same faction on the last square only), then cost above Speed. Step uses the same order, and after the Speed-multiple check it rejects Speed under 10, then a path that is not exactly one square, then difficult terrain before the occupied-end check. On `Err`, the caller keeps the start square and the counter it passed in. `GridError` carries neither.

Distance for two 1×1 squares: `n` diagonals cost `(n / 2) * 15 + (n % 2) * 5`, plus 5 per orthogonal square, with checked arithmetic. The same square is 0. This always starts at the 5-foot diagonal. It does not read the turn counter, terrain, or occupants. Adjacent means Chebyshev distance 1. Reach 5 is adjacent and not the same square. Reach 10 is distance at most 10, or both absolute deltas equal to 2. Reach above 10 is distance at most the reach, with no extra exception. The same square is never in reach.

Range count is `ceil(distance / increment)` and at least 1, so an exact multiple stays in that count. Penalty is `-2 * (count - 1)`. Count above 6 is `BeyondRange`, not a larger penalty. Increment `<= 0` is `InvalidRangeIncrement`. Distance `< 0` is `NegativeDistance`.

### What is not edited

Do not edit `crates/rules/src/check.rs`, `crates/rules/src/combatant.rs`, or `crates/rules/src/dice.rs`. Do not add a modifiers module or a modifiers spec. Do not add a path-search function. Tests live in `grid.rs` under `#[cfg(test)]` and call the public functions. Each test comment cites the Player Core section for that scenario. A test builds a `ScriptedDice` and asserts it is unused.

## QA questions

Designer rulings Q1–Q9. Each row names the scenario or the out-of-scope line that answers it.

| Q | Ruling | Answer |
|---|---|---|
| Q1 | Step and Crawl share the turn's diagonal counter. Crawl itself is out of scope (positioning). | Scenario "Step and Crawl share the diagonal counter". Out of scope: requirement "Step and Crawl share one counter" — this capability does not implement Crawl. |
| Q2 | A path over Speed is rejected with no partial move. | Scenario "Exact Speed". Requirement "Stride follows Speed along the path" rejects that path with no partial move. Requirement "A failed move does nothing" applies no prefix of the path. |
| Q3 | A Speed that is not a multiple of 5 is rejected at validation. The 5-ft floor under penalties is out of scope. | Scenario "Speed not a multiple of five". Validation here is the move's check (`SpeedNotMultipleOfFive`). Creature validation stays unchanged. Out of scope: requirement "Movement Speeds are multiples of five" — a 5-foot Speed floor under penalties is not applied. |
| Q4 | A Step on the counter's 10-ft diagonal is allowed, moves 5 ft, and advances the counter. | Scenario "Step on a 10-ft diagonal costs 5". |
| Q5 | Passing through a same-faction square is allowed. Ending any move there is rejected. RAW's next action moves out is out of scope (encounter-core). | Scenarios "Through an ally" and "Ending in an ally square". Out of scope: requirement "Same faction can be crossed" — the next-action move out is not implemented. |
| Q6 | Corner-cutting is out of scope. Phase 1 has no walls. Future ruling: a diagonal is blocked only when both orthogonal neighbours block. | Requirement "Corner cutting waits for walls" and scenario "A diagonal needs no wall check". The future ruling is recorded there and is not applied. |
| Q7 | Sharing a square with a prone, dead, willing, or Tiny creature is out of scope. | Requirement "Special square sharing is out of scope" and scenario "Prone dead willing and Tiny sharing is out of scope". |
| Q8 | There is no pathfinding API. Callers supply explicit ordered paths. | Design, "No pathfinding API", above. Scenario "An empty path does not search". |
| Q9 | Action cost and turn spending are out of scope (encounter-core). Movement legality and cost, plus Left/Entered events, are in. | Out of scope: requirement "Movement does not spend actions" and scenario "A Stride does not spend actions". In: scenario "Exact Speed" for legality and foot cost, and scenario "Stride emits left and entered events". |

## Deviations

Each item is rule, why not, substitute, intent. D1, D3, D4, D5, D11, D12, D13, and D17 are the checklist rulings as written; they are not deviations. D1 is the per-turn counter. D3 and D4 are the cost formula. D5 is "up to Speed" with no banking. D11 and D12 are adjacency and the 10-foot reach exception. D13 is the increment formula. D17 is the explicit path.

1. **D2, off-turn counter. Interpretation.** Rule: Player Core pg. 421 counts diagonals across your turn and resets at end of turn (https://2e.aonprd.com/Rules.aspx?ID=2357). Why not: a reaction or a readied Stride moves on someone else's turn, and the book does not define that count. Substitute: that action takes a fresh counter and does not replace the stored turn counter. Intent: off-turn movement still alternates, and your own turn's count stays put.

2. **D6, Step on a 10-foot diagonal. Interpretation.** Rule: Player Core pg. 421 counts all your movement, and Step on pg. 418 moves 5 feet (https://2e.aonprd.com/Actions.aspx?ID=2304). Why not: the Step can fall on the diagonal that would otherwise cost 10. Substitute: the Step costs 5 feet, does not spend Speed, and still advances the counter. Intent: Step stays a 5-foot action and the turn's alternation stays honest.

3. **D7, willing means same faction. Deviation.** Rule: Player Core pg. 422 lets you move through a willing creature; an unwilling one needs Tumble Through (https://2e.aonprd.com/Rules.aspx?ID=2360). Why not: Phase 1 has no willingness flag, and Tumble Through needs a skill check. Substitute: equal `FactionId` may be passed through; a different id blocks the square. Intent: allies do not stop a Stride, and enemies do, until Tumble Through exists.

4. **D8, no ending in an occupied square. Deviation.** Rule: Player Core pg. 422 allows a move to end in a willing creature's square when another move immediately leaves (https://2e.aonprd.com/Rules.aspx?ID=2360). Why not: Phase 1 cannot see the next action. Substitute: ending Stride or Step on any other creature is `OccupiedEnd` for same faction and `EnemySquare` for the other faction. Intent: a move does not finish while sharing a square. Encounter-core can restore the follow-up later.

5. **D9, Small and Medium only. Scope.** Rule: Player Core pg. 421 gives six sizes (https://2e.aonprd.com/Rules.aspx?ID=2359). Why not: Phase 1 footprints are one square. Substitute: `size_data` stores all six, and movement, distance, and reach reject the other four with `UnsupportedSize`. Intent: a later size change can reuse the table.

6. **D10, distance between creatures. Interpretation.** Rule: Player Core pg. 428 measures areas like movement and ignores difficult terrain (https://2e.aonprd.com/Rules.aspx?ID=2384). Why not: no passage states creature-to-creature distance. Substitute: grid cost between the two squares, alternation starting at 5, ignoring terrain, creatures, and the turn counter; same square is 0; the two directions match. Intent: reach and range share one measurement.

7. **D14, bad numbers, with Speed 0 kept. Interpretation.** Rule: the book does not define a negative Speed or a non-positive range increment. Why not: callers can still pass them. The checklist's "Speed ≤ 0" would also reject Speed 0, which contradicts edge case E22 and `creature-template`, both of which accept Speed 0. Substitute: `NegativeSpeed`, `InvalidRangeIncrement`, and `NegativeDistance` are errors and are not clamped. Speed 0 remains a legal multiple of 5. Intent: a bad measure fails. An empty Stride at Speed 0 still succeeds.

8. **D15, off the map. Engineering.** Rule: Player Core has no edge-of-map rule. Why not: a path can name a missing square. Substitute: `OffGrid` rejects the whole action. The caller keeps the start and the counter. Intent: no partial move and no stop at the edge.

9. **D16, no Speed floor; immobilized is a flag. Scope.** Rule: Player Core pg. 420 says penalties cannot reduce Speed below 5 feet (https://2e.aonprd.com/Rules.aspx?ID=2346), and pg. 444 says immobilized blocks move actions (https://2e.aonprd.com/Conditions.aspx?ID=81). Why not: no Speed penalty exists yet, and immobilized is not a condition. Substitute: the floor is not applied. `move_trait_allowed` reads a bool. Speed 0 allows an empty Stride and rejects Step. Intent: the check is ready for Crawl, Stand, and Drop Prone, and Speed is used as stored.

10. **Speed that is not a multiple of 5. Architecture.** Rule: grid Speeds are multiples of 5 (https://2e.aonprd.com/Rules.aspx?ID=2347). Why not: `creature-template` accepts any non-negative Speed, including 7. Substitute: movement returns `SpeedNotMultipleOfFive` and does not change creature validation. Intent: Stride does not invent a square cost for a Speed the grid cannot divide.

11. **Range penalty is not applied. Scope, checklist E19.** Rule: Player Core pg. 403 gives -2 per extra increment, and pg. 400 says that untyped penalty adds with the multiple attack penalty (https://2e.aonprd.com/Rules.aspx?ID=2290, https://2e.aonprd.com/Rules.aspx?ID=2282). Why not: attack rolls are encounter-core, and `add-modifiers` is a parallel change. Substitute: 130 feet at increment 60 returns count 3 and penalty -4, and nothing else. Intent: encounter-core can add that -4 to MAP's -5 later. This change does not produce the checklist's summed -9.

12. **Action cost is not spent. Scope.** Rule: Player Core pg. 418 says Stride and Step each cost 1 action. Why not: spending actions is encounter-core. Substitute: the functions do not read or write `actions_remaining` and do not return an action cost. Intent: the grid can be tested before turns exist.

## Risks / Trade-offs

- [Faction stands in for willing] → a same-faction creature that is unwilling cannot be represented. Tumble Through is the later fix.
- [Occupied end is always illegal] → a legal RAW follow-up move is rejected until encounter-core sequences actions.
- [Off-turn is a calling convention] → `stride` will happily update whatever counter it is given. The off-turn test must pass `fresh` and assert the stored counter is untouched.
- [Plain range penalty] → a caller that forgets to add `RangeBand.penalty` will miss the -2. The type is an `i32` on purpose so this module does not grow a modifier enum.
- [Checked overflow] → `i32::MIN` has no absolute value in `i32`. `checked_abs` and `checked_sub` return `Overflow` instead of panicking.
