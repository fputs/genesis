# saving-throws

## Purpose

Resolves Fortitude, Reflex, and Will from an attribute modifier and a proficiency bonus, then applies basic-save damage through the existing blow.

## ADDED Requirements

### Requirement: Check degrees follow the DC bands

A saving throw SHALL be a success when the total is at least the DC, a critical success at the DC plus 10, and a critical failure at the DC minus 10. Every other total SHALL be a failure. The degree SHALL be the existing check's degree.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2286 (Player Core pg. 401, Step 4: Degree of Success)

#### Scenario: Total equal to the DC is a success

- **GIVEN** Fortitude, level 1, Constitution modifier +0, untrained, no other modifier, a scripted natural 15, and DC 15
- **WHEN** the save is resolved
- **THEN** the total is 15 and the degree is success

#### Scenario: Beating the DC by 9 is a success

- **GIVEN** Fortitude, level 1, Constitution modifier +0, untrained, a circumstance bonus of +10 on Fortitude, a scripted natural 14, and DC 15
- **WHEN** the save is resolved
- **THEN** the total is 24 and the degree is success

#### Scenario: Beating the DC by 10 is a critical success

- **GIVEN** Fortitude, level 1, Constitution modifier +0, untrained, a circumstance bonus of +10 on Fortitude, a scripted natural 15, and DC 15
- **WHEN** the save is resolved
- **THEN** the total is 25 and the degree is critical success

#### Scenario: Missing the DC by 9 is a failure

- **GIVEN** Fortitude, level 1, Constitution modifier +0, untrained, no other modifier, a scripted natural 6, and DC 15
- **WHEN** the save is resolved
- **THEN** the total is 6 and the degree is failure

#### Scenario: Missing the DC by 10 is a critical failure

- **GIVEN** Fortitude, level 1, Constitution modifier +0, untrained, no other modifier, a scripted natural 5, and DC 15
- **WHEN** the save is resolved
- **THEN** the total is 5 and the degree is critical failure

### Requirement: Natural 20 and natural 1 shift one degree first

A natural 20 SHALL improve the numeric degree by one step, and a natural 1 SHALL worsen it by one step, before damage is chosen. A critical success SHALL NOT improve further. A critical failure SHALL NOT worsen further.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2287 (Player Core pg. 401, Natural 1 and Natural 20)

#### Scenario: Natural 20 turns a failure into a success

- **GIVEN** Fortitude, level 1, Constitution modifier +0, untrained, a scripted natural 20, and DC 25
- **WHEN** the save is resolved
- **THEN** the total is 20, the numeric degree is failure, and the final degree is success

#### Scenario: Natural 20 turns a success into a critical success

- **GIVEN** Fortitude, level 1, Constitution modifier +0, untrained, a scripted natural 20, and DC 18
- **WHEN** the save is resolved
- **THEN** the total is 20 and the degree is critical success

#### Scenario: Natural 20 does not raise a critical success

- **GIVEN** Fortitude, level 1, Constitution modifier +8, untrained, a scripted natural 20, and DC 18
- **WHEN** the save is resolved
- **THEN** the total is 28 and the degree stays critical success

#### Scenario: Natural 20 turns a critical failure into a failure

- **GIVEN** Fortitude, level 1, Constitution modifier +0, untrained, a scripted natural 20, and DC 30
- **WHEN** the save is resolved
- **THEN** the total is 20, the numeric degree is critical failure, and the final degree is failure

#### Scenario: Natural 1 turns a success into a failure

- **GIVEN** Fortitude, level 1, Constitution modifier +17, untrained, a scripted natural 1, and DC 15
- **WHEN** the save is resolved
- **THEN** the total is 18, the numeric degree is success, and the final degree is failure

#### Scenario: Natural 1 turns a critical success into a success

- **GIVEN** Fortitude, level 1, Constitution modifier +24, untrained, a scripted natural 1, and DC 15
- **WHEN** the save is resolved
- **THEN** the total is 25 and the degree is success

#### Scenario: Natural 1 does not lower a critical failure

- **GIVEN** Fortitude, level 1, Constitution modifier +0, untrained, a scripted natural 1, and DC 15
- **WHEN** the save is resolved
- **THEN** the total is 1 and the degree stays critical failure

### Requirement: Basic saves use four damage multipliers

A basic save SHALL deal no damage on a critical success, half on a success, the listed amount on a failure, and double on a critical failure. Half SHALL round down. Half of 1 SHALL stay 1. Double SHALL apply once.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2297 (Player Core pg. 404, Basic Saving Throws)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2307 (Player Core pg. 407, Doubling and Halving Damage)

#### Scenario: Critical success deals no damage

- **GIVEN** a critical success and listed damage 5
- **WHEN** the basic-save damage is resolved
- **THEN** the damage is 0

#### Scenario: Success halves 5 to 2

- **GIVEN** a success and listed damage 5
- **WHEN** the basic-save damage is resolved
- **THEN** the damage is 2

#### Scenario: Success halves 7 to 3

- **GIVEN** a success and listed damage 7
- **WHEN** the basic-save damage is resolved
- **THEN** the damage is 3

#### Scenario: Success on 1 damage stays 1

- **GIVEN** a success and listed damage 1
- **WHEN** the basic-save damage is resolved
- **THEN** the damage is 1

#### Scenario: Success on 0 damage stays 0

- **GIVEN** a success and listed damage 0
- **WHEN** the basic-save damage is resolved
- **THEN** the damage is 0

#### Scenario: Failure deals the full amount

- **GIVEN** a failure and listed damage 5
- **WHEN** the basic-save damage is resolved
- **THEN** the damage is 5

#### Scenario: Critical failure doubles once

- **GIVEN** a critical failure and listed damage 5
- **WHEN** the basic-save damage is resolved
- **THEN** the damage is 10

#### Scenario: Critical success on 1 damage deals 0

- **GIVEN** a critical success and listed damage 1
- **WHEN** the basic-save damage is resolved
- **THEN** the damage is 0

#### Scenario: Critical failure on 0 damage stays 0

- **GIVEN** a critical failure and listed damage 0
- **WHEN** the basic-save damage is resolved
- **THEN** the damage is 0

### Requirement: A negative amount rolls nothing

A negative listed amount SHALL be rejected before a die is drawn. Hit Points SHALL stay unchanged.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2297 (Player Core pg. 404, Basic Saving Throws)

#### Scenario: A negative amount rolls nothing

- **GIVEN** an Alive defender with save training, 20 Hit Points, listed damage -1, and a scripted natural 10
- **WHEN** a basic save is attempted
- **THEN** the error is NegativeAmount, Hit Points stay 20, and the scripted 10 is unused

### Requirement: Save modifiers use the attribute and proficiency

Fortitude SHALL add Constitution, Reflex Dexterity, and Will Wisdom, then that proficiency bonus, then modifiers on that save only. Untrained SHALL be legal and 0 (DEV-52, proposed). Trained, expert, master, and legendary SHALL be level plus 2, 4, 6, and 8. Proficiency SHALL NOT be a bonus type.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2296 (Player Core pg. 404, Saving Throws)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2281 (Player Core pg. 400, Bonuses)

#### Scenario: Fortitude is Constitution plus trained at level 1

- **GIVEN** Fortitude, level 1, Constitution modifier +2, trained, no other modifier, a scripted natural 10, and DC 15
- **WHEN** the save is resolved
- **THEN** the modifier is +5, the total is 15, and the degree is success

#### Scenario: Reflex is Dexterity plus expert at level 5

- **GIVEN** Reflex, level 5, Dexterity modifier -1, expert, no other modifier, a scripted natural 10, and DC 18
- **WHEN** the save is resolved
- **THEN** the modifier is +8, the total is 18, and the degree is success

#### Scenario: Will is Wisdom plus legendary at level 20

- **GIVEN** Will, level 20, Wisdom modifier +4, legendary, no other modifier, a scripted natural 10, and DC 40
- **WHEN** the save is resolved
- **THEN** the modifier is +32, the total is 42, and the degree is success

#### Scenario: Untrained is 0 at level 20

- **GIVEN** Fortitude, level 20, Constitution modifier +3, untrained, no other modifier, a scripted natural 10, and DC 13
- **WHEN** the save is resolved
- **THEN** the modifier is +3, the total is 13, and the degree is success

#### Scenario: Trained at level 2 with Wisdom 0 is plus 4

- **GIVEN** Will, level 2, Wisdom modifier +0, trained, no other modifier, a scripted natural 10, and DC 14
- **WHEN** the save is resolved
- **THEN** the modifier is +4 and the total is 14

#### Scenario: An item bonus adds beside proficiency

- **GIVEN** Fortitude, level 1, Constitution modifier +2, trained, an item bonus of +1 on Fortitude, a scripted natural 10, and DC 16
- **WHEN** the save is resolved
- **THEN** the modifier is +6 and the total is 16

#### Scenario: Two circumstance bonuses keep the highest

- **GIVEN** Fortitude, level 1, Constitution modifier +2, trained, circumstance bonuses of +2 and +1 on Fortitude, and a scripted natural 10
- **WHEN** the save is resolved against DC 17
- **THEN** the modifier is +7 and the total is 17
- **AND** the modifier is not +8

#### Scenario: An attack penalty does not change Fortitude

- **GIVEN** Fortitude, level 1, Constitution modifier +2, trained, a circumstance penalty of -2 on Attack, a scripted natural 10, and DC 15
- **WHEN** the save is resolved
- **THEN** the modifier is +5 and the total is 15
- **AND** the total is not 13

#### Scenario: Fortitude and Will do not share a bonus

- **GIVEN** level 1, Constitution modifier +2, Wisdom modifier +0, both trained, a circumstance bonus of +2 on Fortitude, a circumstance bonus of +1 on Will, and a scripted natural 10
- **WHEN** each save is resolved against DC 14
- **THEN** the Fortitude modifier is +7 and the Will modifier is +4

#### Scenario: The same modifier twice matches

- **GIVEN** two equal Fortitude trainings at level 1, Constitution modifier +2, trained, and no other modifier
- **WHEN** each modifier is calculated
- **THEN** both modifiers are +5

### Requirement: Unconscious penalizes Reflex only

Dying and unconscious SHALL include the existing Reflex status penalty of -4. Fortitude and Will SHALL NOT include it. Alive SHALL NOT include it.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=95 (Player Core pg. 446, Unconscious)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2325 (Player Core pg. 411, Dying)

#### Scenario: Unconscious Reflex includes minus 4

- **GIVEN** an unconscious defender, Reflex, level 1, Dexterity modifier +0, untrained, a scripted natural 15, and DC 15
- **WHEN** the save is resolved
- **THEN** the modifier is -4, the total is 11, and the degree is failure

#### Scenario: Alive Reflex does not include minus 4

- **GIVEN** an Alive defender, Reflex, level 1, Dexterity modifier +0, untrained, a scripted natural 15, and DC 15
- **WHEN** the save is resolved
- **THEN** the modifier is +0, the total is 15, and the degree is success

#### Scenario: A dying creature's Reflex includes minus 4

- **GIVEN** a defender at dying 1, Reflex, level 1, Dexterity modifier +2, trained, a scripted natural 10, and DC 11
- **WHEN** the save is resolved
- **THEN** the modifier is +1 and the total is 11

#### Scenario: Unconscious Fortitude does not include minus 4

- **GIVEN** an unconscious defender, Fortitude, level 1, Constitution modifier +0, untrained, a scripted natural 15, and DC 15
- **WHEN** the save is resolved
- **THEN** the modifier is +0, the total is 15, and the degree is success

#### Scenario: Unconscious Will does not include minus 4

- **GIVEN** an unconscious defender, Will, level 1, Wisdom modifier +0, untrained, a scripted natural 15, and DC 15
- **WHEN** the save is resolved
- **THEN** the modifier is +0 and the total is 15

### Requirement: A saving throw DC is 10 plus the modifier

A saving throw DC SHALL be 10 plus that save's modifier. This SHALL use the existing DC-from-modifier rule. No Grapple action SHALL be added.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2296 (Player Core pg. 404, Saving Throws)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2285 (Player Core pg. 401, Getting a DC from a Modifier)

#### Scenario: A modifier of plus 7 has DC 17

- **GIVEN** Fortitude, level 1, Constitution modifier +2, trained, an item bonus of +1, and a circumstance bonus of +1, all on Fortitude
- **WHEN** the save DC is taken from that modifier
- **THEN** the DC is 17
- **AND** a scripted natural 10 against that DC totals 17 and is a success

### Requirement: A modifier that does not fit rolls nothing

A proficiency bonus or a save modifier that does not fit the existing check modifier SHALL be rejected before a die is drawn.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2296 (Player Core pg. 404, Saving Throws)

#### Scenario: Trained at the maximum level rolls nothing

- **GIVEN** Fortitude, level 2147483647, Constitution modifier +0, trained, and a scripted natural 10
- **WHEN** the save is attempted
- **THEN** the error is SaveOverflow and the scripted 10 is unused

#### Scenario: A Constitution modifier at the maximum rolls nothing

- **GIVEN** Fortitude, level 1, Constitution modifier 2147483647, untrained, and a scripted natural 10
- **WHEN** the save is attempted
- **THEN** the error is SaveOverflow and the scripted 10 is unused

### Requirement: Doubling that does not fit writes nothing

Doubling that does not fit SHALL leave Hit Points unchanged after the save die has been drawn.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2307 (Player Core pg. 407, Doubling and Halving Damage)

#### Scenario: Doubling the maximum damage writes nothing

- **GIVEN** an Alive defender at 20 Hit Points, Fortitude, level 1, Constitution modifier +0, untrained, a scripted natural 5, DC 15, and listed damage 2147483647
- **WHEN** the basic save is resolved
- **THEN** the degree is critical failure, the error is DamageOverflow, Hit Points stay 20, and the scripted 5 was drawn

### Requirement: Basic-save damage uses the existing blow

Failure SHALL apply as an ordinary blow. Critical failure SHALL apply as a critical blow, already doubled once. Damage modifiers and Shield Block SHALL NOT change it (DEV-53, proposed).

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2324 (Player Core pg. 410, Getting Knocked Out)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2327 (Player Core pg. 411, Taking Damage)
**Source**: https://2e.aonprd.com/Feats.aspx?ID=5212 (Player Core pg. 262, Shield Block)

#### Scenario: Failure to 0 sets dying 1

- **GIVEN** a defender with 5 Hit Points, uses_dying_rules true, Fortitude, level 1, Constitution modifier +0, untrained, a scripted natural 6, DC 15, and listed damage 5
- **WHEN** the basic save is resolved
- **THEN** the degree is failure, the damage is 5, Hit Points are 0, and the defender is dying 1

#### Scenario: Critical failure to 0 sets dying 2

- **GIVEN** a defender with 5 Hit Points, uses_dying_rules true, Fortitude, level 1, Constitution modifier +0, untrained, a scripted natural 5, DC 15, and listed damage 5
- **WHEN** the basic save is resolved
- **THEN** the damage is 10, Hit Points are 0, and the defender is dying 2
- **AND** the defender is not dying 1 and the damage is not 20

#### Scenario: Failure while dying 1 increases to dying 2

- **GIVEN** a defender at dying 1 and 0 Hit Points, uses_dying_rules true, Fortitude, level 1, Constitution modifier +0, untrained, a scripted natural 6, DC 15, and listed damage 1
- **WHEN** the basic save is resolved
- **THEN** the damage is 1 and the defender is dying 2

#### Scenario: Critical failure while dying 1 increases by 2

- **GIVEN** a defender at dying 1 and 0 Hit Points, uses_dying_rules true, Fortitude, level 1, Constitution modifier +0, untrained, a scripted natural 5, DC 15, and listed damage 1
- **WHEN** the basic save is resolved
- **THEN** the damage is 2 and the defender is dying 3

#### Scenario: Without dying rules, 0 Hit Points is death

- **GIVEN** a defender with 5 Hit Points, uses_dying_rules false, Fortitude, level 1, Constitution modifier +0, untrained, a scripted natural 6, DC 15, and listed damage 5
- **WHEN** the basic save is resolved
- **THEN** the damage is 5 and the defender is dead

#### Scenario: A raised shield does not block

- **GIVEN** an Alive defender at 30 Hit Points with a raised steel shield at 20 Hit Points and a ready reaction, Fortitude, level 1, Constitution modifier +0, untrained, a scripted natural 6, DC 15, and listed damage 12
- **WHEN** the basic save is resolved
- **THEN** Hit Points are 18, shield Hit Points stay 20, and the reaction stays ready

#### Scenario: A damage modifier does not change the amount

- **GIVEN** an Alive defender at 30 Hit Points, an untyped penalty of -6 on Damage, Fortitude, level 1, Constitution modifier +0, untrained, a scripted natural 6, DC 15, and listed damage 5
- **WHEN** the basic save is resolved
- **THEN** the damage is 5 and Hit Points are 25

#### Scenario: The same save twice matches

- **GIVEN** two encounters, each with an Alive defender at 30 Hit Points, Fortitude, level 1, Constitution modifier +2, trained, a scripted natural 10, DC 15, and listed damage 5
- **WHEN** each basic save is resolved
- **THEN** both totals are 15, both degrees are success, both damage results are 2, and both defenders have 28 Hit Points

### Requirement: A save that cannot be attempted rolls nothing

An unknown creature, a dead creature, and a creature with no save training SHALL be rejected before a die is drawn.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2296 (Player Core pg. 404, Saving Throws)

#### Scenario: An unknown defender rolls nothing

- **GIVEN** an encounter and a scripted natural 10
- **WHEN** a basic save names a CombatantId that is not in the encounter
- **THEN** the error is UnknownCombatant and the scripted 10 is unused

#### Scenario: A dead defender rolls nothing

- **GIVEN** a dead defender with save training, listed damage -1, and a scripted natural 10
- **WHEN** a basic save is attempted
- **THEN** the error is TargetDead, no Hit Points change, and the scripted 10 is unused

#### Scenario: No save training rolls nothing

- **GIVEN** an Alive defender with no save training, 20 Hit Points, and a scripted natural 10
- **WHEN** a basic save is attempted
- **THEN** the error is NoSavingThrows, Hit Points stay 20, and the scripted 10 is unused

### Requirement: A creature may save against itself

A basic save SHALL be legal when the defender is the creature whose turn is open. The result SHALL NOT be the Strike self-target error.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2296 (Player Core pg. 404, Saving Throws)

#### Scenario: A creature may save against itself

- **GIVEN** an open turn, the active creature at 20 Hit Points, Fortitude, level 1, Constitution modifier +0, untrained, a scripted natural 6, DC 15, and listed damage 5
- **WHEN** that creature attempts the basic save
- **THEN** the result is not the self-target error, the damage is 5, and Hit Points are 15

### Requirement: A delayed creature can save

A delayed creature SHALL be able to attempt a save. The result SHALL NOT be the error used when a delayed creature spends a reaction.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2296 (Player Core pg. 404, Saving Throws)
**Source**: https://2e.aonprd.com/Actions.aspx?ID=2294 (Player Core pg. 416, Delay)

#### Scenario: A delayed creature can save

- **GIVEN** CombatantId(1) and CombatantId(2), Perception +0, scripted initiative naturals 15 then 10, CombatantId(2) Delays on its turn, and CombatantId(2) is Alive at 20 Hit Points with Fortitude, level 1, Constitution modifier +0, untrained
- **WHEN** CombatantId(2) attempts a basic save with a scripted natural 6, DC 15, and listed damage 1
- **THEN** the error is not Delayed, the damage is 1, and Hit Points are 19

### Requirement: Knockout placement follows the existing blow

A save that newly stores dying for a delayed defender, while a turn is open, SHALL place that defender directly before the open creature. Any other new dying value SHALL leave the order unchanged and SHALL leave the pending knockout report.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2324 (Player Core pg. 410, Getting Knocked Out)

#### Scenario: A delayed knockout is placed before the open creature

- **GIVEN** CombatantId(1) and CombatantId(2), Perception +0, scripted initiative naturals 15 then 10, both with uses_dying_rules true, CombatantId(2) Delays, then CombatantId(1)'s turn is open, and CombatantId(2) has 5 Hit Points, Fortitude, level 1, Constitution modifier +0, and untrained
- **WHEN** CombatantId(2) fails a basic save on a scripted natural 6 against DC 15 for listed damage 5
- **THEN** CombatantId(2) is dying 1 and the order is CombatantId(2) then CombatantId(1)

#### Scenario: A knockout that is not delayed leaves the order

- **GIVEN** CombatantId(1) then CombatantId(2), CombatantId(1)'s turn open, CombatantId(2) not delayed, 5 Hit Points, uses_dying_rules true, Fortitude, level 1, Constitution modifier +0, and untrained
- **WHEN** CombatantId(2) fails a basic save on a scripted natural 6 against DC 15 for listed damage 5
- **THEN** the order stays CombatantId(1) then CombatantId(2), CombatantId(2) is dying 1, and the knockout report is pending
