# Proposal

## Why

Phase 1 left skills deferred: Armor Class, attack, Perception, and DCs stay baked, and nothing rolls a skill check (docs/designer/pf2e-mvp-scope.md; the 2026-10-20 architecture decision). The Player Core skill check is the procedure that adds a key attribute, a proficiency rank, and other bonuses and penalties, then compares that total with a DC.

## What Changes

- Add `skills`. One roll uses the existing check. It does not reimplement the four degrees or the natural 1 and natural 20 shift.

- The modifier is the skill's key attribute modifier, plus the proficiency bonus, plus modifiers on that skill only. Untrained is +0. An item bonus still adds when untrained. Proficiency stays out of the bonus types.

- The sixteen Player Core skills other than Lore are the closed set. Each uses the key attribute on its skill page. Lore is not a skill in this change.

- `skill_check` is a primitive for a caller-adjudicated check. It resolves that roll against a caller DC and spends nothing, including when the creature is unconscious. It adds no unconscious penalty. `recall_knowledge` spends 1 action on the actor's turn and returns the same degree. The result is the degree. It does not answer the question. Recall Knowledge's concentrate trait is not stored.

- Demoralize, Trip, Grapple, and Shove are not this change. Each applies a condition, and Grapple and Demoralize also carry a duration. Those actions belong to a later change that depends on `add-conditions-with-durations`. This change does not define frightened, grabbed, restrained, prone-from-Trip, or the 10-minute immunity.

**BREAKING:** `BuildParts`, `Contribution`, and `Creature` gain `skills: Option<Skills>`. `Statistic` gains one variant per skill. `Encounter` gains `skill_check` and `recall_knowledge`. `EncounterError` and `ReadyEncounter` gain `NoSkills` and `SkillOverflow`. The facade's public name set gains `Skill`, `Skills`, and `SkillRoll`.

## Capabilities

### New Capabilities

- `skills`: The sixteen skill modifiers, the skill DC, the skill check, and Recall Knowledge.

### Modified Capabilities

- `creature-template`: `derive` copies optional skill training and does not recompute Armor Class, attack, Perception, the strike, or saves from it.

- `modifiers`: Each skill is its own statistic. The frightened and sickened producer puts a status penalty equal to its value on all sixteen skill statistics. Unconscious does not penalize a skill. The armor check penalty is out of scope; a caller may pass it as an untyped penalty.

- `encounter-turns`: Recall Knowledge spends 1 action and does not increment the attack count. A skill check is a caller-adjudicated primitive: it spends nothing and is legal off the active turn, including for an unconscious creature.

- `api-facade`: The public method set and the public name set include the skill command and its types. `skill_modifier` and `skill_dc` stay off the crate root.

## Impact

New module `crates/rules/src/skill.rs`, with `Skill`, `Skills`, and `SkillRoll` re-exported from the crate root. The encounter calls the existing check. It does not call Strike, damage, Shield Block, or targeting. `crates/rules` gains no dependency. No file on `v1-reference` is named, so none is ported. This change does not edit Hide, Seek, Aid, or initiative.

This pull request is the spec commit. It adds no code, no tests, and no changelog bullet, and it does not archive. Designer rules on the spec here. Code waits for a `spec-approved: <sha>` review from Evan.

The archive commit is last, right before merge, and only after Architect and QA have both passed that head. Proposed DEV-59, DEV-60, and DEV-61 are in design.md. The ruling log is [docs/designer/rules-rulings.md](../../../docs/designer/rules-rulings.md). The code phase adds those three rows to `docs/designer/rules-deviations.md`. The spec commit does not edit that file.
