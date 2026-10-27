# Design

## Context

This change depends on pull request #93, `add-resistances-weaknesses-persistent`, which merges first. That change owns `ConditionSlot`, the `PersistentDamage { damage_type, amount }` variant, and one end-of-turn hook that runs before decrements. `end_turn` already takes the injected dice. This change does not replace that enum, that variant, or that hook. See proposal.md for why the named conditions exist.

`ConditionId` already has `FRIGHTENED` = 2, `SICKENED` = 3, and `UNCONSCIOUS` = 4. A private `ConditionId(1)` is the off-guard id inside `ac_modifiers`. `total` already keeps the highest bonus and the worst penalty of one type, and it keeps both a bonus and a penalty of that type. Frightened and sickened status penalties are validated at construction (R-03, R-04). Different condition ids do not share a duplicate key; the type bucket still keeps one status penalty. `Encounter::strike` compares the attack total with baked Armor Class and does not read the target's conditions. `basic_save` passes only the caller slice into `save_modifier`. Initiative adds Perception to the d20. No file on `v1-reference` is named. The spec commit does not edit `docs/designer/`.

## Goals / Non-Goals

**Goals:**

- One module, `crates/rules/src/conditions.rs`, for the instance, the duration, the pure ticks, and the modifiers those instances contribute.

- `Encounter::apply_condition`, `reduce_condition`, and `remove_condition` are the only new methods. Start, end, Delay, initiative, Strike, and `basic_save` call the module. They do not grow a second stacker.

**Non-Goals:**

- Immunity, weakness, resistance, the persistent blow, and the DC 15 flat check. Those stay in #93. Skills, including Retch and Athletics. Spells and areas. Stunned, slowed, grabbed, blinded, drained, doomed, and the rest of the appendix.

- Moving unconscious or dying into the vec. `Vital` stays the source of the unconscious −4 (DEV-31). Strike still does not read `Vital` (DEV-36).

- Temporary Hit Points. The start tick does not read `temp_rounds` (DEV-32 stays unwired).

- Flat checks. They are not a `Statistic`, and this change adds no modifier on them.

- Per-attacker off-guard. Flanking stays outside the tracker. A tracked off-guard and a caller off-guard still collapse to one −2.

- A thrown weapon whose attack roll is Strength-based. `AttackBasis::Ranged` takes clumsy and not enfeebled. Phase 1 has no thrown profile.

- Crawl, and greater cover from prone. Those are DEV-73. Drop Prone still does not write the condition list.

- A new crate, a new dependency, or a port from `v1-reference`.

## Decisions

### Types

`ConditionKind` is `Frightened`, `Sickened`, `OffGuard`, `Prone`, `Clumsy`, or `Enfeebled`. That is the whole set. A later change adds a variant; this change does not use a trait object or a callback.

`ConditionDuration` is `UntilRemoved`, `Rounds { remaining, on }`, or `UntilEndOfNextTurn { on, due }`. `remaining` is 1 or more at construction. `due` is true when the clock creature's turn is not open. The end tick removes a due instance. An instance that is not due becomes due and stays.

`ConditionSlot` stays the enum from #93. This change adds one variant and does not edit `PersistentDamage`.

```rust
pub enum ConditionSlot {
    PersistentDamage { damage_type: DamageType, amount: i32 },
    Instance {
        kind: ConditionKind,
        value: Option<u32>,
        duration: ConditionDuration,
    },
}
```

`Instance` fields are private. `try_new` is the constructor and returns `ConditionSlot::Instance`. `kind`, `value`, and `duration` read that variant. `PersistentDamage` has none of those. Apply, reduce, remove, and both ticks skip `PersistentDamage`. They do not add, replace, or remove one. An external crate can read an instance and cannot build an illegal one with a struct literal.

```rust
pub enum ConditionKind {
    Frightened,
    Sickened,
    OffGuard,
    Prone,
    Clumsy,
    Enfeebled,
}

pub enum ConditionDuration {
    UntilRemoved,
    Rounds { remaining: u32, on: CombatantId },
    UntilEndOfNextTurn { on: CombatantId, due: bool },
}

pub enum ConditionError {
    ValueRequired,
    EndedValue,
    ValueForbidden,
    EmptyDuration,
}

impl ConditionSlot {
    pub fn try_new(
        kind: ConditionKind,
        value: Option<u32>,
        duration: ConditionDuration,
    ) -> Result<Self, ConditionError>;
    pub fn kind(&self) -> Option<ConditionKind>;
    pub fn value(&self) -> Option<u32>;
    pub fn duration(&self) -> Option<ConditionDuration>;
    pub fn condition_id(self) -> ConditionId;
}
```

Valued kinds are frightened, sickened, clumsy, and enfeebled. They require `Some` of 1 or more. `None` is `ValueRequired`. `Some(0)` is `EndedValue`. Off-guard and prone require `None`. Any `Some` is `ValueForbidden`. `Rounds` with `remaining` 0 is `EmptyDuration`. `try_new` does not tick.

`ConditionId::OFF_GUARD` is 1, `PRONE` is 5, `CLUMSY` is 6, and `ENFEEBLED` is 7. The private `ConditionId(1)` in `modifiers.rs` becomes `ConditionId::OFF_GUARD`. Do not add `with_condition` checks for the new ids. Tests already build `ConditionId(5)` and `ConditionId(6)` as ordinary modifiers.

```rust
pub enum AttackBasis {
    StrengthMelee,
    DexterityMelee,
    Ranged,
}
```

`StrikeProfile` gains `pub basis: AttackBasis` and keeps `damage_type` from #93. Every literal that this change touches sets `StrengthMelee` unless the scenario names another basis, and leaves `damage_type` as #93 set it. The `strike` signature does not change. `end_turn` keeps the dice argument. `gain_persistent_damage` stays.

Alternative considered: store one slot per name and overwrite it. Rejected. A lower value can outlast a higher one, so both instances stay (Rules 2455).

Alternative considered: a public `due` that `apply_condition` trusts. Rejected. The encounter can see the open turn and the caller cannot. `apply_condition` writes `due` as `self.open != Some(on)` and ignores the flag on the argument. Pure tests build the flag they mean through `try_new`.

### Lists

`apply` appends. It does not replace. `active_value` is the highest stored value of a valued kind, or absent when none remain. An unvalued kind is present while any instance of that name remains. Modifiers use that active value once, not once per instance.

`reduce(kind, by)` rejects off-guard and prone with `ValueForbidden`, including a reduction of 0. For a valued kind, `by == 0` changes nothing, including when the kind is absent. Any other `by` subtracts from every instance of that name. `checked_sub` that does not fit, or a result of 0, ends that instance. Other names stay. `remove(kind)` drops every instance of that name. Removing an absent name changes nothing.

`start_tick(list, clock)` subtracts 1 from each `Rounds` whose `on` is `clock`, on that list only. Remaining 0 ends the instance. Other durations stay. Frightened does not decrease.

`end_tick(list, bearer, clock)` does two steps, in this order. First it resolves `UntilEndOfNextTurn` whose `on` is `clock`: a due instance is removed, and an instance that is not due becomes due and stays. Then, only when `bearer == clock`, it reduces frightened by 1. Sickened, clumsy, and enfeebled do not decrease. The order is load-bearing: a due frightened 3 removed before the decrease leaves a frightened 2, which then becomes 1. Decreasing first would leave a different value.

Neither tick rolls, reads a clock, or writes Hit Points, temporary Hit Points, or wounded.

The encounter calls each tick once per bearer, because a duration clocked to creature A can sit on creature B. The pure functions see one list.

Alternative considered: delete the lower instance when a higher one is applied. Rejected. Rules 2455 keeps the longer duration.

### Modifiers

`condition_modifiers(list, statistic, basis)` returns the modifiers for one statistic. `basis` is `None` except for `Statistic::Attack` and `Statistic::Damage`. The result goes through the existing `total`. Level is `None`. Sources are new `ModifierSource` variants, so a caller `SourceId` cannot name them. Sequences are stable: frightened 20, sickened 21, clumsy 22, enfeebled 23, off-guard 24, prone's attack penalty 25. Sort keys continue after `RaisedShield` (11), in that same order. `source_ord` and its test copy both gain an arm.

| Kind | Statistic | Basis | Modifier |
|---|---|---|---|
| Frightened | Attack, Armor Class, Perception, Fortitude, Reflex, Will | any | status −value, `FRIGHTENED` |
| Sickened | those six | any | status −value, `SICKENED` |
| Clumsy | Armor Class, Reflex | | status −value, `CLUMSY` |
| Clumsy | Attack | Dexterity melee or ranged | status −value, `CLUMSY` |
| Enfeebled | Attack and Damage | Strength melee | status −value, `ENFEEBLED` |
| Off-guard | Armor Class | | circumstance −2, `OFF_GUARD`, no rank |
| Prone | Armor Class | | circumstance −2, `OFF_GUARD`, no rank |
| Prone | Attack | any | circumstance −2, `PRONE`, no rank |

Frightened and sickened do not touch damage. Clumsy does not touch Fortitude, Will, Perception, damage, or a Strength melee attack. Enfeebled does not touch Armor Class, ranged attacks, Dexterity melee, or the three saves. Prone's Armor Class penalty uses `OFF_GUARD` so it groups with off-guard and with `ac_modifiers`. A value that does not fit `i32` is `ModifierError::Overflow`.

Alternative considered: emit one pre-stacked penalty. Rejected. A caller frightened 1 has to compete with a tracked frightened 2 inside `total`, and a status bonus has to survive beside the penalty (Rules 2281, Rules 2282).

### Encounter

```rust
impl Encounter {
    pub fn apply_condition(
        &mut self,
        id: CombatantId,
        kind: ConditionKind,
        value: Option<u32>,
        duration: ConditionDuration,
    ) -> Result<(), EncounterError>;

    pub fn reduce_condition(
        &mut self,
        id: CombatantId,
        kind: ConditionKind,
        by: u32,
    ) -> Result<(), EncounterError>;

    pub fn remove_condition(
        &mut self,
        id: CombatantId,
        kind: ConditionKind,
    ) -> Result<(), EncounterError>;
}
```

An unknown bearer is `UnknownCombatant` and stores nothing. `Rounds` and `UntilEndOfNextTurn` also require `on` to be in the encounter; an unknown clock is `UnknownCombatant` and stores nothing. `ConditionError` maps to `EncounterError::Condition`. Apply does not tick and does not spend an action. `ReadyEncounter` gains `Condition(ConditionError)`. The ready match maps `EncounterError::Condition` to that variant. `from_spend` maps it to `ReactiveError::NoReaction`. `as_damage_overflow` maps it to `ReactiveError::DamageOverflow(ModifierError::Overflow)`. Each match names the variant.

`start_turn` ticks round durations for each dead id it skips, before `open_at`. That tick does not open a turn, does not decrease frightened, and does not run the end tick. `start_of_turn_step` then ticks the creature whose turn is actually opening, before actions are regained, and still lowers a raised shield. A `SkipStartHook` start does not tick. Delay does not tick round durations: the start already did.

The persistent hook from #93 runs to completion first. The decrement is the next step, not a second hook and not a step in front of that hook. It runs unless the slot is `Returned` or `SkipStartHook`. Delay calls the end step while the slot is still `Normal`, so the hook and then the decrement run once there. `end_turn` on the turn return opened sees `Returned` or `SkipStartHook` and does not decrement again. `close_actions` then sets the slot to `Normal`, so the next ordinary end decrements. This is the Actions 2294 reading: effects that belong at the end of the delayed turn happen at Delay, and that same turn does not run them a second time. A missed return uses the ordinary start and ticks round durations once. The decrement does not roll and does not apply the persistent blow.

Initiative, before any d20, adds the Perception condition total to Perception. The existing `perception + 20` check uses that sum. The stored total is the d20 plus Perception plus that total. No conditions leaves the total unchanged.

Strike concatenates the attacker's attack modifiers for `profile.basis` with the caller slice. Armor Class is the baked value plus the target's Armor Class condition total. A caller Armor Class modifier still does not apply. Damage concatenates the attacker's damage modifiers for that basis. A `ModifierError` is `DamageOverflow` and draws no attack d20. The flat check is unchanged and is not a condition statistic.

`basic_save` concatenates the defender's modifiers for that save's statistic with the caller slice, then calls `save_modifier` as it does today. The unconscious Reflex −4 still comes from `save_modifier`, not from this vec.

`drop_prone`, `stand`, `stride`, `step`, and Take Cover do not read or write `conditions`.

Rules 2430 places persistent damage before the frightened decrease. #93 owns that hook. This change does not insert another one. A later condition kind is a new `ConditionKind` variant, not a new container.

### Proposed deviations

| ID | Rule | RAW citation | Our behavior | Reason | Owning change |
|---|---|---|---|---|---|
| DEV-73 | Prone restricts movement and grants greater cover | https://2e.aonprd.com/Conditions.aspx?ID=88 (PC 445) | The condition applies its penalties. It does not change Stride, Step, Drop Prone, Stand, or Take Cover, and it does not grant cover | No Crawl in Phase 1. Posture stays caller-held | add-conditions-with-durations |

DEV-29, DEV-31, DEV-32, and DEV-36 stay as they are. Sickened's Retch is a non-goal, not a deviation: the condition does not decrease by itself, and `reduce_condition` is what a later action will call. DEV-33 to DEV-35 and DEV-47 to DEV-49 stay unused. Do not reuse them. Phase 2 reserves DEV-59 through DEV-61 for skills, DEV-62 and DEV-63 for resistances, and DEV-64 through DEV-72 for spells. This change does not use DEV-59 through DEV-72. Its first number is DEV-73.

## Risks / Trade-offs

- [The decrement runs before the persistent blow] → The call sits after the hook from #93 returns. The encounter scenario locks the blow's Hit Points and the frightened decrease on the same end.

- [A returned turn decreases frightened twice] → Delay runs the hook and the decrement while the slot is `Normal`. The returned end sees `Returned` or `SkipStartHook` and skips the decrement. The scenario locks frightened 2 down to 1, then to absent on the next ordinary end.

- [A dead caster's rounds never move] → The skip loop ticks that clock before `open_at`. The dead id does not start a turn. The scenario locks 2 rounds down to 1.

- [Frightened 2 and sickened 1 become −3] → Both modifiers are emitted and `total` keeps the worst status penalty. The attack and Armor Class scenarios lock −2.

- [Prone and off-guard become −4 to Armor Class] → Both penalties use `ConditionId::OFF_GUARD`. The scenario locks one −2.

- [Existing strike totals change] → Every current `StrikeProfile` literal sets `StrengthMelee`. With no conditions the attack and damage totals stay as they are.

- [A new error has no match arm] → `ready_from_encounter`, `from_spend`, and `as_damage_overflow` each name `EncounterError::Condition`.

## Migration Plan

The code phase adds `basis: AttackBasis::StrengthMelee` on every current `StrikeProfile` literal, including the doctest, and adds one arm to each `EncounterError` match. No saved data exists. Revert that commit to roll back. The spec commit has no Rust change. DEV-73 is written into `docs/designer/rules-deviations.md` in the code phase, not in the spec commit. That commit does not write DEV-59 through DEV-72.
