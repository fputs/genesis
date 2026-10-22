# ready Specification

## Purpose

Ready stores one single action and a trigger, then spends the reaction to resolve that action before the creature's next turn. A readied attack uses the attack count stored when Ready resolved.

## ADDED Requirements

### Requirement: The same ready twice matches

The same creature, count, trigger, and readied action SHALL resolve to the same stored ready and the same encounter. Ready SHALL roll nothing while it is stored.

#### Scenario: Storing the same ready twice matches

- **GIVEN** one alive creature with 3 actions and attack count 0
- **WHEN** that creature Readies a non-agile Strike on the trigger that a second creature moves within its 5-foot reach
- **THEN** the stored count is 0
- **AND** the turn has ended with 0 actions
- **AND** the reaction is still ready
- **AND** no die was rolled
- **WHEN** the same ready is stored again from the same start
- **THEN** the stored ready and the encounter match the first result

### Requirement: Ready costs two actions and ends the turn

Ready is a two-action concentrate action on your open turn (Player Core pg. 417, [Actions 2299](https://2e.aonprd.com/Actions.aspx?ID=2299)). It SHALL store the named trigger, one single action or free action, and the attack count at resolution, then end the turn. Actions still unused SHALL be lost. Ready SHALL NOT spend the reaction and SHALL NOT change the attack count.

#### Scenario: One Strike then Ready stores count 1

- **GIVEN** an alive creature with 3 actions and attack count 0
- **WHEN** it resolves one Strike and then Readies a non-agile Strike
- **THEN** the stored count is 1
- **AND** the turn has ended
- **AND** actions remaining are 0
- **AND** the attack count is still 1
- **AND** the reaction is still ready
- **AND** no die was rolled by Ready

#### Scenario: Ready with no earlier attack stores count 0

- **GIVEN** an alive creature with 3 actions and attack count 0
- **WHEN** its first action is Ready
- **THEN** the stored count is 0
- **AND** the turn has ended
- **AND** the reaction is still ready

#### Scenario: Ready is legal after the reaction was spent

- **GIVEN** an alive creature with 3 actions whose reaction is not ready
- **WHEN** it Readies a Strike
- **THEN** the ready is stored
- **AND** the reaction stays not ready

### Requirement: A readied attack uses the stored count

A readied attack SHALL call `multiple_attack_penalty` with `readied_attack_count: Some(snapshot)` and agile from the readied weapon (Player Core pg. 417, Actions 2299; R-05; B-06). It SHALL NOT change the snapshot or the creature's attack count. That creature's next own-turn attack-trait penalty SHALL be 0.

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

A stored ready SHALL last until the start of the readier's next turn, and that start SHALL clear it (Player Core pg. 417, Actions 2299). A trigger after that start SHALL be rejected, and no die SHALL be rolled.

#### Scenario: The next start clears the ready

- **GIVEN** creature 1 has a stored Strike and creature 2 then ends its turn
- **WHEN** creature 1's next turn starts
- **THEN** no ready is stored
- **WHEN** the old trigger is offered
- **THEN** the offer is rejected
- **AND** no die is rolled
- **AND** the reaction granted by that start stays ready

### Requirement: Firing spends the reaction and not an action

Using the readied action SHALL spend the readier's reaction and SHALL NOT spend an action (Player Core pg. 417, Actions 2299). The trigger SHALL be offered by the caller when it occurs. A miss SHALL still spend the reaction and clear the ready. With no reaction, the fire SHALL be rejected, the ready SHALL stay, and no die SHALL be rolled. Firing SHALL NOT end the turn that is open and SHALL NOT spend that creature's actions.

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

A Dying or Unconscious target SHALL be struck against its baked AC, and damage SHALL use the blow rules (Player Core pg. 410, Rules 2289; R-08; DEV-36). Damage greater than 0 to an Unconscious creature at 0 HP SHALL be a knockout. A Dead target SHALL be rejected before the roll: the reaction SHALL stay ready, the stored action SHALL stay, and no die SHALL be rolled.

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

The readied action SHALL be Strike, Stride, Step, or Dismiss (Player Core pg. 417, Actions 2299). A readied Strike SHALL target the trigger's subject. Any other action, including a two-action or three-action activity or a free action that already has a trigger, SHALL be rejected before actions are spent.

**Rule**: Any one single action or free action you can use.
**Why**: Phase 1 can resolve four of those actions and no others.
**Substitute**: Strike, Stride, Step, and Dismiss only. The Strike's target is the trigger subject.
**Intent**: A later change adds an action by extending the enum. The cost and the trigger stay.

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

The trigger SHALL be a named creature entering one square within the readier's reach, or that creature beginning Strike, Stride, Step, Drop Prone, or Stand (Player Core pg. 417, Actions 2299). The caller SHALL supply the entered square. Any other trigger SHALL be rejected before the reaction is spent. A Step into reach SHALL satisfy the reach trigger.

**Rule**: A trigger you choose.
**Why**: Phase 1 can observe a square entered and five begun actions.
**Substitute**: Those two trigger kinds only. One entered square, not the whole path.
**Intent**: A later change adds a trigger by extending the enum.

#### Scenario: Entering a square within 5 feet fires

- **GIVEN** a readier at (0, 0) with reach 5 feet and size Medium, and a stored Strike on creature 2 entering reach
- **WHEN** creature 2 enters (0, 1)
- **THEN** the Strike resolves
- **AND** the reaction is spent

#### Scenario: Entering a square 15 feet away does not fire

- **GIVEN** a readier at (0, 0) with reach 5 feet, and a stored Strike on creature 2 entering reach
- **WHEN** creature 2 enters (0, 3)
- **THEN** the fire is rejected
- **AND** the reaction stays ready
- **AND** the ready stays stored
- **AND** no die is rolled

#### Scenario: A Step into reach fires

- **GIVEN** a stored reach trigger and a creature that Steps into a square 5 feet from the readier
- **WHEN** that Step is offered
- **THEN** the readied action resolves

#### Scenario: Beginning a Strike fires

- **GIVEN** a stored trigger that creature 2 begins a Strike
- **WHEN** creature 2 begins a Strike
- **THEN** the readied action resolves

#### Scenario: Beginning a Seek does not fire

- **GIVEN** a stored trigger that creature 2 begins a Strike
- **WHEN** creature 2 begins a Seek
- **THEN** the fire is rejected
- **AND** the reaction stays ready

### Requirement: A readied Stride uses a fresh diagonal counter

A readied Stride or Step SHALL take its path from the caller and SHALL build a fresh diagonal counter (Player Core pg. 417; DEV-15). The caller's own-turn counter SHALL stay unchanged. An illegal path SHALL be rejected before the reaction is spent, and the stored action SHALL stay.

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

Delay SHALL NOT change another creature's stored count or stored action (Player Core pg. 416).

#### Scenario: A delay does not rewrite a stored count

- **GIVEN** creature 2 has a stored non-agile Strike with snapshot 1
- **WHEN** creature 1 delays and later returns
- **THEN** creature 2's snapshot is still 1

### Requirement: Ready rejects before it spends

Ready SHALL reject, in order, an unknown actor, an unknown subject, a turn that is not open, a creature whose turn it is not, a creature that is not alive, an action outside the four, a free action that already has a trigger, a trigger whose subject is the readier, a Strike aimed at someone other than the trigger subject, fewer than 2 actions, then a round past the maximum. Nothing is stored and no die is rolled.

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

Fire SHALL reject, in order, an unknown creature, no stored ready, a different readied action, a delayed readier, a trigger that does not match, a readier that is not alive, a Dead Strike target, then no reaction. An illegal path SHALL be rejected before the reaction is spent. An attack total past the modifier bounds SHALL be rejected before the d20. Nothing changes and no die is rolled.

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
