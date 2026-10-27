# Design

## Context

`damage_dealt` rolls, applies the penalty floor, and doubles a critical hit. `apply_resolved_blow` then spends Hit Points. Nothing in that path takes a damage type. `end_turn` takes no dice, and its end step does not change Hit Points. `ConditionSlot` is an empty unit struct. `basic_save` passes its listed amount straight to the blow. See proposal.md for why this change exists.

The spec commit does not edit `docs/designer/` or any Rust file. No file on `v1-reference` is named.

## Goals / Non-Goals

**Goals:**

- One pure adjustment: immunity, then the highest weakness of that type, then the highest resistance of that type, minimum 0.
- The three attack paths use it after `damage_dealt` and before Shield Block.
- Persistent damage is one condition per type. The end step applies one ordinary blow of the adjusted sum, then one DC 15 flat check per condition.

**Non-Goals:**

- Assisted recovery. The book leaves the help, the DC, and the action cost to the GM, and a particularly appropriate help "usually" uses DC 10 (Conditions 86). Skills that would supply that help belong to `add-skills`. This change rolls only the end-of-turn DC 15 check. Bleed ending at full Hit Points stays, because Damage Types states it as a rule (Rules 2308), not as a GM example.
- Damage dice on the persistent condition. The stored amount is already the number taken each end of turn. The GM choice between a flat amount and dice (Conditions 86) does not arise.
- The one-minute GM timeout, category defenses (physical, all damage, spells) and the choice of which type a category resistance applies to (Rules 2318), precision damage (Rules 2308), vitality and void, materials, immunity to critical hits, and nonlethal. Bleed does not gain physical resistance in this change, because that category is not representable.
- `basic_save`. The listed amount stays final. `add-spells-and-areas` should call the adjustment when it gives a spell a type, and it should not add a second damage-type enum.
- Durations and every named condition except persistent damage. That is `add-conditions-with-durations`.

## Decisions

### Assumption about the conditions change

`add-conditions-with-durations` is not in this tree. This change assumes it provides a generic condition on `CombatantState` and one end-of-turn tick, and nothing else. No duration field, no frightened value, no start-of-turn countdown.

Stand-in, so the two specs can be reconciled: `ConditionSlot` becomes an enum with one variant this change owns.

```rust
pub enum ConditionSlot {
    PersistentDamage { damage_type: DamageType, amount: i32 },
}
```

A fresh state keeps an empty `conditions` vec. Persistent damage does not end by counting rounds. If the other change adds variants, this tick reads only `PersistentDamage` and runs before any end-of-turn decrease that change adds (Rules 2430). If it uses another container, the payload is still (damage type, positive amount) inside the generic condition.

Alternative considered: a separate `persistent_damage` field beside the empty slot. Rejected. Persistent damage is a condition, and a second container would have to be merged later.

### Types

```rust
pub enum DamageType {
    Bludgeoning,
    Piercing,
    Slashing,
    Bleed,
    Acid,
    Cold,
    Electricity,
    Fire,
    Sonic,
    Force,
    Spirit,
    Mental,
    Poison,
}

pub struct Weakness {
    pub damage_type: DamageType,
    pub value: i32,
}

pub struct Resistance {
    pub damage_type: DamageType,
    pub value: i32,
}

pub struct DamageDefenses {
    pub immunities: Vec<DamageType>,
    pub weaknesses: Vec<Weakness>,
    pub resistances: Vec<Resistance>,
}
```

`MeleeStrike` and `StrikeProfile` gain `damage_type`. `BuildParts` and `Creature` gain `defenses`. `derive` copies both and does not roll. A contribution replaces defenses when `Some`, as a shield does. `try_new` rejects a weakness or resistance value below 1 with `TemplateError::InvalidDefenses`. Empty lists are valid. Duplicate immunity is immune once.

`adjust_damage` is `pub(crate)` and is not a crate-root name. It does not roll.

```rust
pub(crate) fn adjust_damage(
    amount: i32,
    damage_type: DamageType,
    defenses: &DamageDefenses,
) -> Result<i32, ModifierError>;
```

Immune: return 0, and do not add weakness or subtract resistance. Otherwise add the highest matching weakness once (DEV-59), then subtract the highest matching resistance (DEV-60), then clamp at 0. A sum that does not fit in `i32` is `ModifierError::Overflow`. The encounter reports `DamageOverflow` and writes nothing.

Alternative considered: sum two weakness values of one type. Rejected. The book adds each weakness once (Rules 2317) and does not say two values of one type stack. The highest is the one value.

Alternative considered: let a caller pick a lower resistance. Rejected. The book says usually the highest (Rules 2318). The engine always uses the highest.

### Attack order

`damage_dealt` is unchanged. On a hit, Strike, Reactive Strike, and a readied Strike call `adjust_damage` on that result, then Shield Block, then the blow. A miss does not roll damage and does not adjust.

Shield Block's trigger is a positive post-adjustment amount of bludgeoning, piercing, or slashing. Immunity is known before the roll, so a requested block against an immune target or a non-physical type does not require a raised shield and does not spend the reaction. A physical request that is not immune still fails with `NotRaised` before the roll when the shield is not raised. After the roll, a remainder of 0 does not spend the reaction and does not change the shield. Hardness applies to the post-adjustment amount. The existing steel shield has hardness 5, so 12 Slashing with resistance 5 becomes 7, then hardness leaves 2 on the creature and 2 on the shield.

`apply_resolved_blow` and `apply_dice_damage` stay untyped. `basic_save` does not call `adjust_damage`.

Alternative considered: Shield Block before the adjustment. Rejected. The trigger is damage you would take, and Step 3 applies immunity, weakness, and resistance before Hit Points are reduced (Rules 2309, Rules 2310).

### Persistent damage

```rust
impl Encounter {
    pub fn gain_persistent_damage(
        &mut self,
        id: CombatantId,
        damage_type: DamageType,
        amount: i32,
    ) -> Result<(), EncounterError>;

    pub fn end_turn(&mut self, dice: &mut impl Dice) -> Result<(), EncounterError>;
}
```

`gain_persistent_damage` spends nothing and rolls nothing. Amount below 1 is `NonPositivePersistent`. Unknown id is `UnknownCombatant`. Dead is `TargetDead`. The same type replaces the amount only when the new amount is strictly higher. Another type is another slot.

`end_turn` draws no die and changes no Hit Points when that creature has no persistent damage. Otherwise, in `DamageType` order, adjust each entry. Sum in `i64`. A sum that does not fit is `DamageOverflow`, the turn stays open, and no flat check is rolled. A positive sum is one `apply_resolved_blow` at `Success`, not a critical hit. Shield Block is not used. Several types increase dying once (Conditions 86, Rules 2327). The creature is already in the slot of the turn that dropped it, so the order is unchanged and the knockout report is cleared. A sum of 0 skips the blow.

If the creature is already Dead, or the blow leaves it Dead, clear the slots and draw no flat-check die. Otherwise each remaining slot rolls the existing DC 15 flat check, in `DamageType` order. Success and critical success remove that slot. Failure and critical failure keep the amount. There is no critical entry, so those degrees are the flat check's ordinary success and failure (Rules 2278).

Healing to maximum Hit Points removes Bleed slots only (Rules 2308). A shorter heal leaves them. Becoming Dead clears every persistent slot.

### Error mapping

`NonPositivePersistent` is the new `EncounterError`. `from_spend` maps it to `ReactiveError::NoReaction`. `as_damage_overflow` maps it to `ReactiveError::DamageOverflow(ModifierError::Overflow)`. `ready_from_encounter` maps it to `ReadyError::Encounter(ReadyEncounter::NonPositivePersistent)`. Each new variant has its own arm in all three matches.

### Proposed deviations

| ID | Rule | RAW citation | Our behavior | Reason |
|---|---|---|---|---|
| DEV-59 | Two weaknesses of one type | https://2e.aonprd.com/Rules.aspx?ID=2317 (PC 408) | Highest once, not the sum | The book adds each weakness once and does not say two values stack |
| DEV-60 | "Usually the highest" resistance | https://2e.aonprd.com/Rules.aspx?ID=2318 (PC 408) | Always the highest | No GM choice |

The code phase adds those rows and amends the persistent-damage wording of DEV-09 and DEV-29. DEV-33 through DEV-35 and DEV-47 through DEV-49 stay unused.

## Risks / Trade-offs

- [The conditions change defines `ConditionSlot` differently] → This spec states the assumption and the payload. A mismatch is a spec clarification on this pull request, not a second damage model.
- [`end_turn` taking dice touches every caller] → Callers pass the encounter dice. No persistent damage still draws nothing, so existing end-step results stay.
- [Existing Strike literals have no type] → The code phase sets `DamageType::Slashing` and empty defenses on those literals. Slashing keeps Shield Block available. Empty defenses leave the rolled total unchanged.

## Migration Plan

The spec commit adds no Rust, no test, and no changelog bullet. After `spec-approved: <sha>`, tests land, then the code they cover. The Unreleased bullets land in the code phase, before the archive commit. Archive is the last commit, with `openspec archive add-resistances-weaknesses-persistent --yes`, only after Architect and QA have both passed that head.

Rollback is the pull request itself. Nothing is archived in the spec commit.
