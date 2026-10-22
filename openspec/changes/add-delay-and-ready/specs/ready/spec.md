# ready Specification

## Purpose

Ready stores one single action and a trigger, then spends the reaction to resolve that action before the creature's next turn. A readied attack uses the attack count stored when Ready resolved.

## ADDED Requirements

### Requirement: The same ready twice matches

The same creature, count, trigger, and readied action SHALL resolve to the same stored ready and the same encounter. Ready SHALL roll nothing while it is stored.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2299 (Player Core pg. 417, Ready)

#### Scenario: Storing the same ready twice matches

- **GIVEN** Alive CombatantId(1) and Alive CombatantId(2), both Side::Pc, Medium, Perception +0, scripted initiative naturals 15 then 10, CombatantId(1) at (0, 0) with reach 5 and 3 actions and attack count 0
- **WHEN** CombatantId(1) Readies a non-agile Strike, attack bonus +8, on CombatantId(2) entering reach, with ActionCost of 1 action
- **THEN** the stored trigger is MovesWithinReach of CombatantId(2) and the stored action is that Strike
- **AND** the frozen attack count is 0, the turn has ended, actions are 0, and the reaction is ready
- **AND** no die was rolled
- **WHEN** the same ready is stored again from the same start
- **THEN** the stored ready and the encounter match the first result

### Requirement: Ready costs two actions and ends the turn

Ready is a two-action concentrate action on your open turn (https://2e.aonprd.com/Actions.aspx?ID=2299, Player Core pg. 417). It SHALL store the named trigger, one single action or free action, and the attack count at resolution, then end the turn. Actions still unused SHALL be lost. Ready SHALL NOT spend the reaction and SHALL NOT change the attack count.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2299 (Player Core pg. 417, Ready)

#### Scenario: One Strike then Ready stores count 1

- **GIVEN** Alive CombatantId(1) at (0, 0) and Alive CombatantId(2) at (0, 1), both Medium, CombatantId(2) at 20 Hit Points with Armor Class 18, Perception +0, scripted initiative naturals 15 then 10, CombatantId(1) with 3 actions, attack count 0, and attack bonus +8
- **WHEN** CombatantId(1) Strikes with a natural 10 and 0 damage dice, then Readies a non-agile Strike on CombatantId(2) entering reach
- **THEN** the frozen attack count is 1 and the creature's attack count is 1
- **AND** the turn has ended, actions are 0, and the reaction is ready
- **AND** Ready rolled no die

#### Scenario: Ready with no earlier attack stores count 0

- **GIVEN** Alive CombatantId(1) at (0, 0) and Alive CombatantId(2) at (0, 1), both Medium, Perception +0, scripted initiative naturals 15 then 10, CombatantId(1) with 3 actions and attack count 0
- **WHEN** CombatantId(1)'s first action is Ready of a non-agile Strike on CombatantId(2) entering reach, ActionCost of 1 action
- **THEN** the frozen attack count is 0
- **AND** the turn has ended and the reaction is still ready

#### Scenario: Ready is legal after the reaction was spent

- **GIVEN** Alive CombatantId(1) at (0, 0) and Alive CombatantId(2) at (0, 1), both Medium, Perception +0, scripted initiative naturals 15 then 10, CombatantId(1) with 3 actions, attack count 0, and the reaction already spent
- **WHEN** CombatantId(1) Readies a non-agile Strike on CombatantId(2) entering reach
- **THEN** the ready is stored with frozen attack count 0
- **AND** the reaction stays not ready

### Requirement: A readied attack uses the stored count

A readied attack SHALL call `multiple_attack_penalty` with `readied_attack_count: Some(snapshot)` and agile from the readied weapon (https://2e.aonprd.com/Actions.aspx?ID=2299, Player Core pg. 417; R-05; B-06). It SHALL NOT change the snapshot or the creature's attack count. That creature's next own-turn attack-trait penalty SHALL be 0.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2299 (Player Core pg. 417, Ready)

#### Scenario: A readied non-agile Strike after one Strike takes -5

- **GIVEN** Alive CombatantId(1) at (0, 0), Medium, reach 5, with a stored non-agile Strike, snapshot 1, attack bonus +8, and Alive CombatantId(2) at (0, 1), Medium, Armor Class 18, 20 Hit Points
- **WHEN** CombatantId(2) begins a Strike and resolve_reactions runs with a scripted natural 15
- **THEN** the penalty is -5
- **AND** the total is 18
- **AND** the degree is success
- **AND** the attack count is unchanged
- **WHEN** CombatantId(1)'s next turn starts and its attack-trait penalty is read
- **THEN** the penalty is 0

#### Scenario: The first readied non-agile Strike takes 0

- **GIVEN** Alive CombatantId(1) at (0, 0), Medium, reach 5, with a stored non-agile Strike, snapshot 0, attack bonus +8, and Alive CombatantId(2) at (0, 1), Medium, Armor Class 18, 20 Hit Points
- **WHEN** CombatantId(2) begins a Strike and resolve_reactions runs with a scripted natural 10
- **THEN** the penalty is 0
- **AND** the total is 18
- **AND** the degree is success

#### Scenario: A readied non-agile Strike at count 2 takes -10

- **GIVEN** Alive CombatantId(1) at (0, 0), Medium, reach 5, with a stored non-agile Strike, snapshot 2, attack bonus +8, and Alive CombatantId(2) at (0, 1), Medium, Armor Class 18, 20 Hit Points
- **WHEN** CombatantId(2) begins a Strike and resolve_reactions runs with a scripted natural 15
- **THEN** the penalty is -10
- **AND** the total is 13
- **AND** the degree is failure
- **AND** the attack count remains 2
- **WHEN** CombatantId(1)'s next turn starts and its attack-trait penalty is read
- **THEN** the penalty is 0

#### Scenario: A readied agile Strike at count 2 takes -8

- **GIVEN** Alive CombatantId(1) at (0, 0), Medium, reach 5, with a stored agile Strike, snapshot 2, attack bonus +8, and Alive CombatantId(2) at (0, 1), Medium, Armor Class 18, 20 Hit Points
- **WHEN** CombatantId(2) begins a Strike and resolve_reactions runs with a scripted natural 18
- **THEN** the penalty is -8
- **AND** the total is 18
- **AND** the degree is success
- **AND** the penalty is not -10

### Requirement: Ready expires at the next start

A stored ready SHALL last until the start of the readier's next turn, and that start SHALL clear it (https://2e.aonprd.com/Actions.aspx?ID=2299, Player Core pg. 417). A trigger after that start SHALL be rejected, and no die SHALL be rolled.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2299 (Player Core pg. 417, Ready)

#### Scenario: The next start clears the ready

- **GIVEN** Alive CombatantId(1) at (0, 0), Medium, has a stored non-agile Strike on Alive CombatantId(2) at (0, 1), Medium, entering reach, and CombatantId(2) then ends its turn, with a scripted natural 10
- **WHEN** CombatantId(1)'s next turn starts
- **THEN** no ready is stored
- **WHEN** CombatantId(2) enters (0, 1) from (0, 2) and resolve_reactions runs
- **THEN** the offer is rejected
- **AND** the natural 10 is unused
- **AND** the reaction granted by that start stays ready

### Requirement: Firing spends the reaction and not an action

Using the readied action SHALL spend the readier's reaction and SHALL NOT spend an action (https://2e.aonprd.com/Actions.aspx?ID=2299, Player Core pg. 417). The trigger SHALL be offered by the caller when it occurs. A miss SHALL still spend the reaction and clear the ready. With no reaction, the fire SHALL be rejected, the ready SHALL stay, and no die SHALL be rolled. Firing SHALL NOT end the turn that is open and SHALL NOT spend that creature's actions.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2299 (Player Core pg. 417, Ready)

#### Scenario: A hit spends the reaction only

- **GIVEN** Alive CombatantId(2) at (0, 1), Medium, has its turn open with 3 actions, and Alive CombatantId(1) at (0, 0), Medium, reach 5, has a stored non-agile Strike, snapshot 0, a ready reaction, attack bonus +8, and CombatantId(2) has Armor Class 18 and 20 Hit Points
- **WHEN** CombatantId(2) begins a Strike and resolve_reactions runs with a scripted natural 10 and damage 1
- **THEN** CombatantId(1) has no reaction ready
- **AND** CombatantId(1) has no stored ready
- **AND** CombatantId(1)'s actions stay 0
- **AND** CombatantId(2) still has 3 actions
- **AND** CombatantId(2)'s turn stays open

#### Scenario: A miss still spends the reaction

- **GIVEN** Alive CombatantId(1) at (0, 0), Medium, reach 5, with a stored non-agile Strike, snapshot 0, attack bonus +8, a ready reaction, and Alive CombatantId(2) at (0, 1), Medium, Armor Class 18, 20 Hit Points
- **WHEN** CombatantId(2) begins a Strike and resolve_reactions runs with a scripted natural 1
- **THEN** the total is 9
- **AND** the degree is critical failure
- **AND** damage is 0
- **AND** the reaction is spent
- **AND** the ready is cleared

#### Scenario: No reaction leaves the ready stored

- **GIVEN** Alive CombatantId(1) at (0, 0), Medium, reach 5, with a stored non-agile Strike on Alive CombatantId(2) at (0, 1), Medium, and CombatantId(1) has no reaction ready, and a scripted natural 10
- **WHEN** CombatantId(2) begins a Strike and resolve_reactions runs
- **THEN** the error is NoReaction
- **AND** the ready stays stored
- **AND** the natural 10 is unused

### Requirement: A readied Strike against a helpless or dead target

A Dying or Unconscious target SHALL be struck against its baked AC, and damage SHALL use the blow rules (https://2e.aonprd.com/Rules.aspx?ID=2324, Player Core pg. 410; R-08; DEV-36). Damage greater than 0 to an Unconscious creature at 0 HP SHALL be a knockout. A Dead target SHALL be rejected before the roll: the reaction SHALL stay ready, the stored action SHALL stay, and no die SHALL be rolled.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2324 (Player Core pg. 410, Getting Knocked Out)
**Source**: https://2e.aonprd.com/Actions.aspx?ID=2299 (Player Core pg. 417, Ready)

#### Scenario: Dying keeps baked AC on a readied miss

- **GIVEN** Alive CombatantId(1) at (0, 0), Medium, reach 5, with a stored non-agile Strike, snapshot 0, attack bonus +8, and Dying(1) CombatantId(2) at (0, 1), Medium, 0 Hit Points, Armor Class 18
- **WHEN** CombatantId(2) begins a Strike and resolve_reactions runs with a scripted natural 4 and no damage dice
- **THEN** the penalty is 0
- **AND** the total is 12
- **AND** the degree is failure
- **AND** the target stays Dying 1
- **AND** the reaction is spent

#### Scenario: Damage to Dying 1 makes it Dying 2

- **GIVEN** Alive CombatantId(1) at (0, 0), Medium, reach 5, with a stored non-agile Strike, snapshot 0, attack bonus +8, and Dying(1) CombatantId(2) at (0, 1), Medium, 0 Hit Points, Armor Class 18
- **WHEN** CombatantId(2) begins a Strike and resolve_reactions runs with a scripted natural 10 and damage 1
- **THEN** the total is 18
- **AND** the degree is success
- **AND** the target is Dying 2

#### Scenario: Damage to Unconscious at 0 HP is a knockout

- **GIVEN** Alive CombatantId(1) at (0, 0), Medium, reach 5, with a stored non-agile Strike, snapshot 0, attack bonus +8, and Unconscious CombatantId(2) at (0, 1), Medium, 0 Hit Points, wounded 0, uses_dying_rules, Armor Class 18
- **WHEN** CombatantId(2) begins a Strike and resolve_reactions runs with a scripted natural 10 and damage 1
- **THEN** the target is Dying 1
- **AND** its HP stays 0

#### Scenario: A Dead target spends nothing

- **GIVEN** Alive CombatantId(1) at (0, 0), Medium, reach 5, reaction ready, with a stored non-agile Strike on Dead CombatantId(2) at (0, 1), Medium, and a scripted natural 10
- **WHEN** CombatantId(2) begins a Strike and resolve_reactions runs
- **THEN** the error is TargetDead
- **AND** the reaction stays ready
- **AND** the ready stays stored
- **AND** the natural 10 is unused

### Requirement: The readied action is one of four

The readied action SHALL be Strike, Stride, Step, or Dismiss (https://2e.aonprd.com/Actions.aspx?ID=2299, Player Core pg. 417). A readied Strike SHALL target the trigger's subject. Any other action, including a two-action or three-action activity or a free action that already has a trigger, SHALL be rejected before actions are spent.

**Rule**: Any one single action or free action you can use.
**Why**: Phase 1 can resolve four of those actions and no others.
**Substitute**: Strike, Stride, Step, and Dismiss only. The Strike's target is the trigger subject.
**Intent**: A later change adds an action by extending the enum. The cost and the trigger stay.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2299 (Player Core pg. 417, Ready)

#### Scenario: Dismiss drops to no cover

- **GIVEN** Alive CombatantId(1) at (0, 0), Medium, with standard cover and 3 actions, and Alive CombatantId(2) at (0, 2), Medium, and a scripted natural 10
- **WHEN** CombatantId(1) Readies Dismiss on CombatantId(2) entering reach, CombatantId(2) steps from (0, 2) to (0, 1), and resolve_reactions runs
- **THEN** CombatantId(1)'s cover is none
- **AND** the reaction is spent
- **AND** the ready is cleared
- **AND** the natural 10 is unused

#### Scenario: A two-action activity is rejected

- **GIVEN** Alive CombatantId(1) at (0, 0), Medium, with 3 actions, and Alive CombatantId(2) at (0, 1), Medium, and a scripted natural 10
- **WHEN** CombatantId(1) Readies a Strike with an ActionCost of 2 actions, triggered by CombatantId(2) entering reach
- **THEN** the error is NotASingleAction
- **AND** 3 actions remain
- **AND** nothing is stored
- **AND** the natural 10 is unused

#### Scenario: Delay cannot be readied

- **GIVEN** Alive CombatantId(1) at (0, 0), Medium, with 3 actions at the start of its turn, and Alive CombatantId(2) at (0, 1), Medium, and a scripted natural 10
- **WHEN** CombatantId(1) Readies Delay, triggered by CombatantId(2) entering reach
- **THEN** the error is TriggeredFreeAction
- **AND** 3 actions remain
- **AND** nothing is stored
- **AND** the natural 10 is unused

#### Scenario: A readied Strike cannot name a third creature

- **GIVEN** Alive CombatantId(1) at (0, 0), Alive CombatantId(2) at (0, 1), and Alive CombatantId(3) at (1, 0), all Medium, Perception +0, scripted initiative naturals 15 then 10 then 5, CombatantId(1)'s turn open with 3 actions, and a scripted natural 10
- **WHEN** CombatantId(1) Readies a Strike whose target is CombatantId(3) on CombatantId(2) entering reach
- **THEN** the error is NotTheTriggerSubject
- **AND** nothing is stored
- **AND** the natural 10 is unused

### Requirement: The trigger is one of two observed kinds

The trigger SHALL be a named creature entering one square within the readier's reach from outside it, or that creature beginning Strike, Stride, Step, Drop Prone, or Stand (https://2e.aonprd.com/Actions.aspx?ID=2299, Player Core pg. 417). The caller SHALL supply both squares. A move between two squares already in reach SHALL NOT fire. Any other trigger SHALL be rejected before the reaction is spent. A Step into reach from outside SHALL fire.

**Rule**: A trigger you choose.
**Why**: Phase 1 can observe a square entered and five begun actions.
**Substitute**: Those two trigger kinds only. One entered square, not the whole path.
**Intent**: A later change adds a trigger by extending the enum.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2299 (Player Core pg. 417, Ready)

#### Scenario: Entering a square within 5 feet fires

- **GIVEN** Alive CombatantId(1) at (0, 0), Medium, reach 5, with a stored non-agile Strike, snapshot 0, attack bonus +8, and Alive CombatantId(2), Medium, Armor Class 18, moving from (0, 2) into (0, 1), and a scripted natural 10
- **WHEN** CombatantId(2) enters (0, 1)
- **THEN** the penalty is 0, the total is 18, and the degree is success
- **AND** the reaction is spent

#### Scenario: Entering a square 15 feet away does not fire

- **GIVEN** Alive CombatantId(1) at (0, 0), Medium, reach 5, with a stored Strike on CombatantId(2) entering reach, reaction ready, and a scripted natural 10
- **WHEN** CombatantId(2) enters (0, 3) from (0, 4)
- **THEN** the fire is rejected
- **AND** the reaction stays ready
- **AND** the ready stays stored
- **AND** no die is rolled

#### Scenario: A Step into reach fires

- **GIVEN** Alive CombatantId(1) at (0, 0), Medium, reach 5, with a stored Dismiss on CombatantId(2) entering reach from outside, and Alive CombatantId(2), Medium, Speed 25, stepping from (0, 2) to (0, 1), and a scripted natural 10
- **WHEN** that Step is offered and resolve_reactions runs
- **THEN** the Dismiss resolves, the reaction is spent, and the ready is cleared
- **AND** the natural 10 is unused

#### Scenario: Beginning a Strike fires

- **GIVEN** Alive CombatantId(1) at (0, 0), Medium, reach 5, with a stored non-agile Strike, snapshot 0, attack bonus +8, on CombatantId(2) beginning a Strike, and Alive CombatantId(2) at (0, 1), Medium, Armor Class 18, 20 Hit Points
- **WHEN** CombatantId(2) begins a Strike and the d20 reads 10
- **THEN** the penalty is 0, the total is 18, and the degree is success
- **AND** the reaction is spent

#### Scenario: Beginning a Seek does not fire

- **GIVEN** Alive CombatantId(1) at (0, 0), Medium, with a stored trigger that CombatantId(2) begins a Strike, reaction ready, Alive CombatantId(2) at (0, 1), Medium, and a scripted natural 10
- **WHEN** CombatantId(2) begins a Seek
- **THEN** the error is TriggerMismatch
- **AND** the reaction stays ready and the natural 10 is unused

#### Scenario: Moving inside reach does not fire

- **GIVEN** Alive CombatantId(1) at (0, 0), Medium, reach 5, with a stored Strike on CombatantId(2) entering reach, and Alive CombatantId(2), Medium, moving from (0, 1) to (1, 0), and a scripted natural 10
- **WHEN** that move is offered and resolve_reactions runs
- **THEN** the error is TriggerMismatch
- **AND** the reaction stays ready and the ready stays stored
- **AND** the natural 10 is unused

### Requirement: A readied Stride uses a fresh diagonal counter

A readied Stride or Step SHALL take its path from the caller and SHALL build a fresh diagonal counter (https://2e.aonprd.com/Actions.aspx?ID=2299, Player Core pg. 417; DEV-15). The caller's own-turn counter SHALL stay unchanged. An illegal path SHALL be rejected before the reaction is spent, and the stored action SHALL stay.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2299 (Player Core pg. 417, Ready)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2357 (Player Core pg. 421, Diagonal Movement)

#### Scenario: An orthogonal Stride spends the reaction

- **GIVEN** Alive CombatantId(1) at (0, 0), Medium, Speed 25, with a stored Stride, a caller counter that has already seen 1 diagonal, Alive CombatantId(2) at (0, 1), Medium, beginning a Strike, and a scripted natural 10
- **WHEN** CombatantId(2) begins a Strike and resolve_reactions runs with path (1, 0)
- **THEN** CombatantId(1) ends at (1, 0)
- **AND** the distance is 5 feet
- **AND** the reaction is spent
- **AND** the caller's counter still has 1 diagonal
- **AND** the natural 10 is unused

#### Scenario: The first diagonal of a readied Stride costs 5

- **GIVEN** Alive CombatantId(1) at (0, 0), Medium, Speed 25, with a stored Stride, Alive CombatantId(2) at (0, 1), Medium, beginning a Strike, and a scripted natural 10
- **WHEN** CombatantId(2) begins a Strike and resolve_reactions runs with path (1, 1)
- **THEN** the distance is 5 feet
- **AND** CombatantId(1) ends at (1, 1)
- **AND** the natural 10 is unused

#### Scenario: A path beyond Speed spends nothing

- **GIVEN** Alive CombatantId(1) at (0, 0), Medium, Speed 5, with a stored Stride, reaction ready, Alive CombatantId(2) at (0, 1), Medium, beginning a Strike, and a scripted natural 10
- **WHEN** CombatantId(2) begins a Strike and resolve_reactions runs with path (1, 0) then (2, 0)
- **THEN** the error is Grid
- **AND** the reaction stays ready
- **AND** the ready stays stored
- **AND** CombatantId(1) stays at (0, 0)
- **AND** the natural 10 is unused

#### Scenario: A readied Step moves one square

- **GIVEN** Alive CombatantId(1) at (0, 0), Medium, Speed 25, with a stored Step, Alive CombatantId(2) at (0, 1), Medium, beginning a Strike, and a scripted natural 10
- **WHEN** CombatantId(2) begins a Strike and resolve_reactions runs with destination (1, 0)
- **THEN** CombatantId(1) ends at (1, 0)
- **AND** the distance is 5 feet
- **AND** the reaction is spent
- **AND** the natural 10 is unused

### Requirement: Another creature's delay leaves the snapshot

Delay SHALL NOT change another creature's stored count or stored action (https://2e.aonprd.com/Actions.aspx?ID=2294, Player Core pg. 416).

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2299 (Player Core pg. 417, Ready)
**Source**: https://2e.aonprd.com/Actions.aspx?ID=2294 (Player Core pg. 416, Delay)

#### Scenario: A delay does not rewrite a stored count

- **GIVEN** Side::Pc CombatantId(1) at (0, 0) and Side::Pc CombatantId(2) at (0, 1), both Medium, Perception +0, scripted initiative naturals 15 then 10, CombatantId(2) has a stored non-agile Strike with frozen attack count 1, CombatantId(1) is Delayed, CombatantId(2)'s turn has ended, and a scripted natural 10 is held
- **WHEN** CombatantId(1) returns
- **THEN** CombatantId(2)'s stored action is still that Strike and the frozen attack count is 1
- **AND** the natural 10 is unused

### Requirement: Ready rejects before it spends

Ready SHALL reject, in order, an unknown actor, an unknown subject, a turn that is not open, a creature whose turn it is not, a creature that is not alive, an action outside the four, a free action that already has a trigger, a trigger whose subject is the readier, a Strike aimed at someone other than the trigger subject, fewer than 2 actions, then a round past the maximum. Nothing is stored and no die is rolled.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2299 (Player Core pg. 417, Ready)

#### Scenario: An unknown creature is rejected first

- **GIVEN** an encounter of Alive CombatantId(1) at (0, 0), Medium, and no open turn, and a scripted natural 10
- **WHEN** CombatantId(9) Readies a Strike
- **THEN** the error is UnknownCombatant
- **AND** nothing is stored
- **AND** the natural 10 is unused

#### Scenario: An unknown subject is rejected before a closed turn

- **GIVEN** only Alive CombatantId(1) at (0, 0), Medium, and no open turn, and a scripted natural 10
- **WHEN** CombatantId(1) Readies a Strike triggered by CombatantId(9) entering reach
- **THEN** the error is UnknownCombatant
- **AND** nothing is stored
- **AND** the natural 10 is unused

#### Scenario: A trigger about the readier is rejected

- **GIVEN** Alive CombatantId(1) at (0, 0), Medium, turn open with 3 actions and the reaction ready, and a scripted natural 10
- **WHEN** CombatantId(1) Readies a Strike triggered by CombatantId(1) entering reach
- **THEN** the error is TriggerSubjectIsSelf
- **AND** 3 actions remain
- **AND** nothing is stored
- **AND** the natural 10 is unused

#### Scenario: One action left is rejected before a round past the maximum

- **GIVEN** round 4294967295, Alive CombatantId(2) at (0, 1), Medium, turn open with 1 action remaining, Alive CombatantId(1) at (0, 0), Medium, and a scripted natural 10
- **WHEN** CombatantId(2) Readies a Strike on CombatantId(1) entering reach
- **THEN** the error is InsufficientActions
- **AND** the round stays 4294967295
- **AND** 1 action remains
- **AND** nothing is stored
- **AND** the natural 10 is unused

#### Scenario: Ready at the maximum round changes nothing

- **GIVEN** round 4294967295 and Alive CombatantId(2) at (0, 1), Medium, turn open with 3 actions, in order Alive CombatantId(1) at (0, 0) then CombatantId(2), both Medium, and a scripted natural 10
- **WHEN** CombatantId(2) Readies a Strike on CombatantId(1) entering reach
- **THEN** the error is RoundOverflow
- **AND** the round stays 4294967295
- **AND** 3 actions remain
- **AND** nothing is stored
- **AND** the natural 10 is unused

### Requirement: Fire rejects before it rolls

Fire SHALL reject, in order, an unknown readier, an unknown subject, no stored ready, the wrong action, a readier who is not Alive, a delayed readier, a trigger mismatch, a begun Strike out of reach, a Dead target, then no reaction. An illegal path, PositioningError from Dismiss, and an attack total past the bounds SHALL fail before the reaction is spent or the d20 is drawn. PositioningError SHALL be returned. No earlier check writes, and no die is rolled.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2299 (Player Core pg. 417, Ready)

#### Scenario: No stored ready is rejected before a dead target

- **GIVEN** Alive CombatantId(1) at (0, 0), Medium, has no stored ready, and Dead CombatantId(2) at (0, 1), Medium, and a scripted natural 10
- **WHEN** CombatantId(2) begins a Strike and resolve_reactions runs
- **THEN** the result is empty and the error is not TargetDead
- **AND** the natural 10 is unused

#### Scenario: The wrong action is rejected before a delayed readier

- **GIVEN** Delayed CombatantId(1) at (0, 0), Medium, whose stored ready is a non-agile Strike on CombatantId(2) at (0, 1), Medium, reaction ready, and a scripted natural 10
- **WHEN** that stored Strike is checked as a Stride
- **THEN** the error is WrongAction
- **AND** the ready stays stored
- **AND** the natural 10 is unused

#### Scenario: A delayed readier is rejected before the trigger is matched

- **GIVEN** Delayed CombatantId(1) at (0, 0), Medium, reach 5, with a stored non-agile Strike on CombatantId(2) beginning a Strike, Alive CombatantId(2) at (0, 1), Medium, reaction ready, and a scripted natural 10
- **WHEN** CombatantId(2) begins a Strike and resolve_reactions runs
- **THEN** the error is Delayed
- **AND** the ready stays stored
- **AND** the natural 10 is unused

#### Scenario: A dying readier is rejected before a dead target

- **GIVEN** Dying(1) CombatantId(1) at (0, 0), Medium, 0 Hit Points, with a stored non-agile Strike, reaction ready, and Dead CombatantId(2) at (0, 1), Medium, and a scripted natural 10
- **WHEN** CombatantId(2) begins a Strike and resolve_reactions runs
- **THEN** the error is NotConscious
- **AND** the reaction flag is unchanged
- **AND** the ready stays stored
- **AND** the natural 10 is unused

#### Scenario: A dead target is rejected before a missing reaction

- **GIVEN** Alive CombatantId(1) at (0, 0), Medium, reach 5, with a stored non-agile Strike and no reaction, and Dead CombatantId(2) at (0, 1), Medium, and a scripted natural 10
- **WHEN** CombatantId(2) begins a Strike and resolve_reactions runs
- **THEN** the error is TargetDead
- **AND** the ready stays stored
- **AND** the natural 10 is unused

### Requirement: A begun Strike is checked for reach

A readied Strike on a begins-an-action trigger SHALL check range at fire time. Melee SHALL use in_reach, and failure SHALL be OutOfReach. Ranged SHALL use range_band: a count above 6 SHALL fail before the reaction is spent, and a count from 1 to 6 SHALL fire and add that untyped penalty. This SHALL NOT add a deviation. On failure the reaction and the ready SHALL stay, and no die SHALL be rolled.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2299 (Player Core pg. 417, Ready)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2379 (Player Core pg. 426, Range and Reach)

#### Scenario: A begun strike out of reach spends nothing

- **GIVEN** Alive CombatantId(1) at (0, 0), Medium, reach 5, with a stored non-agile Strike on CombatantId(2) beginning a Strike, reaction ready, and a scripted natural 10
- **WHEN** Alive CombatantId(2), Medium, at (0, 3), begins a Strike
- **THEN** the error is OutOfReach
- **AND** the reaction stays ready, the ready stays stored, and the natural 10 is unused

#### Scenario: A readied ranged Strike in the second increment still fires

- **GIVEN** Alive CombatantId(1) at (0, 0), Medium, with a stored ranged Strike, range increment 10 feet, snapshot 0, attack bonus +8, reaction ready, and Alive CombatantId(2) at (0, 3), Medium, Armor Class 18, 20 Hit Points, and a scripted natural 12
- **WHEN** CombatantId(2) begins a Strike and resolve_reactions runs
- **THEN** the range penalty is -2, the total is 18, and the degree is success
- **AND** the reaction is spent and the ready is cleared

#### Scenario: A readied ranged Strike past six increments spends nothing

- **GIVEN** Alive CombatantId(1) at (0, 0), Medium, with a stored ranged Strike, range increment 5 feet, snapshot 0, attack bonus +8, reaction ready, and Alive CombatantId(2) at (0, 7), Medium, and a scripted natural 10
- **WHEN** CombatantId(2) begins a Strike and resolve_reactions runs
- **THEN** the error is Grid(BeyondRange)
- **AND** the reaction stays ready, the ready stays stored, and the natural 10 is unused

### Requirement: Hidden targets stay with the caller

A readied Strike SHALL NOT roll a flat check and SHALL NOT read hidden or undetected. The caller supplies the target. The Strike SHALL use baked Armor Class.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2299 (Player Core pg. 417, Ready)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2285 (Player Core pg. 401, Hidden and Undetected)

#### Scenario: A readied strike draws no flat check

- **GIVEN** Alive CombatantId(1) at (0, 0), Medium, reach 5, with a stored non-agile Strike, snapshot 0, attack bonus +8, and Alive CombatantId(2) at (0, 1), Medium, Armor Class 18, who is hidden from CombatantId(1), and a script of 10 then 5
- **WHEN** the Strike fires
- **THEN** the total is 18 and the degree is success
- **AND** only the natural 10 is drawn
- **AND** the face 5 is unused

### Requirement: Matching reactions resolve in initiative order

resolve_reactions SHALL be the one public call for a trigger. It SHALL resolve every matching readied action and every Reactive Strike, in the current initiative order, each fully before the next (DEV-55). DEV-55 covers Reactive Strike and readied reactions together. A tie SHALL follow DEV-05: enemy before PC, then lower CombatantId. A per-readier fire function SHALL NOT be public. A later reaction whose target is already Dead SHALL be TargetDead and SHALL keep its reaction and its ready.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2299 (Player Core pg. 417, Ready)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2423 (Player Core pg. 435, Step 1: Roll Initiative)

#### Scenario: The first readied reaction kills and the second keeps ready

- **GIVEN** the order is Side::Enemy CombatantId(1) at (0, 0) then Side::Pc CombatantId(2) at (1, 0) then Side::Pc CombatantId(3) at (0, 1), all Medium, reach 5, CombatantId(3) has 9 Hit Points and uses_dying_rules false and Armor Class 18, and both CombatantId(1) and CombatantId(2) have a stored non-agile Strike, snapshot 0, attack bonus +8, on CombatantId(3) beginning a Strike
- **WHEN** CombatantId(3) begins a Strike and resolve_reactions is called once, CombatantId(1) with natural 10 and damage 1d8+1 showing 8, then CombatantId(2) with natural 12
- **THEN** CombatantId(3) is Dead at 0 Hit Points
- **AND** CombatantId(1)'s reaction is spent and that ready is cleared
- **AND** CombatantId(2)'s error is TargetDead, that reaction stays ready, that ready stays stored, and the natural 12 is unused

#### Scenario: An enemy reaction resolves before a tied pc

- **GIVEN** Side::Enemy CombatantId(4) at (0, 0) and Side::Pc CombatantId(1) at (1, 0), both Medium, reach 5, both have initiative total 12, the order is CombatantId(4) then CombatantId(1), both have a stored non-agile Strike, snapshot 0, attack bonus +8, on Alive CombatantId(3) at (0, 1), Medium, 30 Hit Points and Armor Class 18 beginning a Strike
- **WHEN** CombatantId(3) begins a Strike and resolve_reactions is called once, each with natural 10 and damage 1
- **THEN** CombatantId(3) is at 28 Hit Points
- **AND** CombatantId(4)'s reaction is spent before CombatantId(1)'s reaction is spent

#### Scenario: A reactive strike and a ready resolve in one call

- **GIVEN** Side::Enemy CombatantId(1) at (0, 0), Medium, reach 5, attack bonus +8, reaction ready, ReactiveStrike::Available, Side::Pc CombatantId(2) at (1, 0), Medium, reach 5, with a stored non-agile Strike, snapshot 0, attack bonus +8, on CombatantId(3) beginning a Stride, and Alive CombatantId(3) at (0, 1), Medium, 20 Hit Points, Armor Class 18, 3 actions, Perception +0, scripted initiative naturals 15 then 10 then 5
- **WHEN** CombatantId(3) begins a Stride and resolve_reactions is called once, CombatantId(1)'s Reactive Strike with natural 10 and damage 1, then CombatantId(2)'s readied Strike with natural 10 and damage 1
- **THEN** CombatantId(3) is at 18 Hit Points
- **AND** CombatantId(1)'s reaction is spent before CombatantId(2)'s reaction is spent
- **AND** CombatantId(3) still has 3 actions

### Requirement: A knockout abandons the triggering action

If a readied reaction leaves the creature who began Strike, Stride, Step, Stand, or Drop Prone Dying or Dead, that begun action SHALL return NotConscious. It SHALL spend no action and SHALL draw no die. A begun Stride or Step SHALL NOT move them. The readied reaction's result SHALL stay.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2299 (Player Core pg. 417, Ready)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2324 (Player Core pg. 410, Getting Knocked Out)

#### Scenario: A knockout abandons the triggering strike

- **GIVEN** Alive CombatantId(2) at (0, 1), Medium, with 3 actions, 9 Hit Points, wounded 0, uses_dying_rules, and Armor Class 18, and Alive CombatantId(1) at (0, 0), Medium, reach 5, with a stored non-agile Strike, snapshot 0, attack bonus +8, on CombatantId(2) beginning a Strike
- **WHEN** CombatantId(2) begins a Strike, resolve_reactions rolls natural 10 and damage 1d8+1 showing 8, and CombatantId(2) then Strikes
- **THEN** CombatantId(2) is Dying(1) at 0 Hit Points
- **AND** CombatantId(1)'s reaction is spent and the ready is cleared
- **AND** CombatantId(2)'s Strike is NotConscious, CombatantId(2) still has 3 actions, and CombatantId(2)'s natural is unused

#### Scenario: A knockout abandons the triggering stride

- **GIVEN** Alive CombatantId(2) at (0, 1), Medium, Speed 25, with 3 actions, 9 Hit Points, wounded 0, uses_dying_rules, and Armor Class 18, and Alive CombatantId(1) at (0, 0), Medium, reach 5, with a stored non-agile Strike, snapshot 0, attack bonus +8, on CombatantId(2) beginning a Stride, and a scripted natural 4 held for CombatantId(2)
- **WHEN** CombatantId(2) begins a Stride from (0, 1) toward (1, 1), and resolve_reactions rolls natural 10 and damage 1d8+1 showing 8
- **THEN** CombatantId(2) is Dying(1) at 0 Hit Points and stays at (0, 1)
- **AND** CombatantId(1)'s reaction is spent and the ready is cleared
- **AND** CombatantId(2)'s Stride is NotConscious, CombatantId(2) still has 3 actions, and the natural 4 is unused

### Requirement: A dead readier cannot fire

A Dead readier SHALL be NotConscious. The ready and the reaction SHALL stay, and no die SHALL be rolled. This change SHALL NOT clear a ready from a death hook.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2299 (Player Core pg. 417, Ready)

#### Scenario: A dead readier spends nothing

- **GIVEN** Dead CombatantId(1) at (0, 0), Medium, with a stored non-agile Strike, snapshot 0, reaction ready, and Alive CombatantId(2) at (0, 1), Medium, and a scripted natural 10
- **WHEN** CombatantId(2) begins a Strike and resolve_reactions runs
- **THEN** the error is NotConscious
- **AND** the reaction stays ready, the ready stays stored, and the natural 10 is unused

### Requirement: A returned creature may Ready

A creature whose open turn was produced by return SHALL be able to Ready. The cursor SHALL name that creature while the turn is open. Ready SHALL snapshot the attack count the start left, which return did not write. Ready SHALL end that turn. The ready SHALL stay stored until that creature's next start, which SHALL clear it.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2299 (Player Core pg. 417, Ready)
**Source**: https://2e.aonprd.com/Actions.aspx?ID=2294 (Player Core pg. 416, Delay)

#### Scenario: A returned creature may ready

- **GIVEN** Side::Pc CombatantId(1) at (0, 0) and Side::Pc CombatantId(2) at (0, 2), both Medium, Perception +0, scripted initiative naturals 15 then 10, CombatantId(1) Delayed with attack count 0, CombatantId(2)'s turn ended, and a scripted natural 10
- **WHEN** CombatantId(1) returns
- **THEN** the open turn is CombatantId(1) and the cursor names CombatantId(1)
- **WHEN** CombatantId(1) Readies a non-agile Strike, attack bonus +8, on CombatantId(2) entering reach
- **THEN** the frozen attack count is 0
- **AND** the turn has ended, no turn is open, actions are 0, and the reaction is ready
- **AND** the round stays 1 and next_start is CombatantId(2)
- **AND** the ready stays stored and the natural 10 is unused
- **WHEN** CombatantId(2)'s turn starts and ends, and CombatantId(1)'s next turn starts
- **THEN** the round is 2, CombatantId(1)'s turn is open, and no ready is stored
