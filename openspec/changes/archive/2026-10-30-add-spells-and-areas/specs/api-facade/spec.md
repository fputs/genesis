# api-facade

## MODIFIED Requirements

### Requirement: Encounter commands stay public

The crate root SHALL export `begin_encounter` and `Encounter`. Every `Encounter` method that is public today SHALL stay public with the same signature. The only methods added are `breathe_fire` and `telekinetic_projectile`. A command's result SHALL stay that method's current return type.

#### Scenario: An external crate starts and ends one turn

- **GIVEN** hit points 30, armor class 11, perception 0, speed 25, one non-agile 1d8 strike of reach 5, no shield, no Stealth DC, no Aid modifier, no saves, no spellcasting, and `SeededDice` seeded with 1
- **WHEN** an external crate builds that character with `CharacterBuild::try_new`, `derive`, and `new_combatant_state`, begins an encounter for `CombatantId(1)` on `Side::Pc`, then calls `start_turn` and `end_turn`
- **THEN** `begin_encounter` returns Ok, the round is 1, the order is that id, `start_turn` returns that id, and `end_turn` returns Ok
- **AND** the doctest imports only names from the public list

#### Scenario: The public method set is unchanged

- **GIVEN** `Encounter` before this change
- **WHEN** an external crate names its methods
- **THEN** these methods are still public with the same signatures: `round`, `order`, `turn_open`, `state`, `creature`, `start_turn`, `end_turn`, `spend_actions`, `spend_free_action`, `spend_reaction`, `prepare_to_aid`, `aid`, `attack_trait_penalty`, `raise_shield`, `shield_block`, `strike`, `reactive_strike`, `initiative_total`, `turn_slot`, `next_start`, `stored_ready`, `frozen_attack_count`, `attack_count`, `basic_save`, `delay`, `return_from_delay`, `ready`, `readied_attack_penalty`, `resolve_reactions`, `stride`, `step`, `drop_prone`, `stand`, `breathe_fire`, and `telekinetic_projectile`
- **AND** no public method outside that list exists

### Requirement: The crate root exports only the facade names

Aside from `ScriptedDice` under its existing cfg, the crate root SHALL export exactly the names in the scenario below and no other name. A test that reads the crate root's `pub use` names SHALL fail when a name is added or missing.

#### Scenario: The public names are this set

- **GIVEN** the crate root, ignoring the cfg-gated `ScriptedDice` re-export
- **WHEN** its `pub use` names are collected
- **THEN** the set is `ActionCost`, `ActionIdentity`, `ActionTrait`, `Agility`, `AidCircumstance`, `AidOutcome`, `AidPreparation`, `AidRank`, `Aim`, `AreaCreature`, `ArithmeticOverflow`, `BonusType`, `BreatheFireOutcome`, `BuildParts`, `CharacterBuild`, `CombatantId`, `CombatantState`, `Concealment`, `ConditionId`, `ConditionSlot`, `CoverBenefit`, `Creature`, `Detection`, `DetectionError`, `Detections`, `DiagonalCounter`, `Dice`, `DieCount`, `DieCountError`, `DegreeOfSuccess`, `Disruption`, `DyingValue`, `DyingValueError`, `Encounter`, `EncounterError`, `FaceCountError`, `Faces`, `FactionId`, `GridError`, `HpError`, `InitiativeMove`, `LEVEL_RANGE`, `Level`, `LevelError`, `LostStrike`, `MapBounds`, `MeleeStrike`, `Mobility`, `Modifier`, `ModifierError`, `MoveAction`, `MoveOutcome`, `Occupant`, `OccupantPresence`, `ObservedAction`, `Participant`, `PenaltyType`, `PositioningError`, `Posture`, `PostureChange`, `Proficiency`, `ReachError`, `ReachFeet`, `ReactionField`, `ReactionPlace`, `ReactiveError`, `ReactiveOutcome`, `ReactiveStrike`, `ReactiveStrikeInput`, `ReadyActivity`, `ReadyEncounter`, `ReadyError`, `ReadyStrike`, `ReadySuccess`, `ReadyTrigger`, `Relation`, `ResolvedReaction`, `Save`, `SaveRoll`, `SavingThrows`, `SeededDice`, `ShieldBlock`, `ShieldTemplate`, `Side`, `Size`, `SourceId`, `SpellAttackHit`, `SpellAttackOutcome`, `Spellcasting`, `Square`, `SquareEvent`, `Statistic`, `StoredReady`, `StrikeDelivery`, `StrikeOutcome`, `StrikeProfile`, `StrikeRange`, `TemplateError`, `Terrain`, `TurnSlot`, `Vital`, `begin_encounter`, `derive`, and `new_combatant_state`
- **AND** an external crate can import that whole set in one `use`
