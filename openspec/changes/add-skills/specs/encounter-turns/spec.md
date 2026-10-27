# encounter-turns

## ADDED Requirements

### Requirement: Recall Knowledge spends one action and a skill check spends none

Recall Knowledge SHALL spend 1 action on the actor's open Alive turn and SHALL NOT change the attack count. A skill check SHALL spend nothing and SHALL ignore the turn and vital. A rejected Recall Knowledge SHALL draw no die.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2367 (Player Core pg. 231, Recall Knowledge)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2429 (Player Core pg. 436, Step 2: Act)

#### Scenario: Recall Knowledge spends one action

- **GIVEN** the actor's turn is open with 3 actions, Society, level 1, Intelligence modifier +0, untrained, a scripted natural 15, and DC 15
- **WHEN** Recall Knowledge is resolved
- **THEN** 2 actions remain, the attack count is 0, and the degree is success

#### Scenario: Recall Knowledge does not change the attack count

- **GIVEN** the actor's turn is open with 2 actions after a Strike, and the attack count is 1
- **WHEN** Recall Knowledge is resolved on a scripted natural 15 against DC 15, with Society, level 1, Intelligence modifier +0, and untrained
- **THEN** the attack count stays 1 and 1 action remains

#### Scenario: A skill check off the active turn rolls

- **GIVEN** CombatantId(1)'s turn is open with 3 actions, and CombatantId(2) has Athletics, level 1, Strength modifier +0, and untrained
- **WHEN** CombatantId(2) resolves a skill check on a scripted natural 6 against DC 15
- **THEN** the error is not NotYourTurn, CombatantId(1) still has 3 actions, and CombatantId(2) still has 0 actions

#### Scenario: A skill check before any turn rolls

- **GIVEN** initiative has been rolled, no turn is open, and the creature has Athletics, level 1, Strength modifier +0, and untrained
- **WHEN** a skill check is resolved on a scripted natural 6 against DC 15
- **THEN** the error is not TurnNotOpen, the degree is failure, and actions stay 0

#### Scenario: Recall Knowledge before any turn rolls nothing

- **GIVEN** initiative has been rolled, no turn is open, and a scripted natural 10
- **WHEN** Recall Knowledge is resolved
- **THEN** the error is TurnNotOpen and the scripted 10 is unused

#### Scenario: Recall Knowledge off the active turn rolls nothing

- **GIVEN** CombatantId(1)'s turn is open with 3 actions, CombatantId(2) has skills, and a scripted natural 10
- **WHEN** CombatantId(2) resolves Recall Knowledge
- **THEN** the error is NotYourTurn, CombatantId(1) still has 3 actions, and the scripted 10 is unused

#### Scenario: An unconscious actor rolls nothing

- **GIVEN** the actor's turn is open with 3 actions, the actor is then unconscious, and a scripted natural 8
- **WHEN** Recall Knowledge is resolved
- **THEN** the error is NotConscious, 3 actions remain, and the scripted 8 is unused

#### Scenario: An unconscious creature can still roll a skill check

- **GIVEN** an unconscious creature with Athletics, level 1, Strength modifier +0, untrained, and a scripted natural 6
- **WHEN** a skill check is resolved against DC 15
- **THEN** the error is not NotConscious and the degree is failure

#### Scenario: Insufficient actions roll nothing

- **GIVEN** an open turn with 0 actions and a scripted natural 8
- **WHEN** Recall Knowledge is resolved
- **THEN** the error is InsufficientActions, 0 actions remain, and the scripted 8 is unused

#### Scenario: No skill training rolls nothing

- **GIVEN** an open turn with 3 actions, no skill training, and a scripted natural 10
- **WHEN** Recall Knowledge is resolved
- **THEN** the error is NoSkills, 3 actions remain, and the scripted 10 is unused

#### Scenario: A skill check with no training rolls nothing

- **GIVEN** a creature with no skill training and a scripted natural 10
- **WHEN** a skill check is resolved
- **THEN** the error is NoSkills and the scripted 10 is unused

#### Scenario: A skill check draws no flat-check die

- **GIVEN** Athletics, level 1, Strength modifier +0, untrained, DC 15, and a script holding 6 then 19
- **WHEN** the skill check is resolved
- **THEN** the natural result is 6, the degree is failure, and the scripted 19 is unused

#### Scenario: An unknown creature rolls nothing

- **GIVEN** a combatant id that is not in the encounter and a scripted natural 10
- **WHEN** a skill check is resolved
- **THEN** the error is UnknownCombatant and the scripted 10 is unused
