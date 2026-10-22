# encounter-turns

## MODIFIED Requirements

### Requirement: Strike spends one action against baked Armor Class

On an open Alive turn, a Strike SHALL spend 1 action, roll one d20, add the strike's attack bonus and the multiple attack penalty, and compare that total with the target's baked Armor Class. No square and no reach SHALL be required. The degree SHALL use that baked Armor Class. A target whose vital is neither Alive nor Dead keeps that baked Armor Class until wire-dying-into-turns replaces this result (DEV-36). A Reactive Strike while it is not that reactor's open turn SHALL spend no action, SHALL apply a multiple attack penalty of 0, and SHALL leave the stored attack count unchanged. Encounter::strike off the active turn SHALL still return NotYourTurn.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2306 (Player Core pg. 418, Strike)

#### Scenario: The first Strike totals the d20 plus the attack bonus

- **GIVEN** an open turn, attack bonus +8, Armor Class 18, 0 damage dice, an empty modifier list, and a scripted natural 10
- **WHEN** the active combatant Strikes the other
- **THEN** the multiple attack penalty is 0
- **AND** the total is 18
- **AND** the degree is success
- **AND** 2 actions remain
- **AND** the turn stays open

#### Scenario: A not-alive target keeps baked Armor Class

- **GIVEN** CombatantId(1) and CombatantId(2), both with Perception +0, scripted initiative naturals 10 then 8, and CombatantId(2) at 9 Hit Points with uses_dying_rules true and baked Armor Class 18
- **WHEN** CombatantId(1) Strikes with attack bonus +10, natural 15, and damage 1d8+1 showing 8, CombatantId(1)'s turn ends, CombatantId(2)'s turn starts and ends, CombatantId(1)'s next turn starts, and CombatantId(1) Strikes with attack bonus +8, 0 damage dice, and natural 10
- **THEN** CombatantId(2) is not Alive and is not Dead when the second Strike is rolled
- **AND** the second total is 18
- **AND** the second degree is success
- **AND** this baked Armor Class result is replaced by wire-dying-into-turns (DEV-36)

#### Scenario: Strike before the turn starts rolls nothing

- **GIVEN** initiative has been rolled, no turn is open, and a scripted natural 10
- **WHEN** a Strike is attempted
- **THEN** the error is TurnNotOpen
- **AND** the scripted 10 is still unused

#### Scenario: A Strike off the active turn rolls nothing

- **GIVEN** it is another combatant's open turn, and a scripted natural 10
- **WHEN** a combatant who is not active Strikes
- **THEN** the error is NotYourTurn
- **AND** the scripted 10 is still unused

#### Scenario: An unknown target rolls nothing

- **GIVEN** an open turn with 3 actions and a scripted natural 10
- **WHEN** the active combatant Strikes a CombatantId that is not in the encounter
- **THEN** the error is UnknownCombatant
- **AND** 3 actions remain
- **AND** the scripted 10 is still unused

#### Scenario: Striking oneself rolls nothing

- **GIVEN** an open turn with 3 actions and a scripted natural 10
- **WHEN** the active combatant Strikes their own CombatantId
- **THEN** the error is SelfTarget
- **AND** 3 actions remain
- **AND** the scripted 10 is still unused

#### Scenario: A Strike on a dead target spends nothing

- **GIVEN** an open turn with 3 actions, a dead target, and a scripted natural 10
- **WHEN** the active combatant Strikes that target
- **THEN** the error is TargetDead
- **AND** 3 actions remain
- **AND** the scripted 10 is still unused

#### Scenario: A Strike with no actions rolls nothing

- **GIVEN** an open Alive turn with 0 actions and a scripted natural 10
- **WHEN** the active combatant Strikes another creature
- **THEN** the error is InsufficientActions
- **AND** the scripted 10 is still unused
- **AND** the turn stays open

#### Scenario: Armor Class in the caller list does not change the result

- **GIVEN** attack bonus +0, baked Armor Class 12, a scripted natural 10, and a caller circumstance penalty of -2 on Armor Class
- **WHEN** the Strike is resolved
- **THEN** the total is 10
- **AND** the degree is failure

#### Scenario: A reactive Strike off your turn spends no action

- **GIVEN** Alive CombatantId(1) with Perception +0 and a scripted initiative 8, Alive CombatantId(2) with Perception +0 and a scripted initiative 10, CombatantId(2)'s turn open, CombatantId(1) at (0, 0) with melee reach 5, 0 actions, attack count 0, and a ready reaction, CombatantId(2) at (0, 1) with 20 Hit Points and Armor Class 18, both Medium, an ActionCost of 1 action with Manipulate, ReactiveStrike::Available, attack bonus +8, an empty caller modifier list, damage 1d8+4, and a scripted natural 10 then a damage die of 3
- **WHEN** CombatantId(1) makes that Reactive Strike
- **THEN** the result is not NotYourTurn
- **AND** CombatantId(1) still has 0 actions
- **AND** the penalty is 0
- **AND** the stored attack count is still 0
- **AND** CombatantId(1)'s reaction is not ready
- **AND** the open turn is still CombatantId(2)

### Requirement: The multiple attack penalty follows this turn's count

On the striker's turn the penalty SHALL be 0, then -5, then -10, or 0, then -4, then -8 when that strike is agile. A resolved Strike SHALL count, including a miss. A rejected Strike SHALL NOT count. The stored count SHALL be a u8. A Reactive Strike while it is not that reactor's open turn SHALL spend no action, SHALL apply a multiple attack penalty of 0, and SHALL leave that stored count unchanged.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2289 (Player Core pg. 402, Multiple Attack Penalty)

#### Scenario: The second agile Strike takes -4

- **GIVEN** an open turn, an agile strike, attack bonus +8, Armor Class 18, 0 damage dice, and scripted naturals 2 then 14
- **WHEN** both Strikes are resolved
- **THEN** the second multiple attack penalty is -4
- **AND** the second total is 18
- **AND** the second degree is success

#### Scenario: The third non-agile Strike takes -10

- **GIVEN** an open turn, a non-agile strike, attack bonus +8, Armor Class 18, 0 damage dice, and scripted naturals 2, then 2, then 15
- **WHEN** all three Strikes are resolved
- **THEN** the third multiple attack penalty is -10
- **AND** the third total is 13
- **AND** the third degree is failure
- **AND** 0 actions remain
- **AND** the turn stays open

#### Scenario: A rejected Strike does not count

- **GIVEN** a Strike attempted before the turn starts, with a scripted natural 10, attack bonus +8, and Armor Class 18
- **WHEN** that attempt fails, the turn then starts, and the Strike is resolved with that same natural 10
- **THEN** the failed attempt rolls nothing
- **AND** the resolved Strike has multiple attack penalty 0 and total 18

#### Scenario: An attack count at the u8 maximum does not increment

- **GIVEN** the active creature's stored attack count is 255, 3 actions remain, and a scripted natural 10
- **WHEN** that creature Strikes
- **THEN** the error is AttackCountOverflow
- **AND** 3 actions remain
- **AND** the stored count is 255
- **AND** the scripted 10 is still unused

#### Scenario: A reactive Strike off your turn leaves the count

- **GIVEN** Alive CombatantId(1) has resolved two Strikes and ended that turn, so the stored attack count is 2 and 0 actions remain, the reaction is still ready, it is Alive CombatantId(2)'s open turn, CombatantId(1) is at (0, 0) with melee reach 5, CombatantId(2) is at (0, 1) with 20 Hit Points and Armor Class 18, both Medium, an ActionCost of 1 action with Manipulate, ReactiveStrike::Available, attack bonus +8, an empty caller modifier list, damage 1d8+4, and a scripted natural 10 then a damage die of 3
- **WHEN** CombatantId(1) makes that Reactive Strike
- **THEN** the penalty is 0
- **AND** the stored attack count is still 2
- **AND** CombatantId(1) still has 0 actions
- **AND** the open turn is still CombatantId(2)
