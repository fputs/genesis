# Design

## Context

`modifiers` already builds the prone attack penalty, cover, and one off-guard penalty, and `total` folds them. `grid` already answers `in_reach` and `move_trait_allowed` for Small and Medium squares. `add-encounter-turns` names `Side::{Pc, Enemy}` and re-exports it. `CombatantState.conditions` is an empty slot. See proposal.md for why this family is one change. This change names no file on `v1-reference`, so none is ported. `crates/rules` stays dependency-free. Fallible functions return `Result`. No `unwrap`, `expect`, `panic!`, or unchecked indexing. No file under `docs/designer/` is edited here. DEV-23 through DEV-32 are on main in `docs/designer/rules-deviations.md`.

## Goals / Non-Goals

**Goals:**

- One new module, `crates/rules/src/positioning.rs`, re-exported from `lib.rs`.
- Prone, flanking, and Take Cover choose inputs. `attack_modifiers`, `ac_modifiers`, and `total` compute the numbers.
- Flanking geometry is an exact center-line test on 1×1 squares. Reach is `in_reach`. The only team type is `Side`.

**Non-Goals:**

- Editing `modifiers.rs`, `grid.rs`, `combatant.rs`, `check.rs`, or `dice.rs`. No new field on `Creature` or `CombatantState`. No `FactionId` and no second `Side` enum.
- Crawl and a standing Leap's distance. Those distances are `add-crawl-and-leap`. Stride and Step already exist in grid-movement (`grid.rs`). This change does not implement them and does not claim them.
- Reactive Strike, shields, Seek, hidden, undetected, concealed, and unconscious's −4. Hidden off-guard is change 10 (FIX-14).
- Spending actions or reading a turn. Costs are returned data. No trigger flag.

## Decisions

### Call the modifiers that already exist

`attack_penalty(Posture::Prone)` calls `attack_modifiers(0, true, 0)`. `Posture::Standing` calls `attack_modifiers(0, false, 0)`. The zeros omit the multiple attack penalty and Aid. A `ModifierError` becomes `PositioningError::Modifier`. `OffGuard::Yes` is the `true` passed to `ac_modifiers`. `OffGuard::No` is `false`.

`armor_class` picks one `Option<Cover>` for the attack, calls `ac_modifiers(cover, off_guard, false)`, appends `extra`, and returns `total` of that slice. The `false` omits unconscious. `ModifierError::Overflow` is `PositioningError::Modifier`, not a wrapped `i32`.

The cover degree is the higher bonus, per attack, of the stored benefit and the caller cover:

| Source | Melee | Ranged |
|---|---|---|
| no benefit | 0 | 0 |
| `Standard` | +2 | +2 |
| `Greater` | +4 | +4 |
| `GreaterVsRanged` | 0 | +4 |
| `StandardPlusGreaterVsRanged` | +2 | +4 |
| caller `Lesser` / `Standard` / `Greater` | +1 / +2 / +4 | same |

Map the max to one `Cover`: 4 → `Greater`, 2 → `Standard`, 1 → `Lesser`, 0 → `None`. Pass that once. A second cover bonus is rejected (+4, not +6).

### Posture and the benefit are returned values

Every type below derives `Debug, Clone, Copy, PartialEq, Eq`.

```rust
pub enum Posture { Standing, Prone }
pub enum Readiness { Able, Unable }
pub enum NearbyFeature { Present, Absent }
pub enum OccupantPresence { Present, Absent }
pub enum AttackKind { Melee, Ranged }
pub enum CoverBenefit { Standard, Greater, GreaterVsRanged, StandardPlusGreaterVsRanged }
pub enum EndReason { LeftSquare, AttackAction, Unconscious, Dismiss, ForcedMovement }
pub enum ActionTrait { Move }
pub enum ActionCost { Actions { count: u8, traits: &'static [ActionTrait] }, Free }
pub enum MoveAction { Stride, Step, DropProne }

pub struct PostureChange {
    pub posture: Posture,
    pub square: Square,
    pub benefit: Option<CoverBenefit>,
    pub cost: ActionCost,
}

pub struct TakeCoverResult {
    pub benefit: CoverBenefit,
    pub square: Square,
    pub cost: ActionCost,
}

pub enum Mobility { Free, Immobilized }

pub enum OffGuard { Yes, No }

pub enum PositioningError {
    Immobilized,
    NotProne,
    Unsupported,
    ProneRestrictsMovement,
    RequirementNotMet,
    DuplicateId,
    SelfId,
    UnsupportedSize,
    Overflow,
    InvalidReach,
    Modifier(ModifierError),
}
```

`ModifierError` is `Copy`, so `PositioningError` is `Copy`. Geometry is a boolean. There is no trigger field.

`Side` is the enum from `add-encounter-turns`: `Pc` or `Enemy`. This module imports it and does not declare it. `CombatantPos` stores it by value.

```rust
pub struct CombatantPos {
    pub id: CombatantId,
    pub square: Square,
    pub size: Size,
    pub reach_feet: u32,
    pub side: Side,
    pub readiness: Readiness,
    pub posture: Posture,
}
```

No facing field. No faction field. No consent or charm field.

```rust
pub fn drop_prone(
    posture: Posture,
    square: Square,
    mobility: Mobility,
    benefit: Option<CoverBenefit>,
) -> Result<PostureChange, PositioningError>

pub fn stand(
    posture: Posture,
    square: Square,
    mobility: Mobility,
    benefit: Option<CoverBenefit>,
    occupant: OccupantPresence,
) -> Result<PostureChange, PositioningError>

pub fn prone_move(
    posture: Posture,
    mobility: Mobility,
    action: MoveAction,
    counter: DiagonalCounter,
) -> Result<(), PositioningError>

pub fn leap_while_prone(posture: Posture, mobility: Mobility) -> Result<(), PositioningError>

pub fn take_cover(
    posture: Posture,
    square: Square,
    stored: Option<CoverBenefit>,
    caller_cover: Option<Cover>,
    feature: NearbyFeature,
) -> Result<TakeCoverResult, PositioningError>

pub fn dismiss(stored: Option<CoverBenefit>) -> Result<Option<CoverBenefit>, PositioningError>
pub fn dismiss_cost() -> ActionCost

pub fn end_benefit(reason: EndReason, benefit: Option<CoverBenefit>) -> Option<CoverBenefit>

pub fn flanks(
    defender: &CombatantPos,
    attacker: &CombatantPos,
    partners: &[CombatantPos],
) -> Result<bool, PositioningError>

pub fn off_guard(
    defender: &CombatantPos,
    attacker: &CombatantPos,
    attack: AttackKind,
    partners: &[CombatantPos],
    other: OffGuard,
) -> Result<bool, PositioningError>

pub fn attack_penalty(posture: Posture) -> Result<Vec<Modifier>, PositioningError>

pub fn armor_class(
    benefit: Option<CoverBenefit>,
    caller_cover: Option<Cover>,
    off_guard: OffGuard,
    attack: AttackKind,
    extra: &[Modifier],
) -> Result<ModifierTotal, PositioningError>
```

`drop_prone`, `stand`, and `take_cover` do not take `CombatantState`. Tests build a state, do not pass it, and assert actions and the reaction are unchanged. `OccupantPresence::Present` does not change `stand`'s `Ok` result. The returned square is the input square. `PostureChange` has exactly `posture`, `square`, `benefit`, and `cost`. Another creature ending a move in the square is grid-movement's `OccupiedEnd`.

`prone_move` and `leap_while_prone` return `()`. The counter is `Copy` and is taken by value, so an error leaves the caller's counter as it was. Neither function calls `stride` or `step`. Those functions stay in grid-movement. `MoveAction::Stride` and `MoveAction::Step` only name the actions the prone condition rejects.

`dismiss(None)` and `dismiss(Some(_))` are both `Ok(None)`. `dismiss_cost()` is `ActionCost::Free`. `end_benefit` returns `None` for every `EndReason` and does not take posture. `drop_prone` copies `benefit` through. `stand` applies R-11, below, and does not call `end_benefit`.

### Action checks

`drop_prone`, `stand`, and `leap_while_prone` call `move_trait_allowed` first. `prone_move` does the same before the prone restriction.

- `Mobility::Immobilized`: `Immobilized`. Inputs unchanged. This wins over `NotProne` and over `ProneRestrictsMovement`.
- `drop_prone` while prone: `ProneRestrictsMovement`.
- `stand` while standing: `NotProne`.
- `prone_move` while prone, for `Stride`, `Step`, or `DropProne`: `ProneRestrictsMovement`.
- `prone_move` while standing and `Mobility::Free`: `Ok(())`. The counter is unchanged. No distance.
- `leap_while_prone` while prone: `ProneRestrictsMovement`.
- `leap_while_prone` while standing: `Unsupported`. Leap does not require prone. No distance either way. Standing Leap distance is `add-crawl-and-leap`. There is no `crawl` function. `MoveAction` has no Leap variant.
- Success of `drop_prone` or `stand`: the other posture, the same square. Cost is 1 action with the move trait. No trigger flag. Change 8 treats any `PostureChange` whose cost includes `ActionTrait::Move` as the reactive trigger (FIX-06). `drop_prone` keeps the benefit. `stand` adjusts it per R-11.

`take_cover` does not call `move_trait_allowed`. Cost is 1 action and an empty trait list. `Mobility::Immobilized` does not block it.

### Take Cover

Not prone, before merging with a benefit already stored:

| Caller cover | Feature | Result |
|---|---|---|
| Standard | either | `Greater` |
| Greater | either | `Greater` |
| Lesser | Present | `Standard` |
| Lesser | Absent | `RequirementNotMet` |
| None | Present | `Standard` |
| None | Absent | `RequirementNotMet` |

On `RequirementNotMet` the function returns the error and no replacement. A caller that writes the stored benefit only on `Ok` keeps the previous value. The first failure, with nothing stored, leaves Armor Class at the caller lesser bonus of +1.

Prone, the action always succeeds. Being prone meets the requirement. Build the ordinary benefit with the table above, treating the two `RequirementNotMet` rows as no ordinary bonus. Also build `GreaterVsRanged` (0 melee, +4 ranged). Per attack, keep the higher bonus (DEV-24) and store:

| Melee | Ranged | Stored |
|---|---|---|
| +4 | +4 | `Greater` |
| +2 | +4 | `StandardPlusGreaterVsRanged` |
| 0 | +4 | `GreaterVsRanged` |
| +2 | +2 | `Standard` |

With the prone off-guard −2 included, the six Armor Class nets are:

| Situation | Stored | Melee | Ranged |
|---|---|---|---|
| standard | `Greater` | +2 | +2 |
| greater | `Greater` | +2 | +2 |
| feature, no cover | `StandardPlusGreaterVsRanged` | 0 | +2 |
| lesser only | `GreaterVsRanged`, caller lesser stays +1 | −1 | +2 |
| lesser and a feature | `StandardPlusGreaterVsRanged` | 0 | +2 |
| nothing | `GreaterVsRanged` | −2 | +2 |

A later Take Cover merges the same way with the benefit already stored, so stored `Greater` then caller standard stays `Greater` at +4. Query time uses the same per-attack max, so stored `Standard` plus caller `Greater` is +4 without taking the action again.

Stand does not end Take Cover. It drops only the prone-only part (R-11, extending DEV-24). R-11 is recorded in `docs/designer/rules-rulings.md` by a parallel docs pull request. This change does not edit that file.

| Stored before Stand | Stored after Stand |
|---|---|
| `StandardPlusGreaterVsRanged` | `Standard` (+2 melee and ranged, and the creature is not off-guard from prone) |
| `GreaterVsRanged` with no feature and no cover | `None` |
| `Greater` | `Greater` (+4) |
| `Standard` | `Standard` |

The feature is `NearbyFeature`. There is no wall test. Grid has no walls (DEV-22).

### Allies are a Side

DEV-26, from `docs/designer/rules-deviations.md`. Two creatures are allies for flanking if and only if they share a Side (`Side::{Pc, Enemy}`). No consent or charm model. The flankers must share a `Side`, and the target's `Side` must be the other one.

### Flanking checks

`flanks` looks at the attacker plus every partner. Order:

1. Any repeated `CombatantId` in that set is `DuplicateId`.
2. Any id equal to the defender's id is `SelfId`.
3. `in_reach` from each creature to the defender, attacker first and then the partner slice, with that creature's size and the defender's size. `UnsupportedSize` and `Overflow` from that call are returned as those variants.
4. The center `(2x + 1, 2y + 1)` of the defender and of each of those squares, with checked arithmetic. A value that does not fit is `Overflow`.
5. A reach that is not a multiple of 5 is `InvalidReach`. Reach 0 is a multiple of 5.
6. Otherwise `Ok`. A partner flanks when both it and the attacker are in reach, both are `Able`, the two share a `Side`, the defender's `Side` differs, and the center line flanks. Any such partner makes the result `true`. Partner order does not matter. An empty partner list is `false`.

`in_reach` for a reach other than 5, 10, or greater than 10 returns `Ok(false)`. Step 5 still turns a non-multiple of 5 into `InvalidReach` before that boolean is used. Reach `u32::MAX` is a multiple of 5. Compare it as a `u32`. On `(0, 1)` and `(0, -1)` the result is `Ok(true)`, not a wrapped reach.

This function takes no map and does not call the grid's off-map check. Negative squares are legal. It takes no terrain and no third creature. Those do not change the result (there is no facing).

`off_guard`: if the defender is `Prone`, return `Ok(true)` for melee and ranged without reading partners. Otherwise call `flanks` and return its error unchanged. On `Ok`, melee is that boolean or `other == OffGuard::Yes`. Ranged is `OffGuard::Yes` only. `other` is the caller's grabbed (or any other) cause. The Armor Class function is then called once with `OffGuard::Yes`, so two causes are still one −2. A caller status −4 for unconscious is an extra modifier. It is not a second off-guard. The total is −6, not −8.

Threatening (DEV-25) is step 6's reach test plus `Readiness::Able`. `Readiness::Able` is the ledger's `able_to_act`. The template has one melee strike and no weapon inventory, so the supplied reach stands in for a wielded melee weapon or unarmed attack (Player Core pg. 425, Flanking).

### Center line

Translate so the target square occupies `[0, 2] × [0, 2]` in half-square units. Intersect the closed segment between the two centers with the four edges. Use exact rationals or integer cross-multiplication.

A corner counts as touching both edges that meet there (DEV-23). The line flanks if the edges it touches include an opposite pair (left and right, or bottom and top), or if it passes through two opposite corners `(0, 0)` and `(2, 2)`, or `(0, 2)` and `(2, 0)`. Every other hit is not a flank, including a miss, one point, one corner, and two adjacent side interiors.

Squares in the spec, target at `(0, 0)`:

| Flankers | Result | Reach |
|---|---|---|
| `(0, 1)` and `(0, -1)`; `(1, 0)` and `(-1, 0)` | flank | 5 |
| `(1, 1)` and `(-1, -1)` | flank | 5 |
| `(-1, 1)` and `(0, 1)` | not a flank | 5 |
| `(0, 1)` and `(1, 0)` | not a flank | 5 |
| `(-2, -1)` and `(1, 0)` | flank | 10 and 5 |
| `(-2, -2)` and `(0, 1)` | not a flank | 10 and 5 |
| `(0, 2)` and `(0, -1)` | flank | 10 and 5 |
| `(2, 2)` and `(-2, -2)` | flank | 10, and also 15 |
| `(-2, -1)` and `(2, 1)` | flank | 10 |
| `(3, 3)` and `(-3, -3)` | not in reach | 15 |

`(0, -2)` at reach 5 is 10 feet, so the pair with `(0, 1)` is `Ok(false)` at step 6.

The extreme pair is exactly two flankers: target `(i32::MAX - 1, 0)`, flankers `(i32::MAX, 0)` and `(i32::MAX - 2, 0)`, reach 5. The centers do not fit. The result is `Overflow`, never a wrapped coordinate. A Large target on those squares is `UnsupportedSize` at step 3, before `Overflow`.

### Rulings already applied

Designer rulings 1 through 9 are in the spec. Nothing in this change is waiting on a further Designer answer.

| Topic | Ruling |
|---|---|
| Lesser cover alone, not prone | `RequirementNotMet`. Nothing stored. Actions kept. Armor Class stays +1. Lesser plus a feature is standard. |
| Corner | DEV-23, as above. `(-2, -1)` and `(1, 0)` flank. |
| Prone and other cover | DEV-24, the six nets above. |
| Threaten | DEV-25. Reach contains the target, and `Readiness::Able` is the ledger's `able_to_act`. |
| Allies | DEV-26, quoted above. Same `Side::{Pc, Enemy}`. |
| Square | DEV-27. No sharing. Stand ignores an occupant. |
| Crawl and Leap | Distances are `add-crawl-and-leap`. Leap while prone is `ProneRestrictsMovement`. |
| Hidden | FIX-14, change 10 only. |
| Greater cover already stored | Stays greater through another Take Cover. Stand keeps `Greater` and drops only the prone-only part (R-11). Take Cover has no move trait. |
| Cover vs flanking | Cover, terrain, and creatures on the line are not flanking inputs. |

## Risks / Trade-offs

- [Implementation needs `Side` from `add-encounter-turns`] → this module does not declare a stand-in enum. The implementation pull request starts once `Side` is in the crate.
- [Reach stands in for a wielded melee weapon] → a creature with only a ranged attack is not in this crate. The template's strike is melee.
- [Prone off-guard skips partner errors] → `flanks` still reports those errors. `off_guard` returns true first so a bad partner list cannot hide prone.
- [Take Cover can outlive prone] → Stand keeps `Greater` and turns `StandardPlusGreaterVsRanged` into `Standard`. `GreaterVsRanged` alone becomes `None`. Prone off-guard ends because the posture changed.

## Migration Plan

No stored data and no change to an existing public function. The spec-only pull request does not add a changelog bullet and does not archive. The implementation pull request adds one Unreleased bullet, then `openspec archive add-positioning --yes` as its last commit. Rollback of the implementation is reverting the new module and its re-export.
