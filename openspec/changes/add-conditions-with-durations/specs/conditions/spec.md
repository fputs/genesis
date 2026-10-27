# conditions

## Purpose

Applies, tracks, and removes the core combat conditions on a creature, ticks their durations at the start and end of a turn, and turns the active conditions into modifiers for checks and Armor Class.

## ADDED Requirements

### Requirement: The same conditions twice match

The same instances, the same tick, and the same check SHALL produce the same stored list, the same active value, and the same modifiers. A call SHALL NOT roll dice, read a clock, or read a file, and SHALL NOT panic.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2455 (Player Core pg. 442, Conditions Appendix)

#### Scenario: The same conditions twice match

- **GIVEN** frightened 2 until removed, and a scripted d20 holding 8
- **WHEN** the modifiers for an attack roll are produced twice
- **THEN** both results are equal
- **AND** the scripted 8 is still unused

### Requirement: Apply appends one instance

The conditions that can be applied SHALL be frightened, sickened, off-guard, prone, clumsy, and enfeebled. Applying SHALL append one instance and SHALL NOT replace an instance already stored. A round duration SHALL name a clock creature and SHALL start at 1 or more rounds. Apply SHALL NOT tick.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2455 (Player Core pg. 442, Redundant Conditions)

#### Scenario: Frightened 2 is stored at value 2

- **GIVEN** no conditions, and a scripted d20 holding 8
- **WHEN** frightened 2 until removed is applied
- **THEN** one frightened instance is stored at value 2
- **AND** the scripted 8 is still unused

#### Scenario: A second frightened instance stays stored

- **GIVEN** frightened 1 until removed
- **WHEN** frightened 2 until removed is applied
- **THEN** both instances are stored
- **AND** the active value is 2

### Requirement: Illegal values are rejected

Frightened, sickened, clumsy, and enfeebled SHALL require a value of 1 or more. Off-guard and prone SHALL reject a value. A round duration of 0 SHALL be rejected. A rejection SHALL store nothing and SHALL NOT panic.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2456 (Player Core pg. 442, Condition Values)

#### Scenario: Frightened without a value is rejected

- **GIVEN** no conditions
- **WHEN** frightened is applied with no value
- **THEN** the error rejects the missing value
- **AND** no condition is stored

#### Scenario: Frightened 0 is rejected

- **GIVEN** no conditions
- **WHEN** frightened 0 is applied
- **THEN** the error rejects the ended value
- **AND** no condition is stored

#### Scenario: Off-guard with a value is rejected

- **GIVEN** no conditions
- **WHEN** off-guard 1 is applied
- **THEN** the error rejects the value
- **AND** no condition is stored

#### Scenario: A round duration of 0 is rejected

- **GIVEN** no conditions
- **WHEN** sickened 1 is applied for 0 rounds on CombatantId(1)
- **THEN** the error rejects the duration
- **AND** no condition is stored

### Requirement: The active value is the highest

A valued condition SHALL keep every current instance. Its active value SHALL be the highest value still stored. An unvalued condition SHALL be present while any instance of that name remains. Modifiers SHALL use the active value once.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2455 (Player Core pg. 442, Redundant Conditions With Values)

#### Scenario: The active value is the higher frightened

- **GIVEN** frightened 1 and frightened 3, both until removed
- **WHEN** the active value is read
- **THEN** the active value is 3
- **AND** both instances are stored

#### Scenario: The higher frightened ending leaves the lower one

- **GIVEN** frightened 3 for 1 round on CombatantId(1), and frightened 1 until removed
- **WHEN** CombatantId(1)'s start tick runs
- **THEN** frightened 3 is gone
- **AND** the active value is 1

### Requirement: Reduce affects every instance of that name

Reducing a valued condition by N SHALL subtract N from every instance of that name. An instance at 0 SHALL end. Reducing an absent condition, or reducing by 0, SHALL change nothing. Off-guard and prone SHALL reject a reduction. Other names SHALL stay.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2455 (Player Core pg. 442, Redundant Conditions With Values)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2456 (Player Core pg. 442, Condition Values)

#### Scenario: Reduce lowers every frightened instance

- **GIVEN** frightened 3 until removed and frightened 1 until removed
- **WHEN** frightened is reduced by 1
- **THEN** the stored values are 2 and the second instance has ended
- **AND** the active value is 2

#### Scenario: Frightened 1 reduced by 1 ends

- **GIVEN** frightened 1 until removed
- **WHEN** frightened is reduced by 1
- **THEN** frightened is absent

#### Scenario: Reduce on an absent condition changes nothing

- **GIVEN** sickened 1 until removed, and a scripted d20 holding 8
- **WHEN** frightened is reduced by 1
- **THEN** sickened 1 remains and frightened is absent
- **AND** the scripted 8 is still unused

#### Scenario: Reduce by 0 changes nothing

- **GIVEN** frightened 2 until removed
- **WHEN** frightened is reduced by 0
- **THEN** frightened is 2

#### Scenario: Off-guard rejects a reduction

- **GIVEN** off-guard until removed
- **WHEN** off-guard is reduced by 1
- **THEN** the error rejects the value
- **AND** off-guard remains

#### Scenario: Prone rejects a reduction

- **GIVEN** prone until removed
- **WHEN** prone is reduced by 1
- **THEN** the error rejects the value
- **AND** prone remains

### Requirement: Remove clears every instance of that name

Removing a condition SHALL remove every instance of that name, whatever the values and durations. Removing an absent condition SHALL change nothing. Other names SHALL stay.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2455 (Player Core pg. 442, Redundant Conditions)

#### Scenario: Remove clears frightened and leaves sickened

- **GIVEN** frightened 2 until removed and sickened 1 until removed
- **WHEN** frightened is removed
- **THEN** frightened is absent
- **AND** sickened 1 remains

#### Scenario: Remove on an absent condition changes nothing

- **GIVEN** off-guard until removed
- **WHEN** prone is removed
- **THEN** off-guard remains

### Requirement: Frightened and sickened penalize checks and DCs

Each active value SHALL be one status penalty of that size on attacks, Armor Class, Perception, Fortitude, Reflex, and Will. Neither SHALL penalize damage. On one statistic the worse value SHALL be the only status penalty.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=76 (Player Core pg. 444, Frightened)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=91 (Player Core pg. 446, Sickened)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2282 (Player Core pg. 400, Penalties)

#### Scenario: Frightened 2 and sickened 1 are a status penalty of minus 2

- **GIVEN** frightened 2 and sickened 1, both until removed
- **WHEN** attack-roll modifiers are produced
- **THEN** one status penalty of -2 is applied
- **AND** the total penalty is -2

#### Scenario: Frightened does not penalize damage

- **GIVEN** frightened 2 until removed
- **WHEN** damage modifiers are produced
- **THEN** no penalty is applied

#### Scenario: Tracked frightened 2 and a caller frightened 1 are minus 2

- **GIVEN** tracked frightened 2 and a caller frightened 1 status penalty on the attack roll
- **WHEN** the attack roll is totaled
- **THEN** one status penalty of -2 is applied
- **AND** the total penalty is -2

### Requirement: A bonus and a penalty of the same type both apply

Condition penalties SHALL use the existing stacker. A status bonus and a status penalty SHALL both apply. A status penalty and a circumstance penalty SHALL both apply. Two circumstance penalties SHALL keep only the worst.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2281 (Player Core pg. 400, Bonuses)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2282 (Player Core pg. 400, Penalties)

#### Scenario: A status bonus and frightened both apply

- **GIVEN** frightened 2 until removed and a caller status bonus of +1 on the attack roll
- **WHEN** the attack roll is totaled
- **THEN** the status bonus of +1 is applied
- **AND** the status penalty of -2 is applied
- **AND** the total is -1

#### Scenario: Frightened and off-guard both apply to Armor Class

- **GIVEN** frightened 2 until removed and off-guard until removed
- **WHEN** Armor Class modifiers are produced
- **THEN** a status penalty of -2 is applied
- **AND** a circumstance penalty of -2 is applied
- **AND** the total is -4

### Requirement: Off-guard and prone penalize Armor Class and attacks

Off-guard SHALL be a −2 circumstance penalty to Armor Class and SHALL NOT penalize an attack roll. Prone SHALL include that off-guard penalty and a −2 circumstance penalty to attack rolls. Together they SHALL still be one −2 circumstance penalty to Armor Class.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=58 (Player Core pg. 445, Off-Guard)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=88 (Player Core pg. 445, Prone)

#### Scenario: Off-guard is minus 2 circumstance to Armor Class

- **GIVEN** off-guard until removed
- **WHEN** Armor Class modifiers and attack-roll modifiers are produced
- **THEN** Armor Class has one circumstance penalty of -2
- **AND** the attack roll has no penalty from off-guard

#### Scenario: Prone penalizes attacks and Armor Class

- **GIVEN** prone until removed
- **WHEN** Armor Class modifiers and attack-roll modifiers are produced
- **THEN** Armor Class has one circumstance penalty of -2
- **AND** the attack roll has one circumstance penalty of -2

#### Scenario: Prone and off-guard penalize Armor Class once

- **GIVEN** prone until removed and off-guard until removed
- **WHEN** Armor Class modifiers are produced
- **THEN** one circumstance penalty of -2 is applied
- **AND** the total is -2

### Requirement: Clumsy and enfeebled follow the ability lists

Clumsy SHALL penalize Armor Class, Reflex, a Dexterity melee attack, and a ranged attack by its value. It SHALL NOT penalize Fortitude, Will, Perception, damage, or a Strength melee attack. Enfeebled SHALL penalize a Strength melee attack and its damage by its value, and nothing else in this set.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=61 (Player Core pg. 442, Clumsy)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=71 (Player Core pg. 443, Enfeebled)

#### Scenario: Clumsy 2 penalizes Reflex and a ranged attack

- **GIVEN** clumsy 2 until removed
- **WHEN** Reflex modifiers and ranged attack modifiers are produced
- **THEN** each has one status penalty of -2

#### Scenario: Clumsy 2 does not penalize a Strength melee attack

- **GIVEN** clumsy 2 until removed
- **WHEN** Fortitude modifiers and Strength melee attack modifiers are produced
- **THEN** neither has a penalty

#### Scenario: Enfeebled 2 penalizes a Strength melee attack and its damage

- **GIVEN** enfeebled 2 until removed
- **WHEN** Strength melee attack modifiers and that attack's damage modifiers are produced
- **THEN** each has one status penalty of -2

#### Scenario: Enfeebled 2 does not penalize Armor Class

- **GIVEN** enfeebled 2 until removed
- **WHEN** Armor Class modifiers and ranged attack modifiers are produced
- **THEN** neither has a penalty

### Requirement: The end tick decreases frightened after durations

The end tick runs after the persistent-damage hook from add-resistances-weaknesses-persistent. It SHALL remove due instances clocked to that creature, then reduce that creature's frightened by 1. Other values SHALL NOT decrease. An instance that is not yet due SHALL become due and stay. The tick SHALL NOT roll, change Hit Points, or change a persistent slot.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2430 (Player Core pg. 436, Step 3: End Your Turn)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=76 (Player Core pg. 444, Frightened)

#### Scenario: The end tick lowers frightened and not sickened

- **GIVEN** frightened 2 until removed and sickened 1 until removed on the creature whose turn is ending
- **WHEN** that creature's end tick runs
- **THEN** frightened is 1
- **AND** sickened is 1
- **AND** Hit Points are unchanged

#### Scenario: Frightened 1 ends on the end tick

- **GIVEN** frightened 1 until removed on the creature whose turn is ending
- **WHEN** that creature's end tick runs
- **THEN** frightened is absent

#### Scenario: A due frightened 3 leaves the other frightened at 1

- **GIVEN** frightened 3 whose end is due on this creature, and frightened 2 until removed
- **WHEN** that creature's end tick runs
- **THEN** the due instance is gone
- **AND** the active value is 1

#### Scenario: An end that is not yet due survives one tick

- **GIVEN** off-guard clocked to CombatantId(1) and not yet due
- **WHEN** CombatantId(1)'s end tick runs
- **THEN** off-guard remains
- **AND** the next end tick for CombatantId(1) removes it

#### Scenario: The end tick leaves persistent damage

- **GIVEN** persistent Fire 5 and frightened 2 until removed on the creature whose turn is ending
- **WHEN** that creature's end tick runs
- **THEN** frightened is 1
- **AND** persistent Fire 5 remains
- **AND** Hit Points are unchanged

#### Scenario: A due end removes the condition

- **GIVEN** off-guard clocked to CombatantId(1) and already due
- **WHEN** CombatantId(1)'s end tick runs
- **THEN** off-guard is absent

### Requirement: The start tick decreases round durations

The start tick SHALL decrease by 1 each round duration clocked to that creature, on every bearer. Remaining 0 SHALL end it. Other clocks SHALL stay. Frightened SHALL NOT decrease. The tick SHALL NOT roll or change Hit Points.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2428 (Player Core pg. 435, Step 1: Start Your Turn)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2242 (Player Core pg. 302, Durations)

#### Scenario: One start tick lowers 3 rounds to 2

- **GIVEN** sickened 1 for 3 rounds clocked to CombatantId(1), stored on CombatantId(2)
- **WHEN** CombatantId(1)'s start tick runs
- **THEN** 2 rounds remain
- **AND** sickened 1 is still present

#### Scenario: The third start tick ends a 3-round duration

- **GIVEN** sickened 1 for 3 rounds clocked to CombatantId(1)
- **WHEN** CombatantId(1)'s start tick runs three times
- **THEN** sickened is absent

#### Scenario: Another creature's start does not lower that duration

- **GIVEN** sickened 1 for 3 rounds clocked to CombatantId(1), stored on CombatantId(2)
- **WHEN** CombatantId(2)'s start tick runs
- **THEN** 3 rounds remain
- **AND** sickened 1 is still present

#### Scenario: The start tick does not lower frightened

- **GIVEN** frightened 2 until removed
- **WHEN** that creature's start tick runs
- **THEN** frightened is 2

### Requirement: A prone condition does not restrict movement

A prone condition SHALL apply its penalties and SHALL NOT change Stride, Step, Drop Prone, Stand, or Take Cover. It SHALL NOT grant cover.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=88 (Player Core pg. 445, Prone)

#### Scenario: Prone does not stop a Stride

- **GIVEN** an Alive creature whose turn is open, with prone until removed, and a legal Stride
- **WHEN** that creature Strides
- **THEN** the Stride succeeds
- **AND** prone remains
