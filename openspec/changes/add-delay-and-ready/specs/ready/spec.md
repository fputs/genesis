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

- **GIVEN** Alive CombatantId(1) and Alive CombatantId(2) at 20 Hit Points with Armor Class 18, Perception +0, scripted initiative naturals 15 then 10, CombatantId(1) with 3 actions, attack count 0, and attack bonus +8
- **WHEN** CombatantId(1) Strikes with a natural 10 and 0 damage dice, then Readies a non-agile Strike on CombatantId(2) entering reach
- **THEN** the frozen attack count is 1 and the creature's attack count is 1
- **AND** the turn has ended, actions are 0, and the reaction is ready
- **AND** Ready rolled no die

#### Scenario: Ready with no earlier attack stores count 0

- **GIVEN** Alive CombatantId(1) and Alive CombatantId(2), Perception +0, scripted initiative naturals 15 then 10, CombatantId(1) with 3 actions and attack count 0
- **WHEN** CombatantId(1)'s first action is Ready of a non-agile Strike on CombatantId(2) entering reach, ActionCost of 1 action
- **THEN** the frozen attack count is 0
- **AND** the turn has ended and the reaction is still ready

#### Scenario: Ready is legal after the reaction was spent

- **GIVEN** Alive CombatantId(1) and Alive CombatantId(2), Perception +0, scripted initiative naturals 15 then 10, CombatantId(1) with 3 actions, attack count 0, and the reaction already spent
- **WHEN** CombatantId(1) Readies a non-agile Strike on CombatantId(2) entering reach
- **THEN** the ready is stored with frozen attack count 0
- **AND** the reaction stays not ready

### Requirement: A readied attack uses the stored count

A readied attack SHALL call `multiple_attack_penalty` with `readied_attack_count: Some(snapshot)` and agile from the readied weapon (https://2e.aonprd.com/Actions.aspx?ID=2299, Player Core pg. 417; R-05; B-06). It SHALL NOT change the snapshot or the creature's attack count. That creature's next own-turn attack-trait penalty SHALL be 0.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2299 (Player Core pg. 417, Ready)

#### Scenario: A readied non-agile Strike after one Strike takes -5

- **GIVEN** a stored non-agile Strike with snapshot 1, attack bonus +8, and a conscious target with AC 18
- **WHEN** the trigger fires and the d20 reads 15
- **THEN** the penalty is -5
- **AND** the total is 18
- **AND** the degree is success
- **AND** the attack count is unchanged
- **WHEN** that creature's next turn starts and its attack-trait penalty is read
- **THEN** the penalty is 0

#### Scenario: The first readied non-agile Strike takes 0

- **GIVEN** a stored non-agile Strike with snapshot 0, attack bonus +8, and AC 18
- **WHEN** the d20 reads 10
- **THEN** the penalty is 0
- **AND** the total is 18
- **AND** the degree is success

#### Scenario: A readied non-agile Strike at count 2 takes -10

- **GIVEN** a stored non-agile Strike with snapshot 2, attack bonus +8, and AC 18
- **WHEN** the d20 reads 15
- **THEN** the penalty is -10
- **AND** the total is 13
- **AND** the degree is failure
- **AND** the attack count remains 2
- **WHEN** that creature's next turn starts and its attack-trait penalty is read
- **THEN** the penalty is 0

#### Scenario: A readied agile Strike at count 2 takes -8

- **GIVEN** a stored agile Strike with snapshot 2, attack bonus +8, and AC 18
- **WHEN** the d20 reads 18
- **THEN** the penalty is -8
- **AND** the total is 18
- **AND** the degree is success
- **AND** the penalty is not -10

### Requirement: Ready expires at the next start

A stored ready SHALL last until the start of the readier's next turn, and that start SHALL clear it (https://2e.aonprd.com/Actions.aspx?ID=2299, Player Core pg. 417). A trigger after that start SHALL be rejected, and no die SHALL be rolled.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2299 (Player Core pg. 417, Ready)

#### Scenario: The next start clears the ready

- **GIVEN** creature 1 has a stored Strike and creature 2 then ends its turn
- **WHEN** creature 1's next turn starts
- **THEN** no ready is stored
- **WHEN** the old trigger is offered
- **THEN** the offer is rejected
- **AND** no die is rolled
- **AND** the reaction granted by that start stays ready

### Requirement: Firing spends the reaction and not an action

Using the readied action SHALL spend the readier's reaction and SHALL NOT spend an action (https://2e.aonprd.com/Actions.aspx?ID=2299, Player Core pg. 417). The trigger SHALL be offered by the caller when it occurs. A miss SHALL still spend the reaction and clear the ready. With no reaction, the fire SHALL be rejected, the ready SHALL stay, and no die SHALL be rolled. Firing SHALL NOT end the turn that is open and SHALL NOT spend that creature's actions.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2299 (Player Core pg. 417, Ready)

#### Scenario: A hit spends the reaction only

- **GIVEN** creature 2's turn is open with 3 actions, and creature 1 has a stored Strike, snapshot 0, a ready reaction, attack bonus +8, and a target with AC 18
- **WHEN** the trigger fires and the d20 reads 10 and damage is 1
- **THEN** creature 1 has no reaction ready
- **AND** creature 1 has no stored ready
- **AND** creature 1's actions stay 0
- **AND** creature 2 still has 3 actions
- **AND** creature 2's turn stays open

#### Scenario: A miss still spends the reaction

- **GIVEN** a stored Strike with snapshot 0, attack bonus +8, AC 18, and a ready reaction
- **WHEN** the d20 reads 1
- **THEN** the total is 9
- **AND** the degree is critical failure
- **AND** damage is 0
- **AND** the reaction is spent
- **AND** the ready is cleared

#### Scenario: No reaction leaves the ready stored

- **GIVEN** a stored Strike whose creature has no reaction ready
- **WHEN** the trigger fires
- **THEN** the fire is rejected
- **AND** the ready stays stored
- **AND** no die is rolled

### Requirement: A readied Strike against a helpless or dead target

A Dying or Unconscious target SHALL be struck against its baked AC, and damage SHALL use the blow rules (https://2e.aonprd.com/Rules.aspx?ID=2324, Player Core pg. 410; R-08; DEV-36). Damage greater than 0 to an Unconscious creature at 0 HP SHALL be a knockout. A Dead target SHALL be rejected before the roll: the reaction SHALL stay ready, the stored action SHALL stay, and no die SHALL be rolled.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2324 (Player Core pg. 410, Getting Knocked Out)
**Source**: https://2e.aonprd.com/Actions.aspx?ID=2299 (Player Core pg. 417, Ready)

#### Scenario: Dying keeps baked AC on a readied miss

- **GIVEN** a Dying 1 target at 0 HP with AC 18, and a stored non-agile Strike with snapshot 0 and attack bonus +8
- **WHEN** the d20 reads 4 and the damage dice are empty
- **THEN** the penalty is 0
- **AND** the total is 12
- **AND** the degree is failure
- **AND** the target stays Dying 1
- **AND** the reaction is spent

#### Scenario: Damage to Dying 1 makes it Dying 2

- **GIVEN** a Dying 1 target at 0 HP with AC 18, and a stored Strike with snapshot 0 and attack bonus +8
- **WHEN** the d20 reads 10 and damage is 1
- **THEN** the total is 18
- **AND** the degree is success
- **AND** the target is Dying 2

#### Scenario: Damage to Unconscious at 0 HP is a knockout

- **GIVEN** an Unconscious target at 0 HP, wounded 0, with AC 18, and a stored Strike with attack bonus +8
- **WHEN** the d20 reads 10 and damage is 1
- **THEN** the target is Dying 1
- **AND** its HP stays 0

#### Scenario: A Dead target spends nothing

- **GIVEN** a Dead target and a stored Strike whose creature has a ready reaction
- **WHEN** the trigger fires
- **THEN** the fire is rejected
- **AND** the reaction stays ready
- **AND** the ready stays stored
- **AND** no die is rolled

### Requirement: The readied action is one of four

The readied action SHALL be Strike, Stride, Step, or Dismiss (https://2e.aonprd.com/Actions.aspx?ID=2299, Player Core pg. 417). A readied Strike SHALL target the trigger's subject. Any other action, including a two-action or three-action activity or a free action that already has a trigger, SHALL be rejected before actions are spent.

**Rule**: Any one single action or free action you can use.
**Why**: Phase 1 can resolve four of those actions and no others.
**Substitute**: Strike, Stride, Step, and Dismiss only. The Strike's target is the trigger subject.
**Intent**: A later change adds an action by extending the enum. The cost and the trigger stay.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2299 (Player Core pg. 417, Ready)

#### Scenario: Dismiss drops to no cover

- **GIVEN** a creature with standard cover and 3 actions
- **WHEN** it Readies Dismiss and the trigger fires
- **THEN** its cover is none
- **AND** the reaction is spent
- **AND** the ready is cleared

#### Scenario: A two-action activity is rejected

- **GIVEN** an alive creature with 3 actions
- **WHEN** it Readies a two-action activity
- **THEN** the ready is rejected
- **AND** 3 actions remain
- **AND** nothing is stored

#### Scenario: Delay cannot be readied

- **GIVEN** an alive creature with 3 actions at the start of its turn
- **WHEN** it Readies the Delay free action
- **THEN** the ready is rejected
- **AND** 3 actions remain
- **AND** nothing is stored

#### Scenario: A readied Strike cannot name a third creature

- **GIVEN** creatures 1, 2, and 3, and creature 1's turn is open
- **WHEN** creature 1 Readies a Strike whose target is creature 3 on a trigger about creature 2
- **THEN** the ready is rejected
- **AND** nothing is stored

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

- **GIVEN** Alive CombatantId(1) at (0, 0), Medium, reach 5, with a stored Dismiss on CombatantId(2) entering reach from outside, and Alive CombatantId(2), Medium, Speed 25, stepping from (0, 2) to (0, 1)
- **WHEN** that Step is offered
- **THEN** the Dismiss resolves, the reaction is spent, and the ready is cleared

#### Scenario: Beginning a Strike fires

- **GIVEN** Alive CombatantId(1) at (0, 0), Medium, reach 5, with a stored non-agile Strike, snapshot 0, attack bonus +8, on CombatantId(2) beginning a Strike, and Alive CombatantId(2) at (0, 1), Medium, Armor Class 18, 20 Hit Points
- **WHEN** CombatantId(2) begins a Strike and the d20 reads 10
- **THEN** the penalty is 0, the total is 18, and the degree is success
- **AND** the reaction is spent

#### Scenario: Beginning a Seek does not fire

- **GIVEN** Alive CombatantId(1) at (0, 0) with a stored trigger that CombatantId(2) begins a Strike, reaction ready, and a scripted natural 10
- **WHEN** CombatantId(2) begins a Seek
- **THEN** the error is TriggerMismatch
- **AND** the reaction stays ready and the natural 10 is unused

#### Scenario: Moving inside reach does not fire

- **GIVEN** Alive CombatantId(1) at (0, 0), Medium, reach 5, with a stored Strike on CombatantId(2) entering reach, and Alive CombatantId(2), Medium, moving from (0, 1) to (1, 0)
- **WHEN** that move is offered
- **THEN** the error is TriggerMismatch
- **AND** the reaction stays ready and the ready stays stored
- **AND** no die is rolled

### Requirement: A readied Stride uses a fresh diagonal counter

A readied Stride or Step SHALL take its path from the caller and SHALL build a fresh diagonal counter (https://2e.aonprd.com/Actions.aspx?ID=2299, Player Core pg. 417; DEV-15). The caller's own-turn counter SHALL stay unchanged. An illegal path SHALL be rejected before the reaction is spent, and the stored action SHALL stay.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2299 (Player Core pg. 417, Ready)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2357 (Player Core pg. 421, Diagonal Movement)

#### Scenario: An orthogonal Stride spends the reaction

- **GIVEN** a stored Stride, Speed 25, start (0, 0), and a caller counter that has already seen 1 diagonal
- **WHEN** the trigger fires with path (1, 0)
- **THEN** the creature ends at (1, 0)
- **AND** the distance is 5 feet
- **AND** the reaction is spent
- **AND** the caller's counter still has 1 diagonal

#### Scenario: The first diagonal of a readied Stride costs 5

- **GIVEN** a stored Stride, Speed 25, and start (0, 0)
- **WHEN** the trigger fires with path (1, 1)
- **THEN** the distance is 5 feet
- **AND** the creature ends at (1, 1)

#### Scenario: A path beyond Speed spends nothing

- **GIVEN** a stored Stride, Speed 5, and start (0, 0)
- **WHEN** the trigger fires with path (1, 0) then (2, 0)
- **THEN** the fire is rejected
- **AND** the reaction stays ready
- **AND** the ready stays stored
- **AND** the creature is not moved

#### Scenario: A readied Step moves one square

- **GIVEN** a stored Step, Speed 25, and start (0, 0)
- **WHEN** the trigger fires with destination (1, 0)
- **THEN** the creature ends at (1, 0)
- **AND** the distance is 5 feet
- **AND** the reaction is spent

### Requirement: Another creature's delay leaves the snapshot

Delay SHALL NOT change another creature's stored count or stored action (https://2e.aonprd.com/Actions.aspx?ID=2294, Player Core pg. 416).

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2299 (Player Core pg. 417, Ready)
**Source**: https://2e.aonprd.com/Actions.aspx?ID=2294 (Player Core pg. 416, Delay)

#### Scenario: A delay does not rewrite a stored count

- **GIVEN** Side::Pc CombatantId(1) and Side::Pc CombatantId(2), Perception +0, scripted initiative naturals 15 then 10, CombatantId(2) has a stored non-agile Strike with frozen attack count 1, CombatantId(1) is Delayed, and CombatantId(2)'s turn has ended
- **WHEN** CombatantId(1) returns
- **THEN** CombatantId(2)'s stored action is still that Strike and the frozen attack count is 1

### Requirement: Ready rejects before it spends

Ready SHALL reject, in order, an unknown actor, an unknown subject, a turn that is not open, a creature whose turn it is not, a creature that is not alive, an action outside the four, a free action that already has a trigger, a trigger whose subject is the readier, a Strike aimed at someone other than the trigger subject, fewer than 2 actions, then a round past the maximum. Nothing is stored and no die is rolled.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2299 (Player Core pg. 417, Ready)

#### Scenario: An unknown creature is rejected first

- **GIVEN** an encounter of creature 1 and no open turn
- **WHEN** creature 9 Readies a Strike
- **THEN** the error is the unknown creature
- **AND** nothing is stored

#### Scenario: An unknown subject is rejected before a closed turn

- **GIVEN** only creature 1 and no open turn
- **WHEN** creature 1 Readies a Strike triggered by creature 9 entering reach
- **THEN** the error is the unknown creature
- **AND** nothing is stored

#### Scenario: A trigger about the readier is rejected

- **GIVEN** creature 1's turn is open with 3 actions and the reaction ready
- **WHEN** creature 1 Readies a Strike triggered by creature 1 entering reach
- **THEN** the error is that the trigger subject is the readier
- **AND** 3 actions remain
- **AND** nothing is stored

#### Scenario: One action left is rejected before a round past the maximum

- **GIVEN** round 4294967295, creature 2's turn open, and 1 action remaining
- **WHEN** creature 2 Readies a Strike
- **THEN** the error is too few actions
- **AND** the round stays 4294967295
- **AND** 1 action remains
- **AND** nothing is stored

#### Scenario: Ready at the maximum round changes nothing

- **GIVEN** round 4294967295 and creature 2's turn open with 3 actions, in order creature 1 then creature 2
- **WHEN** creature 2 Readies a Strike
- **THEN** the error is the round past the maximum
- **AND** the round stays 4294967295
- **AND** 3 actions remain
- **AND** nothing is stored

### Requirement: Fire rejects before it rolls

Fire SHALL reject, in order, an unknown readier, an unknown subject, no stored ready, the wrong action, a readier who is not Alive, a delayed readier, a trigger mismatch, a begun Strike out of reach, a Dead target, then no reaction. An illegal path, PositioningError from Dismiss, and an attack total past the bounds SHALL fail before the reaction is spent or the d20 is drawn. PositioningError SHALL be returned. No earlier check writes, and no die is rolled.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2299 (Player Core pg. 417, Ready)

#### Scenario: No stored ready is rejected before a dead target

- **GIVEN** creature 1 has no stored ready and creature 2 is Dead
- **WHEN** a Strike trigger is offered for creature 1
- **THEN** the error is that nothing is readied
- **AND** no die is rolled

#### Scenario: The wrong action is rejected before a delayed readier

- **GIVEN** a delayed creature whose stored ready is a Strike
- **WHEN** a Stride is fired for that creature
- **THEN** the error is the wrong action
- **AND** the ready stays stored
- **AND** no die is rolled

#### Scenario: A delayed readier is rejected before the trigger is matched

- **GIVEN** a delayed creature that also has a stored ready
- **WHEN** its trigger is offered
- **THEN** the error is that the creature is delayed
- **AND** the ready stays stored
- **AND** no die is rolled

#### Scenario: A dying readier is rejected before a dead target

- **GIVEN** a Dying readier with a stored Strike and a Dead target
- **WHEN** the trigger fires
- **THEN** the error is that the readier is not conscious
- **AND** the reaction flag is unchanged
- **AND** the ready stays stored

#### Scenario: A dead target is rejected before a missing reaction

- **GIVEN** a stored Strike, a Dead target, and a readier with no reaction
- **WHEN** the trigger fires
- **THEN** the error is the dead target
- **AND** the ready stays stored
- **AND** no die is rolled

### Requirement: A begun Strike is checked for reach

A readied Strike on a begins-an-action trigger SHALL check the subject's square against the readier's melee reach, or against the weapon's range increment when the weapon is ranged, at fire time. Out of reach SHALL be OutOfReach. The reaction and the ready SHALL stay, and no die SHALL be rolled.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2299 (Player Core pg. 417, Ready)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2379 (Player Core pg. 426, Range and Reach)

#### Scenario: A begun strike out of reach spends nothing

- **GIVEN** Alive CombatantId(1) at (0, 0), Medium, reach 5, with a stored non-agile Strike on CombatantId(2) beginning a Strike, reaction ready, and a scripted natural 10
- **WHEN** Alive CombatantId(2), Medium, at (0, 3), begins a Strike
- **THEN** the error is OutOfReach
- **AND** the reaction stays ready, the ready stays stored, and the natural 10 is unused

### Requirement: Hidden targets stay with the caller

A readied Strike SHALL NOT roll a flat check and SHALL NOT read hidden or undetected. The caller supplies the target. The Strike SHALL use baked Armor Class.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2299 (Player Core pg. 417, Ready)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2285 (Player Core pg. 401, Hidden and Undetected)

#### Scenario: A readied strike draws no flat check

- **GIVEN** Alive CombatantId(1) with a stored non-agile Strike, snapshot 0, attack bonus +8, and Alive CombatantId(2) at Armor Class 18 who is hidden from CombatantId(1), and a script of 10 then 5
- **WHEN** the Strike fires
- **THEN** the total is 18 and the degree is success
- **AND** only the natural 10 is drawn
- **AND** the face 5 is unused

### Requirement: Matching reactions resolve in initiative order

When several readied reactions match one trigger, they SHALL resolve in the current initiative order, each fully before the next. A tie SHALL follow DEV-05: enemy before PC, then lower CombatantId (DEV-55). A later reaction whose target is already Dead SHALL be TargetDead and SHALL keep its reaction and its ready.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2299 (Player Core pg. 417, Ready)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2423 (Player Core pg. 435, Step 1: Roll Initiative)

#### Scenario: The first readied reaction kills and the second keeps ready

- **GIVEN** the order is Side::Enemy CombatantId(1) then Side::Pc CombatantId(2) then Side::Pc CombatantId(3), CombatantId(3) has 9 Hit Points and uses_dying_rules false and Armor Class 18, and both CombatantId(1) and CombatantId(2) have a stored non-agile Strike, snapshot 0, attack bonus +8, on CombatantId(3) beginning a Strike
- **WHEN** CombatantId(3) begins a Strike and the reactions resolve, CombatantId(1) with natural 10 and damage 1d8+1 showing 8, then CombatantId(2) with natural 12
- **THEN** CombatantId(3) is Dead at 0 Hit Points
- **AND** CombatantId(1)'s reaction is spent and that ready is cleared
- **AND** CombatantId(2)'s error is TargetDead, that reaction stays ready, that ready stays stored, and the natural 12 is unused

#### Scenario: An enemy reaction resolves before a tied pc

- **GIVEN** Side::Enemy CombatantId(4) and Side::Pc CombatantId(1) both have initiative total 12, the order is CombatantId(4) then CombatantId(1), both have a stored non-agile Strike, snapshot 0, attack bonus +8, on Alive CombatantId(3) at 30 Hit Points and Armor Class 18 beginning a Strike
- **WHEN** CombatantId(3) begins a Strike and both reactions resolve, each with natural 10 and damage 1
- **THEN** CombatantId(3) is at 28 Hit Points
- **AND** CombatantId(4)'s reaction is spent before CombatantId(1)'s reaction is spent

### Requirement: A knockout abandons the triggering Strike

If a readied Strike leaves the triggering creature Dying or Dead, that creature's triggering Strike SHALL then return NotConscious. It SHALL spend no action and SHALL draw no die. The readied Strike's result SHALL stay.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2299 (Player Core pg. 417, Ready)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2324 (Player Core pg. 410, Getting Knocked Out)

#### Scenario: A knockout abandons the triggering strike

- **GIVEN** Alive CombatantId(2) with 3 actions, 9 Hit Points, wounded 0, uses_dying_rules, and Armor Class 18, and Alive CombatantId(1) with a stored non-agile Strike, snapshot 0, attack bonus +8, on CombatantId(2) beginning a Strike
- **WHEN** CombatantId(2) begins a Strike, the readied Strike rolls natural 10 and damage 1d8+1 showing 8, and CombatantId(2) then Strikes
- **THEN** CombatantId(2) is Dying(1) at 0 Hit Points
- **AND** CombatantId(1)'s reaction is spent and the ready is cleared
- **AND** CombatantId(2)'s Strike is NotConscious, CombatantId(2) still has 3 actions, and CombatantId(2)'s natural is unused

### Requirement: A dead readier cannot fire

A Dead readier SHALL be NotConscious. The ready and the reaction SHALL stay, and no die SHALL be rolled. This change SHALL NOT clear a ready from a death hook.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2299 (Player Core pg. 417, Ready)

#### Scenario: A dead readier spends nothing

- **GIVEN** Dead CombatantId(1) with a stored non-agile Strike, snapshot 0, reaction ready, and Alive CombatantId(2) at (0, 1), and a scripted natural 10
- **WHEN** CombatantId(2) begins a Strike and CombatantId(1)'s ready is fired
- **THEN** the error is NotConscious
- **AND** the reaction stays ready, the ready stays stored, and the natural 10 is unused

### Requirement: A returned creature may Ready

A creature whose open turn was produced by return SHALL be able to Ready. Ready SHALL snapshot the attack count the start left, which return did not write.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2299 (Player Core pg. 417, Ready)
**Source**: https://2e.aonprd.com/Actions.aspx?ID=2294 (Player Core pg. 416, Delay)

#### Scenario: A returned creature may ready

- **GIVEN** Side::Pc CombatantId(1) and Side::Pc CombatantId(2), Perception +0, scripted initiative naturals 15 then 10, CombatantId(1) Delayed with attack count 0, and CombatantId(2)'s turn ended
- **WHEN** CombatantId(1) returns and Readies a non-agile Strike on CombatantId(2) entering reach
- **THEN** the frozen attack count is 0
- **AND** the turn has ended, actions are 0, and the reaction is ready
