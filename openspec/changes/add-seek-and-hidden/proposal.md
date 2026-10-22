# Proposal

## Why

A creature can already roll Perception against a baked Stealth DC, and off-guard is already one circumstance penalty, but nothing records whether one creature is observed, concealed, hidden, or undetected by another. Seek, Hide, and the targeting flat checks are that relation. This change is that procedure family.

## What Changes

- Add the `seek-and-hidden` capability. It owns the per-observer relation, the targeting flat checks, Seek, and Hide. It calls the existing check, flat check, Stealth DC, and Armor Class total. It does not reimplement them.
- Subtract before adding. Do not add a second off-guard penalty, a second cover bonus, a second DC formula, or a second flat-check roller. Concealment is `Concealment::{Concealed, NotConcealed}` on the relation, not a replacement for observed, hidden, or undetected. Player Core says concealment does not change the detection category (Rules 2414, Player Core pg. 434). Hidden and undetected pass positioning's `OffGuard::Yes` once. This change does not declare `OffGuard` and does not edit the positioning requirements.
- Seek is `DetectionCost::Seek`: 1 action with the concentrate trait. Positioning's `ActionTrait` is only `Move` (merged in 83eafa5), and that capability is not an archived spec, so this change does not declare a second `ActionCost`. The caller spends 1 action through encounter-turns `spend_actions`. One open Perception check is compared with each qualifying Stealth DC, over an area whose every dimension is at most 30 feet (DEV-08). The Perception gate is initiative's `checked_add(20)`, so Perception 2147483500 is rolled and 2147483628 is `PerceptionOutOfRange`. A critical success moves hidden or undetected to observed. A success moves undetected to hidden and hidden to observed. Failure and a critical failure, which has no entry, leave the relation as it was.
- Hide is `DetectionCost::Hide`: 1 action, with none of the concentrate, move, or manipulate traits. The caller spends it the same way. One open Stealth check is compared with each qualifying Perception DC. The requirement is standard cover, greater cover, or `Concealment::Concealed`. Standard cover is a +2 circumstance bonus and greater cover is +4. The Stealth modifier is the baked Stealth DC minus 10 (proposed DEV-40, amending the "no Hide action" sentence of DEV-10). There is still no Stealth skill. Observers who already have the subject hidden or undetected are skipped.
- A concealed target takes a DC 5 flat check. A hidden target takes DC 11. Both at once roll once at DC 11. An observed target, and any area effect, roll nothing. An undetected target cannot be chosen (proposed DEV-41). A failed hidden flat check reports the action spent and the attack counting for MAP, and does not call `spend_actions` or `Encounter::strike`. The observer of a hidden or undetected creature is off-guard to that creature.
- A seeker or hider who is not `Vital::Alive` returns `NotConscious` and draws no die. `Side` does not choose who may observe whom.
- Losing standard-or-greater cover and concealment ends hidden, and only when the caller calls `refresh`. A Strike while hidden or undetected reads `OffGuard::Yes` and then every observer's relation of the striker becomes observed (proposed DEV-43). Senses, light, and invisibility are not modeled (proposed DEV-42).

Out of this change, and not given a wrong substitute: Sneak, Point Out, invisibility, special senses, light, unnoticed, Avoid Notice, Create a Diversion, Concealing an Object, hazards, and secret doors. Square guessing for an undetected creature is the proposed DEV-41 substitute, not a second procedure. Feeding detection into `Encounter::strike` (off-guard, the DC 11 flat check, and MAP on a failed flat check) is the follow-up change `wire-detection-into-strike`.

This change does not add a ruling. The log is [docs/designer/rules-rulings.md](../../../docs/designer/rules-rulings.md). Proposed DEV-40 through DEV-43 are written in design.md. This change does not edit `docs/designer/`. No other spec's requirement header changes.

## Capabilities

### New Capabilities

- `seek-and-hidden`: Per-observer detection and concealment, targeting flat checks, Seek, and Hide.

### Modified Capabilities

None. `creature-template` already copies an optional Stealth DC and says a Seek check uses that DC without changing the Creature. `dice-and-checks` already resolves a d20 check, a DC of 10 plus a modifier, and one flat check at the highest DC. `modifiers` already totals cover and one off-guard penalty of −2. `add-positioning` already names `OffGuard` and does not take a hidden input. This change calls those results. It does not modify positioning, encounter-turns, or dying-and-victory: those capabilities are merged on main and are not archived specs, so a MODIFIED header cannot match main byte for byte.

## Impact

New module `crates/rules/src/detection.rs`, re-exported from the crate root. It imports `OffGuard` and `MoveAction` and does not define them. It imports `Cover` from modifiers and `Vital` from the dying types. It does not declare `Side`, `ActionCost`, or `ActionTrait`. Callers pass combatant ids, `Detections`, cover, concealment, baked DCs, vital, and dice. The relation is not a field of `Creature` or `CombatantState`. `crates/rules` gains no dependency. This pull request is the spec only: no changelog bullet and no archive. The implementation pull request adds the Unreleased changelog bullet, then archives as its last commit. Implementation starts once `OffGuard`, `MoveAction`, and `Vital::Unconscious` / `Vital::Dying` are public in the crate. `wire-detection-into-strike` is a later change.
