# Design

## Context

`degree_of_success` and `resolve_check` already apply the four bands and the natural 1 and natural 20 shift. `apply_resolved_blow` treats `Failure` and `CriticalFailure` as `HpError::NotAHit`, and it treats `CriticalSuccess` as the critical-blow dying step. `Statistic::Reflex` and `reflex_modifiers` already exist. `Creature` has no save training. See proposal.md for why this change exists.

`Encounter::strike` compares with baked Armor Class, spends one action, and rolls damage through `damage_dealt`. `wire-targeting-into-strike` may change that function's signature. This change does not edit it. This change names no file on `v1-reference`. It does not edit `docs/designer/`.

## Goals / Non-Goals

**Goals:**

- One module, `crates/rules/src/save.rs`, for the three modifiers, the save DC, and the basic-save multiplier.
- `Encounter::basic_save` rolls with `resolve_check` and applies the multiplied amount with `apply_resolved_blow`.
- Fortitude and Will are new statistics. They do not take the unconscious penalty.

**Non-Goals:**

- Editing `Encounter::strike`, `reactive_strike`, or a readied Strike. No flat check and no `Detections` argument.
- Calling `damage_dealt` or Shield Block. The listed amount is already final. DEV-14 and DEV-09 stay on Strike.
- Ability scores, Strength, Intelligence, Charisma, skills, class DC, spells, Grapple, incapacitation, and Hero Points.
- A fourth bonus type. Proficiency stays an addend.

## Decisions

### Training stored on the creature

`SavingThrows` holds `level`, `constitution`, `dexterity`, and `wisdom` as attribute modifiers, plus one `Proficiency` per save. `BuildParts`, `Contribution`, and `Creature` gain `saves: Option<SavingThrows>`. `None` is copied through. A contribution replaces training when `Some`, as a shield does. `derive` copies the option and does not roll.

Alternative considered: a required block on every creature. Rejected. Existing fixtures have no saves, and `None` is the typed "cannot attempt" state.

Alternative considered: store an ability score and use `(score - 10) / 2`. Rejected. Player Core stores the attribute modifier. That division is not this book's procedure.

### Modifier formula

`save_modifier` adds the attribute modifier, the proficiency bonus, and `total` on that statistic. Untrained is 0. The other ranks are `level` plus 2, 4, 6, or 8, checked. Reflex includes `reflex_modifiers(true)` for `Dying` or `Unconscious`. Fortitude and Will use empty producers. The sum must fit `CheckModifier`, which always fits `DcModifier`, so the DC is 10 plus it.

A caller modifier on another statistic is ignored. Proficiency is not a `Modifier`.

### Basic-save amount and the blow

`basic_save_amount` maps the degree: critical success to 0, success to half rounding down, failure to the amount, critical failure to double. Half of 1 stays 1. Half of 0 stays 0. Double of 0 stays 0. A double that does not fit is an error. Negative amounts are rejected by the encounter before this call.

The encounter then calls `apply_resolved_blow` with a translated degree:

| Save degree | Amount | Blow degree |
|---|---|---|
| Critical success | 0 | Success |
| Success | half, minimum 1 when the amount is at least 1 | Success |
| Failure | full | Success |
| Critical failure | double | CriticalSuccess |

`Failure` must not be passed through. That value is `NotAHit` today. `CriticalSuccess` selects the existing dying step of 2 and does not double the amount again.

Alternative considered: teach `apply_resolved_blow` about saves. Rejected. Strike still passes a miss as `Failure`.

### Encounter order

`Encounter::basic_save(defender, save, dc, amount, modifiers, dice)` checks, in order: `UnknownCombatant`, `TargetDead`, `NegativeAmount`, `NoSavingThrows`, then `SaveOverflow`. Those draw no die. It then calls `resolve_check`. A double that does not fit returns `DamageOverflow` and writes no Hit Points. Otherwise it applies the blow. It does not spend an action, the reaction, or an attack count.

When a turn is open it calls `place_delayed_knockout` with the open creature as the anchor. When no turn is open it does not. A non-delayed knockout leaves the pending report and the order.

Alternative considered: skip knockout placement because a save has no attacker. Rejected. The open creature is the turn in which the defender hit 0.

### Statistics

Add `Statistic::Fortitude` and `Statistic::Will` after Reflex, with new sort keys, so existing keys stay. `fortitude_modifiers` and `will_modifiers` return an empty `Vec`. `reflex_modifiers` stays the status penalty of -4 at sequence 10.

### Proposed deviations

| ID | Rule | RAW citation | Our behavior | Reason | Owning change |
|---|---|---|---|---|---|
| DEV-52 | Save rank may be untrained | https://2e.aonprd.com/Rules.aspx?ID=2296 (PC 404) | `Untrained` is legal and is +0 | No class grants trained | add-saving-throws |
| DEV-53 | Basic save is not a Shield Block | https://2e.aonprd.com/Feats.aspx?ID=5212 (PC 262) | `basic_save` never calls the shield path | Strike owns the block | add-saving-throws |
| DEV-54 | Attributes for three saves only. Amends DEV-01 | https://2e.aonprd.com/Rules.aspx?ID=2295 (PC 404) | Constitution, Dexterity, and Wisdom only. AC, attack, Perception, and DCs stay baked | Saves, not full derivation | add-saving-throws |

Unused numbers stay unused: DEV-33 to DEV-35 and DEV-47 to DEV-49.

## Risks / Trade-offs

- [Miss degree on a failed save] → Translate before `apply_resolved_blow`. The dying scenarios lock 1 and 2.
- [Half of 1 becomes 0] → The halving scenario locks damage 1.
- [Strike's signature changes in another change] → This change does not edit `Encounter::strike`.
- [A listed 0 is raised to 1] → Do not call `damage_dealt`. Success on 0 stays 0.

## Migration Plan

The implementation adds `saves: None` at every current `BuildParts`, `Contribution`, and `Creature` literal, then sets `Some` only in the new tests. No saved data exists. Revert that commit to roll back. The spec commit has no Rust change.
