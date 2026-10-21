# modifiers

## MODIFIED Requirements

### Requirement: Condition groups ignore the source key

The group key SHALL be (condition, statistic, bonus or penalty, type), not the source key, after frightened and sickened are rewritten to status. Different types SHALL stack. Unequal condition ids, including a present id against none, SHALL NOT share a source key. A group SHALL keep the worst penalty or the highest bonus. Frightened and sickened use the highest condition value (−value, Player Core p.442, Rules 2455). Otherwise the larger modifier wins, our ruling, not Rules 2455.

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

### Requirement: Condition-group ties break on rank, sequence, and source id

After the worst penalty or the highest bonus, the order SHALL be the highest rank, then the newest sequence, then the lowest source id. Those three SHALL be engineering tiebreaks with no rules effect. The higher level SHALL then win, and a level SHALL beat none. That level step SHALL be an engineering tiebreak with no rules effect. The canonical duplicate-key comparison SHALL decide. This order SHALL be a total order over distinct modifiers.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2455 (Player Core pg. 442, Redundant Conditions With Values; context only, the value step is RAW)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2266 (Player Core pg. 398, Duplicate Effects; context only)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2282 (Player Core pg. 400, Penalties)

#### Scenario: Two frightened penalties report the lower source in both orders

- **GIVEN** two frightened status penalties of -1 on the attack roll, from source 1 and source 2, both at sequence 5 and rank 2
- **WHEN** they are totaled with either modifier first
- **THEN** one status penalty is applied
- **AND** its source id is 1 in both orders
- **AND** its value is -2, the negation of rank 2
- **AND** the attack-roll total is -2 in both orders

#### Scenario: A circumstance and status tie totals -5 in every order

- **GIVEN** condition 10 on the attack roll, a circumstance penalty of -2 and a status penalty of -2, both from source 1 at sequence 5 and rank 1, with no level, plus an unconditioned status penalty of -3 from source 2 at sequence 1
- **WHEN** they are totaled in all 6 orders
- **THEN** the circumstance penalty of -2 applies alone in its type in every order
- **AND** the status penalty of -3 beats the status penalty of -2 in every order
- **AND** every result applies the circumstance penalty of -2 and the status penalty of -3
- **AND** no result applies the status penalty of -2
- **AND** every attack-roll total is -5

#### Scenario: A same-type tie keeps the higher level

- **GIVEN** condition 10 on the attack roll, two status penalties of -2 from source 1 at sequence 5 and rank 1, at level 1 and level 4
- **WHEN** they are totaled with either modifier first
- **THEN** one status penalty is applied
- **AND** its level is 4 in both orders
- **AND** the level 1 penalty is not applied
- **AND** the total is -2 in both orders

#### Scenario: A circumstance and item tie totals -5 in every order

- **GIVEN** on the attack roll, a circumstance penalty of -2 and an item penalty of -2, both from source 1 at sequence 5, condition 10, and rank 1, with no level, plus a circumstance penalty of -3 from source 9 at sequence 1 with no condition
- **WHEN** they are totaled in all 6 orders
- **THEN** every attack-roll total is -5
- **AND** every result applies the item penalty of -2 from source 1 and condition 10
- **AND** every result applies the circumstance penalty of -3 from source 9
- **AND** no result applies the circumstance penalty of -2

#### Scenario: A frightened level tie keeps level 3

- **GIVEN** on the attack roll, two frightened status penalties of -1, both from source 1 at sequence 5 and rank 2, at level 1 and level 3
- **WHEN** they are totaled with either modifier first
- **THEN** one status penalty is applied in both orders
- **AND** its level is 3
- **AND** its value is -2, the negation of rank 2
- **AND** the level 1 penalty is not applied
- **AND** the attack-roll total is -2 in both orders

#### Scenario: Two identical condition penalties apply once

- **GIVEN** condition 10 on the attack roll, two status penalties of -2 from source 1 at sequence 5 and rank 1, both at level 1
- **WHEN** they are totaled with either modifier first
- **THEN** exactly one status penalty is applied in both orders
- **AND** the attack-roll total is -2 in both orders

### Requirement: Comparisons end on the lower source id

Stacking SHALL keep the highest bonus or worst penalty, then the lowest sequence, then the lower condition id, a missing id losing, then the lower source id. A duplicate-key comparison SHALL end on that source id. A condition-group comparison SHALL continue after that source id, in the order its tie requirement states, and SHALL be a total order over distinct modifiers. For any input set, every ordering SHALL keep the same applied modifier and the same total.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2281 (Player Core pg. 400, Bonuses)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2282 (Player Core pg. 400, Penalties)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2266 (Player Core pg. 398, Duplicate Effects)

#### Scenario: A tied circumstance bonus reports the lower source in both orders

- **GIVEN** circumstance bonuses of +2 from source 1 and source 2, both at sequence 3 on the attack roll, with no condition
- **WHEN** they are totaled with either modifier first
- **THEN** one circumstance bonus is applied
- **AND** its source id is 1 in both orders
- **AND** the total is +2 in both orders
- **AND** the tiebreak is recorded as rule, Duplicate Effects and same-type stacking name no further instance once the bonus or penalty is chosen; why not, they are silent when those match; substitute, the lower source id wins; intent, every input order reports the same modifier and the same total

#### Scenario: The lowest sequence beats the lower source id

- **GIVEN** circumstance bonuses of +2 on the attack roll, from source 2 at sequence 1 and from source 1 at sequence 4, with no condition
- **WHEN** they are totaled with either modifier first
- **THEN** one circumstance bonus is applied
- **AND** its source id is 2 in both orders
- **AND** its sequence number is 1
- **AND** the total is +2 in both orders

## ADDED Requirements

### Requirement: A property test checks every ordering

It SHALL add no crate and SHALL NOT read a clock. It SHALL emit random sets, one unconditioned untyped exact duplicate, and one set with both levels absent so the canonical key decides. Every permutation of at most four SHALL be totaled. Each statistic total SHALL equal the best bonus per type plus the worst penalty per type plus every untyped penalty, after condition grouping and the R-01 duplicate-key collapse (one entry per source id, statistic, type, bonus or penalty, condition id).

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2281 (Player Core pg. 400, Bonuses)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2282 (Player Core pg. 400, Penalties)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2266 (Player Core pg. 398, Duplicate Effects; context only)

#### Scenario: Permuted sets match the stacking oracle

- **GIVEN** one fixed seed whose generated sets include one unconditioned untyped exact duplicate and one set with both levels absent, and each set has at most four modifiers
- **WHEN** every permutation of each set is totaled, and the same seed is used again
- **THEN** every ordering of a set keeps the same applied modifiers and the same total
- **AND** each statistic total equals the best bonus per type plus the worst penalty per type plus every untyped penalty, after condition grouping and the R-01 duplicate-key collapse (one entry per source id, statistic, type, bonus or penalty, condition id)
- **AND** the second run generates the same sets
