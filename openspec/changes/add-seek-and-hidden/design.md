# Design

## Context

`degree_of_success`, `resolve_check`, `flat_check`, `flat_checks_for_same_thing`, `fallback_unlisted_critical`, `CheckModifier`, and `DcModifier` already exist. `ac_modifiers(cover, off_guard, unconscious)` and `total` already turn one off-guard bool into a circumstance −2. `Cover` is `Lesser` (+1), `Standard` (+2), or `Greater` (+4). `Creature.stealth_dc` is optional. Merged `add-positioning` (83eafa5) names `OffGuard::{Yes, No}`, `MoveAction::{Stride, Step, DropProne}`, and `ActionTrait::{Move}` only. `ActionCost` is `Actions { count: u8, traits }` or `Free`. Merged `add-encounter-turns` names `Side::{Pc, Enemy}`, `EncounterError::NotConscious`, `EncounterError::PerceptionOutOfRange` (`checked_add(20)`, not `CheckModifier::new`), and `Encounter::spend_actions(&mut self, id: CombatantId, n: u8)`. Merged `add-dying-and-victory` names `Vital::{Alive, Unconscious, Dying(DyingValue), Dead}` with dying values 1, 2, or 3. The crate on this branch still has the older `Vital` until that implementation lands. This spec uses the merged names. See proposal.md for why this family is one change. The behavior to implement is `specs/seek-and-hidden/spec.md`.

This change names no file on `v1-reference`, so none is ported. `crates/rules` stays dependency-free. Fallible functions return `Result`. No `unwrap`, `expect`, `panic!`, or unchecked indexing. No file under `docs/designer/` is edited. No new ruling. The log is `docs/designer/rules-rulings.md`.

## Goals / Non-Goals

**Goals:**

- One new module, `crates/rules/src/detection.rs`, re-exported from `lib.rs`.
- A caller-owned `Detections` map. Functions take it by value and return a new one, except the read-only targeting check.
- Seek, Hide, the targeting flat check, and off-guard all read that relation.
- `OffGuard` and `MoveAction` are imported. `Yes` is the `true` passed once to `ac_modifiers`. `Vital` is imported. This module declares none of them.

**Non-Goals:**

- Editing `check.rs`, `modifiers.rs`, `combatant.rs`, `positioning.rs`, or `encounter.rs`. No field on `Creature` or `CombatantState`. No second `OffGuard`, no second `Cover`, no second `ActionCost`, no second `Side`, no second `Vital`.
- Calling `spend_actions` or `Encounter::strike`, or changing Hit Points. The caller spends Seek and Hide. `off_guard_against` does not spend.
- `wire-detection-into-strike`: detection does not feed `Encounter::strike`. Off-guard from hidden, the DC 11 flat check, and MAP on a failed flat check stay at this API until that change.
- Sneak, Point Out, invisibility, senses, light, unnoticed, Avoid Notice, Create a Diversion, Concealing an Object, hazards, and secret doors. Cone and burst areas. A square guess for an undetected creature (proposed DEV-41).
- Unconscious observers. The caller passes Perception with every modifier already included, including the unconscious −4. This module does not read an observer's `Vital` and does not apply that penalty.
- Prone observers. This module does not change an observer's Perception for prone.

## Decisions

### Subtract before adding

Do not add a roller, a DC formula, a cover bonus, or an off-guard penalty. Call what is already there.

| Need | Call | Do not add |
|---|---|---|
| Seek degree | `degree_of_success` after the initiative gate | `CheckModifier` for Seek's Perception |
| Hide's Stealth check | `resolve_check` | a second degree function |
| DC 10 + modifier | `DcModifier` and `dc_from_modifier` | a second formula |
| Flat check | `flat_check` or `flat_checks_for_same_thing` | a second flat-check rule |
| Off-guard −2 | `OffGuard::Yes` once, then `ac_modifiers(None, true, false)` and `total` | a new penalty or a new enum |
| Cover +2 / +4 | `Cover::Standard` and `Cover::Greater` | a Hide-only cover type |
| Stealth DC | the caller's `stealth_dc` | a skill or a new creature field |

Seek does not call `resolve_check`. `CheckModifier::new` rejects a modifier that cannot be added to 255, which refuses Perception 2147483500. Initiative accepts that modifier: the gate is `checked_add(20)`. After that gate, every face in 1..=20 fits in `i32`. The total is `checked_add` of the modifier and the face, then `degree_of_success`.

Alternative considered: a four-value enum whose `Concealed` variant replaces `Observed`. Rejected. https://2e.aonprd.com/Rules.aspx?ID=2414 (Player Core pg. 434) says concealment does not change the detection category, and an observed creature can still be concealed. `Concealment` sits beside `Detection`.

Alternative considered: a bool for concealment. Rejected. Constitution §6 prefers an enum where a bool would hide the two states.

Alternative considered: folding the flat check into `strike`. Rejected. That is `wire-detection-into-strike`. The caller reads this result.

### Relation

```rust
pub enum Detection {
    Observed,
    Hidden,
    Undetected,
}

pub enum Concealment {
    Concealed,
    NotConcealed,
}

pub struct Relation {
    pub detection: Detection,
    pub concealment: Concealment,
}

/// Caller-owned. Keyed by ordered (observer, subject) CombatantId.
pub struct Detections {
    // private map
}

impl Detections {
    pub fn new() -> Self;
    /// Missing pair: Observed and NotConcealed.
    pub fn get(&self, observer: CombatantId, subject: CombatantId) -> Relation;
    pub fn store(
        &self,
        observer: CombatantId,
        subject: CombatantId,
        relation: Relation,
        stealth_dc: Option<i32>,
    ) -> Result<Self, DetectionError>;
}
```

`Relation`, `Detection`, `Concealment`, `DetectionCost`, `Area`, `OpenRoll`, `HideObserver`, and `TargetingResult` derive `Debug, Clone, Copy, PartialEq, Eq`. `Detections` and the result structs that own it derive `Debug, Clone, PartialEq, Eq` and not `Copy`. `get` does not consult `Vital`. `store` does not take `Vital`: a `Dead` observer or subject is `Ok` when the other checks pass. `store` rejects `SameCreature` when the ids match, and `NoStealthDc` when the detection is `Hidden` or `Undetected` and `stealth_dc` is `None`. Observed, either concealment, needs no Stealth DC. The `Creature` is not mutated.

`Side::{Pc, Enemy}` is not a field and not a parameter. A `Pc` may observe an `Enemy` and an `Enemy` may observe a `Pc`. This module does not declare `Side`.

### Errors

```rust
pub enum DetectionError {
    NotConscious,
    SameCreature,
    NoStealthDc,
    CannotTarget,
    EmptyArea,
    AreaTooLarge,
    DuplicateId,
    Overflow,
    PerceptionOutOfRange,
    RequirementNotMet,
}
```

The names `NotConscious` and `PerceptionOutOfRange` match encounter-turns. The values are `DetectionError`, not `EncounterError`, so this module does not depend on the encounter type. `PerceptionOutOfRange` is the failed `checked_add(20)` gate. `i32::MIN` passes that gate. Perception 2147483627 passes. Perception 2147483628 fails and draws no die.

### Off-guard

```rust
pub fn off_guard_against(relation: Relation, other: OffGuard) -> OffGuard
```

`Hidden` or `Undetected` returns `OffGuard::Yes`. Otherwise the function returns `other`. Concealment does not. The Armor Class scenarios call `ac_modifiers(None, result == OffGuard::Yes, false)` and `total`. The unconscious argument stays `false`. A second `Yes` is still one `Yes`, so the total is −2, not −4. This function does not spend an action.

`OffGuard` is the positioning type. This module does not declare it. Implementation starts once `OffGuard`, `MoveAction`, and the merged `Vital` are public.

### Targeting flat check

```rust
pub struct TargetingResult {
    pub affected: bool,
    pub dc: Option<i32>,
    pub degree: Option<DegreeOfSuccess>,
    pub action_spent: bool,
    pub counts_for_map: bool,
}

pub fn resolve_targeting(
    detections: &Detections,
    observer: CombatantId,
    subject: CombatantId,
    area_effect: bool,
    dice: &mut impl Dice,
) -> Result<TargetingResult, DetectionError>;
```

Observed and `NotConcealed`, or any area effect: `affected`, no die, `dc` and `degree` are `None`, both spend flags false. Undetected and not an area effect: `CannotTarget`, no die, relation unchanged (proposed DEV-41). Otherwise one roll. A hidden failure (`affected` false, DC 11) sets `action_spent` and `counts_for_map`. Every other `Ok` sets both flags false. This function does not call `spend_actions` or `Encounter::strike`.

| Relation | Call |
|---|---|
| observed, Concealed | `flat_check(dice, 5)` |
| hidden, NotConcealed | `flat_check(dice, 11)` |
| hidden, Concealed | `flat_checks_for_same_thing(dice, 5, &[11])` |

The effect treats a critical success as a success and a critical failure as a failure (`fallback_unlisted_critical` with neither critical entry listed). The numeric degree is still returned. Flat checks keep the natural 20 and natural 1 step from `dice-and-checks`.

### Action cost

```rust
pub enum DetectionCost {
    /// 1 action, concentrate. The caller spends it.
    Seek,
    /// 1 action. None of concentrate, move, or manipulate.
    Hide,
}
```

This is not positioning's `ActionCost` and not a `DetectionTrait`. Positioning's `ActionTrait` is only `Move`. Seek needs concentrate. A MODIFIED delta cannot add that trait: `positioning` is not under `openspec/specs/`, so no requirement header on main matches byte for byte. `DetectionCost` is the local name so `lib.rs` does not gain a second `ActionCost`. The caller maps either variant to `spend_actions(id, 1)`. Scenarios that do not name a vital use `Vital::Alive`.

### Seek

```rust
pub struct Area {
    pub length_feet: u32,
    pub width_feet: u32,
    pub height_feet: u32,
}

pub struct OpenRoll {
    pub natural: u8,
    pub total: i32,
    pub degree: DegreeOfSuccess,
}

pub struct SeekResult {
    pub cost: DetectionCost,
    pub roll: Option<OpenRoll>,
    pub detections: Detections,
}

pub fn seek(
    detections: Detections,
    seeker: CombatantId,
    vital: Vital,
    perception_modifier: i32,
    area: Area,
    subjects: &[CombatantId],
    dice: &mut impl Dice,
) -> Result<SeekResult, DetectionError>;
```

`DetectionCost::Seek` means 1 action with the concentrate trait. The area is three dimensions in feet. The caller decides which subjects are inside. This change does not test squares against the area. DEV-08 (Seek, https://2e.aonprd.com/Actions.aspx?ID=2301, Player Core pg. 417): the roll is open, and every dimension above 30 feet is rejected. A cone or a burst is a non-goal. An area of 0 by 0 by 0 feet is `EmptyArea`.

Before any die, in this order: `NotConscious` when `vital` is not `Alive`; `DuplicateId` when a subject id repeats; `SameCreature` when the seeker id is a subject; `EmptyArea`; `AreaTooLarge`; `PerceptionOutOfRange` when `perception_modifier.checked_add(20)` is `None`. A `Dying(1)` seeker in a 35-foot area is `NotConscious`, not `AreaTooLarge`. A duplicate in that area is `DuplicateId`. The seeker listed twice is `DuplicateId`, not `SameCreature`. The seeker listed once in that area is `SameCreature`.

If no subject is hidden or undetected with a Stealth DC, the call succeeds, draws no die, and returns `roll: None`. Otherwise one face is drawn. The total is the Perception modifier plus that face. Seek lists a critical success and a success, and no critical failure. `fallback_unlisted_critical(degree, true, false)` is the effect:

- critical success: hidden or undetected becomes observed
- success: undetected becomes hidden, hidden becomes observed
- failure, including a critical failure read as failure: unchanged

Concealment is copied. Invisibility and an imprecise sense do not change the effect (proposed DEV-42). `OpenRoll` exposes `natural`, `total`, and `degree` (DEV-08, open roll). Perception 2147483500 and a natural 12 total 2147483512 against Stealth DC 17, a critical success.

### Hide

```rust
pub struct HideObserver {
    pub id: CombatantId,
    pub perception_modifier: i32,
    pub cover: Option<Cover>,
}

pub struct HideResult {
    pub cost: DetectionCost,
    pub roll: Option<OpenRoll>,
    pub detections: Detections,
}

pub fn hide(
    detections: Detections,
    subject: CombatantId,
    vital: Vital,
    stealth_dc: Option<i32>,
    observers: &[HideObserver],
    dice: &mut impl Dice,
) -> Result<HideResult, DetectionError>;
```

`DetectionCost::Hide` means 1 action and none of concentrate, move, or manipulate.

The Stealth modifier is `stealth_dc - 10`, then +2 for `Cover::Standard`, +4 for `Cover::Greater`, or +0 when the only qualifier is `Concealed`. `checked_sub` and `checked_add`, then `CheckModifier::new`. `DcModifier::new` rejects a Perception modifier for which `10 + modifier` does not fit. Perception 2147483638 is that failure. `stealth_dc` of `i32::MIN` fails `checked_sub(10)`. `i32::MAX` with greater cover is `(i32::MAX - 10) + 4`, which `CheckModifier::new` rejects. Either failure is `Overflow` and draws no die. This is not `PerceptionOutOfRange`.

Before any die: `NotConscious`, then `DuplicateId`, then `SameCreature` (an observer id equal to the subject), then `NoStealthDc`, then `RequirementNotMet` when no remaining observer has standard cover, greater cover, or `Concealed`, then `Overflow`. Lesser cover does not qualify. An `Unconscious` hider with no Stealth DC is `NotConscious`, not `NoStealthDc`. `NoStealthDc` wins over a cover failure. A duplicate that is also the subject returns `DuplicateId`.

An observer whose current detection of the subject is `Hidden` or `Undetected` is skipped and is not compared. When every observer is skipped, the call succeeds and draws no die. An observer who does not qualify is left unchanged when another does. One `Concealed` observer is enough; the other observer stays as they were and the error is not `RequirementNotMet`.

One `resolve_check` is compared with each qualifying Perception DC, in ascending id. Hide lists a success and no critical success or critical failure. Both critical degrees use the unlisted fallback. Success hides an observed subject. Failure leaves the relation. Concealment is copied. The degree is on the roll (proposed DEV-40, open roll, amending DEV-10's "no Hide action").

```rust
pub fn refresh(
    detections: Detections,
    subject: CombatantId,
    observer: CombatantId,
    cover: Option<Cover>,
) -> Detections;
```

`refresh` is never implicit. A hidden subject with neither standard nor greater cover, and with `NotConcealed`, becomes observed. Lesser cover does not keep hidden. Undetected is unchanged. Concealment is unchanged.

```rust
pub struct StrikeNotice {
    pub off_guard: OffGuard,
    pub detections: Detections,
}

pub fn note_strike(
    detections: Detections,
    striker: CombatantId,
    target: CombatantId,
) -> StrikeNotice;

pub fn note_action(
    detections: Detections,
    actor: CombatantId,
    action: MoveAction,
) -> Detections;
```

`note_strike` reads `off_guard_against` for the target's relation to the striker first, then returns that `OffGuard`. Every stored observer who has the striker `Hidden` or `Undetected` then becomes `Observed`. Concealment on those pairs stays. A pair whose subject is not the striker stays, including the other direction. Designer, round 1: a hidden Strike reveals the striker to every observer (Hide, Actions 2404, Player Core pg. 244), and an undetected Strike becomes observed after the target is off-guard (Sneak, https://2e.aonprd.com/Actions.aspx?ID=2405, Player Core pg. 245). `note_action` for `MoveAction::Stride`, `Step`, or `DropProne` returns the same relations and does not call `refresh` (proposed DEV-43). Neither function rolls.

### Proposed deviations

These rows are proposed. They are not written to `docs/designer/rules-deviations.md`. DEV-38 and DEV-39 belong to shields.

| ID | Rule | RAW citation | Our behavior | Reason |
|---|---|---|---|---|
| DEV-40 | Stealth check for Hide. Amends the "no Hide action" sentence of DEV-10. | https://2e.aonprd.com/Actions.aspx?ID=2404 (PC 244); https://2e.aonprd.com/Rules.aspx?ID=2285 (PC 401) | Hide is 1 action. The modifier is the baked Stealth DC minus 10. The roll is open. No Stealth skill. Absent Stealth DC: cannot Hide, and cannot be hidden or undetected. | No skills in Phase 1. |
| DEV-41 | Targeting an undetected creature | https://2e.aonprd.com/Conditions.aspx?ID=96 (PC 447) | `CannotTarget`. No square and no flat check. Area effects still apply with no roll. The observer is still off-guard. | The guess is hidden information. |
| DEV-42 | Senses, light, and invisibility | https://2e.aonprd.com/Actions.aspx?ID=2301 (PC 417); https://2e.aonprd.com/Rules.aspx?ID=2414 (PC 434) | Every Seek is a precise sense. A listed degree becomes observed or hidden as written. Concealment is a caller value, not dim light. Invisibility is not a condition. | Those subsystems are deferred. |
| DEV-43 | Ending hidden | https://2e.aonprd.com/Actions.aspx?ID=2404 (PC 244); https://2e.aonprd.com/Actions.aspx?ID=2405 (PC 245) | A Strike while hidden or undetected reads `OffGuard::Yes`, then every observer's detection of the striker becomes observed. Stride, Step, and every other action do not clear detection and do not call `refresh`. Sneak is not an action. | Sneak is deferred. The off-guard Strike stays, and the striker is observed by every observer. |

DEV-08 is unchanged: Seek is an open roll, and every dimension above 30 feet is rejected. This change adds no distance penalty.

## Risks / Trade-offs

- [Positioning has not landed `OffGuard` or `MoveAction` in the crate yet, and `Vital` in the crate is still the pre-dying enum] → The implementation tasks do not start until those types are public. This spec does not declare a stand-in.
- [`CheckModifier` rejects a Perception modifier initiative accepts] → Seek uses `checked_add(20)` and `degree_of_success`. Hide's Stealth modifier still uses `CheckModifier`.
- [A hidden creature can Stride and stay hidden until the caller refreshes] → Proposed DEV-43. The Strike exception is specified. Sneak is the follow-up action, not a substitute that keeps the striker hidden.
- [Undetected creatures cannot be attacked by guessing a square] → Proposed DEV-41. Seek is the way onto the hidden DC 11 check.
- [Detection does not change `Encounter::strike`] → `wire-detection-into-strike` applies off-guard, the DC 11 flat check, and MAP on a failed flat check.
- [The same Perception total can be a success against one Stealth DC and a failure against another] → One roll, compared per subject. The scenario with total 18 against DC 17 and DC 27 locks that.

## Migration Plan

No saved encounter and no change to an existing public function. Callers who do not use the new module keep their current results.
