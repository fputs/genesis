# modifiers

## MODIFIED Requirements

### Requirement: Frightened grouping follows the split

A frightened or sickened modifier with no rank, or with a declared type other than status, SHALL be rejected at construction with InvalidValuedCondition and SHALL NOT panic. A penalty of rank above 0 SHALL apply as status -rank and SHALL NOT be rewritten from another type. Rank 0 SHALL stay legal and contribute nothing. Any other condition with no rank SHALL stay legal. A status bonus tagged with either condition SHALL keep its declared type and value and SHALL NOT hide that penalty.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2455 (Player Core pg. 442, Redundant Conditions With Values)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=76 (Player Core pg. 444, Frightened)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=91 (Player Core pg. 446, Sickened)

#### Scenario: A frightened bonus does not hide a frightened penalty

- **GIVEN** a synthetic robustness check, because frightened never grants a bonus: source 7 on the attack roll at sequence 4, with a frightened status bonus of +2 at rank 3 and a frightened status penalty of -1 at rank 1
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

### Requirement: Condition groups ignore the source key

The group key SHALL be (condition, statistic, bonus or penalty, type), not the source key. Different types SHALL stack, and type SHALL NOT be a tiebreak. Unequal condition ids, including a present id against none, SHALL NOT share a source key. A group SHALL keep the worst penalty or the highest bonus. Frightened and sickened use the highest condition value (−value, Player Core p.442, Rules 2455). Otherwise the larger modifier wins, our ruling, not Rules 2455.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2455 (Player Core pg. 442, Redundant Conditions With Values)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2281 (Player Core pg. 400, Bonuses)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2282 (Player Core pg. 400, Penalties)

#### Scenario: A same condition keeps the worse penalty in both orders

- **GIVEN** condition 10 on the attack roll, a circumstance penalty of -1 from source 1 at sequence 5 and rank 1, and a circumstance penalty of -4 from source 2 at sequence 5 and rank 1
- **WHEN** they are totaled with either modifier first
- **THEN** the circumstance penalty is -4 in both orders
- **AND** the total is -4 in both orders

#### Scenario: An older higher condition value beats a newer lower one

- **GIVEN** frightened 3 at sequence 1 and frightened 1 at sequence 2 on the same statistic, the attack roll
- **WHEN** they are totaled with either modifier first
- **THEN** the status penalty is -3 in both orders
- **AND** the total is -3 in both orders

#### Scenario: Frightened 1, 3, and 2 give -3 in all six orders

- **GIVEN** on the attack roll, frightened 1 from source 1 at sequence 1, frightened 3 from source 2 at sequence 2, and frightened 2 from source 3 at sequence 3
- **WHEN** they are totaled in all 6 orders
- **THEN** every result applies one status penalty of -3
- **AND** that penalty has sequence 2
- **AND** every total is -3

#### Scenario: Removing frightened 2 leaves frightened 1

- **GIVEN** frightened 2 from source 1 and frightened 1 from source 2, on the attack roll
- **WHEN** the frightened 2 entry is removed and the remaining list is totaled
- **THEN** the status penalty is -1
- **AND** the total is -1
- **AND** frightened 1 is applied

### Requirement: A property test checks every ordering

Frightened and sickened penalties use status -rank as their value, and rank 0 entries are dropped. A frightened status bonus keeps its declared value. (a) Group by condition, keyed on (condition, statistic, bonus or penalty, type). Grouped value first, then the condition-group ladder. (b) R-01 (source, statistic, type, bonus or penalty), same condition id. (c) Best bonus, worst penalty per type. Untyped stack. Ties SHALL: value, lowest sequence, lower condition id (none loses), lower source id.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2281 (Player Core pg. 400, Bonuses)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2282 (Player Core pg. 400, Penalties)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2266 (Player Core pg. 398, Duplicate Effects; context only)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2455 (Player Core pg. 442, Redundant Conditions With Values)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=76 (Player Core pg. 444, Frightened)

#### Scenario: Permuted sets match the stacking oracle

- **GIVEN** one fixed seed and at least 1,000 random sets of at most four modifiers, plus one unconditioned untyped exact duplicate, one conditioned exact duplicate, and one set with both levels absent. Each draw is a bonus or a penalty; a type of circumstance, item, or status, with untyped allowed for a penalty only; and a statistic of attack roll or Armor Class. Frightened is always status, with rank Some(0..2), ranks 0, 1, and 2. The draws include at least one reserved source
- **WHEN** every generated modifier is built, every permutation of each set is totaled, and the same seed is used again
- **THEN** every generated modifier builds without error
- **AND** the test adds no crate and does not read a clock
- **AND** every ordering of a set keeps the same applied modifiers and the same total
- **AND** each statistic total equals the independent oracle's three stages: Frightened and sickened penalties use status -rank as their value, and rank 0 entries are dropped. A frightened status bonus keeps its declared value. (a) group by condition, keyed on (condition, statistic, bonus or penalty, type). Stage (a) picks the grouped value first, then applies the condition-group ladder (Player Core pg. 442 and pg. 444); (b) collapse same-source duplicates (rule R-01), keyed on (source, statistic, type, bonus or penalty), among entries with the same condition id; (c) for each statistic, keep the best bonus per type, the worst penalty per type, and every untyped penalty (untyped penalties all stack)
- **AND** ties at the stacking step go to value first, then lowest sequence, then lower condition id (an entry with no condition id loses), then lower source id
- **AND** that oracle does not call derived_rank, same_duplicate_key, source_ord, or the comparison chains
- **AND** the second run generates the same sets

#### Scenario: A condition-10 circumstance penalty does not add to an unconditioned one

- **GIVEN** a circumstance penalty of -2 with condition 10 and an unconditioned circumstance penalty of -3, both on the attack roll
- **WHEN** they are totaled
- **THEN** the total is -3
- **AND** the total is not -5
- **AND** one circumstance penalty is applied

## ADDED Requirements

### Requirement: Attack check internal terms use reserved sources

The multiple attack penalty SHALL apply once per check and SHALL reuse MultipleAttackPenalty. The attack check SHALL NOT add a separate internal multiple attack penalty source. The other untyped internal penalty SHALL be the range penalty (Player Core pg. 403, Rules 2290). A reserved source SHALL NOT exempt a term from same-type stacking. A caller SHALL NOT construct a reserved source.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2282 (Player Core pg. 400, Penalties)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2289 (Player Core pg. 402, Multiple Attack Penalty)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2290 (Player Core pg. 403, Range Penalty)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2266 (Player Core pg. 398, Duplicate Effects; context only)

#### Scenario: A caller source 1 does not collapse an internal term

- **GIVEN** the attack check total's internal terms for an untyped multiple attack penalty of -5, a circumstance penalty of -2, and a range penalty of -2, plus a caller untyped penalty of -3 from source 1
- **WHEN** they are totaled on the attack roll
- **THEN** the caller penalty of -3 is applied
- **AND** the multiple attack penalty of -5 and the range penalty of -2 are applied
- **AND** the circumstance penalty of -2 is applied
- **AND** the attack-roll total is -12
- **AND** no internal term is caller source 1
- **AND** the substitute is recorded as rule, Duplicate Effects keys a repeated effect on its source; why not, these internal terms are not caller effects; substitute, each internal term uses a reserved source no caller can construct; intent, caller source 1 cannot collapse an internal term

#### Scenario: A second attack with no caller MAP gets -5 once

- **GIVEN** a second non-agile attack on the creature's own turn and no caller multiple attack penalty
- **WHEN** the attack-roll modifiers for that check are totaled
- **THEN** one untyped penalty of -5 is applied
- **AND** that penalty is applied once
- **AND** the multiple attack penalty is not -10

#### Scenario: An internal circumstance penalty and a caller circumstance penalty total -3

- **GIVEN** an internal circumstance penalty of -2 and a caller circumstance penalty of -3 from source 1
- **WHEN** they are totaled on the attack roll
- **THEN** the total is -3
- **AND** the circumstance penalty is -3
- **AND** the total is not -5
