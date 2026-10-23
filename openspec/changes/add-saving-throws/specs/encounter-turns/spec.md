# encounter-turns

## ADDED Requirements

### Requirement: A basic save is not a Strike

A basic save SHALL spend no action, no reaction, and no attack count. It SHALL draw no flat-check die and no damage die. It SHALL be legal before any turn is open and when it is not the defender's turn. Strike SHALL still spend 1 action on an open turn, compare with baked Armor Class, and SHALL NOT roll a save.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2296 (Player Core pg. 404, Saving Throws)
**Source**: https://2e.aonprd.com/Actions.aspx?ID=2306 (Player Core pg. 418, Strike)

#### Scenario: A save after a Strike is not a Strike

- **GIVEN** CombatantId(1) and CombatantId(2), Perception +0, scripted initiative naturals 10 then 8, CombatantId(1)'s turn open, attack bonus +0, baked Armor Class 11, damage 1d8+0, CombatantId(2) at 30 Hit Points with Reflex, level 1, Dexterity modifier +0, and untrained

- **WHEN** CombatantId(1) Strikes with a scripted natural 11 and a damage die of 8, then CombatantId(2) succeeds at a basic Reflex save on a scripted natural 15 against DC 15 for listed damage 5

- **THEN** the Strike total is 11, the Strike degree is success, and the Strike damage is 8

- **AND** 2 actions remain, the attack count is 1, the save damage is 2, and Hit Points are 20

- **AND** the save does not change the action count or the attack count

#### Scenario: A save draws no flat-check die

- **GIVEN** an Alive defender at 20 Hit Points, Fortitude, level 1, Constitution modifier +0, untrained, listed damage 5, DC 15, and a script holding 6 then 19
- **WHEN** the basic save is resolved
- **THEN** the degree is failure, the damage is 5, and the scripted 19 is unused

#### Scenario: A save before any turn rolls

- **GIVEN** initiative has been rolled, no turn is open, the defender is Alive at 20 Hit Points with Fortitude, level 1, Constitution modifier +0, untrained, and a scripted natural 6
- **WHEN** a basic save is resolved against DC 15 for listed damage 5
- **THEN** the error is not TurnNotOpen, the damage is 5, Hit Points are 15, and actions stay 0

#### Scenario: A save off the active turn rolls

- **GIVEN** CombatantId(1)'s turn is open with 3 actions, and CombatantId(2) is Alive at 20 Hit Points with Will, level 1, Wisdom modifier +0, and untrained
- **WHEN** CombatantId(2) resolves a basic save on a scripted natural 6 against DC 15 for listed damage 5
- **THEN** the error is not NotYourTurn, CombatantId(1) still has 3 actions, and CombatantId(2) has 15 Hit Points
