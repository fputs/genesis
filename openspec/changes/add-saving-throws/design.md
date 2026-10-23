# Design

## Context

`degree_of_success` and `resolve_check` already apply the four bands and the natural 1 and natural 20 shift. `apply_resolved_blow` treats `Failure` and `CriticalFailure` as `HpError::NotAHit`, and it treats `CriticalSuccess` as the critical-blow dying step. An amount of 0 returns before that dying step. `Statistic::Reflex` and `reflex_modifiers` already exist. `Creature` has no save training. See proposal.md for why this change exists.

`Encounter::strike` compares with baked Armor Class, spends one action, and rolls damage through `damage_dealt`. `wire-targeting-into-strike` may change that function's signature. This change does not edit it. This change names no file on `v1-reference`. The spec commit does not edit `docs/designer/`.

## Goals / Non-Goals

**Goals:**

- One module, `crates/rules/src/save.rs`, for the three modifiers, the save DC, and the basic-save multiplier.

- `Encounter::basic_save` rolls with `resolve_check`. A positive amount uses `apply_resolved_blow`. A multiplied 0 does not.

- `Statistic::Fortitude` and `Statistic::Will` stay available for modifiers. They do not take the unconscious penalty.

**Non-Goals:**

- Editing `Encounter::strike`, `reactive_strike`, or a readied Strike. No flat check and no `Detections` argument.

- Calling `damage_dealt` or Shield Block. The listed amount is already final. DEV-14 (Rules 2302) and DEV-09 stay on Strike.

- Ability scores, Strength, Intelligence, Charisma, skills, class DC, spells, Grapple, incapacitation, and Hero Points.

- Evasion, Juggernaut, and Resolve. Monster Core listed saves.

- A fourth bonus type. Proficiency stays an addend. No `fortitude_modifiers` or `will_modifiers` producer.

## Decisions

### Types

`Proficiency` is `Untrained`, `Trained`, `Expert`, `Master`, or `Legendary`. Untrained is +0 (DEV-56). The other ranks are `level` plus 2, 4, 6, or 8.

`Level` is a validated newtype. `Level::new` accepts `1` through `2147483639` (`i32::MAX - 8`). Every other integer is `LevelError::OutOfRange`, so a level whose legendary bonus does not fit in `i32` cannot be represented. `proficiency_bonus` then returns `i32` and does not fail.

`SavingThrows` fields are `level: Level`, `constitution: i32`, `dexterity: i32`, `wisdom: i32`, `fortitude: Proficiency`, `reflex: Proficiency`, and `will: Proficiency`. Those are attribute modifiers, not scores.

`Save` is `Fortitude`, `Reflex`, or `Will`. `Save::statistic` maps those to `Statistic::Fortitude`, `Statistic::Reflex`, and `Statistic::Will`. `basic_save` takes `Save`, not `Statistic`.

```rust
pub enum Proficiency {
    Untrained,
    Trained,
    Expert,
    Master,
    Legendary,
}

pub struct Level(i32);

impl Level {
    pub fn new(level: i32) -> Result<Self, LevelError>;
    pub fn get(self) -> i32;
}

pub enum LevelError {
    OutOfRange,
}
```

```rust
pub struct SavingThrows {
    pub level: Level,
    pub constitution: i32,
    pub dexterity: i32,
    pub wisdom: i32,
    pub fortitude: Proficiency,
    pub reflex: Proficiency,
    pub will: Proficiency,
}

pub enum Save {
    Fortitude,
    Reflex,
    Will,
}

impl Save {
    pub fn statistic(self) -> Statistic;
}
```

```rust
pub fn proficiency_bonus(level: Level, rank: Proficiency) -> i32;

pub enum SaveError {
    NegativeAmount,
    ModifierOverflow(ArithmeticOverflow),
    DoubleOverflow(ArithmeticOverflow),
}

pub fn save_modifier(
    training: &SavingThrows,
    save: Save,
    unconscious: bool,
    modifiers: &[Modifier],
) -> Result<CheckModifier, SaveError>;

pub fn save_dc(modifier: CheckModifier) -> i32;

pub fn basic_save_amount(
    amount: i32,
    degree: DegreeOfSuccess,
) -> Result<i32, SaveError>;
```

```rust
pub struct SaveRoll {
    pub natural: u8,
    pub total: i32,
    pub degree: DegreeOfSuccess,
    pub damage: i32,
}

impl Encounter {
    pub fn basic_save(
        &mut self,
        defender: CombatantId,
        save: Save,
        dc: i32,
        amount: i32,
        modifiers: &[Modifier],
        dice: &mut impl Dice,
    ) -> Result<SaveRoll, EncounterError>;
}
```

`EncounterError::NoSavingThrows` and `EncounterError::SaveOverflow(ArithmeticOverflow)` are the encounter errors. The items above are re-exported from the crate root. `basic_save` is a method on the exported `Encounter`.

`BuildParts`, `Contribution`, and `Creature` gain `saves: Option<SavingThrows>`. `None` is copied through. A contribution replaces training when `Some`, as a shield does. `derive` copies the option and does not roll.

Alternative considered: a required block on every creature. Rejected. Existing fixtures have no saves, and `None` is the typed "cannot attempt" state.

Alternative considered: store an ability score and use `(score - 10) / 2`. Rejected. Player Core stores the attribute modifier. That division is not this book's procedure.

Alternative considered: pass `Statistic` into `basic_save`. Rejected. Attack, Armor Class, Perception, and Damage are not saves. `Save` has the three values.

### Modifier formula

`save_modifier` adds the attribute modifier, `proficiency_bonus`, and `total` of caller modifiers on `save.statistic()`. `unconscious` is true for `Dying` and `Unconscious`. Only Reflex then includes `reflex_modifiers(true)`. Fortitude and Will do not. A caller modifier on another statistic is ignored. Proficiency is not a `Modifier`.

The sum must fit `CheckModifier`. That failure is `SaveError::ModifierOverflow`, and the encounter reports `SaveOverflow` before the die. A fitting `CheckModifier` always fits `DcModifier`, so `save_dc` returns `dc_from_modifier` of that value and does not fail.

### Basic-save amount and the blow

`basic_save_amount` maps the degree: critical success to 0, success to half rounding down, failure to the amount, critical failure to double. Half of 1 stays 1 (Rules 2307). A critical success of 0 is exempt from that minimum. Half of 0 stays 0. Double of 0 stays 0. A negative amount is `SaveError::NegativeAmount`. A double that does not fit is `SaveError::DoubleOverflow`.

This change closes R-02. The deferred halving sentence is now this rule. The Rules 2302 floor stays on Strike (DEV-14). A listed 0 is not raised to 1.

When the multiplied amount is 0, the encounter returns that roll and does not call `apply_resolved_blow`. Hit Points and temporary Hit Points are not written. Dying does not increase. There is no knockout (Rules 2297, Rules 2327).

A positive amount uses this translation. `Failure` must not be passed through. That value is `NotAHit` today. `CriticalSuccess` selects the existing dying step of 2 and does not double the amount again.

| Save degree | Amount | Blow |
|---|---|---|
| Critical success | 0 | skipped |
| Success | half | Success, or skipped at 0 |

| Save degree | Amount | Blow |
|---|---|---|
| Failure | listed amount | Success, or skipped at 0 |
| Critical failure | double | CriticalSuccess, or skipped at 0 |

Alternative considered: teach `apply_resolved_blow` about saves. Rejected. Strike still passes a miss as `Failure`.

### Encounter order

The encounter checks, in order: `UnknownCombatant`, `TargetDead`, `NegativeAmount`, `NoSavingThrows`, then `SaveOverflow`. Those draw no die. It then calls `resolve_check`. A double that does not fit returns `DamageOverflow` and writes no Hit Points. A multiplied 0 returns the roll and skips the blow. Any other amount applies the translated blow. The save does not spend an action, the reaction, or an attack count.

While a turn is open, every knockout moves the defender to directly before the open creature. That is the turn in which the defender hit 0 HP (Rules 2324, FIX-03, R-08). The move applies whether or not the defender had Delayed. The code today returns early unless `TurnState::DelayedTarget`. The code phase removes that gate and uses the open creature as the anchor. When no turn is open, there is no such turn, so the order stays. A multiplied 0 is not a knockout.

Alternative considered: move only a delayed knockout. Rejected. FIX-03 is every reduction to 0 HP.

### Statistics

Add `Statistic::Fortitude` and `Statistic::Will` after Reflex. Their sort keys are 5 and 6, so Reflex stays 4. Extend the modifier groups so those statistics do not share a stack. Do not add `fortitude_modifiers` or `will_modifiers`. `reflex_modifiers` stays the status penalty of -4 at sequence 10.

### Error mapping

`ready_from_encounter` maps `NoSavingThrows` to `ReadyError::Encounter(ReadyEncounter::NoSavingThrows)`. It maps `SaveOverflow(error)` to `ReadyError::Encounter(ReadyEncounter::SaveOverflow(error))`.

`from_spend` maps `NoSavingThrows` to `ReactiveError::NoReaction`. It maps `SaveOverflow(_)` to `ReactiveError::NoReaction`.

`as_damage_overflow` maps `NoSavingThrows` to `ReactiveError::DamageOverflow(ModifierError::Overflow)`. It maps `SaveOverflow(_)` to that same damage overflow. Each new variant has its own arm in all three matches.

### Proposed deviations

| ID | Rule | RAW citation | Our behavior | Reason | Owning change |
|---|---|---|---|---|---|
| DEV-56 | A save may be untrained | https://2e.aonprd.com/Rules.aspx?ID=2296 (PC 404) | `Untrained` is legal and is +0 | No class grants trained | add-saving-throws |

| ID | Rule | RAW citation | Our behavior | Reason | Owning change |
|---|---|---|---|---|---|
| DEV-57 | Shield Block on a basic save | https://2e.aonprd.com/Feats.aspx?ID=5212 (PC 262) | `basic_save` never calls the shield path | See the note under this table | add-saving-throws |

Shield Block (Feats 5212) only matters for an attack that calls for a save. A basic save is not that attack. Strike owns the block.

| ID | Rule | RAW citation | Our behavior | Reason | Owning change |
|---|---|---|---|---|---|
| DEV-58 | Three attributes. Amends DEV-01 | https://2e.aonprd.com/Rules.aspx?ID=2296 (PC 404) | Constitution, Dexterity, and Wisdom only | Saves, not full derivation | add-saving-throws |

DEV-47 and DEV-49 are not entries, only unused, do not reuse. DEV-33 to DEV-35 are the same. DEV-52 through DEV-55 are add-delay-and-ready. They are not reused here.

## Risks / Trade-offs

- [Miss degree on a failed save] → Translate before `apply_resolved_blow`. The dying scenarios lock 1 and 2.

- [Half of 1 becomes 0] → The halving scenario locks damage 1. A critical success of 0 stays 0.

- [Strike's signature changes in another change] → This change does not edit `Encounter::strike`.

- [A listed 0 still calls the blow] → A multiplied 0 does not call `apply_resolved_blow`. Dying stays, and there is no knockout.

- [A new error has no match arm] → The three conversions each name `NoSavingThrows` and `SaveOverflow`.

## Migration Plan

The implementation adds `saves: None` at every current `BuildParts`, `Contribution`, and `Creature` literal, then sets `Some` only in the new tests. No saved data exists. Revert that commit to roll back. The spec commit has no Rust change.
