# Design

## Context

`modifiers` already builds the prone attack penalty, cover, and one off-guard penalty, and `total` folds them. `grid` already answers `in_reach` and `move_trait_allowed` for Small and Medium squares. `CombatantState.conditions` is an empty slot. See proposal.md for why this family is one change. This change names no file on `v1-reference`, so none is ported. `crates/rules` stays dependency-free. Fallible functions return `Result`. No `unwrap`, `expect`, `panic!`, or unchecked indexing.

## Goals / Non-Goals

**Goals:**

- One new module, `crates/rules/src/positioning.rs`, re-exported from `lib.rs`.
- Prone, flanking, and Take Cover choose inputs. `attack_modifiers`, `ac_modifiers`, and `total` compute the numbers.
- Flanking geometry is an exact center-line test on 1×1 squares. Reach is `in_reach`.

**Non-Goals:**

- Editing `modifiers.rs`, `grid.rs`, `combatant.rs`, `check.rs`, or `dice.rs`. No new field on `Creature` or `CombatantState`.
- Crawl, Leap, Stride, Step, and square sharing. This module does not move a creature.
- Reactive Strike, shields, Seek, hidden, undetected, concealed, and unconscious's −4.
- Spending actions or reading a turn. Costs are returned data.

## Decisions

### Call the modifiers that already exist

`attack_penalty(prone)` calls `attack_modifiers(0, prone, 0)` and returns that list. The zeros omit the multiple attack penalty and Aid. `armor_class(cover, off_guard)` calls `ac_modifiers(cover, off_guard, false)`. The `false` omits unconscious. `total` folds the list. A second −2 built in this module is rejected: two off-guard penalties, or a cover bonus added on top of the degree it replaced, would fight the existing sources.

Alternative considered: pass prone and flanking as two caller modifiers with the off-guard condition. Rejected. `ac_modifiers` already emits one off-guard penalty. A second construction can double it.

Off-guard for an attack is true when the defender is prone, or when the attack is melee and a pair including the attacker flanks. Prone is decided first, so an unruled line does not block prone off-guard. If the defender is not prone, a true pair wins over an unruled pair. An unruled pair with no true pair returns `NotYetRuled`.

Cover passed into `ac_modifiers` is one `Option<Cover>`:

- No Take Cover benefit: the caller-supplied degree, including `None`.
- Stored greater: `Cover::Greater` for melee and ranged.
- Stored standard: `Cover::Standard` for melee and ranged.
- Stored greater-against-ranged: `Cover::Greater` for ranged and `None` for melee.

The benefit replaces the caller-supplied degree. It is not a second bonus.

### Posture and the benefit are returned values

```rust
pub enum Posture { Standing, Prone }

pub enum CoverBenefit { Standard, Greater, GreaterVsRanged }

pub enum AttackKind { Melee, Ranged }

pub enum EndReason { LeftSquare, AttackAction, Unconscious, Dismiss }

pub enum ActionCost { Actions { count: u8, traits: &'static [ActionTrait] }, Free }

pub enum ActionTrait { Move }
```

`DropProne`, `Stand`, and `TakeCover` return the new posture, the benefit, and the cost. They do not take `&mut CombatantState`. Tests build a state, do not pass it, and assert actions and the reaction are unchanged. Same pattern as Stride.

Alternative considered: store prone in `CombatantState.conditions`. Rejected. That amends `creature-template`, and the slot is empty on purpose.

### Action checks

`drop_prone` and `stand` call `move_trait_allowed` first.

- Immobilized: `PositioningError::Immobilized`, inputs unchanged.
- `drop_prone` while already prone: `ProneRestrictsMovement`. The prone condition allows only Crawl and Stand as move actions, and Crawl is not this change.
- `stand` while standing: `NotProne`.
- Success: the other posture, same square, benefit unchanged. Cost is 1 action with the move trait.

`prone_move(Stride | Step | DropProne)` while prone returns `ProneRestrictsMovement` and does not call `stride` or `step`. There is no `crawl` or `leap` function.

`take_cover` does not call `move_trait_allowed`. Cost is 1 action and no traits. `dismiss` is `ActionCost::Free` and clears the benefit only.

End reasons `LeftSquare`, `AttackAction`, and `Unconscious` clear the benefit and leave posture alone. The caller still passes underlying cover on the next query.

### Take Cover table

Not prone:

| Caller cover | Nearby feature | Result |
|---|---|---|
| Standard | either | store `Greater` |
| Greater | either | store `Greater` |
| Lesser | either | store `Standard` |
| None | true | store `Standard` |
| None | false | `RequirementNotMet` |

Prone:

| Caller cover | Nearby feature | Result |
|---|---|---|
| None | false | store `GreaterVsRanged` |
| any other | any other | `NotYetRuled`, posture and benefit unchanged |

The nearby feature is a `bool`. There is no wall test and no line-of-effect test. That matches grid, which has no walls (DEV-22).

### Flanking inputs

```rust
pub struct CombatantPos {
    pub id: CombatantId,
    pub square: Square,
    pub size: Size,
    pub reach_feet: u32,
    pub faction: FactionId,
    pub able_to_act: bool,
    pub posture: Posture,
}
```

`flanks(target, a, b)` checks, in order:

1. `a.id == b.id` is `DuplicateId`. Either id equal to `target.id` is `SelfId`.
2. `in_reach` for each flanker. `UnsupportedSize` and `Overflow` propagate. Do not turn them into `false`.
3. If either reach is false, either `able_to_act` is false, the flankers' factions differ, or their faction equals the target's: `Ok(false)`.
4. Otherwise the center line: `Ok(true)`, `Ok(false)`, or `NotYetRuled`.

`able_to_act` is stored as passed. Prone does not write it. This module does not read an unconscious flag.

Threatening is `in_reach` of the caller-supplied reach. Phase 1's strike is one melee strike, so there is no second wielded-weapon flag.

**Rule**: Player Core pg. 425 also requires a melee weapon or an unarmed attack. https://2e.aonprd.com/Rules.aspx?ID=2375
**Why**: The template has one melee strike and a reach, and no weapon inventory.
**Substitute**: In reach with that reach counts as threatening.
**Intent**: A later weapon change can pass a different reach without a second flanking rule.

Ally is faction, same shape as DEV-17. This change does not edit `rules-deviations.md`. Designer can ledger an id when they accept the substitute. The spec already records it.

### Center line

Translate so the target square is `[0, 2] × [0, 2]` in half-square units. The center of square `(x, y)` is `(2x + 1, 2y + 1)` before that translation. Intersect the closed segment between the two centers with the four edges. Use exact rationals or integer cross-multiplication. Checked arithmetic only. Deduplicate points that are the same corner hit by two edges.

Classify the distinct points:

1. The two opposite corners `(0, 0)` and `(2, 2)`, or `(0, 2)` and `(2, 0)`: geometry holds. Decide this before any side assignment.
2. Two points that each lie on one side only, on opposite sides (left/right or top/bottom): geometry holds.
3. Two points that each lie on one side only, on adjacent sides: geometry does not hold.
4. No point, or one point: geometry does not hold.
5. Anything else, including a corner paired with a side that is not the opposite corner: `NotYetRuled`.

Squares in the spec, target at `(0, 0)`:

| Flankers | Class | Reach used in the scenario |
|---|---|---|
| `(0, 1)` and `(0, -1)`; `(1, 0)` and `(-1, 0)` | opposite side interiors | 5 |
| `(1, 1)` and `(-1, -1)`; `(-1, 1)` and `(1, -1)` | opposite corners | 5 |
| `(-1, 1)` and `(0, 1)` | miss | 5 |
| `(0, 1)` and `(1, 0)` | one corner | 5 |
| `(0, 2)` and `(0, -1)` | opposite side interiors | 10 and 5 |
| `(2, 2)` and `(-2, -2)` | opposite corners | 10 |
| `(-2, -1)` and `(2, 1)` | opposite side interiors | 10 |
| `(-2, -2)` and `(0, 1)` | adjacent side interiors | 10 |
| `(-2, -1)` and `(1, 0)` | corner plus a side | 10 and 5, `NotYetRuled` |

`(0, -2)` at reach 5 is not in reach (10 feet). The pair with `(0, 1)` is `Ok(false)` at step 3, before geometry.

Alternative considered: only the four adjacent opposite pairs. Rejected. Reach 10 has opposite side interiors that are not those four pairs, and the book states the center line.

### Errors

```rust
pub enum PositioningError {
    Immobilized,
    NotProne,
    ProneRestrictsMovement,
    RequirementNotMet,
    NotYetRuled,
    DuplicateId,
    SelfId,
    UnsupportedSize,
    Overflow,
}
```

`Immobilized`, `UnsupportedSize`, and `Overflow` are the grid errors, mapped by name. This module adds no dependency and does not panic on them.

## Questions for Designer

Q1 and Q2 are the not-yet-ruled errors. The tasks implement those scenarios as errors. A ruling replaces the scenario; it does not add a second guess in code. Q3–Q6 are the readings the scenarios already test. Overturning one means editing that scenario before implementation is dispatched.

| Q | Question | Where it sits now |
|---|---|---|
| Q1 | A center line hits one corner and the interior of another side, and the two points are not opposite corners. The corner belongs to both edges. Assigning it to the opposite side makes the hit "opposite sides"; assigning it only to the adjacent side does not. Example: `(-2, -1)` and `(1, 0)`. Which assignment is the rule? | Scenario "A corner and a side are not yet ruled" returns `NotYetRuled`. Off-guard uses the same error unless a different pair already flanks or the target is prone. |
| Q2 | Take Cover while prone when the caller also supplies lesser, standard, or greater cover, or a nearby feature. The action upgrades standard to greater with no ranged limit, and the prone special grants greater cover against ranged attacks even with no cover. The book does not say how those combine. | Scenarios "Prone and standard cover are not ruled" and "Prone and a nearby feature are not ruled" return `NotYetRuled`. |
| Q3 | Not prone, already greater cover. "Otherwise, you gain standard cover" can be read as a downgrade. | Scenario "Greater cover is not reduced" keeps greater at +4. Confirm, or replace that scenario. |
| Q4 | Stand does not leave the square and is not an attack, so it is not an end condition. After Stand, the stored greater-against-ranged benefit remains, and prone off-guard ends. | Scenario "Stand keeps the cover benefit". Confirm, or rule that standing ends the prone special. |
| Q5 | The citation for Take Cover records the cost and the cover ladder, not a trait list. This change uses 1 action and no move trait, so immobilized does not block it. | Scenario "Take Cover is one action without the move trait". Confirm the Remaster action has no move trait. https://2e.aonprd.com/Actions.aspx?ID=2307 (Player Core pg. 418) |
| Q6 | Flanking's text does not mention cover, and cover's text does not mention flanking. Cover is not a flanking input. Separately, caller-supplied standard or greater cover is an Armor Class bonus with no melee filter. Only the prone special is ranged-only. There are no walls to decide whether cover lies between a melee attacker and the target. | No scenario takes cover as a flanking input. Confirm both readings, or name the scenario to add. |

## Risks / Trade-offs

- [Reach 10 pairs that clip a corner error until Q1] → callers handle `NotYetRuled`. A clear pair still flanks when another ally's line is unruled.
- [Faction stands in for ally] → a same-faction enemy cannot be represented. Same limit as DEV-17.
- [Reach stands in for a wielded melee weapon] → a creature with only a ranged attack is not in this crate. The template's strike is melee.
- [Take Cover can outlive prone] → Q4. The benefit and the posture are separate values so Stand does not silently rewrite cover.
- [No shield and no hidden parameter] → a caller who adds those bonuses later must not expect this module to emit them. The Armor Class list from this module is only cover and off-guard.

## Migration Plan

No stored data and no change to an existing public function. Rollback is reverting the new module and its re-export.
