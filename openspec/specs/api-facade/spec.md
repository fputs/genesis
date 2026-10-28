# api-facade Specification

## Purpose
Names the rules crate's public surface: encounter commands, the values they return, and the types a caller must build. Every other current root export becomes crate-private, with no change to a fight result.

## Requirements

### Requirement: Encounter commands stay public

The crate root SHALL export begin_encounter and Encounter. Every method public before these changes SHALL stay public with the same signature, except end_turn, ready, and delay SHALL take the injected dice. skill_check and recall_knowledge SHALL return SkillRoll. gain_persistent_damage SHALL stay. SHALL add apply_condition, reduce_condition, and remove_condition, and SHALL NOT add any other method, event enum, or command enum. A command's result SHALL stay its current return type.

#### Scenario: An external crate starts and ends one turn

- **GIVEN** hit points 30, armor class 11, perception 0, speed 25, one non-agile Slashing 1d8 strike of reach 5, empty defenses, no shield, no Stealth DC, no Aid modifier, no saves, no skills, and `SeededDice` seeded with 1
- **WHEN** an external crate builds that character with `CharacterBuild::try_new`, `derive`, and `new_combatant_state`, begins an encounter for `CombatantId(1)` on `Side::Pc`, then calls `start_turn` and `end_turn` with that dice
- **THEN** `begin_encounter` returns Ok, the round is 1, the order is that id, `start_turn` returns that id, and `end_turn` returns Ok
- **AND** the doctest imports only names from the public list

#### Scenario: The public method set is unchanged

- **GIVEN** `Encounter` after this change
- **WHEN** an external crate names its methods
- **THEN** these methods are public: `round`, `order`, `turn_open`, `state`, `creature`, `start_turn`, `end_turn`, `gain_persistent_damage`, `spend_actions`, `spend_free_action`, `spend_reaction`, `prepare_to_aid`, `aid`, `attack_trait_penalty`, `raise_shield`, `shield_block`, `strike`, `reactive_strike`, `initiative_total`, `turn_slot`, `next_start`, `stored_ready`, `frozen_attack_count`, `attack_count`, `basic_save`, `delay`, `return_from_delay`, `ready`, `readied_attack_penalty`, `resolve_reactions`, `stride`, `step`, `drop_prone`, `stand`, `skill_check`, `recall_knowledge`, `apply_condition`, `reduce_condition`, and `remove_condition`
- **AND** `end_turn`, `delay`, and `ready` take the injected dice
- **AND** no other public method was added

### Requirement: The crate root exports only the facade names

Aside from `ScriptedDice` under its existing cfg, the crate root SHALL export exactly the names in the scenario below and no other name. A test that reads the crate root's `pub use` names SHALL fail when a name is added or missing.

#### Scenario: The public names are this set

- **GIVEN** the crate root, ignoring the cfg-gated `ScriptedDice` re-export
- **WHEN** its `pub use` names are collected
- **THEN** the set is `ActionCost`, `ActionIdentity`, `ActionTrait`, `Agility`, `AidCircumstance`, `AidOutcome`, `AidPreparation`, `AidRank`, `ArithmeticOverflow`, `AttackBasis`, `BonusType`, `BuildParts`, `CharacterBuild`, `CombatantId`, `CombatantState`, `Concealment`, `ConditionId`, `ConditionDuration`, `ConditionError`, `ConditionKind`, `ConditionSlot`, `CoverBenefit`, `Creature`, `DamageBasis`, `DamageDefenses`, `DamageType`, `Detection`, `DetectionError`, `Detections`, `DiagonalCounter`, `Dice`, `DieCount`, `DieCountError`, `DegreeOfSuccess`, `Disruption`, `DyingValue`, `DyingValueError`, `Encounter`, `EncounterError`, `FaceCountError`, `Faces`, `FactionId`, `GridError`, `HpError`, `InitiativeMove`, `LEVEL_RANGE`, `Level`, `LevelError`, `LostStrike`, `MapBounds`, `MeleeStrike`, `Mobility`, `Modifier`, `ModifierError`, `MoveAction`, `MoveOutcome`, `Occupant`, `OccupantPresence`, `ObservedAction`, `Participant`, `PenaltyType`, `PositioningError`, `Posture`, `PostureChange`, `Proficiency`, `ReachError`, `ReachFeet`, `ReactionField`, `ReactionPlace`, `ReactiveError`, `ReactiveOutcome`, `ReactiveStrike`, `ReactiveStrikeInput`, `ReadyActivity`, `ReadyEncounter`, `ReadyError`, `ReadyStrike`, `ReadySuccess`, `ReadyTrigger`, `Relation`, `Resistance`, `ResolvedReaction`, `Save`, `SaveRoll`, `SavingThrows`, `SeededDice`, `ShieldBlock`, `ShieldTemplate`, `Side`, `Size`, `Skill`, `SkillRoll`, `Skills`, `SourceId`, `Square`, `SquareEvent`, `Statistic`, `StoredReady`, `StrikeDelivery`, `StrikeOutcome`, `StrikeProfile`, `StrikeRange`, `TemplateError`, `Terrain`, `TurnSlot`, `Vital`, `Weakness`, `begin_encounter`, `derive`, and `new_combatant_state`
- **AND** an external crate can import that whole set in one `use`

### Requirement: Public types keep their public items

A type on the public list SHALL keep each public field, associated constant, and method it has today. This change SHALL NOT narrow one of those to `pub(crate)`.

#### Scenario: Hit points and a condition id stay readable

- **GIVEN** a `CombatantState` from `Encounter::state` and the `ConditionId` type
- **WHEN** an external crate reads `hit_points` and names `ConditionId::FRIGHTENED`
- **THEN** both are public

### Requirement: Removed root names become pub(crate)

Each name in the scenario below SHALL be `pub(crate)` and SHALL NOT appear at the crate root. There SHALL be no public alias for it. An external `use rules::<name>;` SHALL fail to compile with E0432. One `compile_fail,E0432` doctest SHALL cover each name.

#### Scenario: Each removed name is rejected at the root

- **GIVEN** an external crate
- **WHEN** it imports one of `Area`, `AttackBlowError`, `AttackKind`, `AttackTotalTerm`, `CheckModifier`, `CheckRoll`, `CombatantPos`, `Contribution`, `Cover`, `DcModifier`, `DetectionCost`, `EncounterOutcome`, `EndReason`, `FlatRoll`, `FoldError`, `HideObserver`, `HideResult`, `MapInput`, `ModifierTotal`, `MoveInput`, `NearbyFeature`, `OffGuard`, `OpenRoll`, `RangeBand`, `Readiness`, `RosterEntry`, `SaveError`, `SeekResult`, `ShieldError`, `SizeData`, `Space`, `StrikeNotice`, `TakeCoverResult`, `TargetingResult`, `ac_modifiers`, `adjacent`, `aid_check`, `aid_circumstance_bonus`, `apply_attack_blow`, `apply_dice_damage`, `apply_healing`, `apply_resolved_blow`, `armor_class`, `attack_check_total`, `attack_modifiers`, `attack_penalty`, `base_contribution`, `basic_save_amount`, `contributions`, `damage_dealt`, `dc_from_modifier`, `dc_from_total`, `degree_of_success`, `dismiss`, `dismiss_cost`, `distance`, `drop_prone`, `encounter_outcome`, `end_benefit`, `fallback_unlisted_critical`, `flanks`, `flat_check`, `flat_checks_for_same_thing`, `fold_contributions`, `grant_temporary_hit_points`, `hide`, `in_reach`, `leap_while_prone`, `lower_raised_shield`, `move_trait_allowed`, `multiple_attack_penalty`, `note_action`, `note_strike`, `off_guard`, `off_guard_against`, `perception_modifiers`, `proficiency_bonus`, `prone_move`, `raise_shield`, `range_band`, `ready_cost`, `reflex_modifiers`, `refresh`, `resolve_check`, `resolve_targeting`, `save_dc`, `save_modifier`, `seek`, `shield_ac_modifier`, `size_data`, `stand`, `start_of_turn`, `stealth_dc_from_modifier`, `step`, `stride`, `take_cover`, `take_initiative_move`, `total`, or `validate_speed`
- **THEN** rustc reports E0432 for that import
- **AND** the item's own visibility is `pub(crate)`

### Requirement: Methods keep a name a free function loses

`Encounter::stride`, `Encounter::step`, `Encounter::drop_prone`, `Encounter::stand`, and `Encounter::raise_shield` SHALL stay public. Only the free functions of those five names leave the crate root.

#### Scenario: The encounter Stride method stays callable

- **GIVEN** an external crate that can name `Encounter`
- **WHEN** it names `Encounter::stride`
- **THEN** that method is public
- **AND** `use rules::stride;` fails with E0432

### Requirement: No public module

The crate root SHALL NOT expose a public module. An external path through a child module SHALL fail with E0603. This requirement names the path, not which file defines the item.

#### Scenario: A module path is private

- **GIVEN** an external crate
- **WHEN** it writes `use rules::check::degree_of_success;`
- **THEN** rustc reports E0603

#### Scenario: Encounter is not reached through a module path

- **GIVEN** an external crate
- **WHEN** it writes `use rules::encounter::Encounter;`
- **THEN** rustc reports E0603
- **AND** `use rules::Encounter;` still compiles

### Requirement: No deprecation shim

This change SHALL NOT add `#[deprecated]`. A caller inside this crate that named a removed root item SHALL be updated in the same pull request. In-crate code that already names the defining module SHALL keep doing so.

#### Scenario: Nothing is marked deprecated

- **GIVEN** the rules crate after this change
- **WHEN** its source is searched for `deprecated`
- **THEN** no item removed from the root carries that attribute
- **AND** no public re-export remains as a compatibility alias

### Requirement: Fight results do not change

This change SHALL NOT change the value a rules function returns for the same inputs. An existing test SHALL NOT be deleted or loosened.

#### Scenario: Existing assertions still pass

- **GIVEN** the tests that exist before the visibility edit
- **WHEN** they run after it
- **THEN** their assertions pass
- **AND** none of those assertions was weakened or removed

### Requirement: ScriptedDice stays feature-gated

`ScriptedDice` SHALL remain public only under `cfg(any(test, feature = "test-support"))`. The facade doctests SHALL compile without that feature.

#### Scenario: The default doctest does not need the feature

- **GIVEN** a `cargo test` run that does not enable `test-support`
- **WHEN** the public-list doctest compiles
- **THEN** it does not name `ScriptedDice`
- **AND** the `ScriptedDice` re-export is still behind that cfg

### Requirement: No new package and no new dependency

The workspace members SHALL remain exactly `crates/rules`. This change SHALL NOT add a dependency, including a dev-dependency. The external proof SHALL be doctests on this crate.

#### Scenario: The workspace stays one crate

- **GIVEN** the workspace manifest after this change
- **WHEN** its members are read
- **THEN** the only member is `crates/rules`
- **AND** `crates/rules` has no dependency list

### Requirement: Skill helpers stay off the crate root

`skill_modifier` and `skill_dc` SHALL be `pub(crate)` and SHALL NOT appear at the crate root. An external import of either name SHALL fail with E0432.

#### Scenario: Skill helper names are rejected at the root

- **GIVEN** an external crate
- **WHEN** it imports `skill_modifier`, and when it imports `skill_dc`
- **THEN** rustc reports E0432 for each import

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
