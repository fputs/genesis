# reactive-strike Specification

## Purpose
Resolves one Reactive Strike: a melee reaction when a creature inside the reactor's reach uses a manipulate or move action, or makes a ranged attack.

## Requirements

### Requirement: The same reactive strike twice matches

The same inputs SHALL return the same outcome. A call SHALL NOT read a clock, a file, or a global generator. A fallible call SHALL return a typed error and SHALL NOT panic. The rules crate SHALL gain no dependency.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2256 (Player Core pg. 138, Reactive Strike)

#### Scenario: The same reactive strike twice matches

- **GIVEN** two encounters, each with Alive CombatantId(1) and Alive CombatantId(2), both Perception +0, both current Hit Points 20, CombatantId(1) at melee reach 5, and each scripted 10, then 8, then 10, then 3
- **WHEN** each encounter resolves one Reactive Strike from CombatantId(1) at (0, 0) against CombatantId(2) at (-1, 0), both Medium, an ActionCost of 1 action with Manipulate, and ReactiveStrike::Available, with attack bonus +8, Armor Class 18, and damage 1d8+4
- **THEN** both outcomes are equal
- **AND** both naturals are 10, both totals are 18, both degrees are success, both penalties are 0, both damages are 7, and both disruptions are NotDisrupted

### Requirement: A manipulate, move, or ranged attack in reach triggers

An ActionCost SHALL trigger inside the reactor's stored melee reach when its traits include Move, Manipulate, or Ranged. One cost SHALL be one action even when it includes both Move and Manipulate. Interact SHALL be 1 action with Manipulate. A free action with Manipulate other than Release SHALL trigger. Seek SHALL be 1 action with none of those traits. Side SHALL NOT be read. Drop Prone, Stand, Stride, and Crawl SHALL be Move. Point Out SHALL be Manipulate when the caller passes it.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2256 (Player Core pg. 138, Reactive Strike)
**Source**: https://2e.aonprd.com/Actions.aspx?ID=2295 (Player Core pg. 416, Drop Prone)
**Source**: https://2e.aonprd.com/Actions.aspx?ID=2297 (Player Core pg. 416, Interact)
**Source**: https://2e.aonprd.com/Actions.aspx?ID=2300 (Player Core pg. 417, Release)
**Source**: https://2e.aonprd.com/Actions.aspx?ID=2301 (Player Core pg. 417, Seek)
**Source**: https://2e.aonprd.com/Actions.aspx?ID=2303 (Player Core pg. 418, Stand)
**Source**: https://2e.aonprd.com/Actions.aspx?ID=2315 (Player Core pg. 419, Point Out)

#### Scenario: A manipulate action in reach triggers

- **GIVEN** Alive Pc CombatantId(1) at (0, 0) with melee reach 5 and Alive Enemy CombatantId(2) at (-1, 0), both Medium, both current Hit Points 20, both Perception +0, no turn open, an ActionCost of 1 action with Manipulate, ReactiveStrike::Available, attack bonus +8, Armor Class 18, damage 1d8+4, and a script of 10, then 8, then 10, then 3
- **WHEN** CombatantId(1) makes the Reactive Strike
- **THEN** the degree is success, the total is 18, the penalty is 0, the damage is 7, and the disruption is NotDisrupted
- **AND** CombatantId(2) has 13 Hit Points
- **AND** CombatantId(1)'s reaction is not ready
- **AND** CombatantId(1) still has 0 actions
- **AND** no turn is open

#### Scenario: A move action in reach triggers

- **GIVEN** Alive CombatantId(1) at (0, 0) with melee reach 5 and Alive CombatantId(2) at (-1, 0), both Medium and at 20 Hit Points, no turn open, an ActionCost of 1 action with Move, ReactiveStrike::Available, attack bonus +8, Armor Class 18, damage 1d8+4, and a script of 10, then 8, then 10, then 3
- **WHEN** CombatantId(1) makes the Reactive Strike
- **THEN** the degree is success, the damage is 7, and the disruption is NotDisrupted
- **AND** the reaction is not ready

#### Scenario: A ranged attack in reach triggers

- **GIVEN** Alive CombatantId(1) at (0, 0) with melee reach 5 and Alive CombatantId(2) at (-1, 0), both Medium and at 20 Hit Points, no turn open, an ActionCost of 1 action with Ranged, ReactiveStrike::Available, attack bonus +8, Armor Class 18, damage 1d8+4, and a script of 10, then 8, then 10, then 3
- **WHEN** CombatantId(1) makes the Reactive Strike
- **THEN** the degree is success, the damage is 7, and the disruption is NotDisrupted
- **AND** the reaction is not ready

#### Scenario: The same side still triggers

- **GIVEN** Alive Pc CombatantId(1) at (0, 0) with melee reach 5 and Alive Pc CombatantId(2) at (-1, 0), both Medium, both current Hit Points 20, both Perception +0, no turn open, an ActionCost of 1 action with Manipulate, ReactiveStrike::Available, attack bonus +8, Armor Class 18, damage 1d8+4, and a script of 10, then 8, then 10, then 3
- **WHEN** CombatantId(1) makes the Reactive Strike
- **THEN** the degree is success, the damage is 7, and the disruption is NotDisrupted
- **AND** CombatantId(2) has 13 Hit Points
- **AND** the reaction is not ready

#### Scenario: Move and manipulate on one cost trigger once

- **GIVEN** Alive CombatantId(1) at (0, 0) with melee reach 5 and Alive CombatantId(2) at (0, 1) with 20 Hit Points, both Medium, no turn open, an ActionCost of 1 action with both Move and Manipulate, ReactiveStrike::Available, attack bonus +8, Armor Class 18, damage 1d8+4, and a script of 10, then 8, then 10, then 3
- **WHEN** CombatantId(1) makes the Reactive Strike
- **THEN** there is one outcome, the damage is 7, and the disruption is NotDisrupted
- **AND** CombatantId(2) has 13 Hit Points
- **AND** the reaction is not ready
- **AND** CombatantId(1) still has 0 actions

#### Scenario: A free manipulate action other than Release triggers

- **GIVEN** Alive CombatantId(1) at (0, 0) with melee reach 5 and Alive CombatantId(2) at (0, 1) with 20 Hit Points, both Medium, ActionIdentity::Other, ActionCost::Free with Manipulate, ReactiveStrike::Available, attack bonus +8, Armor Class 18, damage 1d8+4, and a script of 10, then 8, then 10, then 3
- **WHEN** CombatantId(1) makes the Reactive Strike
- **THEN** the degree is success, the damage is 7, and the disruption is NotDisrupted
- **AND** the reaction is not ready
- **AND** the error is not NotATrigger

### Requirement: Step and a cost with no triggering trait do not trigger

MoveAction::Step SHALL return NotATrigger even though its cost includes Move. ActionIdentity::Release SHALL return NotATrigger even though its free cost includes Manipulate. Only that identity is exempt. A cost with none of Move, Manipulate, or Ranged, including Seek and a melee Strike, SHALL return NotATrigger. That check SHALL win over size and reach. The reaction SHALL stay as it was, and no attack die SHALL be drawn.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2304 (Player Core pg. 418, Step)
**Source**: https://2e.aonprd.com/Actions.aspx?ID=2300 (Player Core pg. 417, Release)
**Source**: https://2e.aonprd.com/Actions.aspx?ID=2306 (Player Core pg. 418, Strike)
**Source**: https://2e.aonprd.com/Actions.aspx?ID=2301 (Player Core pg. 417, Seek)

#### Scenario: Step does not trigger

- **GIVEN** Alive CombatantId(1) of size Large at (0, 0) with melee reach 7, Alive CombatantId(2) of size Medium at (0, 1), MoveAction::Step whose cost is 1 action with Move, ReactiveStrike::Available, a ready reaction, and a scripted natural 10 after initiative
- **WHEN** CombatantId(1) attempts the Reactive Strike
- **THEN** the error is NotATrigger
- **AND** the error is not UnsupportedSize and not InvalidReach
- **AND** the reaction is still ready
- **AND** the scripted 10 is still unused

#### Scenario: A melee Strike does not trigger

- **GIVEN** Alive CombatantId(1) at (0, 0) with melee reach 5 and Alive CombatantId(2) at (0, 1), both Medium, an ActionCost of 1 action with none of Move, Manipulate, or Ranged, ReactiveStrike::Available, a ready reaction, and a scripted natural 10 after initiative
- **WHEN** CombatantId(1) attempts the Reactive Strike
- **THEN** the error is NotATrigger
- **AND** the reaction is still ready
- **AND** the scripted 10 is still unused

#### Scenario: Release does not trigger

- **GIVEN** Alive CombatantId(1) at (0, 0) with melee reach 5 and Alive CombatantId(2) at (0, 1), both Medium, ActionIdentity::Release, ActionCost::Free with Manipulate, ReactiveStrike::Available, a ready reaction, and a scripted natural 10 after initiative
- **WHEN** CombatantId(1) attempts the Reactive Strike
- **THEN** the error is NotATrigger
- **AND** the reaction is still ready
- **AND** the scripted 10 is still unused

#### Scenario: Seek does not trigger

- **GIVEN** Alive CombatantId(1) at (0, 0) with melee reach 5 and Alive CombatantId(2) at (0, 1), both Medium, Seek as 1 action with none of Move, Manipulate, or Ranged, ReactiveStrike::Available, a ready reaction, and a scripted natural 10 after initiative
- **WHEN** CombatantId(1) attempts the Reactive Strike
- **THEN** the error is NotATrigger
- **AND** the reaction is still ready
- **AND** the scripted 10 is still unused

### Requirement: Reach is the stored melee reach

The reach SHALL be the reactor's stored melee ReachFeet. Checks SHALL run UnsupportedSize, then Overflow, then InvalidReach, then OutOfReach. A reach that is not a multiple of 5 SHALL be InvalidReach. Reach 0 SHALL be legal and SHALL NOT threaten. The same square SHALL NOT be in reach. Any size but Small or Medium SHALL be UnsupportedSize. A distance that does not fit SHALL be Overflow, including when the reach is not a multiple of 5.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2379 (Player Core pg. 426, Range and Reach)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2359 (Player Core pg. 421, Size, Space, and Reach)

#### Scenario: Out of reach does not trigger

- **GIVEN** Alive CombatantId(1) at (0, 0) with melee reach 5 and Alive CombatantId(2) at (0, 2), both Medium, an ActionCost of 1 action with Manipulate, ReactiveStrike::Available, a ready reaction, and a scripted natural 10 after initiative
- **WHEN** CombatantId(1) attempts the Reactive Strike
- **THEN** the error is OutOfReach
- **AND** the reaction is still ready
- **AND** the scripted 10 is still unused

#### Scenario: Reach 0 does not trigger

- **GIVEN** Alive CombatantId(1) at (0, 0) with melee reach 0 and Alive CombatantId(2) at (0, 1), both Medium, an ActionCost of 1 action with Manipulate, ReactiveStrike::Available, a ready reaction, and a scripted natural 10 after initiative
- **WHEN** CombatantId(1) attempts the Reactive Strike
- **THEN** the error is OutOfReach
- **AND** the error is not InvalidReach
- **AND** the reaction is still ready
- **AND** the scripted 10 is still unused

#### Scenario: The same square is not in reach

- **GIVEN** Alive CombatantId(1) and Alive CombatantId(2) both at (0, 0), CombatantId(1) at melee reach 5, both Medium, an ActionCost of 1 action with Manipulate, ReactiveStrike::Available, a ready reaction, and a scripted natural 10 after initiative
- **WHEN** CombatantId(1) attempts the Reactive Strike
- **THEN** the error is OutOfReach
- **AND** the reaction is still ready
- **AND** the scripted 10 is still unused

#### Scenario: A reactive reach that is not a multiple of 5 is rejected

- **GIVEN** Alive CombatantId(1) at (0, 0) with melee reach 7 and Alive CombatantId(2) at (0, 1), both Medium, an ActionCost of 1 action with Manipulate, ReactiveStrike::Available, a ready reaction, and a scripted natural 10 after initiative
- **WHEN** CombatantId(1) attempts the Reactive Strike
- **THEN** the error is InvalidReach
- **AND** the error is not OutOfReach
- **AND** the reaction is still ready
- **AND** the scripted 10 is still unused

#### Scenario: A large creature is unsupported

- **GIVEN** Alive Large CombatantId(1) at (0, 0) with melee reach 7 and Alive Medium CombatantId(2) at (0, 1), an ActionCost of 1 action with Manipulate, ReactiveStrike::Available, a ready reaction, and a scripted natural 10 after initiative
- **WHEN** CombatantId(1) attempts the Reactive Strike
- **THEN** the error is UnsupportedSize
- **AND** the error is not InvalidReach
- **AND** the reaction is still ready
- **AND** the scripted 10 is still unused

#### Scenario: A distance that does not fit is overflow

- **GIVEN** Alive CombatantId(1) at (0, 0) with melee reach 7 and Alive CombatantId(2) at (i32::MIN, 0), both Medium, an ActionCost of 1 action with Manipulate, ReactiveStrike::Available, a ready reaction, and a scripted natural 10 after initiative
- **WHEN** CombatantId(1) attempts the Reactive Strike
- **THEN** the error is Overflow
- **AND** the error is not InvalidReach
- **AND** the reaction is still ready
- **AND** the scripted 10 is still unused

#### Scenario: A 10-ft reach diagonal is in reach

- **GIVEN** Alive CombatantId(1) at (0, 0) with melee reach 10 and Alive CombatantId(2) at (2, 2), both Medium and at 20 Hit Points, an ActionCost of 1 action with Manipulate, ReactiveStrike::Available, attack bonus +8, Armor Class 18, damage 1d8+4, and a script of 10, then 8, then 10, then 3
- **WHEN** CombatantId(1) makes the Reactive Strike
- **THEN** the degree is success and the damage is 7
- **AND** the reaction is not ready

#### Scenario: A 10-ft reach does not reach three squares straight

- **GIVEN** Alive CombatantId(1) at (0, 0) with melee reach 10 and Alive CombatantId(2) at (3, 0), both Medium, an ActionCost of 1 action with Manipulate, ReactiveStrike::Available, a ready reaction, and a scripted natural 10 after initiative
- **WHEN** CombatantId(1) attempts the Reactive Strike
- **THEN** the error is OutOfReach
- **AND** the reaction is still ready
- **AND** the scripted 10 is still unused

### Requirement: A move that starts outside reach does not trigger

Only the actor's square at the moment of the action SHALL be tested. No path SHALL be read. A Move whose square is outside reach SHALL be OutOfReach (proposed DEV-45).

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2256 (Player Core pg. 138, Reactive Strike)

#### Scenario: A move that starts outside reach does not trigger

- **GIVEN** Alive CombatantId(1) at (0, 0) with melee reach 5 and Alive CombatantId(2) at (0, 2), both Medium, an ActionCost of 1 action with Move, ReactiveStrike::Available, a ready reaction, no path argument, and a scripted natural 10 after initiative
- **WHEN** CombatantId(1) attempts the Reactive Strike
- **THEN** the error is OutOfReach
- **AND** the reaction is still ready
- **AND** the scripted 10 is still unused

### Requirement: The reaction is spent through encounter turns

A resolved Strike SHALL spend the reactor's reaction only through spend_reaction. A second call SHALL return NoReaction. A reactor whose reaction spend_reaction has already cleared, including for Aid, SHALL return NoReaction and SHALL draw no die. The spend SHALL NOT change either creature's actions and SHALL NOT end the turn. It SHALL be legal before the first turn and on the reactor's own turn.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2432 (Player Core pg. 436, Reactions in Encounters)

#### Scenario: A second reactive strike is NoReaction

- **GIVEN** a successful manipulate Reactive Strike that consumed a natural 10 and a damage die of 3, and a further scripted natural 4
- **WHEN** CombatantId(1) attempts that Reactive Strike again
- **THEN** the error is NoReaction
- **AND** the scripted 4 is still unused
- **AND** CombatantId(2) still has 13 Hit Points

#### Scenario: A reaction already spent is NoReaction

- **GIVEN** Alive CombatantId(1) at (0, 0) with melee reach 5 and Alive CombatantId(2) at (0, 1), both Medium, and CombatantId(1)'s reaction already spent through Encounter::spend_reaction, as Aid would, an ActionCost of 1 action with Manipulate, ActionIdentity::Other, ReactiveStrike::Available, and a scripted natural 10 after initiative
- **WHEN** CombatantId(1) attempts the Reactive Strike
- **THEN** the error is NoReaction
- **AND** the scripted 10 is still unused
- **AND** the reaction is still not ready

#### Scenario: Actor actions stay unchanged

- **GIVEN** CombatantId(1) with Perception +0 and a scripted initiative natural 8, CombatantId(2) with Perception +0 and a scripted initiative natural 10, CombatantId(2)'s turn open with 3 actions, both Medium and Alive, CombatantId(1) at (0, 0) with reach 5 and a ready reaction, CombatantId(2) at (0, 1), an ActionCost of 1 action with Manipulate, ReactiveStrike::Available, attack bonus +8, Armor Class 18, damage 1d8+4, and a scripted natural 10 then a damage die of 3
- **WHEN** CombatantId(1) makes the Reactive Strike
- **THEN** the spend succeeds
- **AND** CombatantId(2) still has 3 actions
- **AND** the open turn is still CombatantId(2)
- **AND** CombatantId(1) still has 0 actions
- **AND** CombatantId(1)'s reaction is not ready

#### Scenario: A reactive strike on your turn spends no action

- **GIVEN** Alive CombatantId(1) at (0, 0) with melee reach 5, current Hit Points 20, Perception +0, a ready reaction, and an open turn with 3 actions, Alive CombatantId(2) at (0, 1) with current Hit Points 20 and Perception +0, both Medium, an ActionCost of 1 action with Manipulate, ReactiveStrike::Available, attack bonus +8, Armor Class 18, damage 1d8+4, and a script of initiative 10, then 8, then natural 10, then damage 3
- **WHEN** CombatantId(1) makes the Reactive Strike
- **THEN** CombatantId(1) still has 3 actions
- **AND** the open turn is still CombatantId(1)
- **AND** CombatantId(1)'s reaction is not ready
- **AND** the penalty is 0 and the damage is 7
- **AND** CombatantId(2) has 13 Hit Points

### Requirement: The penalty stays 0 and the count stays

This Strike SHALL use penalty 0 whether or not it is agile, on the reactor's turn and off it. It SHALL NOT change the stored attack count. The reactor's next own-turn Strike SHALL use the count from before this reaction. A caller attack penalty SHALL still apply.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2256 (Player Core pg. 138, Reactive Strike)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2289 (Player Core pg. 402, Multiple Attack Penalty)

#### Scenario: Reactive strike does not use MAP

- **GIVEN** CombatantId(1)'s turn is open, two non-agile Strikes have already resolved against Armor Class 18 with attack bonus +8 and naturals 2 then 2, the stored count is 2, 1 action remains, and the Reactive Strike is Agility::Agile with attack bonus +8, a caller circumstance penalty of -2 on Attack, natural 18, and damage 1d8+4 showing 3
- **WHEN** CombatantId(1) makes that Reactive Strike against CombatantId(2) in reach with an ActionCost of 1 action with Manipulate and ReactiveStrike::Available
- **THEN** the penalty is 0
- **AND** the total is 24
- **AND** the total is not 16 and not 14
- **AND** the degree is success and the damage is 7
- **AND** the stored count is still 2
- **AND** 1 action remains

#### Scenario: Reactive strike does not count toward MAP

- **GIVEN** CombatantId(1)'s turn is open, one non-agile Strike has resolved with natural 2, attack bonus +8, and Armor Class 18, the stored count is 1, and 2 actions remain
- **WHEN** CombatantId(1) makes a Reactive Strike with natural 10, damage 1d8+4 showing 3, an ActionCost of 1 action with Manipulate, and ReactiveStrike::Available, and then Strikes with natural 12
- **THEN** the Reactive Strike penalty is 0 and its damage is 7
- **AND** the stored count after the Reactive Strike is still 1
- **AND** the following Strike has penalty -5 and total 15
- **AND** that penalty is not -10 and that total is not 10

### Requirement: A critical hit disrupts a manipulate action

A CriticalSuccess on a cost that includes Manipulate SHALL return Disruption::Disrupted, including when that cost also includes Move. Every other degree SHALL return NotDisrupted, as SHALL a critical hit on a move-only cost or on Ranged. On the actor's open turn the action's cost SHALL stay spent. The caller SHALL void the action's effects. This function SHALL NOT void them, SHALL NOT change the actor's action count, and SHALL NOT refund the action.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2256 (Player Core pg. 138, Reactive Strike)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2342 (Player Core pg. 415, Disrupting Actions)

#### Scenario: A critical hit disrupts a manipulate action

- **GIVEN** Alive CombatantId(2)'s turn is open with 2 actions remaining, Alive CombatantId(2) at (0, 1) with 20 Hit Points, Alive CombatantId(1) at (0, 0) with melee reach 5, a ready reaction, and 0 actions, both Medium, attack bonus +10, Armor Class 15, an ActionCost of 1 action with Manipulate, ReactiveStrike::Available, damage 1d8+4, and a script of initiative 8, then 10, then natural 15, then damage 3
- **WHEN** CombatantId(1) makes the Reactive Strike
- **THEN** the total is 25 and the degree is critical success
- **AND** the damage is 14
- **AND** the disruption is Disrupted, and the caller voids the manipulate action's effects
- **AND** the cost stays spent
- **AND** CombatantId(2) has 6 Hit Points and still has 2 actions
- **AND** the open turn is still CombatantId(2)
- **AND** the reaction is not ready

#### Scenario: A critical hit does not disrupt a move

- **GIVEN** no turn is open, Alive CombatantId(1) at (0, 0) with reach 5 and Alive CombatantId(2) at (0, 1) with 20 Hit Points, both Medium, attack bonus +10, Armor Class 15, an ActionCost of 1 action with Move, ReactiveStrike::Available, damage 1d8+4, and a script of 10, then 8, then natural 15, then damage 3
- **WHEN** the Reactive Strike is resolved
- **THEN** the degree is critical success and the damage is 14
- **AND** the disruption is NotDisrupted
- **AND** the reaction is not ready

#### Scenario: A critical hit does not disrupt a ranged attack

- **GIVEN** no turn is open, Alive CombatantId(1) at (0, 0) with reach 5 and Alive CombatantId(2) at (0, 1) with 20 Hit Points, both Medium, attack bonus +10, Armor Class 15, an ActionCost of 1 action with Ranged, ReactiveStrike::Available, damage 1d8+4, and a script of 10, then 8, then natural 15, then damage 3
- **WHEN** the Reactive Strike is resolved
- **THEN** the degree is critical success and the damage is 14
- **AND** the disruption is NotDisrupted

#### Scenario: A natural 20 that is only a success does not disrupt

- **GIVEN** Alive CombatantId(1) at (0, 0) with melee reach 5 and Alive CombatantId(2) at (0, 1) with 20 Hit Points, both Medium, attack bonus +0, Armor Class 25, an ActionCost of 1 action with Manipulate, ReactiveStrike::Available, damage 1d8+4, and a script of initiative 10, then 8, then natural 20, then damage 3
- **WHEN** the Reactive Strike is resolved
- **THEN** the total is 20 and the degree is success
- **AND** the damage is 7
- **AND** the damage is not 14
- **AND** the disruption is NotDisrupted
- **AND** CombatantId(2) has 13 Hit Points

#### Scenario: Actor actions are unchanged on Disrupted

- **GIVEN** Alive CombatantId(2)'s turn is open with 1 action remaining, Alive CombatantId(2) at (0, 1) with 20 Hit Points, Alive CombatantId(1) at (0, 0) with melee reach 5, a ready reaction, and 0 actions, both Medium, attack bonus +10, Armor Class 15, an ActionCost of 1 action with Manipulate, ReactiveStrike::Available, damage 1d8+4, and a script of initiative 8, then 10, then natural 15, then damage 3
- **WHEN** CombatantId(1) makes the Reactive Strike
- **THEN** the disruption is Disrupted
- **AND** CombatantId(2) still has 1 action
- **AND** that count is not 2 and not 0
- **AND** the open turn is still CombatantId(2)

### Requirement: A reactor that is not Alive cannot react

A reactor whose vital is not Alive SHALL return NotConscious. The reaction SHALL stay as it was, and no attack die SHALL be drawn. Dying, Unconscious, and Dead SHALL each do this. NotConscious SHALL win over Unavailable and over SelfTarget.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2429 (Player Core pg. 436, Step 2: Act)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=95 (Player Core pg. 446, Unconscious)

#### Scenario: A dying creature cannot react

- **GIVEN** CombatantId(1) is Dying(1) at 0 Hit Points with the reaction ready, at (0, 0) with melee reach 5, Alive CombatantId(2) at (0, 1), both Medium, an ActionCost of 1 action with Manipulate, ReactiveStrike::Available, and a scripted natural 10 after initiative
- **WHEN** CombatantId(1) attempts the Reactive Strike
- **THEN** the error is NotConscious
- **AND** the reaction is still ready
- **AND** the scripted 10 is still unused

#### Scenario: An unconscious creature cannot react

- **GIVEN** CombatantId(1) is Unconscious at 0 Hit Points with the reaction ready, at (0, 0) with melee reach 5, Alive CombatantId(2) at (0, 1), both Medium, an ActionCost of 1 action with Manipulate, ReactiveStrike::Available, and a scripted natural 10 after initiative
- **WHEN** CombatantId(1) attempts the Reactive Strike
- **THEN** the error is NotConscious
- **AND** the reaction is still ready
- **AND** the scripted 10 is still unused

#### Scenario: A dead creature cannot react

- **GIVEN** CombatantId(1) began Alive, so the reaction is ready, and is then Dead at (0, 0) with melee reach 5 with that reaction still ready, Alive CombatantId(2) at (0, 1), both Medium, an ActionCost of 1 action with Manipulate, ReactiveStrike::Available, and a scripted natural 10 after initiative
- **WHEN** CombatantId(1) attempts the Reactive Strike
- **THEN** the error is NotConscious
- **AND** the error is not NoReaction
- **AND** the reaction is still ready
- **AND** the scripted 10 is still unused

#### Scenario: Not conscious wins over unavailable

- **GIVEN** CombatantId(1) is Dying(1) at 0 Hit Points with the reaction ready, at (0, 0) with melee reach 5, and the call passes ReactiveStrike::Unavailable and an ActionCost of 1 action with Manipulate against Alive CombatantId(2) at (0, 1), both Medium
- **WHEN** CombatantId(1) attempts the Reactive Strike
- **THEN** the error is NotConscious
- **AND** the error is not Unavailable
- **AND** the reaction is still ready
- **AND** no attack die is drawn

#### Scenario: A dying self target is not conscious

- **GIVEN** CombatantId(1) is Dying(1) at 0 Hit Points with the reaction ready, at (0, 0) with melee reach 5, size Medium, and the actor is CombatantId(1)
- **WHEN** that Reactive Strike is attempted with ReactiveStrike::Available and an ActionCost of 1 action with Manipulate
- **THEN** the error is NotConscious
- **AND** the error is not SelfTarget
- **AND** the reaction is still ready
- **AND** no attack die is drawn

### Requirement: Unavailable spends nothing

ReactiveStrike::Unavailable SHALL return Unavailable for an Alive reactor and SHALL spend nothing (proposed DEV-44). Unavailable SHALL win over NotATrigger and over InvalidReach.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2256 (Player Core pg. 138, Reactive Strike)

#### Scenario: Unavailable spends nothing

- **GIVEN** an Alive CombatantId(1) at (0, 0) with a ready reaction, melee reach 7, and size Large, MoveAction::Step, ReactiveStrike::Unavailable, Alive Medium CombatantId(2) at (0, 1), and a scripted natural 10 after initiative
- **WHEN** CombatantId(1) attempts the Reactive Strike
- **THEN** the error is Unavailable
- **AND** the error is not NotATrigger and not InvalidReach
- **AND** the reaction is still ready
- **AND** the scripted 10 is still unused

### Requirement: Damage follows apply_resolved_blow

A hit SHALL apply damage through apply_resolved_blow. Temporary Hit Points SHALL be removed first. A miss SHALL spend the reaction, deal 0, and draw no damage die. This call SHALL NOT call note_hit_points.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2306 (Player Core pg. 418, Strike)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2321 (Player Core pg. 410, Temporary Hit Points)

#### Scenario: Temporary Hit Points are spent first

- **GIVEN** Alive CombatantId(1) at (0, 0) with melee reach 5, Alive CombatantId(2) at (0, 1) with 20 current Hit Points, 5 temporary Hit Points, and wounded 0, both Medium, attack bonus +8, Armor Class 18, an ActionCost of 1 action with Manipulate, ReactiveStrike::Available, damage 1d8+4, and a script of initiative 10, then 8, then natural 10, then damage 3
- **WHEN** the Reactive Strike is resolved
- **THEN** the damage is 7
- **AND** temporary Hit Points are 0
- **AND** current Hit Points are 18
- **AND** wounded stays 0
- **AND** vital stays Alive

#### Scenario: A failure spends the reaction and deals no damage

- **GIVEN** Alive CombatantId(1) at (0, 0) with melee reach 5, Alive CombatantId(2) at (0, 1) with 20 Hit Points, both Medium, attack bonus +0, Armor Class 18, an ActionCost of 1 action with Manipulate, ReactiveStrike::Available, a script of initiative 10, then 8, then natural 9, and a scripted damage die of 8
- **WHEN** the Reactive Strike is resolved
- **THEN** the total is 9 and the degree is failure
- **AND** the damage is 0 and the disruption is NotDisrupted
- **AND** the scripted 8 is still unused
- **AND** Hit Points stay 20
- **AND** the reaction is not ready
- **AND** the penalty is 0

#### Scenario: A critical failure spends the reaction and deals no damage

- **GIVEN** Alive CombatantId(1) at (0, 0) with melee reach 5, Alive CombatantId(2) at (0, 1) with 20 Hit Points, both Medium, attack bonus +0, Armor Class 18, an ActionCost of 1 action with Manipulate, ReactiveStrike::Available, a script of initiative 10, then 8, then natural 5, and a scripted damage die of 8
- **WHEN** the Reactive Strike is resolved
- **THEN** the degree is critical failure
- **AND** the damage is 0 and the disruption is NotDisrupted
- **AND** the scripted 8 is still unused
- **AND** Hit Points stay 20
- **AND** the reaction is not ready

### Requirement: A critical hit can drop the target to Dying 2

A critical hit that drops a uses_dying_rules creature from above 0 Hit Points to 0 SHALL be apply_resolved_blow's critical blow. With wounded 0 that vital SHALL be Dying(2), current Hit Points SHALL be 0, and the pending initiative report SHALL be recorded. The reaction SHALL be spent. This change SHALL NOT define a second dying formula.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2324 (Player Core pg. 410, Getting Knocked Out)

#### Scenario: A critical hit drops a dying-rules target to Dying 2

- **GIVEN** uses_dying_rules, Alive CombatantId(2) at 5 current Hit Points and 20 maximum with wounded 0, Alive CombatantId(1) at (0, 0) with melee reach 5 and a ready reaction, CombatantId(2) at (0, 1), both Medium, attack bonus +10, Armor Class 15, an ActionCost of 1 action with Manipulate, ReactiveStrike::Available, damage 1d8+4, and a script of initiative 10, then 8, then natural 15, then damage 3
- **WHEN** CombatantId(1) makes the Reactive Strike and the report is taken
- **THEN** the damage is 14
- **AND** current Hit Points are 0
- **AND** vital is Dying(2)
- **AND** wounded is 0
- **AND** the take is Some(InitiativeMove::Pending)
- **AND** the reaction is not ready

### Requirement: A reactive strike does not use Shield Block

This Strike SHALL call apply_resolved_blow and SHALL NOT take a Shield Block argument. A raised shield SHALL keep its Hit Points. The target's reaction SHALL stay ready. Shield Block is `wire-shield-block-into-reactive-strike`.

**Source**: https://2e.aonprd.com/Feats.aspx?ID=5212 (Player Core pg. 262, Shield Block)

#### Scenario: A reactive strike does not reduce a raised shield

- **GIVEN** Alive CombatantId(2) at 20 Hit Points with a raised steel shield at 20 Hit Points and hardness 5, and a ready reaction, Alive CombatantId(1) at (0, 0) with melee reach 5 and a ready reaction, CombatantId(2) at (0, 1), both Medium, an ActionCost of 1 action with Manipulate, ReactiveStrike::Available, attack bonus +8, Armor Class 18, damage 1d8+4, and a script of 10, then 8, then natural 10, then damage 3
- **WHEN** CombatantId(1) makes the Reactive Strike
- **THEN** the damage is 7
- **AND** CombatantId(2) has 13 Hit Points
- **AND** the shield has 20 Hit Points
- **AND** CombatantId(2)'s reaction is still ready

### Requirement: An actor that is not Alive is TargetNotConscious

An actor whose vital is not Alive SHALL return TargetNotConscious. Dying, Unconscious, and Dead SHALL each do this. That check SHALL sit where TargetDead sat, and TargetDead SHALL NOT be a variant. It SHALL win over NoReaction. The reaction SHALL stay as it was, and no attack die SHALL be drawn.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2256 (Player Core pg. 138, Reactive Strike)

#### Scenario: An unconscious target returns TargetNotConscious

- **GIVEN** Alive CombatantId(1) at (0, 0) with melee reach 5 and a ready reaction, Unconscious CombatantId(2) at (0, 1) with 0 Hit Points, both Medium, an ActionCost of 1 action with Manipulate, ReactiveStrike::Available, and a scripted natural 10 after initiative
- **WHEN** CombatantId(1) attempts the Reactive Strike
- **THEN** the error is TargetNotConscious
- **AND** the reaction is still ready
- **AND** the scripted 10 is still unused

#### Scenario: A dying target in reach is TargetNotConscious

- **GIVEN** Alive CombatantId(1) at (0, 0) with melee reach 5 and a ready reaction, Dying(1) CombatantId(2) at (0, 1) with 0 Hit Points, both Medium, an ActionCost of 1 action with Manipulate, ReactiveStrike::Available, and a scripted natural 10 after initiative
- **WHEN** CombatantId(1) attempts the Reactive Strike
- **THEN** the error is TargetNotConscious
- **AND** the reaction is still ready
- **AND** the scripted 10 is still unused

#### Scenario: A dead target in reach is TargetNotConscious

- **GIVEN** Alive CombatantId(1) at (0, 0) with reach 5 and a reaction that is not ready, Dead CombatantId(2) at (0, 1), both Medium, an ActionCost of 1 action with Manipulate, ReactiveStrike::Available, and a scripted natural 10 after initiative
- **WHEN** CombatantId(1) attempts the Reactive Strike
- **THEN** the error is TargetNotConscious
- **AND** the error is not NoReaction
- **AND** the reaction is still not ready
- **AND** the scripted 10 is still unused

### Requirement: Errors have one order

Checks SHALL run in this order: UnknownCombatant for the reactor, then the actor, NotConscious, SelfTarget, Unavailable, NotATrigger, UnsupportedSize, Overflow, InvalidReach, OutOfReach, TargetNotConscious, then NoReaction. A failure before the d20 SHALL draw no die and leave the reaction unchanged. DamageOverflow is the exception and SHALL consume both dice, leaving the reaction ready and Hit Points unchanged. AttackOverflow SHALL reject bonus + modifiers + 20 over i32 before the d20.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2256 (Player Core pg. 138, Reactive Strike)

#### Scenario: A reactive strike on oneself rolls nothing

- **GIVEN** Alive CombatantId(1) at (0, 0) with melee reach 5, size Medium, a ready reaction, and a scripted natural 10 after initiative
- **WHEN** CombatantId(1) attempts a Reactive Strike against CombatantId(1) with an ActionCost of 1 action with Manipulate and ReactiveStrike::Available
- **THEN** the error is SelfTarget
- **AND** the reaction is still ready
- **AND** the scripted 10 is still unused

#### Scenario: An unknown reactor rolls nothing

- **GIVEN** an encounter of CombatantId(1) and CombatantId(2), and a scripted natural 10 after initiative
- **WHEN** CombatantId(9) attempts a Reactive Strike against CombatantId(2)
- **THEN** the error is UnknownCombatant
- **AND** the scripted 10 is still unused

#### Scenario: An unknown actor rolls nothing

- **GIVEN** an encounter of CombatantId(1) and CombatantId(2), CombatantId(1) Alive with a ready reaction, and a scripted natural 10 after initiative
- **WHEN** CombatantId(1) attempts a Reactive Strike against CombatantId(9)
- **THEN** the error is UnknownCombatant
- **AND** the reaction is still ready
- **AND** the scripted 10 is still unused

#### Scenario: A dead actor out of reach is out of reach

- **GIVEN** Alive CombatantId(1) at (0, 0) with reach 5 and a ready reaction, Dead CombatantId(2) at (0, 2), both Medium, an ActionCost of 1 action with Manipulate, ReactiveStrike::Available, and a scripted natural 10 after initiative
- **WHEN** CombatantId(1) attempts the Reactive Strike
- **THEN** the error is OutOfReach
- **AND** the error is not TargetNotConscious
- **AND** the reaction is still ready
- **AND** the scripted 10 is still unused

#### Scenario: An attack bonus that does not fit spends nothing

- **GIVEN** Alive CombatantId(1) at (0, 0) with melee reach 5 and a ready reaction, Alive CombatantId(2) at (0, 1), both Medium, an ActionCost of 1 action with Manipulate, ReactiveStrike::Available, attack bonus 2147483647, an empty caller modifier list, and a scripted natural 10 after initiative
- **WHEN** the Reactive Strike is attempted
- **THEN** the error is AttackOverflow
- **AND** the reaction is still ready
- **AND** the scripted 10 is still unused

#### Scenario: An attack bonus of 2147483627 and a natural 20 fit

- **GIVEN** Alive CombatantId(1) at (0, 0) with melee reach 5 and a ready reaction, Alive CombatantId(2) at (0, 1) with 20 Hit Points, both Medium, an ActionCost of 1 action with Manipulate, ReactiveStrike::Available, attack bonus 2147483627, an empty caller modifier list, Armor Class 18, 0 damage dice with bonus 0, and a script of initiative 10, then 8, then natural 20
- **WHEN** the Reactive Strike is resolved
- **THEN** the result is Ok and the total is 2147483647
- **AND** the degree is critical success
- **AND** the reaction is not ready

#### Scenario: A damage total that does not fit spends nothing

- **GIVEN** Alive CombatantId(1) at (0, 0) with melee reach 5 and a ready reaction, Alive CombatantId(2) at (0, 1) with current Hit Points 20, both Medium, attack bonus +8, Armor Class 18, an ActionCost of 1 action with Manipulate, ReactiveStrike::Available, damage 1d8 with bonus 2147483647, and a script of initiative 10, then 8, then natural 10, then damage 1, then 4
- **WHEN** the Reactive Strike is resolved
- **THEN** the error is DamageOverflow
- **AND** the natural 10 and the damage die of 1 were consumed
- **AND** the scripted 4 is still unused
- **AND** the reaction is still ready
- **AND** Hit Points stay 20
