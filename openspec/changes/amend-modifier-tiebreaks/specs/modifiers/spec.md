# modifiers

## MODIFIED Requirements

### Requirement: Same-type stacking

On one statistic, for one type, only the highest bonus and only the worst penalty SHALL apply across condition groups. Another statistic SHALL NOT share that choice. When different sources tie, the lowest sequence number SHALL be the one reported as applied. A bonus and a penalty of the same type SHALL both apply. Every untyped penalty on that statistic SHALL be added. A sum that overflows SHALL return a typed error and SHALL NOT saturate.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2281 (Player Core pg. 400, Bonuses)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2282 (Player Core pg. 400, Penalties)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2289 (Player Core pg. 402, Multiple Attack Penalty)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2290 (Player Core pg. 403, Range Penalty)

#### Scenario: Two circumstance bonuses keep the highest

- **GIVEN** circumstance bonuses of +2 and +1 from different sources
- **WHEN** they are totaled
- **THEN** the circumstance bonus is +2
- **AND** the total is +2

#### Scenario: +2 circumstance with -2 circumstance equals 0

- **GIVEN** a circumstance bonus of +2 and a circumstance penalty of -2
- **WHEN** they are totaled
- **THEN** both apply
- **AND** the total is 0

#### Scenario: Status +1 and status -2 total -1

- **GIVEN** a status bonus of +1 and a status penalty of -2 from different sources
- **WHEN** they are totaled
- **THEN** both apply
- **AND** the total is -1

#### Scenario: Frightened 2 and sickened 1 equal -2

- **GIVEN** frightened 2 as a status penalty of -2 and sickened 1 as a status penalty of -1, from different conditions
- **WHEN** they are totaled
- **THEN** the status penalty is -2
- **AND** the total is -2

#### Scenario: Untyped penalties stack

- **GIVEN** untyped penalties of -2 and -3 from different sources
- **WHEN** they are totaled
- **THEN** the total is -5

#### Scenario: MAP -5 plus range -2 equals -7

- **GIVEN** the existing multiple attack penalty of -5 for a second non-agile attack on the creature's own turn, and a range penalty of -2 untyped from another source
- **WHEN** they are totaled on the attack roll
- **THEN** the untyped total is -7
- **AND** that -5 is the value the existing multiple attack penalty returns

#### Scenario: An overflowing modifier sum returns an error

- **GIVEN** untyped penalties of -2147483648 and -1 from different sources
- **WHEN** they are totaled
- **THEN** the result is a typed overflow error and not a total
- **AND** the result is not a saturated value

#### Scenario: The same modifiers total twice

- **GIVEN** one circumstance bonus of +2 and one untyped penalty of -3
- **WHEN** the total is taken twice
- **THEN** the two totals are equal

#### Scenario: A tied highest bonus reports the lowest sequence

- **GIVEN** circumstance bonuses of +2 from two sources, with sequence numbers 4 and 1
- **WHEN** they are totaled
- **THEN** the applied circumstance bonus has sequence number 1
- **AND** the total is +2

#### Scenario: A tied worst penalty reports the lowest sequence

- **GIVEN** circumstance penalties of -2 from two sources, with sequence numbers 7 and 3
- **WHEN** they are totaled
- **THEN** the applied circumstance penalty has sequence number 3
- **AND** the total is -2

### Requirement: One effect and the highest condition rank

Each modifier SHALL carry a source id, a sequence number, and its statistic. The key (source id, statistic, type, bonus or penalty) SHALL become one before the sum and SHALL NOT group different conditions. Within a key, the higher level wins, then the higher rank, otherwise the newer (greater) sequence. Value SHALL NOT override that. A repeated condition on one statistic SHALL keep its highest rank, otherwise the newer (greater) sequence. A condition with no rank SHALL apply once per statistic.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2266 (Player Core pg. 398, Duplicate Effects)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2282 (Player Core pg. 400, Penalties)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2455 (Player Core pg. 442, Redundant Conditions With Values)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=76 (Player Core pg. 444, Frightened)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=91 (Player Core pg. 446, Sickened)

#### Scenario: A duplicate same-source untyped penalty counts once

- **GIVEN** two untyped penalties of -2 that share a source id and have sequence numbers 1 and 2
- **WHEN** they are totaled
- **THEN** the untyped penalty is -2 once
- **AND** the total is -2

#### Scenario: A tied source keeps the newer modifier

- **GIVEN** two circumstance bonuses of +2 that share a source id and level, with sequence numbers 1 and 4
- **WHEN** they are resolved
- **THEN** the kept modifier has sequence number 4
- **AND** the total is +2

#### Scenario: A higher level beats a higher value

- **GIVEN** one source, a level 2 circumstance bonus of +1 with sequence 1, and a level 1 circumstance bonus of +4 with sequence 2
- **WHEN** they are resolved
- **THEN** the kept modifier is the level 2 bonus of +1
- **AND** the total is +1

#### Scenario: A newer lower value wins

- **GIVEN** one source and no levels, an untyped penalty of -2 with sequence 1, and an untyped penalty of -5 with sequence 9
- **WHEN** they are totaled
- **THEN** the kept modifier has sequence number 9
- **AND** the total is -5

#### Scenario: A repeated condition keeps its highest rank

- **GIVEN** frightened 1 and frightened 2
- **WHEN** the conditions are reduced and totaled
- **THEN** the kept rank is 2
- **AND** there is one status penalty of -2
- **AND** frightened 1 is not a separate modifier

#### Scenario: Off-guard from two causes applies once

- **GIVEN** off-guard from prone and off-guard from flanking
- **WHEN** Armor Class modifiers are totaled
- **THEN** the circumstance penalty is -2 once

#### Scenario: A bonus and a penalty from one source both apply

- **GIVEN** source 7 on the attack roll at sequence 5, with a status bonus of +2 and a status penalty of -1
- **WHEN** they are totaled with either modifier first
- **THEN** the status bonus of +2 is applied
- **AND** the status penalty of -1 is applied
- **AND** the attack-roll total is +1 in either order

#### Scenario: One source on two statistics both apply

- **GIVEN** source 7 at sequence 5, with a status bonus of +2 to the attack roll and a circumstance penalty of -1 to Armor Class
- **WHEN** they are resolved together with either modifier first
- **THEN** the attack-roll status bonus of +2 is applied
- **AND** the attack-roll total is +2
- **AND** the Armor Class circumstance penalty of -1 is applied
- **AND** the Armor Class total is -1

### Requirement: Duplicate key comparison is a total order

Within a duplicate key the kept modifier SHALL follow a total order: higher level, a level over none, higher rank, a rank over none, then the greater sequence. If those tie, ruling R-01 (an extension consistent with RAW via Rules 2282, not a deviation) SHALL keep the higher bonus or the more negative penalty. If the value ties, the lower condition id SHALL win, and a missing condition id SHALL lose. The lower source id SHALL then win. It is an engineering tiebreak, not a deviation.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2266 (Player Core pg. 398, Duplicate Effects)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2282 (Player Core pg. 400, Penalties)

#### Scenario: An equal sequence keeps the higher bonus

- **GIVEN** source 1 on the attack roll, circumstance bonuses of +1 and +4, both at sequence 4, with no level
- **WHEN** they are totaled with either modifier first
- **THEN** one bonus is applied
- **AND** that bonus is +4
- **AND** the attack-roll total is +4 in either order

#### Scenario: An equal sequence keeps the worse penalty

- **GIVEN** source 2 on the attack roll, untyped penalties of -1 and -8, both at sequence 7, with no level
- **WHEN** they are totaled with either modifier first
- **THEN** one penalty is applied
- **AND** that penalty is -8
- **AND** the attack-roll total is -8 in either order

#### Scenario: The lower condition id wins an equal value

- **GIVEN** source 9 on the attack roll, two circumstance bonuses of +2, both level 2, rank 1, and sequence 5, with condition ids 8 and 3
- **WHEN** they are totaled with either modifier first
- **THEN** one modifier is applied
- **AND** its condition id is 3
- **AND** the attack-roll total is +2 in either order

#### Scenario: A three-way tie is the same in all six orders

- **GIVEN** source 11 on the attack roll, three circumstance bonuses of +2, all level 1, rank 1, and sequence 8, with condition ids 6, 1, and 4
- **WHEN** they are totaled in all 6 orders
- **THEN** every result is the same
- **AND** one modifier is applied
- **AND** its condition id is 1
- **AND** the attack-roll total is +2

### Requirement: Resolution ignores input order

For any input set, every ordering SHALL produce the same total and the same applied modifiers. Modifiers that differ in type, or in bonus versus penalty, SHALL both apply, including equal values from one source at one sequence. Each reported total SHALL belong to one statistic and SHALL NOT add another statistic.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2266 (Player Core pg. 398, Duplicate Effects)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2281 (Player Core pg. 400, Bonuses)

#### Scenario: Equal values of different types total +5 in every order

- **GIVEN** source 7 on the attack roll at sequence 5, with a status bonus of +2 and a circumstance bonus of +2, plus source 8 circumstance bonus of +3 at sequence 5
- **WHEN** the three modifiers are totaled in every order
- **THEN** every attack-roll total is +5
- **AND** every result applies the status bonus of +2
- **AND** every result applies the circumstance bonus of +3
- **AND** no result applies the circumstance bonus of +2

#### Scenario: Every input order gives an identical result

- **GIVEN** one attack-roll set: condition 10 circumstance penalties of -1 from source 1 and -4 from source 2, both sequence 5 and rank 1; frightened status penalties of -1 from source 1 and source 2, both sequence 5 and rank 2; circumstance bonuses of +2 from source 1 and source 2, both sequence 3 and no condition; frightened 3 from source 4 at sequence 1 and level 1; sickened 1 from source 4 at sequence 2 and level 5
- **WHEN** every ordering of that set is totaled
- **THEN** every result is identical
- **AND** the totals are equal
- **AND** the applied modifiers are equal

### Requirement: Frightened grouping follows the split

Frightened and sickened grouping SHALL key on (condition, statistic) only, never on a source key, after the bonus and penalty split and before duplicate-key collapse. Those penalties SHALL contribute one status penalty equal to the negation of the kept rank. A bonus tagged with either condition SHALL keep its declared type and value and SHALL NOT hide a penalty of that condition.

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

## ADDED Requirements

### Requirement: Condition groups ignore the source key

Condition modifiers SHALL group by (condition, statistic) only, never by source key, after the bonus and penalty split. Unequal condition ids, including a present id against none, SHALL NOT share a source key. Within a group the highest rank SHALL apply. When ranks tie, the newer sequence SHALL apply. When rank and sequence also tie, the worse penalty or the higher bonus SHALL apply. An exact tie SHALL break on the lower source id.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2455 (Player Core pg. 442, Redundant Conditions With Values)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2282 (Player Core pg. 400, Penalties)

#### Scenario: A same condition keeps the worse penalty in both orders

- **GIVEN** condition 10 on the attack roll, a circumstance penalty of -1 from source 1 at sequence 5 and rank 1, and a circumstance penalty of -4 from source 2 at sequence 5 and rank 1
- **WHEN** they are totaled with either modifier first
- **THEN** the circumstance penalty is -4 in both orders
- **AND** the total is -4 in both orders

#### Scenario: Two frightened penalties report the lower source in both orders

- **GIVEN** two frightened status penalties of -1 on the attack roll, from source 1 and source 2, both at sequence 5 and rank 2
- **WHEN** they are totaled with either modifier first
- **THEN** one status penalty is applied
- **AND** its source id is 1 in both orders
- **AND** its value is -2, the negation of rank 2
- **AND** the attack-roll total is -2 in both orders

### Requirement: Comparisons end on the lower source id

Every comparison SHALL end on the lower source id, so the resolution, the total, and the reported modifier SHALL be independent of input order. Stacking SHALL keep the highest bonus or worst penalty, then the lowest sequence, then the lower condition id, a missing id losing, then the lower source id. Duplicate-key and condition-group comparisons SHALL end on that same source id. For any input set, every ordering SHALL produce an identical result.

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

### Requirement: Conditions stay separate state

Conditions SHALL stay separate state. Collapsing SHALL affect only the numeric total. Frightened and sickened from one source SHALL both remain when the worse status penalty is applied. Decrementing frightened SHALL leave sickened present.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2455 (Player Core pg. 442, Redundant Conditions With Values)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=76 (Player Core pg. 444, Frightened)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=91 (Player Core pg. 446, Sickened)

#### Scenario: Frightened and sickened stay present while the worse penalty applies

- **GIVEN** one caller's list for one attack roll and source 4, holding frightened 3 as a status penalty at sequence 1 and level 1, and sickened 1 as a status penalty at sequence 2 and level 5
- **WHEN** the list is totaled
- **THEN** the status penalty is -3
- **AND** the total is -3
- **AND** the list still contains frightened at rank 3 and sickened at rank 1
- **WHEN** frightened is decremented to rank 2 and the list is totaled again
- **THEN** the status penalty is -2
- **AND** the total is -2
- **AND** the list still contains sickened at rank 1
