# Design

## Context

`Creature` already stores one optional shield: hardness, maximum Hit Points, broken threshold, and that shield's AC bonus, each as `i32` on `ShieldTemplate`. `CombatantState` already stores `shield_hit_points` and `shield_raised`. `ac_modifiers` builds cover, off-guard, and unconscious, and `total` keeps the highest circumstance bonus and applies a circumstance penalty beside it. `ac_modifiers` does not take a shield. Positioning's `armor_class` already appends `extra`. See proposal.md for why this change exists.

`add-dying-and-victory` defines `apply_resolved_blow(state, creature, amount, degree)`. That function is the only creature-damage formula: massive damage, temporary Hit Points, then current Hit Points, then dying. Its `HpError::InconsistentVital` is the variant this change returns through `AttackBlowError::HitPoints`. `add-encounter-turns` defines `HpError::NotAHit`, baked Armor Class on Strike, `spend_actions`, `spend_reaction`, `reaction_ready`, and the start step that runs before actions return. `Side` is `Side::{Pc, Enemy}`. `OffGuard` is positioning's `enum OffGuard`. This change reuses those names and does not declare them.

This change names no file on `v1-reference`, so none is ported. It does not edit `docs/designer/`.

## Goals / Non-Goals

**Goals:**

- One module, `crates/rules/src/shield.rs`, for Raise a Shield, the circumstance bonus, Shield Block, and lowering the shield at the start of that combatant's turn.
- Shield Block reduces the attack amount, then calls `apply_resolved_blow` with the remainder.
- Broken and destroyed are read from shield Hit Points. They are not a second stored flag.

**Non-Goals:**

- A new `Side` or any other team type. Victory and flanking keep `Side::{Pc, Enemy}`.
- Putting the raised bonus into Strike's degree. Strike keeps baked Armor Class. A follow-up, `wire-shield-ac-into-strike`, would pass this total into the attack DC. This change does not.
- Shield bash, a shield boss, shield spikes, tower-shield cover, Take Cover's +4, Repair, a speed penalty, hands, and disarm.
- An Unconscious creature who is already raising a shield. That case is `wire-dying-into-turns`.
- Damage types, resistance, weakness, or immunity (DEV-09). Persistent damage is not an attack and is not a Shield Block trigger.
- Editing `docs/designer/rules-deviations.md` or `docs/designer/rules-rulings.md`.

## Decisions

### Where Shield Block plugs in

```rust
pub enum ShieldBlock {
    Requested,
    Declined,
}

pub fn apply_attack_blow(
    state: &mut CombatantState,
    creature: &Creature,
    amount: i32,
    degree: DegreeOfSuccess,
    block: ShieldBlock,
) -> Result<(), AttackBlowError>
```

`amount` is already `damage_dealt`'s result, after the penalty floor and critical doubling (R-07). This function does not roll and does not double.

1. `Failure` or `CriticalFailure` returns `AttackBlowError::HitPoints(HpError::NotAHit)` and writes nothing. The reaction stays ready. This wins over every later check, including a broken shield and a hardness of -1 on a state that skipped validation.
2. A negative `amount` returns `AttackBlowError::NegativeAmount` and writes nothing. This applies to `Declined` and to `Requested`.
3. `ShieldBlock::Declined` calls `apply_resolved_blow` with the original `amount` and leaves the shield and the reaction unchanged. It does not check `NotConscious`, `NoShield`, `NotRaised`, `Broken`, or `NoReaction`.
4. The remaining steps apply only to `Requested`.
5. A combatant who is not `Vital::Alive` returns `AttackBlowError::NotConscious`.
6. No shield returns `AttackBlowError::NoShield`.
7. A shield that is not raised returns `AttackBlowError::NotRaised`.
8. A shield at 0, or at or below its broken threshold, returns `AttackBlowError::Broken`. Destroyed uses `Broken` here. Raise still returns `Destroyed` and `Broken` as two errors.
9. A reaction that is not ready returns `AttackBlowError::NoReaction`. A second requested block in the same round is this error.
10. Amount 0 returns `Ok` and writes nothing. The reaction stays ready.
11. Otherwise `remainder` is `max(0, amount as i64 - hardness as i64)`. Amount `2147483647` and hardness 5 produce `2147483642`, which fits in `i32`. Call `apply_resolved_blow(state, creature, remainder, degree)`. On `Err`, write nothing else. On `Ok`, subtract `remainder` from shield Hit Points, floored at 0, and clear `reaction_ready`.

There is no hardness-sign guard on this path. A shield with hardness below 0, Hit Points below 1, an AC bonus below 1, a broken threshold below 0, or a broken threshold that is not below its Hit Points is `TemplateError::InvalidShield` at build and is not a Creature. The precedence cases that mention hardness -1 construct a state without `derive`, only so `NotAHit` is shown to win.

Hardness is subtracted once. Shield Block says the shield prevents damage up to its Hardness and then both the creature and the shield take the rest (https://2e.aonprd.com/Feats.aspx?ID=5212, Player Core pg. 262; https://2e.aonprd.com/Rules.aspx?ID=2180, Player Core pg. 274). The item-damage sentence is that same subtraction (https://2e.aonprd.com/Rules.aspx?ID=2160, Player Core pg. 269). A steel shield and 12 damage leave the shield at 13, not 18.

Alternative considered: teach `apply_resolved_blow` about shields. Rejected. That function is also the direct blow, and a direct call must not change the shield.

Alternative considered: a second Hit Point formula in `shield.rs`. Rejected. Massive damage, temporary Hit Points, and dying stay inside `apply_resolved_blow`.

Alternative considered: an illegal request passes the full amount. Rejected. Raise already errors on the same states. Declining is the only path that applies the full amount.

### Strike

`strike` takes `block: ShieldBlock`. Existing call sites pass `ShieldBlock::Declined`. A requested block checks steps 5 through 9 before any die is drawn. `Declined` does not. An error maps into `EncounterError` and spends no action:

| Blow error | EncounterError |
|---|---|
| `NotConscious` | `TargetNotConscious` |
| `NoShield` | `NoShield` |
| `NotRaised` | `NotRaised` |
| `Broken` | `Broken` |
| `NoReaction` | `NoReaction` |
| `NegativeAmount` | `NegativeAmount` |
| `HitPoints(err)` | `HitPoints(err)` |

`EncounterError::NotConscious` stays the attacker's error from `add-encounter-turns`. This change does not replace it. `TargetNotConscious` is the target's block error. `NoReaction` already exists. The other variants are new. A miss does not resolve a block. A hit resolves it with `damage_dealt`'s amount and does not also call `apply_resolved_blow`. When that block is `Requested` and legal, `strike` spends the defender's reaction through `spend_reaction` and does not call `apply_attack_blow` on that path. The degree still uses baked Armor Class, so a natural 11, attack bonus +0, and baked Armor Class 11 is a success even when a steel shield is raised.

`add-dying-and-victory` adds the requirement `Strike damage uses resolved blows`. It is not in `openspec/specs/` until that change is archived. This change modifies that requirement. The header stays `Strike damage uses resolved blows`. Shield Hit Points stay unchanged unless a legal Shield Block is requested. Archive this change only after `add-encounter-turns` and `add-dying-and-victory` are archived, and only after that header is present in `openspec/specs/encounter-turns/spec.md`.

### Raise, lower, and the bonus

`raise_shield` checks, in order: no shield (`NoShield`), not Alive (`NotConscious`), 0 Hit Points (`Destroyed`), Hit Points at or below the broken threshold (`Broken`). It sets `shield_raised` and does not change actions or the reaction. Raising an already raised shield is `Ok` and does not add a second bonus. It does not return `InsufficientActions`. That error belongs to `spend_actions`.

`Encounter::raise_shield` rejects `NotYourTurn` when the id is not the active combatant, before it changes actions or the shield. It runs the same shield checks next, and only then calls `spend_actions(id, 1)`. The flag is set after that spend succeeds. A broken shield therefore does not spend an action.

A legal block reports that the reaction must be spent, then writes the remainder. `apply_attack_blow` is the direct entry: it clears `reaction_ready` itself when that spend is required, so a state test needs no encounter. `Encounter::shield_block` is the encounter entry. It uses the same checks, calls `spend_reaction` for that spend, and does not call `apply_attack_blow` afterward. `spend_reaction` would otherwise see a reaction that was already cleared and return `NoReaction`. One private resolve feeds both entries. `spend_reaction` already allows an Alive creature to spend during another creature's turn.

`lower_raised_shield` clears `shield_raised` and changes nothing else. `start_turn` calls it on the combatant who is actually starting, after the round increment has succeeded and before that step grants 3 actions and a reaction. `end_turn` does not call it. `RoundOverflow` returns before that start, so the shield stays raised. `start_turn` skips `Vital::Dead` and does not call `lower_raised_shield` for that combatant. A Dead combatant's raised shield adds no modifier.

```rust
pub fn shield_ac_modifier(
    state: &CombatantState,
    creature: &Creature,
) -> Option<Modifier>
```

`None` when the combatant is Dead, the shield is absent, lowered, broken, or destroyed, or the listed bonus is not positive. `Some` when it is raised and intact. There is no `Result` and no `ShieldError::Modifier`. The caller passes that modifier in positioning's `armor_class(..., extra)`. `ac_modifiers` gains no shield argument. The modifier is reserved `ModifierSource::RaisedShield`, sort key `(11, 0)`, sequence 11, circumstance, Armor Class, no condition. Callers cannot construct it. Sequence 11 is after `AttackCircumstance` at `(9, 0)` on main and after `UnconsciousReflex` at `(10, 0)` reserved by `add-dying-and-victory`. It is not one of those keys.

`ShieldTemplate` stays four `i32` fields. A non-negative newtype would retitle creature-template's stored shield, which this change does not do. `TemplateError::InvalidShield` rejects hardness below 0, Hit Points below 1, an AC bonus below 1, a broken threshold below 0, and a broken threshold that is not below the shield's Hit Points, on `CharacterBuild::try_new` and on fold. The authored buckler of hardness 3, Hit Points 8, broken threshold 4, and AC bonus +1 still derives. A book buckler in the shield scenarios is hardness 3, Hit Points 6, broken threshold 3, AC bonus +1 (`Shields.aspx?ID=17`). A steel shield is hardness 5, Hit Points 20, broken threshold 10, AC bonus +2 (`Shields.aspx?ID=19`).

Broken is `shield Hit Points > 0` and `shield Hit Points <= broken threshold`. Destroyed is 0. The armor exception in the broken condition does not apply, so there is no status penalty of -1, -2, or -3.

A prone combatant with stored `GreaterVsRanged`, a raised steel shield, and `OffGuard::Yes` totals 0 against melee and +2 against ranged. Each attack applies one circumstance bonus: melee keeps the shield's +2, ranged keeps greater cover's +4. Off-guard's -2 applies to both (https://2e.aonprd.com/Conditions.aspx?ID=58, Player Core pg. 445; https://2e.aonprd.com/Rules.aspx?ID=2282, Player Core pg. 400).

### Proposed ledger rows

Do not edit `docs/designer/rules-deviations.md` in this change. Designer records these. The ids are proposals.

| ID | Rule | RAW citation | Our behavior | Reason | Owning change |
|---|---|---|---|---|---|
| DEV-38 | Shield Block availability | https://2e.aonprd.com/Feats.aspx?ID=5212 (Player Core pg. 262); https://2e.aonprd.com/Rules.aspx?ID=2180 (Player Core pg. 274) | Any creature whose `Creature` has a shield may Shield Block while that shield is raised. There is no feat and no class feature. | Feats and classes are out of Phase 1. | 7 |
| DEV-39 | Wielding a shield | https://2e.aonprd.com/Actions.aspx?ID=2316 (Player Core pg. 419); buckler hand rule https://2e.aonprd.com/Shields.aspx?ID=17 (Player Core pg. 274) | A shield on the `Creature` is wielded. No hands, no free-hand check, and no disarm. | No held-item slots in Phase 1. DEV-12 already keeps held items from dropping. | 7 |

(DEV-38, proposed). (DEV-39, proposed).

DEV-09 is cited, not restated: every attack amount is physical, so Shield Block takes no damage type. FIX-11 is cited: the trigger is any attack's physical damage, not a blow that is only named Strike. Persistent damage is not that trigger (https://2e.aonprd.com/Feats.aspx?ID=5212, Player Core pg. 262). The public entry is `apply_attack_blow`. Phase 1's Strike calls it on a hit.

### Errors and purity

`ShieldError` is `NoShield`, `Broken`, `Destroyed`, and `NotConscious`. `InsufficientActions` stays on `EncounterError`, from `spend_actions`. `AttackBlowError` is `HitPoints(HpError)`, `NotConscious`, `NoShield`, `NotRaised`, `Broken`, `NoReaction`, and `NegativeAmount`. `EncounterError` gains `TargetNotConscious` for the target's block error and keeps the attacker's `NotConscious`. `InsufficientActions` is returned by `spend_actions`, not by a second writer. No string errors. No `unwrap`, `expect`, or `panic!`. No `unsafe`, no new dependency, no clock, and no dice. `raise_shield` and `apply_attack_blow` take `&mut CombatantState`. `shield_ac_modifier` does not.

## Risks / Trade-offs

- [Strike's baked Armor Class ignores the raised bonus] → The bonus is still a real total, and the blocked-Strike scenario asserts success against Armor Class 11. `wire-shield-ac-into-strike` is the follow-up. This change does not take it.
- [Shield Block is only physical damage from an attack] → Persistent damage is applied with `apply_resolved_blow` and does not spend the reaction. A second requested block in the same round is `NoReaction`.
- [Two entries can spend the reaction] → `apply_attack_blow` clears `reaction_ready` for a direct test. `Encounter::shield_block` spends it only through `spend_reaction` and does not call `apply_attack_blow` after that. Raise does not touch it. The start step grants it.
- [`add-dying-and-victory` is not implemented yet] → Implementation waits until `apply_resolved_blow` and `Vital::Dying` exist. This spec pull request does not add them.
- [Signed shield fields can still be built by hand] → `derive` will not produce them. The blow path does not add a second hardness error. `NotAHit` still wins on a miss.

## Migration Plan

Spec only in this pull request. No changelog bullet and no archive. The implementation pull request adds `shield.rs`, `ModifierSource::RaisedShield`, `ShieldBlock` on `strike`, the `EncounterError` variants, the build check, and the start-step call. It adds the Unreleased changelog bullet before it is done. `openspec archive add-shields --yes` is the last commit of that pull request, and only after `add-encounter-turns` and `add-dying-and-victory` are archived. Confirm `Strike damage uses resolved blows` is in `openspec/specs/encounter-turns/spec.md` first. Rollback is reverting that pull request. There is no saved encounter to migrate.
