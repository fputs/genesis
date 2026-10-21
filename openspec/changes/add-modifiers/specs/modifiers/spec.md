# modifiers

## Purpose

Folds circumstance, item, status, and untyped modifiers onto attack rolls, Armor Class, Perception, and damage, and rejects an untyped bonus or a zero value.

## ADDED Requirements

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

For one type, only the highest bonus and only the worst penalty SHALL apply. When different sources tie for that bonus or that penalty, the lowest sequence number SHALL be the one reported as applied. A bonus and a penalty of the same type SHALL both apply. Every untyped penalty SHALL be added. A sum that overflows SHALL return a typed error and SHALL NOT saturate.

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

Each modifier SHALL carry a source id and a sequence number. Modifiers that share a source id SHALL become one before any sum: the higher level or rank wins, otherwise the newer sequence number, which is the greater one. Value SHALL NOT decide that tie. A repeated condition SHALL keep only its highest rank before that sum. Frightened and sickened SHALL then contribute one status penalty equal to the negation of that rank. A condition with no rank SHALL apply only once.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2266 (Player Core pg. 398, Duplicate Effects)
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
