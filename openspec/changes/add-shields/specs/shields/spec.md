# shields

## Purpose

Raises a shield for its circumstance bonus until the defender's next turn starts, and reduces one physical attack by Shield Block before the resolved blow.

## ADDED Requirements

### Requirement: The creature's shield is wielded

A shield on the Creature SHALL count as wielded. Shield Hit Points and the raised flag SHALL live on the combatant. The Creature SHALL keep the maximum.

**Rule**: Raise a Shield requires wielding a shield.
**Why not**: Phase 1 has no hands or disarm.
**Substitute**: the Creature's shield is wielded.
**Intent**: there is no hand check.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2316 (Player Core pg. 419, Raise a Shield)

#### Scenario: A fresh steel shield is not raised

- **GIVEN** CombatantId(1) and a steel shield of hardness 5, Hit Points 20, broken threshold 10, and AC bonus +2
- **WHEN** a state is created from that Creature
- **THEN** shield Hit Points are 20, the shield is not raised, it is not broken, and it is not destroyed
- **AND** the Creature's shield is still hardness 5, Hit Points 20, broken threshold 10, and AC bonus +2

#### Scenario: No shield cannot be raised

- **GIVEN** an Alive combatant with 3 actions and no shield
- **WHEN** Raise a Shield is attempted
- **THEN** the result is NoShield, 3 actions remain, and the shield is not raised

### Requirement: Raise a Shield spends 1 action

Raise a Shield SHALL spend 1 action and set the shield raised. It SHALL NOT spend the reaction. A second raise SHALL spend another action and SHALL NOT add the bonus twice. With 0 actions the result SHALL be InsufficientActions and the shield SHALL stay down.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2316 (Player Core pg. 419, Raise a Shield)
**Source**: https://2e.aonprd.com/Shields.aspx?ID=19 (Player Core pg. 274, Steel Shield)

#### Scenario: Raising a steel shield leaves 2 actions

- **GIVEN** an Alive combatant with 3 actions, a ready reaction, and a steel shield that is not raised
- **WHEN** Raise a Shield is used
- **THEN** the shield is raised, 2 actions remain, and the reaction is still ready
- **AND** with no cover and not off-guard, the circumstance bonus is +2 and the Armor Class modifier total is +2
- **AND** the Creature's shield Hit Points are still 20

#### Scenario: Raising again does not stack the bonus

- **GIVEN** an Alive combatant with 3 actions and a steel shield
- **WHEN** Raise a Shield is used twice
- **THEN** 1 action remains, the shield is raised, and the circumstance bonus is +2
- **AND** the circumstance bonus is not +4

#### Scenario: No actions does not raise the shield

- **GIVEN** an Alive combatant with 0 actions and a steel shield
- **WHEN** Raise a Shield is attempted
- **THEN** the result is InsufficientActions, the shield is not raised, and actions stay 0

#### Scenario: The same raise twice matches

- **GIVEN** two equal Alive states, each with 3 actions and a buckler of hardness 3, Hit Points 6, broken threshold 3, and AC bonus +1
- **WHEN** Raise a Shield is resolved on each
- **THEN** the states are equal, both shields are raised, both have 2 actions, and both circumstance bonuses are +1

### Requirement: A broken or destroyed shield cannot be raised

A shield with Hit Points above 0 and at or below its Broken Threshold SHALL be broken. A shield at 0 SHALL be destroyed. Raise a Shield SHALL return Broken or Destroyed and SHALL write nothing.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=60 (Player Core pg. 442, Broken)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2181 (Player Core pg. 274, Shield Statistics)

#### Scenario: A broken shield cannot be raised

- **GIVEN** an Alive combatant with 3 actions and a steel shield at 10 Hit Points, broken threshold 10
- **WHEN** Raise a Shield is attempted
- **THEN** the result is Broken, 3 actions remain, and the shield is not raised

#### Scenario: A destroyed shield cannot be raised

- **GIVEN** an Alive combatant with 3 actions and a steel shield at 0 Hit Points
- **WHEN** Raise a Shield is attempted
- **THEN** the result is Destroyed, 3 actions remain, and the shield is not raised

### Requirement: Only an Alive combatant can raise a shield

A combatant who is not Alive SHALL NOT raise a shield. The result SHALL be NotConscious and the state SHALL stay unchanged.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2316 (Player Core pg. 419, Raise a Shield)

#### Scenario: A dying creature cannot raise a shield

- **GIVEN** vital Dying(1), current Hit Points 0, 3 actions, and a steel shield at 20 Hit Points
- **WHEN** Raise a Shield is attempted
- **THEN** the result is NotConscious, 3 actions remain, and the shield is not raised

### Requirement: The bonus lasts until the next turn starts

The shield SHALL stay raised when this turn ends and when another creature's turn starts. The start of this combatant's turn SHALL lower it before actions are regained and SHALL NOT change shield Hit Points.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2316 (Player Core pg. 419, Raise a Shield)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2428 (Player Core pg. 435, Start Your Turn)

#### Scenario: Ending the turn keeps the shield raised

- **GIVEN** CombatantId(1) with a steel shield and Perception +0, and CombatantId(2) with Perception +0, scripted initiative naturals 10 then 8
- **WHEN** CombatantId(1)'s turn starts, the shield is raised, and that turn ends
- **THEN** the shield is still raised, shield Hit Points are 20, actions are 0, and the reaction is ready
- **AND** the circumstance bonus is +2

#### Scenario: Another turn does not lower the shield

- **GIVEN** the same pair, with CombatantId(1)'s shield already raised and that turn already ended
- **WHEN** CombatantId(2)'s turn starts and ends
- **THEN** CombatantId(1)'s shield is still raised and the circumstance bonus is +2

#### Scenario: The next turn starts with the shield lowered

- **GIVEN** the same pair, after CombatantId(2)'s turn has ended, with CombatantId(1)'s shield still raised at 20 Hit Points
- **WHEN** CombatantId(1)'s next turn starts
- **THEN** the shield is not raised, shield Hit Points are 20, actions are 3, and the reaction is ready
- **AND** with no cover and not off-guard, the Armor Class modifier total is 0

### Requirement: The raised bonus is one circumstance bonus

While raised, and not broken or destroyed, Armor Class SHALL take that shield's listed value as one circumstance bonus in the existing total. Cover SHALL keep the higher bonus. Off-guard SHALL still apply. Equal inputs SHALL match.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2281 (Player Core pg. 400, Bonuses)
**Source**: https://2e.aonprd.com/Actions.aspx?ID=2316 (Player Core pg. 419, Raise a Shield)

#### Scenario: A buckler grants plus 1

- **GIVEN** a raised buckler of hardness 3, Hit Points 6, broken threshold 3, and AC bonus +1, with no cover and not off-guard
- **WHEN** Armor Class modifiers are totaled
- **THEN** the circumstance bonus is +1 and the Armor Class modifier total is +1
- **AND** the circumstance bonus is not +2

#### Scenario: Standard cover and a steel shield stay plus 2

- **GIVEN** a raised steel shield and standard cover, not off-guard
- **WHEN** Armor Class modifiers are totaled
- **THEN** one circumstance bonus of +2 is applied and the Armor Class modifier total is +2
- **AND** the total is not +4

#### Scenario: Greater cover beats the steel shield

- **GIVEN** a raised steel shield and greater cover, not off-guard
- **WHEN** Armor Class modifiers are totaled
- **THEN** the circumstance bonus is +4 and the Armor Class modifier total is +4
- **AND** the steel +2 is not also applied

#### Scenario: A steel shield and off-guard total 0

- **GIVEN** a raised steel shield, off-guard, and no cover
- **WHEN** Armor Class modifiers are totaled
- **THEN** the circumstance bonus is +2, the circumstance penalty is -2, and the Armor Class modifier total is 0

#### Scenario: A lowered shield adds nothing beside cover

- **GIVEN** a steel shield that is not raised, and standard cover
- **WHEN** Armor Class modifiers are totaled
- **THEN** the circumstance bonus is +2 and the Armor Class modifier total is +2
- **AND** no shield bonus is applied

#### Scenario: Unconscious keeps the raised shield bonus

- **GIVEN** vital Unconscious, a raised steel shield, off-guard, and the unconscious penalty, with no cover
- **WHEN** Armor Class modifiers are totaled
- **THEN** the total is -4, from a circumstance bonus of +2, a circumstance penalty of -2, and a status penalty of -4
- **AND** the shield is still raised
- **AND** the total is not -6

#### Scenario: The same Armor Class total twice matches

- **GIVEN** two equal states, each with a raised steel shield and standard cover
- **WHEN** Armor Class modifiers are totaled for each
- **THEN** both totals are +2 and the states are equal

### Requirement: Broken and destroyed grant no bonus

A broken or destroyed shield SHALL add no circumstance bonus, including when the raised flag is still set. It SHALL NOT take the broken-armor status penalty.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=60 (Player Core pg. 442, Broken)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2181 (Player Core pg. 274, Shield Statistics)

#### Scenario: A broken raised shield adds no bonus

- **GIVEN** a raised steel shield at 10 Hit Points, broken threshold 10, with no cover and not off-guard
- **WHEN** Armor Class modifiers are totaled
- **THEN** the Armor Class modifier total is 0
- **AND** the total is not -1, -2, or -3

### Requirement: A negative shield bonus is rejected

A raised shield whose AC bonus is not positive SHALL return a typed error and SHALL NOT report a total.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2281 (Player Core pg. 400, Bonuses)

#### Scenario: A negative shield bonus is rejected

- **GIVEN** a raised shield with AC bonus -1, hardness 5, Hit Points 20, and broken threshold 10
- **WHEN** Armor Class modifiers are totaled
- **THEN** the result is Bonus

### Requirement: Shield Block needs no feat

A creature with a shield SHALL be able to Shield Block while that shield is raised. No feat SHALL be required.

**Rule**: Shield Block is a general feat or a class feature.
**Why not**: Feats and classes are out of Phase 1.
**Substitute**: any creature with a shield can Shield Block.
**Intent**: the reaction needs no feat record.

**Source**: https://2e.aonprd.com/Feats.aspx?ID=5212 (Player Core pg. 262, Shield Block)

#### Scenario: Hardness 5 turns 12 damage into 7

- **GIVEN** an Alive combatant at 30 current Hit Points, maximum 30, 2 actions, a ready reaction, and a raised steel shield at 20 Hit Points, with a scripted d20 showing 15
- **WHEN** an attack blow of 12 at Success requests Shield Block
- **THEN** current Hit Points are 23, shield Hit Points are 13, the shield is not broken, and the reaction is not ready
- **AND** 2 actions remain, the d20 is unused, and the Creature's shield Hit Points are still 20
- **AND** shield Hit Points are not 18

### Requirement: Hardness is subtracted once

On a legal Shield Block, apply_resolved_blow SHALL receive the blow minus hardness, floored at 0. The shield SHALL lose that same remainder, floored at 0. Hardness SHALL be applied once. Hit Points above 0 and at or below the Broken Threshold SHALL be broken. Hit Points 0 SHALL be destroyed.

**Source**: https://2e.aonprd.com/Feats.aspx?ID=5212 (Player Core pg. 262, Shield Block)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2181 (Player Core pg. 274, Shield Statistics)

#### Scenario: Damage equal to hardness deals 0

- **GIVEN** an Alive combatant at 30 current Hit Points, a ready reaction, and a raised steel shield at 20 Hit Points
- **WHEN** an attack blow of 5 at Success requests Shield Block
- **THEN** current Hit Points stay 30, shield Hit Points stay 20, and the reaction is not ready

#### Scenario: Damage below hardness deals 0

- **GIVEN** an Alive combatant at 30 current Hit Points, a ready reaction, and a raised steel shield at 20 Hit Points
- **WHEN** an attack blow of 4 at Success requests Shield Block
- **THEN** current Hit Points stay 30, shield Hit Points stay 20, and the reaction is not ready

#### Scenario: A buckler turns 8 damage into 5

- **GIVEN** an Alive combatant at 30 current Hit Points, a ready reaction, and a raised buckler at 6 Hit Points, hardness 3, broken threshold 3
- **WHEN** an attack blow of 8 at Success requests Shield Block
- **THEN** current Hit Points are 25, shield Hit Points are 1, the shield is broken, and it is not destroyed
- **AND** the reaction is not ready

#### Scenario: The broken threshold breaks the steel shield

- **GIVEN** an Alive combatant at 30 current Hit Points, a ready reaction, and a raised steel shield at 20 Hit Points, broken threshold 10
- **WHEN** an attack blow of 15 at Success requests Shield Block
- **THEN** current Hit Points are 20, shield Hit Points are 10, the shield is broken, and it is not destroyed

#### Scenario: Reducing the shield to 0 destroys it

- **GIVEN** an Alive combatant at 40 current Hit Points, a ready reaction, and a raised steel shield at 20 Hit Points
- **WHEN** an attack blow of 25 at Success requests Shield Block
- **THEN** current Hit Points are 20, shield Hit Points are 0, and the shield is destroyed
- **AND** with no cover and not off-guard, the Armor Class modifier total is 0

#### Scenario: Damage past 0 Hit Points destroys the shield at 0

- **GIVEN** an Alive combatant at 40 current Hit Points, a ready reaction, and a raised steel shield at 20 Hit Points
- **WHEN** an attack blow of 30 at Success requests Shield Block
- **THEN** current Hit Points are 15 and shield Hit Points are 0
- **AND** shield Hit Points are not -5

### Requirement: An illegal block passes the full amount

A hit that does not legally Shield Block SHALL pass its full amount to apply_resolved_blow. Shield Hit Points and the reaction SHALL stay unchanged. There SHALL be no damage type (DEV-09). The trigger SHALL be any attack blow (FIX-11).

**Source**: https://2e.aonprd.com/Feats.aspx?ID=5212 (Player Core pg. 262, Shield Block)

#### Scenario: Declining the block leaves the shield intact

- **GIVEN** an Alive combatant at 30 current Hit Points, a ready reaction, and a raised steel shield at 20 Hit Points
- **WHEN** an attack blow of 12 at Success does not request Shield Block
- **THEN** current Hit Points are 18, shield Hit Points stay 20, and the reaction stays ready

#### Scenario: A shield that is not raised does not block

- **GIVEN** an Alive combatant at 30 current Hit Points, a ready reaction, and a steel shield at 20 Hit Points that is not raised
- **WHEN** an attack blow of 12 at Success requests Shield Block
- **THEN** current Hit Points are 18, shield Hit Points stay 20, and the reaction stays ready

#### Scenario: A spent reaction does not block

- **GIVEN** an Alive combatant at 30 current Hit Points, a reaction that is not ready, and a raised steel shield at 20 Hit Points
- **WHEN** an attack blow of 12 at Success requests Shield Block
- **THEN** current Hit Points are 18, shield Hit Points stay 20, and the reaction stays not ready

#### Scenario: A broken shield does not block

- **GIVEN** an Alive combatant at 30 current Hit Points, a ready reaction, and a raised steel shield at 10 Hit Points, broken threshold 10
- **WHEN** an attack blow of 12 at Success requests Shield Block
- **THEN** current Hit Points are 18, shield Hit Points stay 10, and the reaction stays ready

#### Scenario: A dying creature does not block

- **GIVEN** vital Dying(1), current Hit Points 0, maximum 9, uses_dying_rules, a ready reaction, and a raised steel shield at 20 Hit Points
- **WHEN** an attack blow of 8 at Success requests Shield Block
- **THEN** vital is Dying(2), shield Hit Points stay 20, and the reaction stays ready

### Requirement: A miss does not block

Failure and CriticalFailure SHALL return NotAHit and SHALL write nothing. The reaction SHALL stay ready. This change SHALL NOT declare NotAHit.

**Source**: https://2e.aonprd.com/Feats.aspx?ID=5212 (Player Core pg. 262, Shield Block)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2306 (Player Core pg. 418, Strike)

#### Scenario: A failure is not a blow

- **GIVEN** an Alive combatant at 30 current Hit Points, a ready reaction, and a raised steel shield at 20 Hit Points
- **WHEN** an attack blow of 12 at Failure requests Shield Block
- **THEN** the result is NotAHit, current Hit Points stay 30, shield Hit Points stay 20, and the reaction stays ready

#### Scenario: A critical failure is not a blow

- **GIVEN** an Alive combatant at 30 current Hit Points, a ready reaction, and a raised steel shield at 20 Hit Points
- **WHEN** an attack blow of 12 at CriticalFailure requests Shield Block
- **THEN** the result is NotAHit, current Hit Points stay 30, shield Hit Points stay 20, and the reaction stays ready

### Requirement: The resolved blow is the only creature damage

apply_resolved_blow SHALL be the only creature-damage step. Massive damage, temporary Hit Points, and dying SHALL follow it. A direct call SHALL NOT change the shield. Equal blocks SHALL match.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2332 (Player Core pg. 412, Massive Damage)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2321 (Player Core pg. 410, Temporary Hit Points)

#### Scenario: A blocked 22 is not massive damage

- **GIVEN** uses_dying_rules, maximum Hit Points 10, current Hit Points 10, wounded 0, a ready reaction, and a raised steel shield at 20 Hit Points
- **WHEN** an attack blow of 22 at Success requests Shield Block
- **THEN** current Hit Points are 0, vital is Dying(1), and vital is not Dead
- **AND** shield Hit Points are 3, the shield is broken, and the reaction is not ready

#### Scenario: The same 22 without a block is massive damage

- **GIVEN** uses_dying_rules, maximum Hit Points 10, current Hit Points 10, wounded 0, a ready reaction, and a raised steel shield at 20 Hit Points
- **WHEN** an attack blow of 22 at Success does not request Shield Block
- **THEN** vital is Dead, no dying value is stored, shield Hit Points stay 20, and the reaction stays ready

#### Scenario: Temporary Hit Points are spent by the resolved blow

- **GIVEN** an Alive combatant at 9 current Hit Points, maximum 9, 5 temporary Hit Points with 3 ticks left, a ready reaction, and a raised steel shield at 20 Hit Points
- **WHEN** an attack blow of 12 at Success requests Shield Block
- **THEN** temporary Hit Points are 0, rounds are 0, current Hit Points are 7, and vital is Alive
- **AND** shield Hit Points are 13

#### Scenario: A critical blow is not doubled again

- **GIVEN** uses_dying_rules, maximum Hit Points 10, current Hit Points 10, wounded 0, a ready reaction, and a raised steel shield at 20 Hit Points
- **WHEN** an attack blow of 20 at CriticalSuccess requests Shield Block
- **THEN** current Hit Points are 0, vital is Dying(2), and vital is not Dead
- **AND** shield Hit Points are 5 and the shield is broken

#### Scenario: A remainder of 0 does not increase dying

- **GIVEN** vital Dying(1), current Hit Points 0, maximum 9, a ready reaction, and a raised steel shield at 20 Hit Points
- **WHEN** an attack blow of 4 at Success requests Shield Block
- **THEN** vital is still Dying(1), shield Hit Points stay 20, and the reaction is not ready

#### Scenario: Temporary Hit Points do not stop the dying increase

- **GIVEN** vital Dying(1), current Hit Points 0, maximum 9, 5 temporary Hit Points with 3 ticks left, a ready reaction, and a raised steel shield at 20 Hit Points
- **WHEN** an attack blow of 8 at Success requests Shield Block
- **THEN** temporary Hit Points are 2 with 3 ticks left, vital is Dying(2), and shield Hit Points are 17

#### Scenario: A direct resolved blow does not touch the shield

- **GIVEN** an Alive combatant at 30 current Hit Points, a ready reaction, and a raised steel shield at 20 Hit Points
- **WHEN** apply_resolved_blow is called with amount 12 and Success
- **THEN** current Hit Points are 18, shield Hit Points stay 20, the shield stays raised, and the reaction stays ready

#### Scenario: An inconsistent vital writes nothing

- **GIVEN** vital Alive at 0 current Hit Points, maximum 9, a ready reaction, and a raised steel shield at 20 Hit Points
- **WHEN** an attack blow of 12 at Success requests Shield Block
- **THEN** the result is InconsistentVital, current Hit Points stay 0, shield Hit Points stay 20, and the reaction stays ready

#### Scenario: The same block twice matches

- **GIVEN** two equal Alive states at 30 current Hit Points, each with a ready reaction and a raised steel shield at 20 Hit Points
- **WHEN** an attack blow of 12 at Success requests Shield Block on each
- **THEN** the states are equal, both current Hit Points are 23, and both shield Hit Points are 13

### Requirement: Negative hardness writes nothing

Hardness below 0 SHALL return a typed error and SHALL NOT change Hit Points, shield Hit Points, or the reaction.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2181 (Player Core pg. 274, Shield Statistics)

#### Scenario: A negative hardness writes nothing

- **GIVEN** an Alive combatant at 30 current Hit Points, a ready reaction, and a raised shield of hardness -1, Hit Points 20, broken threshold 10, and AC bonus +2
- **WHEN** an attack blow of 12 at Success requests Shield Block
- **THEN** the result is Hardness, current Hit Points stay 30, shield Hit Points stay 20, and the reaction stays ready

### Requirement: A hitting Strike keeps baked Armor Class

A hitting Strike SHALL compare the target's baked Armor Class. The raised bonus SHALL NOT change that degree. Creature damage from that hit SHALL go through Shield Block, when requested, and then apply_resolved_blow. A miss SHALL NOT change the shield.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2306 (Player Core pg. 418, Strike)
**Source**: https://2e.aonprd.com/Feats.aspx?ID=5212 (Player Core pg. 262, Shield Block)

#### Scenario: A blocked Strike still hits baked Armor Class

- **GIVEN** CombatantId(1) with a steel shield, 30 current Hit Points, maximum 30, and baked Armor Class 11, and CombatantId(2) with attack bonus +0, both Perception +0, scripted initiative naturals 10 then 8
- **WHEN** CombatantId(1) raises the shield and ends the turn, and CombatantId(2) Strikes with a natural 11, damage 1d8+4 showing 8, and Shield Block requested
- **THEN** the degree is success, CombatantId(1)'s current Hit Points are 23, and shield Hit Points are 13
- **AND** CombatantId(1)'s reaction is not ready, and CombatantId(2) has 2 actions

#### Scenario: A Strike that does not block leaves the shield

- **GIVEN** a target with a steel shield at 20 Hit Points that is not raised, 20 current Hit Points, maximum 20, attack bonus +0, baked Armor Class 11, a natural 11, and damage 1d8 showing 8
- **WHEN** that Strike does not request Shield Block
- **THEN** current Hit Points are 12, shield Hit Points stay 20, and the shield is not raised
