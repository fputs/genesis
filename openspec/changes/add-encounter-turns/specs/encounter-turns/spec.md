# encounter-turns

## Purpose

Runs one encounter from Perception initiative through rounds, turns, and Strike, including the three-action economy and caller modifiers on the attack.

## ADDED Requirements

### Requirement: The same encounter twice matches

The same participants and the same scripted dice SHALL produce the same order and the same Strike result. Resolution SHALL NOT read a file, the network, a clock, or a global generator.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2423 (Player Core pg. 435, Step 1: Roll Initiative)

#### Scenario: The same encounter twice matches

- **GIVEN** two encounters, each with PC CombatantId(1) at Perception +5 and PC CombatantId(2) at Perception +0, attack bonus +0, Armor Class 11, damage 1d8+4, and the script 10, 14, 12, 3
- **WHEN** each rolls initiative and CombatantId(1) Strikes CombatantId(2)
- **THEN** both orders are CombatantId(1) then CombatantId(2)
- **AND** both Strike totals are 12 and both damage results are 7
- **AND** each script is fully consumed
- **AND** the arithmetic is 10 + 5 = 15, 14 + 0 = 14, then 12 against Armor Class 11, then 3 + 4 = 7

### Requirement: Initiative is one Perception total

Each combatant SHALL add one d20 to that creature's Perception modifier. The ranking SHALL use that total. A natural 20 or a natural 1 SHALL NOT change the total. Dice SHALL be drawn in ascending CombatantId order.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2423 (Player Core pg. 435, Step 1: Roll Initiative)

#### Scenario: Perception is added to the d20

- **GIVEN** one combatant with Perception modifier +5 and a scripted natural 10
- **WHEN** initiative is rolled
- **THEN** that combatant's total is 15
- **AND** one d20 was rolled

#### Scenario: A natural 20 does not add 10

- **GIVEN** CombatantId(1) with Perception +0 and CombatantId(2) with Perception +5, and scripted naturals 20 then 14
- **WHEN** initiative is rolled
- **THEN** CombatantId(1) has total 20 and CombatantId(2) has total 19
- **AND** CombatantId(1) acts earlier

#### Scenario: A natural 1 does not subtract 10

- **GIVEN** one combatant with Perception modifier +19 and a scripted natural 1
- **WHEN** initiative is rolled
- **THEN** the total is 20

#### Scenario: A negative Perception modifier is added

- **GIVEN** one combatant with Perception modifier -3 and a scripted natural 8
- **WHEN** initiative is rolled
- **THEN** the total is 5

#### Scenario: Roster order does not choose the die

- **GIVEN** CombatantId(5) listed before CombatantId(1), Perception +0 and +5, and scripted naturals 10 then 14
- **WHEN** initiative is rolled
- **THEN** CombatantId(1) has total 15 and CombatantId(5) has total 14

### Requirement: A bad roster rolls nothing

A duplicate CombatantId SHALL return DuplicateCombatant. An empty roster SHALL return EmptyRoster. No die SHALL be rolled.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2423 (Player Core pg. 435, Step 1: Roll Initiative)

#### Scenario: A duplicate id rolls nothing

- **GIVEN** two participants with CombatantId(1) and a scripted natural 8
- **WHEN** initiative is rolled
- **THEN** the error is DuplicateCombatant
- **AND** the scripted 8 is still unused

#### Scenario: An empty roster rolls nothing

- **GIVEN** no participants and a scripted natural 8
- **WHEN** initiative is rolled
- **THEN** the error is EmptyRoster
- **AND** the scripted 8 is still unused

### Requirement: Perception is checked against a d20 of 20

Before any die is drawn, Perception SHALL be rejected when it cannot be added to 20. Perception 2147483627 and a natural 20 SHALL total 2147483647. Perception 2147483628 SHALL return PerceptionOutOfRange and SHALL draw no die. i32::MIN SHALL be accepted, and the total SHALL be the face plus i32::MIN.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2423 (Player Core pg. 435, Step 1: Roll Initiative)

#### Scenario: Perception 2147483627 and a natural 20 fit

- **GIVEN** Perception modifier 2147483627 and a scripted natural 20
- **WHEN** initiative is rolled
- **THEN** the total is 2147483647
- **AND** the scripted 20 was drawn

#### Scenario: Perception 2147483628 rolls nothing

- **GIVEN** Perception modifier 2147483628 and a scripted natural 20
- **WHEN** initiative is rolled
- **THEN** the error is PerceptionOutOfRange
- **AND** the scripted 20 is still unused

#### Scenario: Perception at i32::MIN is added

- **GIVEN** Perception modifier -2147483648 and a scripted natural 1
- **WHEN** initiative is rolled
- **THEN** the total is -2147483647
- **AND** the scripted 1 was drawn

### Requirement: The higher total acts earlier

The higher initiative total SHALL act earlier than a lower total. The participant's team SHALL be Side, Pc or Enemy.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2423 (Player Core pg. 435, Step 1: Roll Initiative)

#### Scenario: A higher PC total acts before a lower enemy total

- **GIVEN** PC CombatantId(1) with Perception +8 and enemy CombatantId(2) with Perception +5, and scripted naturals 10 then 10
- **WHEN** initiative is rolled
- **THEN** the totals are 18 and 15
- **AND** the order is CombatantId(1) then CombatantId(2)

### Requirement: A PC and an enemy on the same total give the enemy the earlier turn

On an equal total, an enemy SHALL act before a PC. Same-Side ties SHALL use the lower CombatantId, which is DEV-05 amended to that order. Side SHALL be the participant's Side.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2423 (Player Core pg. 435, Step 1: Roll Initiative)

#### Scenario: An enemy acts before a tied PC

- **GIVEN** PC CombatantId(1) and enemy CombatantId(9), both with Perception +0, and scripted naturals 10 then 10
- **WHEN** initiative is rolled
- **THEN** both totals are 10
- **AND** the order is CombatantId(9) then CombatantId(1)

#### Scenario: Two tied PCs act by the lower id

- **GIVEN** two PCs, CombatantId(4) and CombatantId(2), both with Perception +0, and scripted naturals 8 then 8
- **WHEN** initiative is rolled
- **THEN** both totals are 8
- **AND** the order is CombatantId(2) then CombatantId(4)

#### Scenario: Two tied enemies act by the lower id

- **GIVEN** two enemies, CombatantId(8) and CombatantId(3), both with Perception +0, and scripted naturals 7 then 7
- **WHEN** initiative is rolled
- **THEN** both totals are 7
- **AND** the order is CombatantId(3) then CombatantId(8)

#### Scenario: A three-way tie puts both enemies first

- **GIVEN** PC CombatantId(1), enemy CombatantId(2), and enemy CombatantId(5), each with Perception +0, and scripted naturals 12, 12, 12
- **WHEN** initiative is rolled
- **THEN** every total is 12
- **AND** the order is CombatantId(2), then CombatantId(5), then CombatantId(1)

#### Scenario: Six combatants all appear in the order

- **GIVEN** PC 0, enemy 1, PC 2, enemy 3, PC 4, and enemy 5, each with Perception +0, and scripted naturals 10, 10, 12, 12, 8, 8 in CombatantId order
- **WHEN** initiative is rolled
- **THEN** the order is CombatantId(3), CombatantId(2), CombatantId(1), CombatantId(0), CombatantId(5), CombatantId(4)

### Requirement: The start step changes nothing for an Alive creature

The Alive start step SHALL run before actions are regained. It SHALL NOT roll a die and SHALL NOT change Hit Points, temporary Hit Points, or wounded.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2428 (Player Core pg. 435, Step 1: Start Your Turn)

#### Scenario: An alive start rolls nothing

- **GIVEN** an Alive creature at 12 Hit Points and a scripted natural 8
- **WHEN** that creature's turn starts
- **THEN** the scripted 8 is still unused
- **AND** Hit Points stay 12
- **AND** wounded stays 0
- **AND** the creature has 3 actions

### Requirement: A turn regains 3 actions and 1 reaction

After the start step, an Alive creature SHALL have exactly 3 actions and one reaction ready. A second start while that turn is open SHALL return TurnInProgress and SHALL NOT grant more actions.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2428 (Player Core pg. 435, Step 1: Start Your Turn)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2429 (Player Core pg. 436, Step 2: Act)

#### Scenario: Start sets 3 actions and a reaction

- **GIVEN** Alive CombatantId(1) with Perception +0 and a scripted natural 10, and no turn has started
- **WHEN** the first turn starts
- **THEN** the initiative total is 10 and the round is 1
- **AND** CombatantId(1) has 3 actions
- **AND** that combatant's reaction is ready

#### Scenario: A second start does not add actions

- **GIVEN** Alive CombatantId(1) whose turn has started with 3 actions and a ready reaction
- **WHEN** start is requested again
- **THEN** the error is TurnInProgress
- **AND** CombatantId(1) still has 3 actions
- **AND** the reaction is still ready

#### Scenario: The next start sets 3 rather than adding leftovers

- **GIVEN** a combatant who ended a turn with 2 actions still unspent
- **WHEN** that combatant's next turn starts
- **THEN** the combatant has 3 actions

#### Scenario: A spent reaction is ready again next turn

- **GIVEN** a combatant who spent the reaction on the previous turn
- **WHEN** that combatant's next turn starts and the reaction is spent twice
- **THEN** the reaction is ready after the start
- **AND** the first spend succeeds
- **AND** the second spend is NoReaction

#### Scenario: An unused reaction is still one reaction

- **GIVEN** a combatant who did not spend the reaction last turn
- **WHEN** that combatant's next turn starts and the reaction is spent twice
- **THEN** the first spend succeeds
- **AND** the second spend is NoReaction

### Requirement: Reactions are ready before the first turn

After initiative and before any turn starts, each Alive combatant's reaction SHALL be ready and that combatant SHALL have 0 actions. A non-Alive combatant's reaction SHALL NOT be ready.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2432 (Player Core pg. 436, Reactions in Encounters)

#### Scenario: A reaction before the first turn spends once

- **GIVEN** initiative has been rolled, no turn has started, and a scripted natural 8
- **WHEN** one Alive combatant spends a reaction, and spends it again
- **THEN** the first spend succeeds and rolls nothing
- **AND** the second spend is NoReaction
- **AND** that combatant still has 0 actions
- **AND** the scripted 8 is still unused

### Requirement: A creature that is not Alive cannot act

A combatant that is not Alive SHALL have 0 actions and no reaction. Dead combatants SHALL be skipped. A combatant whose vital is neither Alive nor Dead SHALL open a turn and close it empty. An action, free action, reaction, or Strike for that combatant SHALL return NotConscious and SHALL NOT change the reaction. If the active creature stops being Alive during the open turn, later spends SHALL return NotConscious, and end_turn SHALL close the turn once and run the end step once.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2429 (Player Core pg. 436, Step 2: Act)

#### Scenario: A not-alive turn opens with no actions

- **GIVEN** CombatantId(1) and CombatantId(2), both Alive, both with Perception +0, scripted initiative naturals 10 then 8, CombatantId(2) at 9 Hit Points with uses_dying_rules true and Armor Class 11
- **WHEN** CombatantId(1)'s turn starts and CombatantId(1) Strikes CombatantId(2) with attack bonus +0, natural 11, and damage 1d8+1 showing 8
- **THEN** CombatantId(2) has 0 Hit Points
- **AND** CombatantId(2) is not Alive and is not Dead
- **WHEN** CombatantId(1)'s turn ends and the next turn starts
- **THEN** the open turn is CombatantId(2)
- **AND** CombatantId(2) has 0 actions
- **AND** the reaction is not ready
- **WHEN** CombatantId(2) spends a reaction, spends 1 action, spends a free action, and Strikes with a scripted natural 4
- **THEN** each error is NotConscious
- **AND** the reaction is still not ready
- **AND** actions stay 0
- **AND** the scripted 4 is still unused
- **WHEN** the turn ends
- **THEN** no turn is open
- **AND** CombatantId(2) still has 0 actions

#### Scenario: An active creature that stops being Alive cannot keep acting

- **GIVEN** CombatantId(1)'s turn opened while Alive, with 2 actions and a ready reaction, CombatantId(2) next, a scripted natural 8, and CombatantId(1)'s vital then neither Alive nor Dead
- **WHEN** CombatantId(1) spends 1 action
- **THEN** the error is NotConscious
- **AND** 2 actions remain
- **AND** the reaction is still ready
- **AND** the scripted 8 is still unused
- **WHEN** the turn ends, and a turn end is requested again
- **THEN** the first end succeeds
- **AND** the second end is TurnNotOpen
- **AND** the round is still 1
- **AND** the next start is CombatantId(2)

#### Scenario: A dead combatant cannot spend a reaction

- **GIVEN** a dead combatant and an Alive combatant whose reaction is ready
- **WHEN** the dead combatant spends a reaction
- **THEN** the error is NotConscious
- **AND** the Alive combatant's reaction is still ready
- **AND** the dead combatant's reaction is unchanged

### Requirement: Ending a turn drops actions and keeps the reaction

Ending a turn SHALL set that combatant's actions to 0 and SHALL keep an unspent reaction ready. The end step SHALL run once. The end step SHALL NOT roll a die and SHALL NOT change Hit Points, temporary Hit Points, or wounded. end_turn with no open turn SHALL return TurnNotOpen.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2430 (Player Core pg. 436, Step 3: End Your Turn)

#### Scenario: Ending early drops leftover actions and keeps the reaction

- **GIVEN** an open turn with 2 actions left and the reaction ready, current Hit Points 12, and a scripted natural 8
- **WHEN** the turn ends
- **THEN** actions are 0
- **AND** the reaction is still ready
- **AND** Hit Points stay 12
- **AND** temporary Hit Points stay 0
- **AND** wounded stays 0
- **AND** the scripted 8 is still unused

#### Scenario: End before a start returns an error

- **GIVEN** initiative has been rolled and no turn has started
- **WHEN** a turn end is requested
- **THEN** the error is TurnNotOpen
- **AND** every combatant still has 0 actions

### Requirement: Spending the last action leaves the turn open

Rules 2429 ends the turn after the third action. This handoff is engineering: the caller SHALL call end_turn. Nothing further can be spent, so the handoff has no rules effect. No end-of-turn effect SHALL fire before end_turn. end_turn SHALL advance once. A second end_turn SHALL return TurnNotOpen.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2429 (Player Core pg. 436, Step 2: Act)

#### Scenario: Spending the last action leaves the turn open

- **GIVEN** Alive CombatantId(1) then Alive CombatantId(2), with CombatantId(1)'s turn open, 3 actions, and the reaction ready
- **WHEN** 3 actions are spent
- **THEN** the open turn is still CombatantId(1)
- **AND** CombatantId(1) has 0 actions
- **AND** the reaction is still ready
- **WHEN** the turn ends, and a turn end is requested again
- **THEN** the second end is TurnNotOpen
- **AND** the round is still 1
- **AND** the next start is CombatantId(2)

### Requirement: An action spend is all or nothing

A spend of 1, 2, or 3 actions SHALL subtract that many at once during an open Alive turn. A spend the combatant cannot finish SHALL return InsufficientActions and leave the count unchanged. A cost of 0 or greater than 3 SHALL return InvalidActionCost.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2336 (Player Core pg. 414, Activities)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2429 (Player Core pg. 436, Step 2: Act)

#### Scenario: Spending 1 leaves 2

- **GIVEN** an open turn with 3 actions
- **WHEN** 1 action is spent
- **THEN** 2 actions remain
- **AND** the turn stays open

#### Scenario: Spending 2 leaves 1 and rolls nothing

- **GIVEN** an open turn with 3 actions and a scripted natural 8
- **WHEN** 2 actions are spent
- **THEN** 1 action remains
- **AND** the scripted 8 is still unused

#### Scenario: Spending 2 with 1 left fails

- **GIVEN** an open turn with 1 action left
- **WHEN** 2 actions are spent
- **THEN** the error is InsufficientActions
- **AND** 1 action remains

#### Scenario: A cost of 0 or 4 is rejected

- **GIVEN** an open turn with 3 actions
- **WHEN** a cost of 0 is spent, and a cost of 4 is spent
- **THEN** both errors are InvalidActionCost
- **AND** 3 actions remain

### Requirement: An untriggered free action does not spend an action

During an open Alive turn, an untriggered free action SHALL leave the action count and the reaction unchanged and SHALL NOT end the turn. With no open turn, that free action SHALL return TurnNotOpen.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2336 (Player Core pg. 414, Activities)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2429 (Player Core pg. 436, Step 2: Act)

#### Scenario: A free action leaves the remaining action

- **GIVEN** an open turn with 1 action left and the reaction ready
- **WHEN** a free action is spent
- **THEN** 1 action remains
- **AND** the reaction is still ready
- **AND** the turn stays open

#### Scenario: An untriggered free action off turn fails

- **GIVEN** initiative has been rolled and no turn is open
- **WHEN** an untriggered free action is spent
- **THEN** the error is TurnNotOpen

### Requirement: Spending a reaction does not take an action

An Alive creature SHALL be able to spend a ready reaction during another creature's turn. The spend SHALL clear that reaction and SHALL NOT change the active creature's actions, roll a die, or end the turn. A second spend SHALL return NoReaction.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2432 (Player Core pg. 436, Reactions in Encounters)

#### Scenario: A reaction during the turn leaves the actions

- **GIVEN** an open turn with 3 actions, the reaction ready, and a scripted natural 8
- **WHEN** the reaction is spent
- **THEN** 3 actions remain
- **AND** the turn stays open
- **AND** the reaction is not ready
- **AND** the scripted 8 is still unused

#### Scenario: Another creature spends a reaction during the turn

- **GIVEN** CombatantId(1)'s turn is open with 3 actions, CombatantId(2) is Alive with a ready reaction, and a scripted natural 8
- **WHEN** CombatantId(2) spends a reaction
- **THEN** the spend succeeds
- **AND** CombatantId(1) still has 3 actions
- **AND** the open turn is still CombatantId(1)
- **AND** the scripted 8 is still unused
- **WHEN** CombatantId(2) spends a reaction again
- **THEN** the error is NoReaction

### Requirement: The round is a u32

The round SHALL be a u32 and SHALL start at 1. Moving the cursor past the last index SHALL wrap it and SHALL add 1 to the round. An add that does not fit SHALL return RoundOverflow and SHALL leave the round, the order, and the open turn unchanged.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2429 (Player Core pg. 436, Step 2: Act)

#### Scenario: The second round starts with the first living combatant

- **GIVEN** Alive CombatantId(1) then Alive CombatantId(2), both with Perception +0, and scripted naturals 10 then 8
- **WHEN** each of them starts and ends one turn
- **THEN** the round is 2 before the next start
- **AND** the next start is CombatantId(1)

#### Scenario: After 2 ends, a dead 3 wraps to 1 in round 2

- **GIVEN** CombatantId(1), CombatantId(2), and CombatantId(3), each with Perception +0, scripted naturals 12, 11, and 10, and CombatantId(3) dead
- **WHEN** CombatantId(1) and CombatantId(2) each start and end
- **THEN** the round is still 1 after CombatantId(2) ends
- **AND** the next start is CombatantId(1) in round 2

#### Scenario: A dead first combatant starts the other in round 1

- **GIVEN** CombatantId(1) and CombatantId(2), both with Perception +0, scripted naturals 10 then 5, and CombatantId(1) dead at the start
- **WHEN** the first turn starts
- **THEN** the started combatant is CombatantId(2)
- **AND** the round is 1

#### Scenario: A round at the u32 maximum does not advance

- **GIVEN** the round is 4294967295, the order is CombatantId(1) then CombatantId(2), and CombatantId(2)'s turn is open
- **WHEN** that turn ends
- **THEN** the error is RoundOverflow
- **AND** the round is 4294967295
- **AND** the order is unchanged
- **AND** the open turn is still CombatantId(2)

### Requirement: A dead combatant is skipped

A dead combatant SHALL remain in the order and SHALL NOT start a turn. The next combatant who is not dead SHALL start instead. Only Vital::Dead is skipped.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2428 (Player Core pg. 435, Step 1: Start Your Turn)

#### Scenario: A dead combatant in the middle is skipped

- **GIVEN** the order CombatantId(1), CombatantId(2), CombatantId(3), with CombatantId(2) already dead
- **WHEN** the first turn starts and ends, and the next turn starts
- **THEN** the first start is CombatantId(1)
- **AND** the next start is CombatantId(3)
- **AND** the round is still 1
- **AND** CombatantId(2) has 0 actions

#### Scenario: The only living combatant starts the next round

- **GIVEN** CombatantId(1) alive and CombatantId(2) dead, with CombatantId(1) earlier in the order
- **WHEN** CombatantId(1) starts and ends a turn, and a start is requested
- **THEN** the round is 2
- **AND** the started combatant is CombatantId(1)

#### Scenario: No living combatant returns an error

- **GIVEN** every combatant is already dead and the round is 1
- **WHEN** a start is requested
- **THEN** the error is NoLivingCombatant
- **AND** the round stays 1
- **AND** the order is unchanged

### Requirement: Strike spends one action against baked Armor Class

On an open Alive turn, a Strike SHALL spend 1 action, roll one d20, add the strike's attack bonus and the multiple attack penalty, and compare that total with the target's baked Armor Class. No square and no reach SHALL be required. The degree SHALL use that baked Armor Class. A target whose vital is neither Alive nor Dead keeps that baked Armor Class until wire-dying-into-turns replaces this result (DEV-36).

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

### Requirement: Strike passes caller modifiers through the attack total

A Strike SHALL pass the caller's attack-statistic modifiers through the attack check total. A modifier on another statistic SHALL NOT change that total. Damage modifiers SHALL NOT change it either. A caller untyped penalty SHALL stack with the multiple attack penalty.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2282 (Player Core pg. 400, Penalties)

#### Scenario: The second Strike includes MAP and a caller circumstance penalty

- **GIVEN** an open turn, a non-agile strike, attack bonus +9, Armor Class 16, 0 damage dice, scripted naturals 2 then 12, and a caller circumstance penalty of -2 on the second Strike
- **WHEN** both Strikes are resolved
- **THEN** the second multiple attack penalty is -5
- **AND** the second total is 14
- **AND** the second degree is failure

#### Scenario: Two caller circumstance bonuses keep the higher on a Strike

- **GIVEN** attack bonus +0, Armor Class 10, 0 damage dice, caller circumstance bonuses of +4 and +1 from different sources, and a scripted natural 10
- **WHEN** the Strike is resolved
- **THEN** the total is 14
- **AND** the total is not 15

#### Scenario: A circumstance bonus and penalty on the attack both apply

- **GIVEN** attack bonus +0, baked Armor Class 10, 0 damage dice, a caller circumstance bonus of +2, a caller circumstance penalty of -2, and a scripted natural 10
- **WHEN** the Strike is resolved
- **THEN** the total is 10
- **AND** the degree is success
- **AND** the total is not 12
- **AND** the total is not 8

#### Scenario: A damage penalty does not change the attack total

- **GIVEN** attack bonus +8, Armor Class 18, 0 damage dice, a scripted natural 12, a caller attack circumstance penalty of -2, and a caller damage penalty of -3
- **WHEN** the Strike is resolved
- **THEN** the total is 18
- **AND** the degree is success

#### Scenario: A caller untyped penalty stacks with MAP

- **GIVEN** a second non-agile Strike, attack bonus +0, Armor Class 11, 0 damage dice, a scripted natural 12, and a caller untyped penalty of -5
- **WHEN** the Strike is resolved
- **THEN** the multiple attack penalty is -5
- **AND** the total is 2
- **AND** the total is not 7
- **AND** the degree is failure

### Requirement: The multiple attack penalty follows this turn's count

On the striker's turn the penalty SHALL be 0, then -5, then -10, or 0, then -4, then -8 when that strike is agile. A resolved Strike SHALL count, including a miss. A rejected Strike SHALL NOT count. The stored count SHALL be a u8.

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

### Requirement: A reaction attack penalty stays 0

An attack-trait check while it is not that creature's open turn SHALL take penalty 0 and SHALL NOT change that creature's count. The turn SHALL be taken from the encounter. This change SHALL NOT keep a penalty frozen from Ready.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2289 (Player Core pg. 402, Multiple Attack Penalty)
**Source**: https://2e.aonprd.com/Actions.aspx?ID=2299 (Player Core pg. 417, Ready)

#### Scenario: A reaction after two Strikes keeps penalty 0

- **GIVEN** a non-agile creature that has resolved two Strikes this turn, and whose turn has since ended
- **WHEN** an attack-trait penalty is read for that creature while the turn is not open
- **THEN** the penalty is 0
- **AND** the read rolls no die
- **WHEN** that creature's next turn starts and the penalty is read
- **THEN** the penalty is 0

### Requirement: The degree after the natural shift selects damage

A success SHALL deal one damage roll. A critical success SHALL double that total. A failure or a critical failure SHALL deal no damage and SHALL NOT roll a damage die. The degree after a natural 20 or natural 1 SHALL be the degree that selects damage. strike SHALL pass critical only when that degree is a critical success.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2306 (Player Core pg. 418, Strike)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2287 (Player Core pg. 401, Natural 1 and Natural 20)

#### Scenario: A success deals the damage roll once

- **GIVEN** attack bonus +8, Armor Class 18, a scripted natural 10, and damage 1d8+4 with a scripted damage die of 3
- **WHEN** the Strike is resolved
- **THEN** the damage is 7
- **AND** one damage die was rolled

#### Scenario: A critical success doubles one damage roll

- **GIVEN** attack bonus +10, Armor Class 15, a scripted natural 15, damage 1d8+4, a scripted damage die of 3, and another scripted damage die of 6
- **WHEN** the Strike is resolved
- **THEN** the damage is 14
- **AND** the scripted 6 is still unused

#### Scenario: A failure rolls no damage die

- **GIVEN** attack bonus +0, Armor Class 18, a scripted natural 9, and a scripted damage die of 8
- **WHEN** the Strike is resolved
- **THEN** the degree is failure
- **AND** the damage is 0
- **AND** the scripted 8 is still unused

#### Scenario: A critical failure rolls no damage die

- **GIVEN** attack bonus +0, Armor Class 18, a scripted natural 5, and a scripted damage die of 8
- **WHEN** the Strike is resolved
- **THEN** the degree is critical failure
- **AND** the damage is 0
- **AND** the scripted 8 is still unused

#### Scenario: A natural 20 that becomes a success deals damage once

- **GIVEN** attack bonus +0, Armor Class 25, a scripted natural 20, and damage 1d8+4 with a scripted damage die of 3
- **WHEN** the Strike is resolved
- **THEN** the degree is success
- **AND** the damage is 7

#### Scenario: A natural 20 that stays a failure deals no damage

- **GIVEN** attack bonus +0, Armor Class 30, a scripted natural 20, and a scripted damage die of 8
- **WHEN** the Strike is resolved
- **THEN** the total is 20
- **AND** the degree is failure
- **AND** the damage is 0
- **AND** the scripted 8 is still unused

#### Scenario: A natural 1 that becomes a failure deals no damage

- **GIVEN** attack bonus +17, Armor Class 15, a scripted natural 1, and a scripted damage die of 8
- **WHEN** the Strike is resolved
- **THEN** the degree is failure
- **AND** the damage is 0
- **AND** the scripted 8 is still unused

#### Scenario: A natural 1 that becomes a success deals damage once

- **GIVEN** attack bonus +24, Armor Class 15, a scripted natural 1, and damage 1d8 with a scripted damage die of 4
- **WHEN** the Strike is resolved
- **THEN** the degree is success
- **AND** the damage is 4

### Requirement: A failure or critical failure short-circuits with no damage

A Strike at Failure or CriticalFailure SHALL return Ok with damage 0, the action spent, and no damage die drawn. Hit Points, temporary Hit Points, wounded, and vital stay unchanged. `strike` SHALL NOT return `HpError::NotAHit`. `HpError::NotAHit` is defined only so `add-dying-and-victory` can return it from `apply_resolved_blow`. This change SHALL NOT add a public damage function beside `apply_dice_damage`.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2306 (Player Core pg. 418, Strike)

#### Scenario: A failure short-circuits with no damage

- **GIVEN** an open turn, 3 actions, attack bonus +0, Armor Class 18, the target at 20 current Hit Points, 5 temporary Hit Points, wounded 0, and vital Alive, damage 1d8+4, a scripted natural 9, and a scripted damage die of 8
- **WHEN** `strike` resolves the Strike
- **THEN** the degree is failure
- **AND** the damage is 0
- **AND** 2 actions remain
- **AND** the scripted 8 is still unused
- **AND** current Hit Points stay 20
- **AND** temporary Hit Points stay 5
- **AND** wounded stays 0
- **AND** vital stays Alive

#### Scenario: A critical failure short-circuits with no damage

- **GIVEN** an open turn, 3 actions, attack bonus +0, Armor Class 18, the target at 20 current Hit Points, 5 temporary Hit Points, wounded 0, and vital Alive, damage 1d8+4, a scripted natural 5, and a scripted damage die of 8
- **WHEN** `strike` resolves the Strike
- **THEN** the degree is critical failure
- **AND** the damage is 0
- **AND** 2 actions remain
- **AND** the scripted 8 is still unused
- **AND** current Hit Points stay 20
- **AND** temporary Hit Points stay 5
- **AND** wounded stays 0
- **AND** vital stays Alive

### Requirement: Strike damage follows R-07 before Hit Points

Dice and modifiers SHALL be added, then penalties. The floor of 1 SHALL apply only when penalties reduce a pre-penalty total of at least 1 to 0 or below, and it SHALL be applied before doubling, so that crit deals 2. A total clamped at 0 SHALL stay 0 on a crit. Ruling R-07 and DEV-14 fix this order.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2302 (Player Core pg. 406, Damage Rolls)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2307 (Player Core pg. 407, Doubling and Halving)

#### Scenario: Penalties reduce a hit to 1 damage

- **GIVEN** attack bonus +0, Armor Class 11, a scripted natural 11, a damage die showing 4, damage bonus +0, and an untyped damage penalty of -6
- **WHEN** the Strike is resolved
- **THEN** the damage is 1

#### Scenario: The same penalties on a crit deal 2

- **GIVEN** attack bonus +10, Armor Class 11, a scripted natural 11, a damage die showing 4, damage bonus +0, and an untyped damage penalty of -6
- **WHEN** the Strike is resolved
- **THEN** the damage is 2

#### Scenario: A pre-penalty 2 with a -2 penalty deals 1, or 2 on a crit

- **GIVEN** a damage die showing 2, damage bonus +0, and an untyped damage penalty of -2, once with attack bonus +0 against Armor Class 11 and natural 11, and once with attack bonus +10 against Armor Class 11 and natural 11
- **WHEN** both Strikes are resolved
- **THEN** the non-critical damage is 1
- **AND** the critical damage is 2

#### Scenario: A clamped total stays 0 on a hit and on a crit

- **GIVEN** damage 1d4-5, a damage die showing 1, and no damage penalty, once with attack bonus +0 against Armor Class 11 and natural 11, and once with attack bonus +10 against Armor Class 11 and natural 11
- **WHEN** both Strikes are resolved
- **THEN** both results deal 0

#### Scenario: A negative modifier plus a penalty stays 0 on a crit

- **GIVEN** attack bonus +10, Armor Class 11, a scripted natural 11, damage 1d4-5, a damage die showing 1, and an untyped damage penalty of -1
- **WHEN** the Strike is resolved
- **THEN** the damage is 0

#### Scenario: Two damage dice are doubled once

- **GIVEN** attack bonus +10, Armor Class 11, a scripted natural 11, and damage 2d6+1 with scripted damage dice 3 and 4
- **WHEN** the Strike is resolved
- **THEN** the damage is 16
- **AND** two damage dice were rolled

#### Scenario: Zero damage dice deal the bonus

- **GIVEN** attack bonus +0, Armor Class 11, a scripted natural 11, 0 damage dice, damage bonus +4, and a scripted damage die of 8
- **WHEN** the Strike is resolved
- **THEN** the damage is 4
- **AND** the scripted 8 is still unused

#### Scenario: Two circumstance bonuses to damage keep the higher

- **GIVEN** attack bonus +0, Armor Class 11, a scripted natural 11, a damage die showing 1, damage bonus +0, and damage circumstance bonuses of +4 and +1
- **WHEN** the Strike is resolved
- **THEN** the damage is 5

### Requirement: Strike damage removes temporary Hit Points first

Damage from a Strike SHALL remove temporary Hit Points first. Only the remainder SHALL reduce current Hit Points. Damage of 0 SHALL leave both unchanged. One helper SHALL apply that amount. apply_dice_damage SHALL stay unchanged.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2321 (Player Core pg. 410, Temporary Hit Points)

#### Scenario: Damage removes temporary Hit Points, then Hit Points

- **GIVEN** 20 current Hit Points, 5 temporary Hit Points, attack bonus +0, Armor Class 11, a scripted natural 11, and damage 1d8+4 with a scripted damage die of 3
- **WHEN** the Strike is resolved
- **THEN** temporary Hit Points are 0
- **AND** current Hit Points are 18

#### Scenario: Damage within temporary Hit Points leaves Hit Points

- **GIVEN** 20 current Hit Points, 5 temporary Hit Points, attack bonus +0, Armor Class 11, a scripted natural 11, and damage 1d4 with a scripted damage die of 3
- **WHEN** the Strike is resolved
- **THEN** temporary Hit Points are 2
- **AND** current Hit Points stay 20

#### Scenario: Zero damage leaves temporary Hit Points

- **GIVEN** 5 temporary Hit Points, attack bonus +0, Armor Class 11, a scripted natural 11, and damage 1d4-5 with a scripted damage die of 1
- **WHEN** the Strike is resolved
- **THEN** temporary Hit Points stay 5
- **AND** current Hit Points are unchanged

### Requirement: Reaching 0 Hit Points floors at 0

Current Hit Points from a Strike SHALL NOT go below 0. With uses_dying_rules false, 0 Hit Points SHALL mark the combatant dead. Wounded and shield Hit Points SHALL stay unchanged.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2324 (Player Core pg. 410, Getting Knocked Out)

#### Scenario: Without dying rules, 0 Hit Points is death

- **GIVEN** a creature with 9 Hit Points, uses_dying_rules false, attack bonus +0, Armor Class 11, a scripted natural 11, and damage 1d8+1 with a scripted damage die of 8
- **WHEN** the Strike is resolved
- **THEN** current Hit Points are 0
- **AND** the combatant is dead
- **AND** wounded stays 0
- **AND** the Creature still has 9 Hit Points

#### Scenario: A Strike does not change the shield

- **GIVEN** a target with a shield of 20 Hit Points that is not raised, attack bonus +0, Armor Class 11, a scripted natural 11, and damage 1d4 with a scripted damage die of 4
- **WHEN** that Strike is resolved
- **THEN** the shield still has 20 Hit Points
- **AND** the shield is not raised

### Requirement: An overflowing Strike changes nothing

An attack total that overflows SHALL return AttackOverflow of that modifier error. A damage sum that overflows SHALL return DamageOverflow of that modifier error. Dice already drawn SHALL stay drawn. Actions, Hit Points, and the attack count SHALL stay as they were. The sum of the damage dice fits in i32 because the dice count and the faces are each at most 255. Adding the damage bonus SHALL be checked.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2302 (Player Core pg. 406, Damage Rolls)

#### Scenario: An overflowing attack total leaves the turn

- **GIVEN** an open turn with 3 actions, attack bonus +0, Armor Class 11, 0 damage dice, a scripted natural 11, a following scripted 8, and two caller untyped penalties of -2147483648 from different sources
- **WHEN** the Strike is attempted
- **THEN** the error is AttackOverflow of the modifier overflow
- **AND** the scripted 11 was drawn
- **AND** the scripted 8 is still unused
- **AND** 3 actions remain
- **AND** the next Strike on that turn has multiple attack penalty 0

#### Scenario: An overflowing damage total leaves the turn

- **GIVEN** an open turn with 3 actions, 9 current Hit Points, 0 temporary Hit Points, attack bonus +0, Armor Class 11, a scripted natural 11, damage bonus 2147483647, and a damage die showing 1
- **WHEN** the Strike hits
- **THEN** the error is DamageOverflow of the modifier overflow
- **AND** both the natural 11 and the damage die were drawn
- **AND** 3 actions remain
- **AND** current Hit Points stay 9
- **AND** the next Strike on that turn has multiple attack penalty 0

### Requirement: The encounter module alone mutates its combatants

In the encounter module, no public free function SHALL take a mutable participant or a mutable combatant state. `apply_dice_damage` in the combatant module stays. A state copied out of the encounter SHALL NOT be the encounter's state.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2428 (Player Core pg. 435, Step 1: Start Your Turn)

#### Scenario: A copied state does not change the encounter

- **GIVEN** an encounter whose CombatantId(1) has 0 actions before any turn
- **WHEN** a copied state for CombatantId(1) is given 9 actions
- **THEN** the encounter still shows 0 actions for CombatantId(1)
