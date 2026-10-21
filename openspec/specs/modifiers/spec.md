# modifiers Specification

## Purpose
Folds circumstance, item, status, and untyped modifiers onto attack rolls, Armor Class, Perception, and damage, and rejects an untyped bonus or a zero value.

## Requirements

### Requirement: Bonus and penalty types

A bonus SHALL be circumstance, item, or status. A penalty SHALL be circumstance, item, status, or untyped. Proficiency SHALL NOT be a modifier type, because AC, attack, Perception, and DCs are baked (DEV-01). An item modifier SHALL be constructible under these rules, and Phase 1 SHALL produce none.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2281 (Player Core pg. 400, Bonuses)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2282 (Player Core pg. 400, Penalties)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2295 (Player Core pg. 404, Armor Class)

#### Scenario: An item bonus is accepted

- **GIVEN** an item bonus of +2 and an item bonus of +1 with different source ids
- **WHEN** they are totaled
- **THEN** the item bonus is +2
- **AND** the item type was accepted

#### Scenario: Proficiency is not a modifier type

- **GIVEN** the bonus types and the penalty types
- **WHEN** they are listed
- **THEN** the bonus types are circumstance, item, and status
- **AND** the penalty types are circumstance, item, status, and untyped
- **AND** proficiency is not among them

#### Scenario: Phase 1 produces no item modifier

- **GIVEN** the Phase 1 modifiers for an attack roll, Armor Class, Perception, and damage
- **WHEN** their types are listed
- **THEN** none of them is item

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

#### Scenario: A condition bonus and penalty both apply

- **GIVEN** condition 10 on the attack roll, a circumstance bonus of +2 and a circumstance penalty of -1, from the same source at the same sequence
- **WHEN** they are totaled with either modifier first
- **THEN** both modifiers are applied in both orders
- **AND** the total is +1 in both orders

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

### Requirement: Construction rejects an illegal value

A bonus value SHALL be greater than 0 and a penalty value SHALL be less than 0. Any other value, including zero and an untyped bonus, SHALL return a typed construction error and SHALL NOT produce a modifier.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2282 (Player Core pg. 400, Penalties)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2456 (Player Core pg. 442, Condition Values)

#### Scenario: An untyped bonus is rejected

- **GIVEN** a bonus of +2 with the untyped penalty type
- **WHEN** a modifier is created
- **THEN** the error is the untyped-bonus error
- **AND** no modifier is returned

#### Scenario: A zero modifier is rejected

- **GIVEN** a circumstance value of 0
- **WHEN** a modifier is created
- **THEN** the error is the zero-value error
- **AND** no modifier is returned

#### Scenario: A negative bonus is rejected

- **GIVEN** a circumstance bonus of -1
- **WHEN** a modifier is created
- **THEN** the result is a typed construction error
- **AND** no modifier is returned

#### Scenario: A positive penalty is rejected

- **GIVEN** a circumstance penalty of +1
- **WHEN** a modifier is created
- **THEN** the result is a typed construction error
- **AND** no modifier is returned

### Requirement: One effect and the highest condition rank

Each modifier SHALL carry a source id, a sequence number, and its statistic. The key (source id, statistic, type, bonus or penalty) SHALL become one before the sum and SHALL NOT group different conditions. Within a key, the higher level wins, then the higher rank, otherwise the newer (greater) sequence. Value SHALL NOT override that. A repeated condition on one statistic SHALL use the condition-group order. A condition with no rank SHALL apply once per statistic.

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

### Requirement: Totals and flat checks are not clamped

A total SHALL NOT be clamped and MAY be negative. A DC from a total SHALL equal 10 plus that total and SHALL NOT be clamped. A flat check SHALL NOT take a modifier total.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2285 (Player Core pg. 401, Getting a DC from a Modifier)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2278 (Player Core pg. 400, Flat Checks)

#### Scenario: A negative total

- **GIVEN** a circumstance bonus of +1 and untyped penalties that sum to -7
- **WHEN** they are totaled
- **THEN** the total is -6
- **AND** the total is not raised to 0

#### Scenario: A DC below 10

- **GIVEN** a modifier total of -6
- **WHEN** a DC is taken from that total
- **THEN** the DC is 4

#### Scenario: A flat check ignores modifiers

- **GIVEN** a flat check at DC 11, a modifier total of +9, and a scripted natural 10
- **WHEN** the flat check is resolved
- **THEN** the total is 10 and the degree is failure
- **AND** the modifier total is not added

#### Scenario: Insertion order does not change the total

- **GIVEN** a status bonus of +1 and a circumstance penalty of -2 from different sources
- **WHEN** they are totaled with either modifier first
- **THEN** both totals are -1

### Requirement: Attacker and defender modifiers stay apart

Attack rolls, Armor Class, Perception, and damage SHALL each keep their own modifiers. Those collections SHALL NOT compete, so a penalty on one SHALL NOT cancel or replace a penalty on another.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=88 (Player Core pg. 445, Prone)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=58 (Player Core pg. 445, Off-Guard)

#### Scenario: A prone attacker and an off-guard defender both apply

- **GIVEN** a prone penalty of -2 circumstance on the attack roll and an off-guard penalty of -2 circumstance on Armor Class
- **WHEN** the attack roll and the Armor Class are totaled separately
- **THEN** the attack roll includes -2 circumstance
- **AND** the Armor Class includes -2 circumstance
- **AND** neither total drops the other's penalty

### Requirement: Phase 1 application points

An attack roll SHALL take the existing multiple attack penalty, including agile, as one untyped penalty, plus prone at -2 circumstance and an Aid circumstance modifier. The Aid action is out of scope. A zero result SHALL NOT be added. Armor Class SHALL take cover at +1, +2, or +4 circumstance, off-guard at -2 circumstance, and unconscious at -4 status. Perception SHALL take unconscious at -4 status. This change SHALL NOT add a shield bonus and SHALL NOT define a second multiple attack penalty.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2289 (Player Core pg. 402, Multiple Attack Penalty)
**Source**: https://2e.aonprd.com/Actions.aspx?ID=2292 (Player Core pg. 416, Aid)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2372 (Player Core pg. 424, Cover)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=58 (Player Core pg. 445, Off-Guard)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=95 (Player Core pg. 446, Unconscious)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2301 (Player Core pg. 406, Damage Rolls)

#### Scenario: MAP combines with circumstance and status

- **GIVEN** the existing multiple attack penalty of -5, a circumstance penalty of -2, and a status penalty of -4, from different sources
- **WHEN** the attack roll is totaled
- **THEN** the total includes -11

#### Scenario: Agile MAP stays the existing untyped penalty

- **GIVEN** the existing multiple attack penalty of -4 for a second agile attack on the creature's own turn, and a circumstance penalty of -2
- **WHEN** the attack roll is totaled
- **THEN** the total includes -6
- **AND** that -4 is the value the existing multiple attack penalty returns

#### Scenario: An Aid success adds +1 circumstance

- **GIVEN** an Aid circumstance result of +1 and no other circumstance bonus
- **WHEN** the attack roll is totaled
- **THEN** the circumstance bonus is +1

#### Scenario: An Aid failure adds no modifier

- **GIVEN** an Aid result of 0
- **WHEN** it is offered to the attack roll
- **THEN** the attack roll gains no circumstance modifier

#### Scenario: Cover is a circumstance bonus

- **GIVEN** lesser cover, standard cover, and greater cover, each totaled alone
- **WHEN** each Armor Class total is taken
- **THEN** the bonuses are +1, +2, and +4 circumstance

#### Scenario: Off-guard is -2 circumstance to Armor Class

- **GIVEN** off-guard and no other Armor Class modifier
- **WHEN** Armor Class modifiers are totaled
- **THEN** the circumstance penalty is -2

#### Scenario: Unconscious is -4 status to Armor Class and Perception

- **GIVEN** an unconscious creature
- **WHEN** Armor Class modifiers and Perception modifiers are totaled separately
- **THEN** each total includes a status penalty of -4
- **AND** the two collections are separate

#### Scenario: A shield bonus is not applied here

- **GIVEN** a creature whose shield grants +2 Armor Class, and no cover, off-guard, or unconscious
- **WHEN** this change's Armor Class modifiers are totaled
- **THEN** the total from this change is 0
- **AND** the shield bonus is not one of those modifiers

#### Scenario: Damage keeps the highest circumstance bonus

- **GIVEN** damage circumstance bonuses of +4 and +1 from different sources
- **WHEN** the damage modifiers are totaled
- **THEN** the circumstance bonus is +4

### Requirement: Duplicate key comparison is a total order

Within a duplicate key the kept modifier SHALL follow a total order: higher level, a level over none, higher rank, a rank over none, then the greater sequence. If those tie, ruling R-01 (an extension consistent with RAW via Rules 2282, not a deviation) SHALL keep the higher bonus or the more negative penalty. If the value ties, the lower condition id SHALL win and a missing id SHALL lose, then the lower source id. Both steps SHALL be engineering tiebreaks with no rules effect, not a deviation.

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

### Requirement: Damage sets its statistic explicitly

A damage modifier SHALL set its statistic to damage. It SHALL NOT take the attack-roll statistic by default. A damage modifier and an attack modifier that share a source SHALL both remain. Each statistic SHALL keep its own total.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2301 (Player Core pg. 406, Damage Rolls)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2266 (Player Core pg. 398, Duplicate Effects)

#### Scenario: A same-source damage modifier and attack modifier do not merge

- **GIVEN** source 4 at sequence 2, a damage circumstance bonus of +2 and an attack-roll circumstance bonus of +3
- **WHEN** they are resolved together in either order
- **THEN** both modifiers remain
- **AND** the damage total is +2
- **AND** the attack-roll total is +3

### Requirement: Condition groups ignore the source key

Condition modifiers SHALL group by (condition, statistic), not by source key, after the bonus and penalty split. Unequal condition ids, including a present id against none, SHALL NOT share a source key. Order SHALL be the worst penalty or the highest bonus. For a condition whose modifier is derived from its value (frightened, sickened: −value), that is the highest condition value (Player Core p.442, Rules 2455). Otherwise the larger modifier wins (our ruling, see the non-derived requirement).

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2455 (Player Core pg. 442, Redundant Conditions With Values)
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

After the worst penalty or the highest bonus, the order SHALL be the highest rank, then the newest sequence, then the lowest source id. Those three SHALL be engineering tiebreaks with no rules effect.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2455 (context only; the value step is RAW, the rank/sequence/source-id tiebreaks are engineering, see R-01 clarification in docs/designer/rules-rulings.md)

#### Scenario: Two frightened penalties report the lower source in both orders

- **GIVEN** two frightened status penalties of -1 on the attack roll, from source 1 and source 2, both at sequence 5 and rank 2
- **WHEN** they are totaled with either modifier first
- **THEN** one status penalty is applied
- **AND** its source id is 1 in both orders
- **AND** its value is -2, the negation of rank 2
- **AND** the attack-roll total is -2 in both orders

### Requirement: A non-derived modifier keeps the larger one

When a condition's penalty or bonus is not derived from its value, the larger modifier SHALL win: the worst penalty or the highest bonus. This SHALL be our extension, consistent with Rules 2282 (Player Core pg. 400), and SHALL NOT be the text of Rules 2455. It SHALL be a ruling, not a deviation.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2282 (Player Core pg. 400, Penalties)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2455 (Player Core pg. 442, Redundant Conditions With Values)

#### Scenario: A non-derived penalty keeps the larger modifier

- **GIVEN** condition 10 on the attack roll, a circumstance penalty of -4 from source 1 at sequence 5 and rank 1, and a circumstance penalty of -1 from source 2 at sequence 5 and rank 2
- **WHEN** they are totaled with either modifier first
- **THEN** the circumstance penalty is -4 in both orders
- **AND** the kept rank is 1
- **AND** the total is -4 in both orders

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

#### Scenario: The lowest sequence beats the lower source id

- **GIVEN** circumstance bonuses of +2 on the attack roll, from source 2 at sequence 1 and from source 1 at sequence 4, with no condition
- **WHEN** they are totaled with either modifier first
- **THEN** one circumstance bonus is applied
- **AND** its source id is 2 in both orders
- **AND** its sequence number is 1
- **AND** the total is +2 in both orders

### Requirement: Conditions stay separate state

Conditions SHALL stay separate state. Collapsing SHALL affect only the numeric total. Frightened and sickened from one source SHALL both remain when the worse status penalty is applied. Decrementing frightened SHALL leave sickened present. Removing one condition SHALL leave the other present at its rank.

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

#### Scenario: Removing frightened leaves sickened 1

- **GIVEN** one caller's list for one attack roll and source 4, holding frightened 3 as a status penalty at sequence 1 and level 1, and sickened 1 as a status penalty at sequence 2 and level 5
- **WHEN** the list is totaled
- **THEN** the status penalty is -3
- **AND** the total is -3
- **WHEN** frightened is removed entirely and the list is totaled again
- **THEN** the status penalty is -1
- **AND** the total is -1
- **AND** the list still contains sickened at rank 1
- **AND** frightened is absent

### Requirement: A zero rank contributes nothing

A frightened or sickened rank of 0 SHALL contribute no modifier and SHALL NOT appear among the applied modifiers. Another condition on that statistic SHALL still apply. A rank of 0 alone SHALL total 0.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=76 (Player Core pg. 444, Frightened)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=91 (Player Core pg. 446, Sickened)

#### Scenario: Frightened 0 plus sickened 1 gives -1

- **GIVEN** frightened 0 as a status penalty at rank 0 and sickened 1 as a status penalty at rank 1, on the attack roll
- **WHEN** they are totaled
- **THEN** the total is -1
- **AND** sickened 1 is applied
- **AND** frightened is not among the applied modifiers

#### Scenario: Frightened 0 alone gives total 0

- **GIVEN** frightened 0 alone as a status penalty at rank 0 on the attack roll
- **WHEN** it is totaled
- **THEN** the total is 0
- **AND** no modifier is applied
