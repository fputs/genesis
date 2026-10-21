# modifiers

## MODIFIED Requirements

### Requirement: Condition-group ties break on rank, sequence, and source id

After the worst penalty or the highest bonus, the order SHALL be the highest rank, then the newest sequence, then the lowest source id. Those SHALL be engineering tiebreaks with no rules effect. After a tied source id, the earlier type SHALL win: circumstance, item, status, untyped, an engineering tiebreak with no RAW basis. The higher level SHALL then win, and a level SHALL beat none. The canonical duplicate-key comparison SHALL decide. This order SHALL be a total order over distinct modifiers.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2455 (Player Core pg. 442, Redundant Conditions With Values; context only, the value step is RAW)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2266 (Player Core pg. 398, Duplicate Effects)
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
- **THEN** every attack-roll total is -5
- **AND** every result applies the circumstance penalty of -2
- **AND** every result applies the status penalty of -3
- **AND** no result applies the status penalty of -2
- **AND** the type tiebreak is recorded as rule, same-type stacking and redundant conditions name no order among circumstance, item, status, and untyped once the value, rank, sequence, and source id match; why not, they are silent when the types differ; substitute, the earlier type in that order wins; intent, every input order reports the same modifier and the same total

#### Scenario: A same-type tie keeps the higher level

- **GIVEN** condition 10 on the attack roll, two status penalties of -2 from source 1 at sequence 5 and rank 1, at level 1 and level 4
- **WHEN** they are totaled with either modifier first
- **THEN** one status penalty is applied
- **AND** its level is 4 in both orders
- **AND** the level 1 penalty is not applied
- **AND** the total is -2 in both orders

### Requirement: Comparisons end on the lower source id

Stacking SHALL keep the highest bonus or worst penalty, then the lowest sequence, then the lower condition id, a missing id losing, then the lower source id. A duplicate-key comparison SHALL end on that source id. A condition-group comparison SHALL continue after that source id, in the order its tie requirement states, and SHALL be a total order over distinct modifiers. For any input set, every ordering SHALL produce an identical result.

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

The implementation SHALL include a property test that uses a fixed seed and SHALL add no crate. Its generated sets SHALL include a same-source, same-value collision across type and one across level, and each set SHALL hold at most four modifiers. The test SHALL shuffle each set through every ordering and SHALL assert that the totals and the kept modifiers are identical for every ordering. The same seed SHALL produce the same sets. The test SHALL NOT read a clock or a global generator.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2266 (Player Core pg. 398, Duplicate Effects)

#### Scenario: Shuffled collisions match in every order

- **GIVEN** one fixed seed whose generated sets include a same-source, same-value collision across type and one across level, and each set has at most four modifiers
- **WHEN** every ordering of each set is totaled, and the same seed is used again
- **THEN** every ordering of a set has the same total
- **AND** every ordering of a set keeps the same modifiers
- **AND** the second run generates the same sets
