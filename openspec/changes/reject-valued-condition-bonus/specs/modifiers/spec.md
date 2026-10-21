# modifiers

## REMOVED Requirements

### Requirement: Frightened grouping follows the split

**Reason**: The requirement now rejects a frightened or sickened bonus, so the old name no longer describes it.
**Migration**: Use "Valued conditions are validated at construction". Its scenarios move with it. The bonus-versus-penalty scenario is renamed to "A condition bonus does not hide that condition's penalty".

## ADDED Requirements

### Requirement: Valued conditions are validated at construction

A frightened or sickened modifier with no rank, non-status type, or a bonus at any rank including 0, SHALL be rejected at construction with InvalidValuedCondition and SHALL NOT panic. A penalty of rank above 0 SHALL apply as status -rank and SHALL NOT be rewritten from another type. A penalty of rank 0 SHALL stay legal and contribute nothing. Any other condition with no rank SHALL stay legal. A bonus and a penalty of a condition whose modifiers are not derived from its value SHALL both apply.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2455 (Player Core pg. 442, Redundant Conditions With Values)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=76 (Player Core pg. 444, Frightened)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=91 (Player Core pg. 446, Sickened)

#### Scenario: A condition bonus does not hide that condition's penalty

- **GIVEN** condition 10, a condition whose modifiers are not derived from its value, from source 7 on the attack roll at sequence 4, with a status bonus of +2 at rank 3 and a status penalty of -1 at rank 1
- **WHEN** they are totaled with either modifier first
- **THEN** the status bonus of +2 is applied
- **AND** the status penalty of -1 is applied
- **AND** the attack-roll total is +1 in either order

#### Scenario: Frightened 2 on attack and Armor Class both survive

- **GIVEN** frightened 2 on the attack roll and frightened 2 on Armor Class, same source, in one list
- **WHEN** they are resolved together in either order
- **THEN** both modifiers survive
- **AND** the attack total is -2
- **AND** the Armor Class total is -2

#### Scenario: Frightened with rank None is rejected

- **GIVEN** a frightened penalty with no rank
- **WHEN** it is created
- **THEN** the result is a typed construction error
- **AND** no modifier is returned
- **AND** creation does not panic

#### Scenario: Frightened rank 2 declared circumstance is rejected

- **GIVEN** a frightened circumstance penalty of rank 2
- **WHEN** it is created
- **THEN** the result is a typed construction error
- **AND** no modifier is returned
- **AND** the penalty is not rewritten to status

#### Scenario: Sickened with no rank is rejected

- **GIVEN** a sickened penalty with no rank
- **WHEN** it is created
- **THEN** the result is a typed construction error
- **AND** no modifier is returned
- **AND** creation does not panic

#### Scenario: The QA frightened list is rejected

- **GIVEN** a circumstance penalty of -4 from source 1 at sequence 1, frightened, with no rank, and a status penalty of -1 from source 2 at sequence 2, frightened, at rank 2
- **WHEN** that input is created
- **THEN** construction returns a typed error
- **AND** the list is not totaled

#### Scenario: Two status frightened rank 2 penalties total -2

- **GIVEN** two frightened status penalties of rank 2 on the attack roll, from source 1 at sequence 1 and source 2 at sequence 2
- **WHEN** they are totaled with either modifier first
- **THEN** one status penalty is applied in both orders
- **AND** its value is -2, the negation of rank 2
- **AND** the attack-roll total is -2 in both orders

#### Scenario: Sickened with item type and rank 1 is rejected

- **GIVEN** a sickened item penalty of rank 1
- **WHEN** it is created
- **THEN** the result is InvalidValuedCondition
- **AND** no modifier is returned
- **AND** creation does not panic

#### Scenario: Frightened untyped with rank 1 is rejected

- **GIVEN** a frightened untyped penalty of rank 1
- **WHEN** it is created
- **THEN** the result is InvalidValuedCondition
- **AND** no modifier is returned
- **AND** creation does not panic

#### Scenario: Frightened bonus rank 0 +2 is rejected

- **GIVEN** a frightened status bonus of +2 at rank 0
- **WHEN** it is created
- **THEN** the result is InvalidValuedCondition
- **AND** no modifier is returned
- **AND** creation does not panic

#### Scenario: Frightened bonus rank 2 +2 is rejected

- **GIVEN** a frightened status bonus of +2 at rank 2
- **WHEN** it is created
- **THEN** the result is InvalidValuedCondition
- **AND** no modifier is returned
- **AND** creation does not panic

#### Scenario: Sickened bonus rank 1 +1 is rejected

- **GIVEN** a sickened status bonus of +1 at rank 1
- **WHEN** it is created
- **THEN** the result is InvalidValuedCondition
- **AND** no modifier is returned
- **AND** creation does not panic

## MODIFIED Requirements

### Requirement: A property test checks every ordering

Frightened and sickened penalties use status -rank. Rank 0 entries are dropped. The generator never draws a frightened or sickened bonus. (a) Group by condition, keyed on (condition, statistic, bonus or penalty, type). Grouped value first, then the condition-group ladder. (b) R-01 (source, statistic, type, bonus or penalty), same condition id. (c) Best bonus, worst penalty per type. Untyped stack. Ties SHALL: value, lowest sequence, lower condition id (none loses), lower source id.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2281 (Player Core pg. 400, Bonuses)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2282 (Player Core pg. 400, Penalties)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2266 (Player Core pg. 398, Duplicate Effects; context only)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2455 (Player Core pg. 442, Redundant Conditions With Values)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=76 (Player Core pg. 444, Frightened)

#### Scenario: Permuted sets match the stacking oracle

- **GIVEN** one fixed seed and at least 1,000 random sets of at most four modifiers, plus one unconditioned untyped exact duplicate, one conditioned exact duplicate, and one set with both levels absent. Each draw is a bonus or a penalty; a type of circumstance, item, or status, with untyped allowed for a penalty only; and a statistic of attack roll or Armor Class. A frightened draw is a status penalty, with rank Some(0..2), ranks 0, 1, and 2. The generator never draws a frightened or sickened bonus. The draws include at least one reserved source
- **WHEN** every generated modifier is built, every permutation of each set is totaled, and the same seed is used again
- **THEN** every generated modifier builds without error
- **AND** the test adds no crate and does not read a clock
- **AND** every ordering of a set keeps the same applied modifiers and the same total
- **AND** each statistic total equals the independent oracle's three stages: Frightened and sickened penalties use status -rank as their value, and rank 0 entries are dropped. The generator never draws a frightened or sickened bonus. (a) group by condition, keyed on (condition, statistic, bonus or penalty, type). Stage (a) picks the grouped value first, then applies the condition-group ladder (Player Core pg. 442 and pg. 444); (b) collapse same-source duplicates (rule R-01), keyed on (source, statistic, type, bonus or penalty), among entries with the same condition id; (c) for each statistic, keep the best bonus per type, the worst penalty per type, and every untyped penalty (untyped penalties all stack)
- **AND** ties at the stacking step go to value first, then lowest sequence, then lower condition id (an entry with no condition id loses), then lower source id
- **AND** that oracle does not call derived_rank, same_duplicate_key, source_ord, or the comparison chains
- **AND** the second run generates the same sets

#### Scenario: A condition-10 circumstance penalty does not add to an unconditioned one

- **GIVEN** a circumstance penalty of -2 with condition 10 and an unconditioned circumstance penalty of -3, both on the attack roll
- **WHEN** they are totaled
- **THEN** the total is -3
- **AND** the total is not -5
- **AND** one circumstance penalty is applied
