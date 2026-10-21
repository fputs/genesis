# creature-template

## Purpose

Stores baked creature numbers for Phase 1, including one melee strike and whether a creature that reaches 0 Hit Points dies or gains dying.

## ADDED Requirements

### Requirement: Baked statistics stay as authored

A template SHALL store Hit Points, Armor Class, the Perception modifier, and Speed in feet as the authored numbers. Loading a template SHALL NOT recompute those numbers from level or from how many creatures are in the roster.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2319 (Player Core, Hit Points)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2285 (Player Core pg. 401, a DC is 10 plus the modifier)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2345 (Player Core pg. 420, Speed)

#### Scenario: Authored numbers are kept

- **GIVEN** a template with 20 Hit Points, Armor Class 18, Perception modifier +5, and Speed 25
- **WHEN** the template is loaded
- **THEN** those four numbers are 20, 18, +5, and 25

#### Scenario: A one-creature roster does not change the numbers

- **GIVEN** that same template as the only creature in a roster
- **WHEN** the template is loaded
- **THEN** Hit Points, Armor Class, Perception modifier, and Speed are unchanged

### Requirement: One melee strike

A template SHALL include exactly one melee strike. That strike SHALL record an attack bonus, a damage dice count, die faces, a damage bonus, reach in feet, and whether the weapon is agile.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2306 (Player Core pg. 418, Strike)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2301 (Player Core pg. 406, Damage Rolls)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2289 (Player Core pg. 402, agile)

#### Scenario: Strike fields are kept

- **GIVEN** a melee strike with attack bonus +7, damage 1d8+4, reach 5 feet, and agile false
- **WHEN** the template is loaded
- **THEN** the strike has attack bonus +7, 1 die of 8 faces, bonus +4, reach 5, and agile false

#### Scenario: The agile flag is kept

- **GIVEN** a melee strike with agile true, attack bonus +6, damage 1d6+2, and reach 5 feet
- **WHEN** the template is loaded
- **THEN** the agile flag is true and the other strike numbers are unchanged

### Requirement: Shield bonus is stored per shield

An optional shield SHALL store hardness, Hit Points, broken threshold, and that shield's own AC bonus. The AC bonus SHALL remain the stored value for that shield. The template SHALL NOT replace it with a flat +2.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2181 (Player Core pg. 274, Shield Statistics)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2281 (Player Core pg. 400, a buckler grants +1)

#### Scenario: A buckler bonus stays +1

- **GIVEN** a shield with hardness 3, Hit Points 8, broken threshold 4, and AC bonus +1
- **WHEN** the template is loaded
- **THEN** those four shield numbers are 3, 8, 4, and +1

#### Scenario: A second shield keeps its own bonus

- **GIVEN** another template whose shield has hardness 5, Hit Points 20, broken threshold 10, and AC bonus +2
- **WHEN** that template is loaded
- **THEN** the AC bonus is +2 and the first shield's AC bonus is still +1

#### Scenario: No shield is omitted

- **GIVEN** a template with no shield
- **WHEN** the template is loaded
- **THEN** it has no shield numbers and no AC bonus is invented

### Requirement: Optional stealth DC

An optional stealth DC SHALL equal 10 plus the creature's Stealth modifier. A Seek check against that creature SHALL use this DC. Omitting the field SHALL store no Stealth DC.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2301 (Player Core pg. 417, Seek)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2285 (Player Core pg. 401, DC = 10 + modifier)

#### Scenario: Stealth modifier plus 10

- **GIVEN** a Stealth modifier of +7
- **WHEN** the stealth DC is baked
- **THEN** stealth_dc is 17

#### Scenario: Seek uses the baked DC

- **GIVEN** a creature with stealth_dc 17 and a seeker with Perception modifier +5 and a scripted natural 12
- **WHEN** the Seek check is resolved against that stealth DC
- **THEN** the total is 17 and the degree is success

#### Scenario: Omitted stealth DC stores nothing

- **GIVEN** a template that omits stealth_dc
- **WHEN** the template is loaded
- **THEN** no Stealth DC is stored

### Requirement: Aid modifier against DC 15

When an Aid modifier is baked, an Aid check SHALL add that modifier to a d20 and compare the total with DC 15. Success SHALL grant a +1 circumstance bonus. Failure SHALL grant 0. Critical failure SHALL grant a -1 circumstance penalty. Critical success SHALL grant a +2 circumstance bonus.

**Rule**: Player Core Aid gives +2 circumstance on a critical success, +3 at master, and +4 at legendary. https://2e.aonprd.com/Actions.aspx?ID=2292
**Why**: The baked modifier has no proficiency rank, so master and legendary cannot be chosen.
**Substitute**: Critical success on this Aid check always grants +2 circumstance.
**Intent**: A critical success still helps more than a success, at the usual trained value.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2292 (Player Core pg. 416, Aid)

#### Scenario: Success grants +1

- **GIVEN** an aid modifier of +7 and a scripted natural 10
- **WHEN** the Aid check is resolved against DC 15
- **THEN** the total is 17, the degree is success, and the circumstance bonus is +1

#### Scenario: Critical success grants +2

- **GIVEN** an aid modifier of +7 and a scripted natural 18
- **WHEN** the Aid check is resolved against DC 15
- **THEN** the total is 25, the degree is critical success, and the circumstance bonus is +2

#### Scenario: Failure grants 0

- **GIVEN** an aid modifier of +7 and a scripted natural 4
- **WHEN** the Aid check is resolved against DC 15
- **THEN** the total is 11, the degree is failure, and the circumstance bonus is 0

#### Scenario: Critical failure grants -1

- **GIVEN** an aid modifier of +4 and a scripted natural 1
- **WHEN** the Aid check is resolved against DC 15
- **THEN** the total is 5, the degree is critical failure, and the circumstance penalty is -1

#### Scenario: A natural 20 uses the fixed critical bonus

- **GIVEN** an aid modifier of +0 and a scripted natural 20
- **WHEN** the Aid check is resolved against DC 15
- **THEN** the numeric total 20 is a success, the natural 20 makes it a critical success, and the circumstance bonus is +2

### Requirement: Plain data and CombatantId

Templates SHALL be plain data with no fixed party size. A roster of one creature and a roster of more than four creatures SHALL both be valid. Each creature SHALL be identified by a CombatantId holding a u32, and that id SHALL NOT be limited to a four-slot party index.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2421 (Player Core, Encounter Mode)

#### Scenario: One creature is a valid roster

- **GIVEN** one template identified by CombatantId(0)
- **WHEN** the roster is accepted
- **THEN** the roster length is 1 and the id is 0

#### Scenario: More than four creatures is a valid roster

- **GIVEN** six templates identified by CombatantId(0) through CombatantId(5)
- **WHEN** the roster is accepted
- **THEN** the roster length is 6

#### Scenario: A large id is valid

- **GIVEN** a template identified by CombatantId(4000000000)
- **WHEN** the id is read
- **THEN** the id value is 4000000000

### Requirement: Validation rejects impossible statistics

Validation SHALL reject a template whose Hit Points are 0 or less. Validation SHALL reject a template whose Speed is negative. Speed 0 SHALL be accepted.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2324 (Player Core pg. 410, creatures are not reduced below 0 Hit Points)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2345 (Player Core pg. 420, Speed)

#### Scenario: Zero Hit Points is rejected

- **GIVEN** a template whose other numbers are valid and whose Hit Points are 0
- **WHEN** the template is validated
- **THEN** validation rejects it

#### Scenario: Negative Hit Points are rejected

- **GIVEN** a template whose Hit Points are -1
- **WHEN** the template is validated
- **THEN** validation rejects it

#### Scenario: Negative Speed is rejected

- **GIVEN** a template whose Speed is -5 and whose Hit Points are 12
- **WHEN** the template is validated
- **THEN** validation rejects it

#### Scenario: Speed zero is accepted

- **GIVEN** a template with 12 Hit Points and Speed 0
- **WHEN** the template is validated
- **THEN** validation accepts it and the stored Speed is 0

### Requirement: Aid preparation is not stored

The template SHALL NOT store whether the creature has prepared to Aid. That prepared state belongs to the later Aid change. A template SHALL be valid without any prepared-to-help flag.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2292 (Player Core pg. 416, Aid requires preparing to help)

#### Scenario: No prepared flag is required

- **GIVEN** a template with aid modifier +6 and no prepared-to-help value
- **WHEN** the template is loaded
- **THEN** it is valid and it does not record a prepared-to-help state

### Requirement: uses_dying_rules defaults to death at 0 Hit Points

A template SHALL store uses_dying_rules as a bool. When the caller omits it, the value SHALL be false. A creature with uses_dying_rules false that reaches 0 Hit Points SHALL be dead and SHALL NOT gain dying. A creature with uses_dying_rules true that reaches 0 Hit Points SHALL gain dying as encounter-core specifies and SHALL NOT be dead only because it reached 0 Hit Points. This capability SHALL NOT specify how the dying value changes.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2324 (Player Core pg. 410, Getting Knocked Out)

#### Scenario: Omitting the flag defaults to false

- **GIVEN** a template that does not set uses_dying_rules
- **WHEN** the template is loaded
- **THEN** uses_dying_rules is false

#### Scenario: Without the flag, 0 Hit Points is death

- **GIVEN** a creature with 9 Hit Points, uses_dying_rules false, and a scripted d8 of 8 for damage 1d8+1
- **WHEN** that damage is applied
- **THEN** the creature has 0 Hit Points, is dead, and has no dying condition

#### Scenario: With the flag, 0 Hit Points enters dying

- **GIVEN** a creature with 9 Hit Points, uses_dying_rules true, and a scripted d8 of 8 for damage 1d8+1
- **WHEN** that damage is applied
- **THEN** the creature has 0 Hit Points, is not dead, and gains dying as encounter-core specifies

#### Scenario: Damage that leaves Hit Points above 0 does not kill

- **GIVEN** a creature with 9 Hit Points, uses_dying_rules false, and a scripted d8 of 1 for damage 1d8+1
- **WHEN** that damage is applied
- **THEN** the creature has 7 Hit Points and is not dead
