# api-facade

## ADDED Requirements

### Requirement: StrikeProfile records the attack basis

`StrikeProfile` SHALL keep its fields, including `damage_type`, and SHALL gain `attack_basis` and `damage_basis`. `attack_basis` SHALL be Strength melee, Dexterity melee, or ranged. `damage_basis` SHALL be Strength or not Strength.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=61 (Player Core pg. 442, Clumsy)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=71 (Player Core pg. 443, Enfeebled)

#### Scenario: A Strength melee profile keeps the old fields

- **GIVEN** attack bonus +4, not agile, one 8-faced damage die, damage bonus +1, damage type Slashing, attack basis Strength melee, and damage basis Strength
- **WHEN** an external crate builds that `StrikeProfile`
- **THEN** those fields read back as given

### Requirement: ConditionSlot gains one instance variant

`ConditionSlot::PersistentDamage` SHALL stay a damage type and an amount. This change SHALL add one instance variant and SHALL NOT add, replace, or remove a persistent slot. An instance SHALL expose its kind, its value when it has one, and its duration. A fresh combatant state SHALL still hold an empty list.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2455 (Player Core pg. 442, Appendix entry, Redundant Conditions)

#### Scenario: A fresh combatant has no conditions

- **GIVEN** a creature and CombatantId(1)
- **WHEN** `new_combatant_state` builds the state
- **THEN** the condition list is empty

#### Scenario: A stored frightened 2 can be read

- **GIVEN** frightened 2 until removed applied to a combatant
- **WHEN** an external crate reads that combatant's conditions
- **THEN** the slot's kind is frightened, its value is 2, and its duration is until removed

#### Scenario: Removing frightened leaves persistent damage

- **GIVEN** persistent Fire 5 and frightened 2 until removed on one combatant
- **WHEN** frightened is removed
- **THEN** frightened is absent
- **AND** persistent Fire 5 remains

## MODIFIED Requirements

### Requirement: Encounter commands stay public

The crate root SHALL export `begin_encounter` and `Encounter`. Methods public on add-resistances-weaknesses-persistent SHALL stay public. `end_turn` and `delay` SHALL take the injected dice. This change SHALL add `apply_condition`, `reduce_condition`, and `remove_condition`, and no other method. `gain_persistent_damage` SHALL stay.

#### Scenario: An external crate starts and ends one turn

- **GIVEN** hit points 30, armor class 11, perception 0, speed 25, one non-agile Slashing 1d8 strike of reach 5, empty defenses, no shield, no Stealth DC, no Aid modifier, no saves, and `SeededDice` seeded with 1
- **WHEN** an external crate builds that character with `CharacterBuild::try_new`, `derive`, and `new_combatant_state`, begins an encounter for `CombatantId(1)` on `Side::Pc`, then calls `start_turn` and `end_turn` with that dice
- **THEN** `begin_encounter` returns Ok, the round is 1, the order is that id, `start_turn` returns that id, and `end_turn` returns Ok
- **AND** the doctest imports only names from the public list

#### Scenario: The public method set is unchanged

- **GIVEN** `Encounter` after this change
- **WHEN** an external crate names its methods
- **THEN** these methods are public: `round`, `order`, `turn_open`, `state`, `creature`, `start_turn`, `end_turn`, `gain_persistent_damage`, `spend_actions`, `spend_free_action`, `spend_reaction`, `prepare_to_aid`, `aid`, `attack_trait_penalty`, `raise_shield`, `shield_block`, `strike`, `reactive_strike`, `initiative_total`, `turn_slot`, `next_start`, `stored_ready`, `frozen_attack_count`, `attack_count`, `basic_save`, `delay`, `return_from_delay`, `ready`, `readied_attack_penalty`, `resolve_reactions`, `stride`, `step`, `drop_prone`, `stand`, `apply_condition`, `reduce_condition`, and `remove_condition`
- **AND** `end_turn` and `delay` take the injected dice
- **AND** no other public method was added

### Requirement: The crate root exports only the facade names

Aside from `ScriptedDice` under its existing cfg, the crate root SHALL export exactly the names in the scenario below and no other name. A test that reads the crate root's `pub use` names SHALL fail when a name is added or missing.

#### Scenario: The public names are this set

- **GIVEN** the crate root, ignoring the cfg-gated `ScriptedDice` re-export
- **WHEN** its `pub use` names are collected
- **THEN** the set is `ActionCost`, `ActionIdentity`, `ActionTrait`, `Agility`, `AidCircumstance`, `AidOutcome`, `AidPreparation`, `AidRank`, `ArithmeticOverflow`, `AttackBasis`, `BonusType`, `BuildParts`, `CharacterBuild`, `CombatantId`, `CombatantState`, `Concealment`, `ConditionId`, `ConditionDuration`, `ConditionError`, `ConditionKind`, `ConditionSlot`, `CoverBenefit`, `Creature`, `DamageBasis`, `DamageDefenses`, `DamageType`, `Detection`, `DetectionError`, `Detections`, `DiagonalCounter`, `Dice`, `DieCount`, `DieCountError`, `DegreeOfSuccess`, `Disruption`, `DyingValue`, `DyingValueError`, `Encounter`, `EncounterError`, `FaceCountError`, `Faces`, `FactionId`, `GridError`, `HpError`, `InitiativeMove`, `LEVEL_RANGE`, `Level`, `LevelError`, `LostStrike`, `MapBounds`, `MeleeStrike`, `Mobility`, `Modifier`, `ModifierError`, `MoveAction`, `MoveOutcome`, `Occupant`, `OccupantPresence`, `ObservedAction`, `Participant`, `PenaltyType`, `PositioningError`, `Posture`, `PostureChange`, `Proficiency`, `ReachError`, `ReachFeet`, `ReactionField`, `ReactionPlace`, `ReactiveError`, `ReactiveOutcome`, `ReactiveStrike`, `ReactiveStrikeInput`, `ReadyActivity`, `ReadyEncounter`, `ReadyError`, `ReadyStrike`, `ReadySuccess`, `ReadyTrigger`, `Relation`, `Resistance`, `ResolvedReaction`, `Save`, `SaveRoll`, `SavingThrows`, `SeededDice`, `ShieldBlock`, `ShieldTemplate`, `Side`, `Size`, `SourceId`, `Square`, `SquareEvent`, `Statistic`, `StoredReady`, `StrikeDelivery`, `StrikeOutcome`, `StrikeProfile`, `StrikeRange`, `TemplateError`, `Terrain`, `TurnSlot`, `Vital`, `Weakness`, `begin_encounter`, `derive`, and `new_combatant_state`
- **AND** an external crate can import that whole set in one `use`
