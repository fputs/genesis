# Design

## Context

`Creature` already stores one optional shield: hardness, maximum Hit Points, broken threshold, and that shield's AC bonus. `CombatantState` already stores `shield_hit_points` and `shield_raised`. `ac_modifiers` builds cover, off-guard, and unconscious, and `total` keeps the highest circumstance bonus and applies a circumstance penalty beside it. `ac_modifiers` does not take a shield. See proposal.md for why this change exists.

`add-dying-and-victory` defines `apply_resolved_blow(state, creature, amount, degree)`. That function is the only creature-damage formula: massive damage, temporary Hit Points, then current Hit Points, then dying. `add-encounter-turns` defines `HpError::NotAHit`, baked Armor Class on Strike, and the start step that runs before actions return. This change does not edit those requirement headers. They are not in `openspec/specs/`.

This change names no file on `v1-reference`, so none is ported. It does not edit `docs/designer/`.

## Goals / Non-Goals

**Goals:**

- One module, `crates/rules/src/shield.rs`, for Raise a Shield, the circumstance bonus, Shield Block, and lowering the shield at the start of that combatant's turn.
- Shield Block reduces the attack amount, then calls `apply_resolved_blow` with the remainder.
- Broken and destroyed are read from shield Hit Points. They are not a second stored flag.

**Non-Goals:**

- A new `Side` or any other team type. Victory and flanking keep `Side::{Pc, Enemy}`.
- Putting the raised bonus into Strike's degree. Strike keeps baked Armor Class. A follow-up, `wire-shield-ac-into-strike`, would pass this total into the attack DC. This change does not.
- Shield bash, a shield boss, shield spikes, tower-shield cover, Repair, a speed penalty, hands, and disarm.
- Damage types, resistance, weakness, or immunity (DEV-09).
- Editing `docs/designer/rules-deviations.md` or `docs/designer/rules-rulings.md`.

## Decisions

### Where Shield Block plugs in

`apply_attack_blow(state, creature, amount, degree, block)` is the attack entry. `amount` is already `damage_dealt`'s result, after the penalty floor and critical doubling (R-07). This function does not roll and does not double.

1. `Failure` or `CriticalFailure` returns `AttackBlowError::HitPoints(HpError::NotAHit)` and writes nothing. The reaction stays ready.
2. A legal block is `block`, `Success` or `CriticalSuccess`, `Vital::Alive`, a shield that is raised, not broken, and not destroyed, `reaction_ready`, hardness at least 0, and `amount > 0`.
3. On a legal block, `remainder = max(0, amount - hardness)` using checked arithmetic in `i64`. Call `apply_resolved_blow(state, creature, remainder, degree)`. On `Err`, write nothing else. On `Ok`, subtract `remainder` from shield Hit Points, floored at 0, and clear `reaction_ready`.
4. Otherwise call `apply_resolved_blow` with the original `amount`. Leave shield Hit Points and the reaction as they were.
5. Hardness below 0, when step 2 would have been legal except for the sign, returns `AttackBlowError::Hardness` and writes nothing.

Hardness is subtracted once. Shield Block says the shield prevents damage up to its Hardness and then both the creature and the shield take the rest (Feats 5212, Rules 2180). The general item-hardness sentence (Rules 2160) is that same subtraction, not a second one. A steel shield and 12 damage leave the shield at 13, not 18.

Alternative considered: teach `apply_resolved_blow` about shields. Rejected. That function is also the direct blow, and a direct call must not change the shield.

Alternative considered: a second Hit Point formula in `shield.rs`. Rejected. Massive damage, temporary Hit Points, and dying stay inside `apply_resolved_blow`.

### Strike

`strike` gains `shield_block: bool`. A hit calls `apply_attack_blow` with `damage_dealt`'s amount and does not also call `apply_resolved_blow`. A miss does not call `apply_attack_blow`. Existing call sites pass `false`. The degree still uses baked Armor Class, so a natural 11, attack bonus +0, and baked Armor Class 11 is a success even when a steel shield is raised.

`add-dying-and-victory` says a Strike leaves shield Hit Points unchanged. Its scenario is a shield that is not raised. A Strike that does not block still leaves them unchanged. A legal block is the only writer of shield Hit Point loss. This change does not modify that requirement header.

### Raise, lower, and the bonus

`raise_shield` checks, in order: no shield (`NoShield`), not Alive (`NotConscious`), 0 Hit Points (`Destroyed`), Hit Points at or below the broken threshold (`Broken`), 0 actions (`InsufficientActions`). Otherwise it subtracts 1 action and sets `shield_raised`. The reaction is not spent. Raising an already raised shield spends another action and does not add a second bonus.

`lower_raised_shield` clears `shield_raised` and changes nothing else. The encounter start step calls it on the combatant who is starting, before that step grants 3 actions and a reaction. `end_turn` does not call it. Another combatant's start lowers only that combatant.

`armor_class` calls `ac_modifiers(cover, off_guard, unconscious)`, appends one shield modifier when the shield is raised and intact, and returns `total`. `ac_modifiers` gains no shield argument. The modifier is reserved `ModifierSource::RaisedShield`, sort key `(11, 0)`, sequence 11, circumstance, Armor Class, no condition. Callers cannot construct it. A listed bonus that is not positive returns `ShieldError::Bonus` only when the modifier would be added. Sequence 11 sits after `UnconsciousReflex` at 10 from `add-dying-and-victory`.

A buckler is hardness 3, Hit Points 6, broken threshold 3, bonus +1 (`Shields.aspx?ID=17`). A steel shield is hardness 5, Hit Points 20, broken threshold 10, bonus +2 (`Shields.aspx?ID=19`). The creature-template scenario that stores +1 with Hit Points 8 and broken threshold 4 is an authored example, not that buckler. This change does not edit it.

Broken is `shield Hit Points > 0` and `shield Hit Points <= broken threshold`. Destroyed is 0. The armor exception in the broken condition does not apply, so there is no status penalty of -1, -2, or -3.

### Proposed ledger rows

Do not edit `docs/designer/rules-deviations.md` in this change. Designer records these. The ids are proposals; DEV-33 through DEV-35 are unused and DEV-36 and DEV-37 are taken.

| ID | Rule | RAW citation | Our behavior | Reason | Owning change |
|---|---|---|---|---|---|
| DEV-38 | Shield Block availability | https://2e.aonprd.com/Feats.aspx?ID=5212 (PC 262); https://2e.aonprd.com/Rules.aspx?ID=2180 (PC 274) | Any creature whose `Creature` has a shield may Shield Block while that shield is raised. There is no feat and no class feature. | Feats and classes are out of Phase 1. | 7 |
| DEV-39 | Wielding a shield | https://2e.aonprd.com/Actions.aspx?ID=2316 (PC 419); buckler hand rule https://2e.aonprd.com/Shields.aspx?ID=17 (PC 274) | A shield on the `Creature` is wielded. No hands, no free-hand check, and no disarm. | No held-item slots in Phase 1. DEV-12 already keeps held items from dropping. | 7 |

DEV-09 is cited, not restated: every attack amount is physical, so Shield Block takes no damage type. FIX-11 is cited: the trigger is any attack blow, not a blow that is only named Strike. The public entry is `apply_attack_blow`. Phase 1's Strike calls it on a hit.

### Errors and purity

`ShieldError` is `NoShield`, `Broken`, `Destroyed`, `NotConscious`, `InsufficientActions`, `Bonus`, and `Modifier(ModifierError)`. `AttackBlowError` is `HitPoints(HpError)` and `Hardness`. No string errors. No `unwrap`, `expect`, or `panic!`. No `unsafe`, no new dependency, no clock, and no dice. `raise_shield` and `apply_attack_blow` take `&mut CombatantState`. `armor_class` does not.

## Risks / Trade-offs

- [Strike's baked Armor Class ignores the raised bonus] → The bonus is still a real total, and the blocked-Strike scenario asserts success against Armor Class 11. `wire-shield-ac-into-strike` is the follow-up. This change does not take it.
- [The dying Strike sentence says shield Hit Points stay unchanged] → The no-block Strike still does. The block is specified only here, and this change does not rewrite that header.
- [Two writers of `reaction_ready`] → Only `apply_attack_blow` clears it for a block. The encounter still grants it at the start of the turn. Raise does not touch it.
- [`add-dying-and-victory` is not implemented yet] → Implementation waits until `apply_resolved_blow` and `Vital::Dying` exist. This spec pull request does not add them.

## Migration Plan

Spec only in this pull request. No changelog bullet and no archive. The implementation pull request adds `shield.rs`, the reserved modifier source, the `shield_block` argument on `strike`, and the start-step call. It adds the Unreleased changelog bullet before it is done. `openspec archive add-shields --yes` is the last commit of that pull request. Rollback is reverting that pull request. There is no saved encounter to migrate.
