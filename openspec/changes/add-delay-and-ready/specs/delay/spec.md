# delay

## Purpose

Leaves the initiative order at the start of a turn and returns later at a new place. The regained reaction stays ready and cannot be spent until the creature returns.

## ADDED Requirements

### Requirement: The same delay twice matches

The same participants, scripted dice, Delay, and return SHALL produce the same order, actions, reaction flag, and temporary Hit Points. A call SHALL NOT read a file, the network, a clock, or a global generator, and SHALL NOT panic.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2294 (Player Core pg. 416, Delay)

#### Scenario: The same delay twice matches

- **GIVEN** CombatantId(1) and CombatantId(2), both Alive, Perception +0, scripted initiative naturals 15 then 10, and CombatantId(1) has 4 temporary Hit Points with 2 ticks
- **WHEN** CombatantId(1)'s turn starts, CombatantId(1) Delays, CombatantId(2)'s turn starts and ends, and CombatantId(1) returns, and the same steps are resolved again from the same start
- **THEN** both results have order CombatantId(2) then CombatantId(1), round 1, CombatantId(1) open with 3 actions and the reaction ready, CombatantId(1)'s initiative total 15, CombatantId(2)'s total 10, and 4 temporary Hit Points with 2 ticks

### Requirement: Delay is a free action at the start of your turn

Delay SHALL be legal only on the open turn `start_turn` just granted, with 3 actions, attack count 0, and the reaction ready. A turn opened by return SHALL be TriggerNotMet. Delay SHALL spend no action and no reaction. It SHALL set actions to 0, close the turn, and mark the creature delayed.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2294 (Player Core pg. 416, Delay)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2428 (Player Core pg. 435, Step 1: Start Your Turn)

#### Scenario: Delay spends nothing and closes the turn

- **GIVEN** CombatantId(1), CombatantId(2), and CombatantId(3), all Alive, Perception +0, scripted initiative naturals 15 then 10 then 5, and CombatantId(1)'s turn has just started
- **WHEN** CombatantId(1) Delays
- **THEN** CombatantId(1) has 0 actions, the reaction is ready, no turn is open, and the next start is CombatantId(2)
- **AND** no die is rolled

#### Scenario: An action already spent is not the start

- **GIVEN** CombatantId(1)'s turn is open with 2 actions left and the reaction ready
- **WHEN** CombatantId(1) Delays
- **THEN** the error is TriggerNotMet, 2 actions remain, the turn stays open, and CombatantId(1) is not delayed

#### Scenario: A spent reaction is not the start

- **GIVEN** CombatantId(1)'s turn is open with 3 actions and the reaction already spent
- **WHEN** CombatantId(1) Delays
- **THEN** the error is TriggerNotMet, 3 actions remain, the reaction stays unready, and the turn stays open

### Requirement: Return changes place and keeps the rolled total

Return SHALL be a free action at the end of another creature's turn. Returned creatures SHALL be inserted immediately after that creature, enemies before PCs, then lower CombatantId (DEV-52, DEV-05). The first SHALL open with 3 actions. Each initiative total SHALL stay on its creature. Return SHALL NOT spend the reaction.

**Rule**: When several creatures return at the end of the same turn, the book does not order them. https://2e.aonprd.com/Actions.aspx?ID=2294 (Player Core pg. 416, Delay)
**Why**: Two inserts at one anchor need one order or the replay changes.
**Substitute**: Enemies before PCs, then lower CombatantId, the same tiebreak as initiative (DEV-05, DEV-52).
**Intent**: The same ids and sides always produce the same order.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2294 (Player Core pg. 416, Delay)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2423 (Player Core pg. 435, Step 1: Roll Initiative)

#### Scenario: Return acts after the creature who just ended

- **GIVEN** CombatantId(1), CombatantId(2), and CombatantId(3), all Side::Pc, Perception +0, scripted initiative naturals 15 then 10 then 5, CombatantId(1) has Delayed, and CombatantId(2)'s turn has ended
- **WHEN** CombatantId(1) returns
- **THEN** the order is CombatantId(2), CombatantId(1), CombatantId(3)
- **AND** CombatantId(1)'s initiative total is 15, CombatantId(2)'s is 10, and CombatantId(3)'s is 5
- **AND** the round is 1, CombatantId(1)'s turn is open, and CombatantId(1) has 3 actions and the reaction ready

#### Scenario: Two returns at one end go by the lower id

- **GIVEN** CombatantId(1), CombatantId(2), CombatantId(3), and CombatantId(4), all Side::Pc, Perception +0, scripted initiative naturals 20 then 15 then 10 then 5, CombatantId(1) and CombatantId(3) are delayed, and CombatantId(4)'s turn has just ended
- **WHEN** CombatantId(3) and CombatantId(1) return together
- **THEN** the order is CombatantId(2), CombatantId(4), CombatantId(1), CombatantId(3)
- **AND** the round is 1 and the open turn is CombatantId(1)
- **AND** CombatantId(1)'s initiative total is 20, CombatantId(2)'s is 15, CombatantId(3)'s is 10, and CombatantId(4)'s is 5
- **AND** CombatantId(3) is not delayed and is not the open turn

#### Scenario: An enemy returns before a pc

- **GIVEN** Side::Pc CombatantId(1) with initiative total 20, Side::Pc CombatantId(2) with total 15, Side::Enemy CombatantId(3) with total 10, and Side::Pc CombatantId(4) with total 5, CombatantId(1) and CombatantId(3) delayed, and CombatantId(4)'s turn just ended
- **WHEN** CombatantId(1) and CombatantId(3) return together
- **THEN** the order is CombatantId(2), CombatantId(4), CombatantId(3), CombatantId(1)
- **AND** the round is 1 and the open turn is CombatantId(3)
- **AND** CombatantId(3)'s initiative total is 10 and CombatantId(1)'s is 20

### Requirement: The reaction is available while delayed and cannot be spent

The start SHALL regain the reaction before Delay. While the creature is delayed, that flag SHALL stay true. Every reaction precheck SHALL return Delayed after NotConscious and before any action or die is spent. That covers Shield Block on a Strike, Reactive Strike, Aid, and readied fire. A spend SHALL NOT clear the flag. After return, the first spend SHALL succeed and the second SHALL be NoReaction.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2294 (Player Core pg. 416, Delay)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2428 (Player Core pg. 435, Step 1: Start Your Turn)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2432 (Player Core pg. 436, Reactions in Encounters)

#### Scenario: A spend while delayed leaves the reaction ready

- **GIVEN** CombatantId(1) has Delayed and the reaction is ready, and a scripted natural 8
- **WHEN** CombatantId(1) spends a reaction
- **THEN** the error is Delayed, the reaction is still ready, and the natural 8 is unused

#### Scenario: A shield block while delayed spends nothing

- **GIVEN** Side::Pc CombatantId(1) and Side::Enemy CombatantId(2), Perception +0, scripted initiative naturals 15 then 10, CombatantId(1) Alive at 30 Hit Points with Armor Class 18, a steel shield at 20 Hit Points that the start lowered, reaction ready, and Delayed, and CombatantId(2)'s turn open with 3 actions
- **WHEN** CombatantId(2) Strikes CombatantId(1) with attack bonus +8, ShieldBlock::Requested, a scripted natural 10, and damage 1d8+4 with a die of 5
- **THEN** the error is Delayed
- **AND** CombatantId(2) still has 3 actions
- **AND** the natural 10 and the damage die are unused
- **AND** CombatantId(1)'s reaction stays ready and CombatantId(1) stays delayed

#### Scenario: A reactive strike while delayed spends nothing

- **GIVEN** CombatantId(1) is Delayed at (0, 0) with melee reach 5, Medium, reaction ready, and attack bonus +8, and Alive CombatantId(2) at (0, 1), Medium, whose turn is open with 3 actions, and a scripted natural 10
- **WHEN** CombatantId(1) makes a Reactive Strike against CombatantId(2) for a 1-action Manipulate cost
- **THEN** the error is Delayed
- **AND** CombatantId(2) still has 3 actions and the natural 10 is unused
- **AND** CombatantId(1)'s reaction stays ready

#### Scenario: Aid while delayed spends nothing

- **GIVEN** Side::Pc CombatantId(1) is Delayed, reaction ready, aid modifier +7, AidRank::BelowMaster, and prepared for Side::Pc CombatantId(2), whose turn is open with 3 actions, and a scripted natural 10
- **WHEN** CombatantId(1) aids CombatantId(2)
- **THEN** the error is Delayed
- **AND** CombatantId(2) still has 3 actions and the natural 10 is unused
- **AND** CombatantId(1)'s reaction stays ready and the preparation stays

#### Scenario: Return leaves one reaction

- **GIVEN** CombatantId(1) Delayed with the reaction ready, and CombatantId(2)'s turn has ended
- **WHEN** CombatantId(1) returns and spends the reaction twice
- **THEN** the first spend succeeds and the second is NoReaction

### Requirement: Return does not repeat the start hook

Return SHALL NOT change temporary Hit Points, their ticks, Hit Points, or wounded, and a shield the start already lowered SHALL stay down. A later resume in that return SHALL also stay unticked. Phase 1 SHALL NOT add an end-of-turn step. Persistent damage (DEV-29) and frightened are not modeled; when they exist, they fire on Delay. Return never ticks. A missed arrival uses the ordinary start, which changes no Hit Points today and ticks once when DEV-32 is wired.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2294 (Player Core pg. 416, Delay)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2428 (Player Core pg. 435, Step 1: Start Your Turn)
**Source**: DEV-29 in docs/designer/rules-deviations.md. Persistent damage is not modeled.
**Source**: DEV-32 in docs/designer/rules-deviations.md. The temporary Hit Point tick is the recipient's start.

#### Scenario: Return leaves temporary Hit Points as the start left them

- **GIVEN** CombatantId(1) is Alive at 9 Hit Points with 4 temporary Hit Points and 2 ticks, and CombatantId(2) is Alive, both Perception +0, scripted initiative naturals 15 then 10
- **WHEN** CombatantId(1)'s turn starts, CombatantId(1) Delays, CombatantId(2)'s turn ends, and CombatantId(1) returns
- **THEN** CombatantId(1) has 4 temporary Hit Points with 2 ticks, 9 Hit Points, and 3 actions
- **AND** the return rolls no die

#### Scenario: The later returned creature is not ticked

- **GIVEN** CombatantId(1), CombatantId(2), CombatantId(3), and CombatantId(4), Perception +0, scripted initiative naturals 20 then 15 then 10 then 5, CombatantId(1) and CombatantId(3) each Alive at 9 Hit Points with 4 temporary Hit Points and 2 ticks, both delayed, and CombatantId(4)'s turn has just ended
- **WHEN** CombatantId(3) and CombatantId(1) return, CombatantId(1)'s opened turn ends, and the next start is resolved
- **THEN** CombatantId(3)'s turn is open with 3 actions and the reaction ready
- **AND** CombatantId(1) and CombatantId(3) each have 4 temporary Hit Points with 2 ticks and 9 Hit Points
- **AND** no die is rolled

#### Scenario: A lowered shield stays down

- **GIVEN** CombatantId(1)'s shield was raised and the start lowered it before Delay, and CombatantId(1) then returns
- **WHEN** the shield is read
- **THEN** the shield is not raised and its Hit Points are unchanged

### Requirement: Missing the return plays the next turn at the old place

If a creature is still delayed when its slot is reached, the delayed actions SHALL stay lost. That arrival SHALL be a real next turn at the original place, with the ordinary start. Order and each initiative total SHALL stay. A Dying arrival SHALL open with 0 actions and no reaction. A Dead arrival SHALL be skipped and the delayed mark SHALL clear. The next living creature SHALL open.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2294 (Player Core pg. 416, Delay)

#### Scenario: A full round without returning plays the next turn in place

- **GIVEN** CombatantId(1), CombatantId(2), and CombatantId(3), Perception +0, scripted initiative naturals 15 then 10 then 5, and CombatantId(1) has 4 temporary Hit Points with 3 ticks at 9 Hit Points
- **WHEN** CombatantId(1)'s turn starts, CombatantId(1) Delays, CombatantId(2) and CombatantId(3) each start and end, and the next start is resolved
- **THEN** the round is 2 and the order is CombatantId(1), CombatantId(2), CombatantId(3)
- **AND** CombatantId(1)'s initiative total is 15, CombatantId(2)'s is 10, and CombatantId(3)'s is 5
- **AND** CombatantId(1)'s turn is open with 3 actions and the reaction ready
- **AND** CombatantId(1) has 4 temporary Hit Points with 3 ticks

#### Scenario: A sole creature's next start is the new turn

- **GIVEN** only CombatantId(1), Alive, Perception +0, a scripted initiative natural 10, 9 Hit Points, and 4 temporary Hit Points with 2 ticks
- **WHEN** the turn starts, CombatantId(1) Delays, and the next start is resolved
- **THEN** the round is 2, the initiative total is 10, the turn is open with 3 actions and the reaction ready
- **AND** temporary Hit Points stay 4 and the ticks stay 2
- **AND** Hit Points stay 9

#### Scenario: Returning after the last creature acts in this round

- **GIVEN** CombatantId(1), CombatantId(2), and CombatantId(3), all Side::Pc, Perception +0, scripted initiative naturals 15 then 10 then 5, and CombatantId(1) has Delayed
- **WHEN** CombatantId(2) and CombatantId(3) each start and end, and CombatantId(1) returns
- **THEN** the round is 1 and the order is CombatantId(2), CombatantId(3), CombatantId(1)
- **AND** CombatantId(1)'s turn is open with 3 actions
- **AND** CombatantId(1)'s initiative total is 15, CombatantId(2)'s is 10, and CombatantId(3)'s is 5
- **WHEN** CombatantId(1) ends the turn and the next start is resolved
- **THEN** the round is 2 and the open turn is CombatantId(2) with 3 actions

#### Scenario: A mid-round return keeps the next turn in this round

- **GIVEN** CombatantId(1), CombatantId(2), and CombatantId(3), all Side::Pc, Perception +0, scripted initiative naturals 15 then 10 then 5, and CombatantId(1) has Delayed
- **WHEN** CombatantId(2)'s turn ends, CombatantId(1) returns, CombatantId(1)'s turn ends, and the next start is resolved
- **THEN** the round is 1 and the open turn is CombatantId(3) with 3 actions
- **AND** the order is CombatantId(2), CombatantId(1), CombatantId(3)

#### Scenario: A second delay after return writes nothing

- **GIVEN** Side::Pc CombatantId(1) and CombatantId(2), Perception +0, scripted initiative naturals 15 then 10, CombatantId(1) Delayed, and CombatantId(2)'s turn ended
- **WHEN** CombatantId(1) returns and then Delays
- **THEN** the error is TriggerNotMet
- **AND** the order stays CombatantId(2), CombatantId(1), the round stays 1, and CombatantId(1) stays open with 3 actions and is not delayed

#### Scenario: Ready ending the turn blocks delay

- **GIVEN** Side::Pc CombatantId(1) and CombatantId(2), Perception +0, scripted initiative naturals 15 then 10, and CombatantId(1) has Readied a Strike and ended that turn
- **WHEN** CombatantId(1) Delays
- **THEN** the error is TurnNotOpen
- **AND** CombatantId(1) is not delayed and the order is unchanged

#### Scenario: A dying creature's missed turn opens with no actions

- **GIVEN** CombatantId(1), CombatantId(2), and CombatantId(3), Perception +0, scripted initiative naturals 15 then 10 then 5, CombatantId(1) Delayed, then Dying(1) at 0 Hit Points
- **WHEN** CombatantId(2) and CombatantId(3) each start and end, and the next start is resolved
- **THEN** the round is 2 and the order is CombatantId(1), CombatantId(2), CombatantId(3)
- **AND** CombatantId(1)'s turn is open with 0 actions and the reaction not ready
- **AND** CombatantId(1) is not delayed

#### Scenario: A dead creature's missed turn is skipped

- **GIVEN** CombatantId(1), CombatantId(2), and CombatantId(3), Perception +0, scripted initiative naturals 15 then 10 then 5, CombatantId(1) Delayed, then Dead
- **WHEN** CombatantId(2) and CombatantId(3) each start and end, and the next start is resolved
- **THEN** the round is 2 and the open turn is CombatantId(2) with 3 actions
- **AND** CombatantId(1) is not delayed and has 0 actions

#### Scenario: A delayed creature knocked out stays delayed

- **GIVEN** CombatantId(1) is Delayed, Alive, uses_dying_rules, 9 Hit Points, wounded 0, Armor Class 18, reaction ready, and CombatantId(2)'s turn is open with 3 actions and attack bonus +8
- **WHEN** CombatantId(2) Strikes with ShieldBlock::Declined, a scripted natural 10, and damage 1d8+1 with a die of 8
- **THEN** CombatantId(1) is Dying(1) at 0 Hit Points and stays delayed
- **AND** CombatantId(1)'s reaction stays ready
- **AND** CombatantId(2) has 2 actions

### Requirement: Delay does not change the attack count

Delay and return SHALL NOT write the attack count. The first Strike after return SHALL use the count the start set.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2289 (Player Core pg. 402, Multiple Attack Penalty)
**Source**: https://2e.aonprd.com/Actions.aspx?ID=2294 (Player Core pg. 416, Delay)

#### Scenario: The first Strike after return has no penalty

- **GIVEN** CombatantId(1) resolved two non-agile Strikes, ended that turn, started the next turn, Delayed, and returned, with attack bonus +8, Armor Class 18, 0 damage dice, and a scripted natural 10
- **WHEN** CombatantId(1) Strikes
- **THEN** the multiple attack penalty is 0 and the total is 18
- **AND** the degree is success

### Requirement: Delay rejects before it changes the order

Delay SHALL check UnknownCombatant, then TurnNotOpen, then NotYourTurn, then NotConscious, then TriggerNotMet, then RoundOverflow. Return SHALL check an empty list, then a duplicate id, then UnknownCombatant, then TurnInProgress, then NotDelayed, then NotConscious, then TriggerNotMet. A failure SHALL write nothing and SHALL roll nothing.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2294 (Player Core pg. 416, Delay)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2429 (Player Core pg. 436, Step 2: Act)

#### Scenario: An unknown creature is not a closed turn

- **GIVEN** only CombatantId(1), initiative rolled, and no turn open
- **WHEN** CombatantId(9) Delays
- **THEN** the error is UnknownCombatant and CombatantId(1) is not delayed

#### Scenario: Delay before a start is TurnNotOpen

- **GIVEN** CombatantId(1), initiative rolled, and no turn open
- **WHEN** CombatantId(1) Delays
- **THEN** the error is TurnNotOpen and CombatantId(1) has 0 actions

#### Scenario: Delaying on another turn is NotYourTurn

- **GIVEN** CombatantId(2)'s turn is open and CombatantId(1) is Alive with the reaction ready
- **WHEN** CombatantId(1) Delays
- **THEN** the error is NotYourTurn and the open turn is still CombatantId(2)

#### Scenario: A dying creature is NotConscious

- **GIVEN** CombatantId(1)'s turn is open, CombatantId(1) is Dying(1) at 0 Hit Points with 0 actions, and the reaction is not ready
- **WHEN** CombatantId(1) Delays
- **THEN** the error is NotConscious, actions stay 0, and CombatantId(1) is not delayed

#### Scenario: Round overflow does not delay

- **GIVEN** the round is 4294967295, the order is CombatantId(1) then CombatantId(2), and CombatantId(2)'s turn has just started with 3 actions and the reaction ready
- **WHEN** CombatantId(2) Delays
- **THEN** the error is RoundOverflow
- **AND** the round is 4294967295, the turn stays open, 3 actions remain, and CombatantId(2) is not delayed

#### Scenario: Return during a turn writes nothing

- **GIVEN** CombatantId(1) is delayed and CombatantId(2)'s turn is open
- **WHEN** CombatantId(1) returns
- **THEN** the error is TurnInProgress, CombatantId(1) stays delayed, and the open turn is CombatantId(2)

#### Scenario: A duplicate return writes nothing

- **GIVEN** CombatantId(1) is delayed and no turn is open after CombatantId(2) ended
- **WHEN** CombatantId(1) is listed twice in one return
- **THEN** the error is DuplicateCombatant and CombatantId(1) stays delayed

#### Scenario: A dying delayed creature cannot return

- **GIVEN** CombatantId(1) is delayed and is Dying(1) at 0 Hit Points, and CombatantId(2)'s turn has ended
- **WHEN** CombatantId(1) returns
- **THEN** the error is NotConscious and CombatantId(1) stays delayed
