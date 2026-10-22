# Proposal

## Why

A creature can already roll Perception against a baked Stealth DC, and off-guard is already one circumstance penalty, but nothing records whether one creature is observed, concealed, hidden, or undetected by another. Seek, Hide, and the targeting flat checks are that relation. This change is that procedure family.

## What Changes

- Add the `seek-and-hidden` capability. It owns the per-observer relation, the targeting flat checks, Seek, and Hide. It calls the existing check, flat check, Stealth DC, and Armor Class total. It does not reimplement them.
- Subtract before adding. Do not add a second off-guard penalty, a second cover bonus, a second DC formula, or a second flat-check roller. Concealed is a flag on the relation, not a replacement for observed, hidden, or undetected: Player Core says the concealed condition does not change the detection category. Hidden and undetected pass positioning's `OffGuard::Yes` once. This change does not declare `OffGuard` and does not edit the positioning requirements.
- Seek is 1 action with the concentrate trait: one open Perception check against each qualifying Stealth DC, over an area whose every dimension is at most 30 feet (DEV-08). A critical success moves hidden or undetected to observed. A success moves undetected to hidden and hidden to observed. Failure and a critical failure, which has no entry, leave the relation as it was.
- Hide is 1 action: one open Stealth check against each qualifying Perception DC. The requirement is standard cover, greater cover, or the concealed flag. Standard cover is a +2 circumstance bonus and greater cover is +4. The Stealth modifier is the baked Stealth DC minus 10 (proposed DEV-38, amending the "no Hide action" sentence of DEV-10). There is still no Stealth skill.
- A concealed target takes a DC 5 flat check. A hidden target takes DC 11. Both at once roll once at DC 11. An observed target, and any area effect, roll nothing. An undetected target cannot be chosen (proposed DEV-39). The observer of a hidden or undetected creature is off-guard to that creature.
- Losing standard-or-greater cover and the concealed flag ends hidden. A Strike while hidden reads `OffGuard::Yes` and then that observer's detection becomes observed. Other actions do not end hidden (proposed DEV-41). Senses, light, and invisibility are not modeled (proposed DEV-40).

Out of this change, and not given a wrong substitute: Sneak, Point Out, invisibility, special senses, light, unnoticed, Avoid Notice, Create a Diversion, Concealing an Object, hazards, and secret doors. Square guessing for an undetected creature is the proposed DEV-39 substitute, not a second procedure.

This change does not add a ruling. The log is [docs/designer/rules-rulings.md](../../../docs/designer/rules-rulings.md). Proposed DEV-38 through DEV-41 are written in design.md. This change does not edit `docs/designer/`. No other spec's requirement header changes.

## Capabilities

### New Capabilities

- `seek-and-hidden`: Per-observer detection and the concealed flag, targeting flat checks, Seek, and Hide.

### Modified Capabilities

None. `creature-template` already copies an optional Stealth DC and says a Seek check uses that DC without changing the Creature. `dice-and-checks` already resolves a d20 check, a DC of 10 plus a modifier, and one flat check at the highest DC. `modifiers` already totals cover and one off-guard penalty of −2. `add-positioning` already names `OffGuard` and does not take a hidden input. This change calls those results.

## Impact

New module `crates/rules/src/detection.rs`, re-exported from the crate root. It imports `OffGuard` and does not define it. It imports `Cover` from modifiers. Callers pass combatant ids, the relation, cover, the concealed flag, baked DCs, and dice. The relation is not a field of `Creature` or `CombatantState`. `crates/rules` gains no dependency. This pull request is the spec only: no changelog bullet and no archive. The implementation pull request adds the Unreleased changelog bullet, then archives as its last commit. Implementation starts once `OffGuard` from `add-positioning` is public in the crate.
