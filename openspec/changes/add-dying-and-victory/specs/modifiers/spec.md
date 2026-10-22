# modifiers

## ADDED Requirements

### Requirement: Reflex takes the unconscious penalty

Reflex SHALL be its own statistic and SHALL NOT share a stack with attack rolls, Armor Class, Perception, or damage. Unconscious SHALL add a status penalty of -4 to Reflex, Armor Class, and Perception. When the creature is not unconscious, those producers SHALL add nothing.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=95 (Player Core pg. 446, Unconscious)

#### Scenario: A conscious creature adds no Reflex penalty

- **GIVEN** a creature that is not unconscious
- **WHEN** Reflex modifiers are totaled
- **THEN** the total from this producer is 0

#### Scenario: Reflex and Armor Class both keep the penalty

- **GIVEN** an unconscious creature and no other modifiers, with off-guard passed only to Armor Class
- **WHEN** Reflex and Armor Class are totaled separately
- **THEN** Reflex includes a status penalty of -4 at sequence 10
- **AND** Armor Class includes a status penalty of -4 at sequence 6
- **AND** neither total drops the other's penalty

### Requirement: Unconscious is a reserved condition

Unconscious SHALL use ConditionId::UNCONSCIOUS, value 4, and no rank. Its group key SHALL be that id, the statistic, a penalty, and status. It SHALL NOT be a valued condition. A rank, a non-status type, a value other than -4, or a bonus SHALL return InvalidValuedCondition. Producers SHALL use UnconsciousArmorClass at sequence 6, UnconsciousPerception at sequence 7, and UnconsciousReflex at sequence 10, each in the reserved-source match. A caller SHALL NOT construct those sources.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=95 (Player Core pg. 446, Unconscious)
**Source**: Ruling R-09 in docs/designer/rules-rulings.md

#### Scenario: Unconscious is -4 status to Reflex

- **GIVEN** an unconscious creature and no other Reflex modifier
- **WHEN** Reflex modifiers are totaled
- **THEN** the total includes a status penalty of -4 from condition id 4 at sequence 10

#### Scenario: A ranked unconscious modifier is rejected

- **GIVEN** condition id 4 with a rank, a status penalty of -4
- **WHEN** it is constructed
- **THEN** the result is InvalidValuedCondition

#### Scenario: A non-status unconscious modifier is rejected

- **GIVEN** condition id 4 with no rank and a circumstance penalty of -4
- **WHEN** it is constructed
- **THEN** the result is InvalidValuedCondition

#### Scenario: An unconscious value other than -4 is rejected

- **GIVEN** condition id 4 with no rank and a status penalty of -3
- **WHEN** it is constructed
- **THEN** the result is InvalidValuedCondition

#### Scenario: An unconscious bonus is rejected

- **GIVEN** condition id 4 with no rank and a status bonus of +4
- **WHEN** it is constructed
- **THEN** the result is InvalidValuedCondition

### Requirement: Unconscious stacks in the status bucket

The unconscious penalty SHALL compete in the status bucket, where the worst status penalty wins, and SHALL add to circumstance penalties and to status bonuses. Armor Class for Dying or Unconscious SHALL include off-guard at -2 circumstance from ac_modifiers. Perception and Reflex SHALL NOT include that off-guard penalty.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=95 (Player Core pg. 446, Unconscious)
**Source**: Ruling R-09 in docs/designer/rules-rulings.md

#### Scenario: Frightened 2 and unconscious total -6 on Armor Class

- **GIVEN** frightened 2 and unconscious, with off-guard, and no other Armor Class modifier
- **WHEN** Armor Class is totaled
- **THEN** the status penalty is -4, the off-guard circumstance penalty is -2, and the total is -6

#### Scenario: Frightened 5 and unconscious total -7 on Armor Class

- **GIVEN** frightened 5 and unconscious, with off-guard, and no other Armor Class modifier
- **WHEN** Armor Class is totaled
- **THEN** the status penalty is -5, the off-guard circumstance penalty is -2, and the total is -7

#### Scenario: Frightened 3, sickened 1, and unconscious total -4 on Reflex

- **GIVEN** frightened 3, sickened 1, and unconscious, and no other Reflex modifier
- **WHEN** Reflex is totaled
- **THEN** the status penalty is -4 and the total is -4

#### Scenario: Unconscious and a status bonus total -3 on Reflex

- **GIVEN** unconscious and a status bonus of +1, and no other Reflex modifier
- **WHEN** Reflex is totaled
- **THEN** the status penalty and the status bonus both apply and the total is -3

#### Scenario: Frightened 4 tied with unconscious totals -4 on Reflex

- **GIVEN** frightened 4 and unconscious, and no other Reflex modifier
- **WHEN** Reflex is totaled
- **THEN** the status penalty is -4 and the total is -4
