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
- **THEN** both results have order CombatantId(2) then CombatantId(1), round 1, CombatantId(1) open with 3 actions and the reaction ready, and 4 temporary Hit Points with 2 ticks

### Requirement: Delay is a free action at the start of your turn

Delay SHALL be legal only on that creature's open turn when the start granted 3 actions, the attack count is 0, and the reaction is ready. It SHALL spend no action and no reaction. It SHALL set actions to 0, close the turn, and mark the creature delayed.

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

Return SHALL be a free action at the end of another creature's turn. It SHALL place every returned creature immediately after that creature, lowest CombatantId first. The lowest id SHALL hold the open turn and SHALL have 3 actions. The Perception total SHALL stay the rolled number. Return SHALL NOT spend the reaction.

**Rule**: When several creatures return at the end of the same turn, the book does not order them. https://2e.aonprd.com/Actions.aspx?ID=2294 (Player Core pg. 416, Delay)
**Why**: Two inserts at one anchor need one order or the replay changes.
**Substitute**: Lowest CombatantId acts first. (DEV-52, proposed)
**Intent**: The same ids always produce the same order.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2294 (Player Core pg. 416, Delay)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2423 (Player Core pg. 435, Step 1: Roll Initiative)

#### Scenario: Return acts after the creature who just ended

- **GIVEN** CombatantId(1), CombatantId(2), and CombatantId(3), Perception +0, scripted initiative naturals 15 then 10 then 5, CombatantId(1) has Delayed, and CombatantId(2)'s turn has ended
- **WHEN** CombatantId(1) returns
- **THEN** the order is CombatantId(2), CombatantId(1), CombatantId(3)
- **AND** the initiative totals are 15, 10, and 5
- **AND** the round is 1, CombatantId(1)'s turn is open, and CombatantId(1) has 3 actions and the reaction ready

#### Scenario: Two returns at one end go by the lower id

- **GIVEN** CombatantId(1), CombatantId(2), CombatantId(3), and CombatantId(4), Perception +0, scripted initiative naturals 20 then 15 then 10 then 5, CombatantId(1) and CombatantId(3) are delayed, and CombatantId(4)'s turn has just ended
- **WHEN** CombatantId(3) and CombatantId(1) return together
- **THEN** the order is CombatantId(2), CombatantId(4), CombatantId(1), CombatantId(3)
- **AND** the open turn is CombatantId(1)
- **AND** CombatantId(3) is not delayed and is not the open turn

### Requirement: The reaction is available while delayed and cannot be spent

The start SHALL regain the reaction before Delay. While the creature is delayed, that flag SHALL stay true, and a spend SHALL return Delayed and SHALL NOT clear it. After return, the first spend SHALL succeed and the second SHALL be NoReaction.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2294 (Player Core pg. 416, Delay)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2428 (Player Core pg. 435, Step 1: Start Your Turn)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2432 (Player Core pg. 436, Reactions in Encounters)

#### Scenario: A spend while delayed leaves the reaction ready

- **GIVEN** CombatantId(1) has Delayed and the reaction is ready, and a scripted natural 8
- **WHEN** CombatantId(1) spends a reaction
- **THEN** the error is Delayed, the reaction is still ready, and the natural 8 is unused

#### Scenario: Return leaves one reaction

- **GIVEN** CombatantId(1) Delayed with the reaction ready, and CombatantId(2)'s turn has ended
- **WHEN** CombatantId(1) returns and spends the reaction twice
- **THEN** the first spend succeeds and the second is NoReaction

### Requirement: Return does not repeat the start hook

Return SHALL NOT change temporary Hit Points, their ticks, Hit Points, or wounded, and a shield the start already lowered SHALL stay down. A creature whose turn opens later in that same return SHALL also stay unticked. Persistent damage is not modeled (DEV-29) and SHALL NOT be added. Delay SHALL NOT add an end-of-turn damage step. A missed return SHALL arrive through the ordinary start, which still changes no Hit Points.

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

If a creature is still delayed when its slot is reached, the delayed turn's actions SHALL stay lost. That arrival SHALL be the next turn at the original place, including the start hook. The order and the Perception total SHALL stay unchanged. A not-Alive arrival SHALL open with 0 actions and no reaction.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2294 (Player Core pg. 416, Delay)

#### Scenario: A full round without returning plays the next turn in place

- **GIVEN** CombatantId(1), CombatantId(2), and CombatantId(3), Perception +0, scripted initiative naturals 15 then 10 then 5, and CombatantId(1) has 4 temporary Hit Points with 3 ticks at 9 Hit Points
- **WHEN** CombatantId(1)'s turn starts, CombatantId(1) Delays, CombatantId(2) and CombatantId(3) each start and end, and the next start is resolved
- **THEN** the round is 2 and the order is CombatantId(1), CombatantId(2), CombatantId(3)
- **AND** the initiative totals are 15, 10, and 5
- **AND** CombatantId(1)'s turn is open with 3 actions and the reaction ready
- **AND** CombatantId(1) has 4 temporary Hit Points with 3 ticks

#### Scenario: A sole creature's next start is the new turn

- **GIVEN** only CombatantId(1), Alive, Perception +0, a scripted initiative natural 10, 9 Hit Points, and 4 temporary Hit Points with 2 ticks
- **WHEN** the turn starts, CombatantId(1) Delays, and the next start is resolved
- **THEN** the round is 2, the initiative total is 10, the turn is open with 3 actions and the reaction ready
- **AND** temporary Hit Points stay 4 and the ticks stay 2
- **AND** Hit Points stay 9

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
