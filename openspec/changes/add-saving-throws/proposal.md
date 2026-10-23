# Proposal

## Why

The encounter rolls a Strike against baked Armor Class and applies that blow, and Reflex already has an unconscious penalty, but nothing rolls Fortitude, Reflex, or Will. A basic save is the procedure that turns those three checks into damage.

## What Changes

- Add `saving-throws`. One roll uses the existing check. It does not reimplement the four degrees or the natural 1 and natural 20 shift.

- The modifier is the attribute modifier, plus the proficiency bonus, plus modifiers on that statistic. Fortitude uses Constitution. Reflex uses Dexterity. Will uses Wisdom. Proficiency stays out of the bonus types.

- A basic save deals no damage on a critical success, half on a success, the full amount on a failure, and double on a critical failure. Half rounds down, and half of 1 stays 1. The amount is doubled once.

- The encounter applies that amount through the existing blow. A save failure is an ordinary blow. A save critical failure is a critical blow. Strike is not called and is not edited.

- Player Core stores attribute modifiers, not ability scores. This change does not apply `(score − 10) / 2`.

**BREAKING:** `BuildParts`, `Contribution`, and `Creature` gain `saves: Option<SavingThrows>`. `Statistic` gains `Fortitude` and `Will`. `EncounterError` gains `NoSavingThrows` and `SaveOverflow`.

## Capabilities

### New Capabilities

- `saving-throws`: The three save modifiers, the four degrees, natural 1 and natural 20, and basic-save damage.

### Modified Capabilities

- `creature-template`: `derive` copies optional save training and does not recompute Armor Class, attack, Perception, or the strike from it.

- `modifiers`: Fortitude and Will are their own statistics. Unconscious does not penalize them. Reflex keeps its existing penalty.

- `encounter-turns`: A basic save spends no action and no attack count. Strike still uses baked Armor Class. The save's damage uses the existing blow.

## Impact

New module `crates/rules/src/save.rs`, re-exported from the crate root. The encounter calls the existing check and the existing blow. It does not call the damage-dealt function, Shield Block, or targeting. `crates/rules` gains no dependency. No file on `v1-reference` is named, so none is ported.

This pull request is the spec commit of the single-PR pilot. It adds no code, no tests, and no changelog bullet, and it does not archive. Designer rules on the spec here. Code waits for a `spec-approved: <sha>` comment from Evan.

The archive commit is last, right before merge, and only after Architect and QA have both passed that head. Proposed DEV-52, DEV-53, and DEV-54 are in design.md. The ruling log is [docs/designer/rules-rulings.md](../../../docs/designer/rules-rulings.md). This change does not edit `docs/designer/`.
