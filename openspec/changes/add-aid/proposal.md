# Proposal

## Why

Aid on the creature is a baked modifier and a DC 15 check. The fight cannot spend an action to prepare, spend the reaction when an ally rolls, or scale the critical-success bonus by rank. This change is that procedure.

## What Changes

- Add the `aid` capability: one action on your turn to prepare, then the reaction when the caller resolves an ally's attack check. The reaction spends the single reaction through `spend_reaction`.
- The check stays the baked aid modifier against DC 15 (DEV-02, DEV-04). Success is +1 circumstance, failure is 0, and a critical failure is -1. A critical success is +1, +2, +3, or +4 from `AidRank` (proposed DEV-50). The book's critical success is +2, +3 at master, and +4 at legendary.
- The circumstance is returned for that one check. It is not stored on the ally. A later check that does not pass it gains nothing.
- An aider who is not `Vital::Alive` returns `NotConscious` and draws no die. That error wins over the later Aid errors.
- Same Side is willing, including Enemy with Enemy. You cannot aid yourself. Distance, a GM-chosen skill, a shifted DC, and a repeat penalty are not modeled (proposed DEV-51).
- The build still does not store preparation. Omitting the rank stores `AidRank::Trained`.

Out of scope: skills, saves, attributes, a level-based proficiency bonus, legendary as its own rank, proximity, and teaching `Encounter::strike` to remember an Aid bonus. This change does not edit `docs/designer/`.

## Capabilities

### New Capabilities

- `aid`: Prepare on your turn, the DC 15 reaction, the one-check circumstance, and Aid error precedence.

### Modified Capabilities

- `creature-template`: `Aid modifier against DC 15`. The critical-success bonus follows `AidRank` instead of the fixed +2. The header is the requirement in `openspec/specs/creature-template/spec.md`.

## Impact

`AidRank`, `AidCircumstance`, and `AidPreparation` are new public types. `aid_circumstance_bonus` gains the rank and returns `AidCircumstance`. `Encounter` gains `prepare_to_aid` and `aid`. `EncounterError` gains `NotPrepared`, `NoAidModifier`, `NotAnAlly`, and `AidOverflow`. `CombatantState` gains `aid_preparation`. `begin_encounter` clears it. A successful `start_turn` clears it for the creature who starts. `crates/rules` gains no dependency. This pull request is the spec only: no changelog bullet and no archive. The implementation pull request adds the Unreleased changelog bullet, then archives as its last commit, only after `Vital::Unconscious` and `Vital::Dying` are public.
