# Proposal

## Why

Aid on the creature is a baked modifier and a DC 15 check. The fight cannot spend an action to prepare, spend the reaction when an ally rolls, or grant the book's critical-success bonus. This change is that procedure.

## What Changes

- Add the `aid` capability: one action on your turn to prepare for one named ally, then the reaction when the caller resolves that ally's attack roll. The reaction spends the single reaction through `spend_reaction`. Aid and Reactive Strike share that reaction. After a reactive strike, Aid is `NoReaction` and draws no die. The reverse case lives in the reactive-strike spec (PR #55).
- The check stays the baked aid modifier against DC 15 (DEV-02, DEV-04). Success is +1 circumstance, failure adds nothing, and a critical failure is a −1 circumstance penalty. A critical success is +2 below master, +3 at master, and +4 at legendary (`AidRank::{BelowMaster, Master, Legendary}`). This follows the book and supersedes DEV-03. The designer ledger is updated in a later docs pull request.
- `prepare_to_aid` names one ally. That preparation ends at the aider's next `start_turn`, or when `aid` returns `Ok`, whichever comes first (proposed DEV-50).
- The circumstance is returned for that one check. It is not stored on the ally. `Zero` adds no modifier. `Minus1` is a circumstance penalty. Same-type circumstance modifiers keep the highest bonus and the worst penalty, and a bonus and a penalty both apply (Rules 2281, Rules 2282).
- An aider who is not `Vital::Alive` returns `NotConscious` and draws no die. `TurnNotOpen` wins over that error. A dead ally is not an error.
- The book's trigger is an ally about to use an action that requires a skill check or attack roll. This change narrows that trigger to an attack roll (proposed DEV-51). Willing is the same `Side`, which is DEV-17 and DEV-26, not a new id. You cannot aid yourself. Distance, a shifted DC, and a repeat penalty are not modeled.
- The build still does not store preparation. Omitting the rank stores `AidRank::BelowMaster`.

Out of scope: skills, saves, attributes, a level-based proficiency bonus, proximity, and teaching `Encounter::strike` to remember an Aid bonus. This change does not edit `docs/designer/`.

The log is [docs/designer/rules-rulings.md](../../../docs/designer/rules-rulings.md).

## Capabilities

### New Capabilities

- `aid`: Prepare on your turn, the DC 15 reaction, the one-check circumstance, and Aid error precedence.

### Modified Capabilities

- `creature-template`: `Aid modifier against DC 15`. The critical-success bonus follows the book: +2 below master, +3 at master, and +4 at legendary. The header is the requirement in `openspec/specs/creature-template/spec.md`.

## Impact

`AidRank`, `AidCircumstance`, and `AidPreparation` are new public types. `AidPreparation::Prepared` names one `CombatantId`. `aid_circumstance_bonus` gains the rank and returns `AidCircumstance`. The Aid roll checks that the modifier plus 20 fits in `i32`. It does not use `CheckModifier`'s `0..=255` gate. `Encounter` gains `prepare_to_aid` and `aid`. `EncounterError` gains `NotPrepared`, `NoAidModifier`, `NotAnAlly`, `NotThePreparedAlly`, and `AidOverflow(ArithmeticOverflow)`. `CombatantState` gains `aid_preparation`. `begin_encounter` clears it. A successful `start_turn` clears it for the creature who starts. An `aid` that returns `Ok` clears it too. `crates/rules` gains no dependency. This pull request is the spec only: no changelog bullet and no archive. The implementation pull request adds the Unreleased changelog bullet, then archives as its last commit, only after `Vital::Unconscious` and `Vital::Dying` are public.
