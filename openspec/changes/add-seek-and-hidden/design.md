# Design

## Context

`resolve_check`, `flat_check`, `flat_checks_for_same_thing`, `fallback_unlisted_critical`, `CheckModifier`, and `DcModifier` already exist. `ac_modifiers(cover, off_guard, unconscious)` and `total` already turn one off-guard bool into a circumstance −2. `Cover` is `Lesser` (+1), `Standard` (+2), or `Greater` (+4). `Creature.stealth_dc` is optional. `add-positioning` names `OffGuard::{Yes, No}` and does not take a hidden input. See proposal.md for why this family is one change. The behavior to implement is `specs/seek-and-hidden/spec.md`.

This change names no file on `v1-reference`, so none is ported. `crates/rules` stays dependency-free. Fallible functions return `Result`. No `unwrap`, `expect`, `panic!`, or unchecked indexing. No file under `docs/designer/` is edited. No new ruling. The log is `docs/designer/rules-rulings.md`.

## Goals / Non-Goals

**Goals:**

- One new module, `crates/rules/src/detection.rs`, re-exported from `lib.rs`.
- A caller-owned relation. Functions take it by value and return a new one.
- Seek, Hide, the targeting flat check, and off-guard all read that relation.
- `OffGuard` is imported. `Yes` is the `true` passed once to `ac_modifiers`.

**Non-Goals:**

- Editing `check.rs`, `modifiers.rs`, `combatant.rs`, `positioning.rs`, or `encounter.rs`. No field on `Creature` or `CombatantState`. No second `OffGuard` and no second `Cover`.
- Spending actions, calling `strike`, or changing Hit Points. The caller spends, including when a flat check fails.
- Sneak, Point Out, invisibility, senses, light, unnoticed, Avoid Notice, Create a Diversion, Concealing an Object, hazards, and secret doors.
- A square guess for an undetected creature (proposed DEV-39).

## Decisions

### Subtract before adding

Do not add a roller, a DC formula, a cover bonus, or an off-guard penalty. Call what is already there.

| Need | Call | Do not add |
|---|---|---|
| Perception or Stealth check | `resolve_check` | a second degree function |
| DC 10 + modifier | `DcModifier` and `dc_from_modifier` | a second formula |
| Flat check | `flat_check` or `flat_checks_for_same_thing` | a second flat-check rule |
| Off-guard −2 | `OffGuard::Yes` once, then `ac_modifiers(None, true, false)` and `total` | a new penalty or a new enum |
| Cover +2 / +4 | `Cover::Standard` and `Cover::Greater` | a Hide-only cover type |
| Stealth DC | the caller's `stealth_dc` | a skill or a new creature field |

Alternative considered: a four-value enum whose `Concealed` variant replaces `Observed`. Rejected. Player Core pg. 444 says the concealed condition does not change the detection category, and an observed creature can still be concealed. The flag sits beside `Detection`.

Alternative considered: folding the flat check into `strike`. Rejected. That rewrites an `encounter-turns` requirement. The caller reads this result and spends the action either way.

### Relation

```rust
pub enum Detection { Observed, Hidden, Undetected }

pub struct Relation {
    pub detection: Detection,
    pub concealed: bool,
}
```

Every type below derives `Debug, Clone, Copy, PartialEq, Eq` except where noted. A missing pair is `Observed` and `concealed: false`. `store` rejects `SameCreature` when the ids match, and `NoStealthDc` when the detection is `Hidden` or `Undetected` and `stealth_dc` is `None`. Observed, concealed or not, needs no Stealth DC. The `Creature` is not mutated.

### Off-guard

```rust
pub fn off_guard_against(relation: Relation, other: OffGuard) -> OffGuard
```

`Hidden` or `Undetected` returns `OffGuard::Yes`. Otherwise the function returns `other`. Concealed does not. The Armor Class scenarios call `ac_modifiers(None, result == OffGuard::Yes, false)` and `total`. The unconscious argument stays `false`. A second `Yes` is still one `Yes`, so the total is −2, not −4.

`OffGuard` is the positioning type. This module does not declare it. Implementation starts once that type is public. Alternative considered: return a bool and let the caller remember the name. Rejected. The spec's result is `OffGuard`.

### Targeting flat check

Observed and not concealed, or any area effect: success, no die, target affected. Undetected and not an area effect: `CannotTarget`, no die, relation unchanged (proposed DEV-39). Otherwise one roll:

| Relation | Call |
|---|---|
| observed, concealed | `flat_check(dice, 5)` |
| hidden, not concealed | `flat_check(dice, 11)` |
| hidden, concealed | `flat_checks_for_same_thing(dice, 5, &[11])` |

The effect treats a critical success as a success and a critical failure as a failure (`fallback_unlisted_critical` with neither critical entry listed). The numeric degree is still returned. This function does not spend an action.

### Seek

Seek returns 1 action and `DetectionTrait::Concentrate`. This module's `ActionCost` is not positioning's. Positioning's trait list has only `Move`. Adding `Concentrate` there would edit that spec. Rejected.

The area is three `u32` dimensions in feet. Any value above 30 is `AreaTooLarge` (DEV-08). The caller decides which subjects are inside. This change does not test squares against the area.

Before any die, in this order: `DuplicateId` if a subject id repeats; `SameCreature` if the seeker id is a subject; `AreaTooLarge`; `Overflow` if `CheckModifier::new` rejects the Perception modifier. Perception `2147483647` fails that constructor because it cannot be added to 255. A list that is both a duplicate and over 30 feet returns `DuplicateId`. The seeker listed twice returns `DuplicateId`, not `SameCreature`. The seeker listed once inside an area of 35 feet returns `SameCreature`.

If no subject is hidden or undetected with a Stealth DC, the call succeeds, draws no die, and returns the same relations. Otherwise one `resolve_check` is compared with each qualifying Stealth DC, in ascending `CombatantId`. Seek lists a critical success and a success, and no critical failure. `fallback_unlisted_critical(degree, true, false)` is the effect:

- critical success: hidden or undetected becomes observed
- success: undetected becomes hidden, hidden becomes observed
- failure, including a critical failure read as failure: unchanged

The concealed flag is copied. Invisibility and an imprecise sense do not change the effect (proposed DEV-40). The numeric degree is on the roll. The roll is open (DEV-08).

### Hide

The Stealth modifier is `stealth_dc - 10`, then +2 for `Cover::Standard`, +4 for `Cover::Greater`, or +0 when the only qualifier is `concealed`. `checked_sub` and `checked_add`. `CheckModifier::new` rejects a modifier that cannot be added to 255. `DcModifier::new` rejects a Perception modifier for which `10 + modifier` does not fit. Perception `2147483638` is that failure. Either failure is `Overflow` and draws no die.

Before any die: `DuplicateId`, then `SameCreature` (an observer id equal to the subject), then `NoStealthDc`, then `RequirementNotMet` when no observer has standard cover, greater cover, or `concealed`. Lesser cover does not qualify. `NoStealthDc` wins over a cover failure. A duplicate that is also the subject returns `DuplicateId`. An observer who does not qualify is left unchanged when another does.

One `resolve_check` is compared with each qualifying Perception DC, in ascending id. Hide lists a success and no critical success or critical failure. Both critical degrees use the unlisted fallback. Success hides an observed subject. Hidden stays hidden. Undetected stays undetected. Failure leaves the relation. The concealed flag is copied. The cost on success is 1 action and an empty trait list. The degree is on the roll (proposed DEV-38, open roll, amending DEV-10's "no Hide action").

`refresh(relation, cover)` : a hidden subject with neither standard nor greater cover, and with `concealed == false`, becomes observed. Lesser cover does not keep hidden. Undetected is unchanged.

`note_strike(relations, target)` reads `off_guard_against` first. If that observer's detection was `Hidden`, it becomes `Observed` and the flag stays. Other observers stay. `Undetected` stays undetected and is still `OffGuard::Yes`. `note_action` for Stride, Step, or anything other than that Strike returns the relations unchanged (proposed DEV-41). Neither function rolls.

### Proposed deviations

These rows are proposed. They are not written to `docs/designer/rules-deviations.md`.

| ID | Rule | RAW citation | Our behavior | Reason |
|---|---|---|---|---|
| DEV-38 | Stealth check for Hide. Amends the "no Hide action" sentence of DEV-10. | https://2e.aonprd.com/Actions.aspx?ID=2404 (PC 244); https://2e.aonprd.com/Rules.aspx?ID=2285 (PC 401) | Hide is 1 action. The modifier is the baked Stealth DC minus 10. The roll is open. No Stealth skill. Absent Stealth DC: cannot Hide, and cannot be hidden or undetected. | No skills in Phase 1. |
| DEV-39 | Targeting an undetected creature | https://2e.aonprd.com/Conditions.aspx?ID=96 (PC 447) | `CannotTarget`. No square and no flat check. Area effects still apply with no roll. The observer is still off-guard. | The guess is hidden information. |
| DEV-40 | Senses, light, and invisibility | https://2e.aonprd.com/Actions.aspx?ID=2301 (PC 417); https://2e.aonprd.com/Rules.aspx?ID=2414 (PC 444) | Every Seek is a precise sense. A listed degree becomes observed or hidden as written. Concealment is a caller flag, not dim light. Invisibility is not a condition. | Those subsystems are deferred. |
| DEV-41 | Ending hidden | https://2e.aonprd.com/Actions.aspx?ID=2404 (PC 244) | A Strike while hidden reads `OffGuard::Yes`, then that observer's detection becomes observed. Stride, Step, and every other action do not clear hidden. Sneak is not an action. | Sneak and the rest of that list are deferred. The off-guard Strike stays. |

DEV-08 is unchanged: Seek is an open roll, and every dimension above 30 feet is rejected. This change adds no distance penalty.

## Risks / Trade-offs

- [Positioning has not landed `OffGuard` in the crate yet] → The implementation tasks do not start until that type is public. This spec does not declare a stand-in.
- [A hidden creature can Stride and stay hidden] → Proposed DEV-41. The Strike exception is specified. Sneak is the follow-up that should replace the "other actions do nothing" substitute.
- [Undetected creatures cannot be attacked by guessing a square] → Proposed DEV-39. Seek is the way onto the hidden DC 11 check.
- [The same Perception total can be a success against one Stealth DC and a failure against another] → One roll, compared per subject. The scenario with total 18 against DC 17 and DC 27 locks that.

## Migration Plan

No saved encounter and no change to an existing public function. Callers who do not use the new module keep their current results.
