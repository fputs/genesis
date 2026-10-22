# reactive-strike

## Purpose

Resolves one Reactive Strike: a melee reaction when a creature inside the reactor's reach uses a manipulate or move action, or makes a ranged attack.

## ADDED Requirements

### Requirement: The same reactive strike twice matches

The same inputs SHALL return the same outcome. A call SHALL NOT read a clock, a file, or a global generator. A fallible call SHALL return a typed error and SHALL NOT panic. The rules crate SHALL gain no dependency.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2256 (Player Core pg. 138, Reactive Strike)

#### Scenario: The same reactive strike twice matches

- **GIVEN** two encounters, each with Alive CombatantId(1) and Alive CombatantId(2), both Perception +0, both current Hit Points 20, CombatantId(1) at melee reach 5, and each scripted 10, then 8, then 10, then 3
- **WHEN** each encounter resolves one Reactive Strike from CombatantId(1) at (0, 0) against CombatantId(2) at (-1, 0), both Medium, Activity::Manipulate, and ReactiveStrike::Available, with attack bonus +8, Armor Class 18, and damage 1d8+4
- **THEN** both outcomes are equal
- **AND** both naturals are 10, both totals are 18, both degrees are success, both penalties are 0, both damages are 7, and both disruptions are NotDisrupted

### Requirement: A manipulate, move, or ranged attack in reach triggers

Manipulate, Move, and RangedAttack SHALL trigger inside the reactor's stored melee reach. Drop Prone, Stand, Stride, and Crawl SHALL be Move. Point Out SHALL be Manipulate when the caller passes it. Side SHALL NOT be read.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2256 (Player Core pg. 138, Reactive Strike)
**Source**: https://2e.aonprd.com/Actions.aspx?ID=2295 (Player Core pg. 416, Drop Prone)
**Source**: https://2e.aonprd.com/Actions.aspx?ID=2315 (Player Core pg. 419, Point Out)

#### Scenario: A manipulate action in reach triggers

- **GIVEN** Alive Pc CombatantId(1) at (0, 0) with melee reach 5 and Alive Enemy CombatantId(2) at (-1, 0), both Medium, both current Hit Points 20, both Perception +0, no turn open, Activity::Manipulate, ReactiveStrike::Available, attack bonus +8, Armor Class 18, damage 1d8+4, and a script of 10, then 8, then 10, then 3
- **WHEN** CombatantId(1) makes the Reactive Strike
- **THEN** the degree is success, the total is 18, the penalty is 0, the damage is 7, and the disruption is NotDisrupted
- **AND** CombatantId(2) has 13 Hit Points
- **AND** CombatantId(1)'s reaction is not ready
- **AND** CombatantId(1) still has 0 actions
- **AND** no turn is open

#### Scenario: A move action in reach triggers

- **GIVEN** Alive CombatantId(1) at (0, 0) with melee reach 5 and Alive CombatantId(2) at (-1, 0), both Medium and at 20 Hit Points, no turn open, Activity::Move, ReactiveStrike::Available, attack bonus +8, Armor Class 18, damage 1d8+4, and a script of 10, then 8, then 10, then 3
- **WHEN** CombatantId(1) makes the Reactive Strike
- **THEN** the degree is success, the damage is 7, and the disruption is NotDisrupted
- **AND** the reaction is not ready

#### Scenario: A ranged attack in reach triggers

- **GIVEN** Alive CombatantId(1) at (0, 0) with melee reach 5 and Alive CombatantId(2) at (-1, 0), both Medium and at 20 Hit Points, no turn open, Activity::RangedAttack, ReactiveStrike::Available, attack bonus +8, Armor Class 18, damage 1d8+4, and a script of 10, then 8, then 10, then 3
- **WHEN** CombatantId(1) makes the Reactive Strike
- **THEN** the degree is success, the damage is 7, and the disruption is NotDisrupted
- **AND** the reaction is not ready

### Requirement: Step and any other activity do not trigger

Activity::Step and Activity::Other SHALL return NotATrigger. That check SHALL win over size and reach. The reaction SHALL stay as it was, and no attack die SHALL be drawn.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2304 (Player Core pg. 418, Step)
**Source**: https://2e.aonprd.com/Actions.aspx?ID=2306 (Player Core pg. 418, Strike)

#### Scenario: Step does not trigger

- **GIVEN** Alive CombatantId(1) of size Large at (0, 0) with melee reach 7, Alive CombatantId(2) of size Medium at (0, 1), Activity::Step, ReactiveStrike::Available, a ready reaction, and a scripted natural 10 after initiative
- **WHEN** CombatantId(1) attempts the Reactive Strike
- **THEN** the error is NotATrigger
- **AND** the error is not UnsupportedSize and not InvalidReach
- **AND** the reaction is still ready
- **AND** the scripted 10 is still unused

#### Scenario: A melee Strike does not trigger

- **GIVEN** Alive CombatantId(1) at (0, 0) with melee reach 5 and Alive CombatantId(2) at (0, 1), both Medium, Activity::Other, ReactiveStrike::Available, a ready reaction, and a scripted natural 10 after initiative
- **WHEN** CombatantId(1) attempts the Reactive Strike
- **THEN** the error is NotATrigger
- **AND** the reaction is still ready
- **AND** the scripted 10 is still unused

### Requirement: Reach is the stored melee reach

The reach SHALL be the reactor's stored melee ReachFeet. A reach that is not a multiple of 5 SHALL be InvalidReach. Reach 0 SHALL be legal and SHALL NOT threaten. The same square SHALL NOT be in reach. Any size but Small or Medium SHALL be UnsupportedSize. A distance that does not fit SHALL be Overflow.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2379 (Player Core pg. 426, Range and Reach)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2359 (Player Core pg. 421, Size, Space, and Reach)

#### Scenario: Out of reach does not trigger

- **GIVEN** Alive CombatantId(1) at (0, 0) with melee reach 5 and Alive CombatantId(2) at (0, 2), both Medium, Activity::Manipulate, ReactiveStrike::Available, a ready reaction, and a scripted natural 10 after initiative
- **WHEN** CombatantId(1) attempts the Reactive Strike
- **THEN** the error is OutOfReach
- **AND** the reaction is still ready
- **AND** the scripted 10 is still unused

#### Scenario: Reach 0 does not trigger

- **GIVEN** Alive CombatantId(1) at (0, 0) with melee reach 0 and Alive CombatantId(2) at (0, 1), both Medium, Activity::Manipulate, ReactiveStrike::Available, a ready reaction, and a scripted natural 10 after initiative
- **WHEN** CombatantId(1) attempts the Reactive Strike
- **THEN** the error is OutOfReach
- **AND** the error is not InvalidReach
- **AND** the reaction is still ready
- **AND** the scripted 10 is still unused

#### Scenario: The same square is not in reach

- **GIVEN** Alive CombatantId(1) and Alive CombatantId(2) both at (0, 0), CombatantId(1) at melee reach 5, both Medium, Activity::Manipulate, ReactiveStrike::Available, a ready reaction, and a scripted natural 10 after initiative
- **WHEN** CombatantId(1) attempts the Reactive Strike
- **THEN** the error is OutOfReach
- **AND** the reaction is still ready
- **AND** the scripted 10 is still unused

#### Scenario: A reach that is not a multiple of 5 is rejected

- **GIVEN** Alive CombatantId(1) at (0, 0) with melee reach 7 and Alive CombatantId(2) at (0, 1), both Medium, Activity::Manipulate, ReactiveStrike::Available, a ready reaction, and a scripted natural 10 after initiative
- **WHEN** CombatantId(1) attempts the Reactive Strike
- **THEN** the error is InvalidReach
- **AND** the error is not OutOfReach
- **AND** the reaction is still ready
- **AND** the scripted 10 is still unused

#### Scenario: A large creature is unsupported

- **GIVEN** Alive Large CombatantId(1) at (0, 0) with melee reach 7 and Alive Medium CombatantId(2) at (0, 1), Activity::Manipulate, ReactiveStrike::Available, a ready reaction, and a scripted natural 10 after initiative
- **WHEN** CombatantId(1) attempts the Reactive Strike
- **THEN** the error is UnsupportedSize
- **AND** the error is not InvalidReach
- **AND** the reaction is still ready
- **AND** the scripted 10 is still unused

#### Scenario: A distance that does not fit is overflow

- **GIVEN** Alive CombatantId(1) at (0, 0) with melee reach 5 and Alive CombatantId(2) at (i32::MIN, 0), both Medium, Activity::Manipulate, ReactiveStrike::Available, a ready reaction, and a scripted natural 10 after initiative
- **WHEN** CombatantId(1) attempts the Reactive Strike
- **THEN** the error is Overflow
- **AND** the reaction is still ready
- **AND** the scripted 10 is still unused

### Requirement: A move that starts outside reach does not trigger

Only the actor's square at the moment of the action SHALL be tested. No path SHALL be read. A Move whose square is outside reach SHALL be OutOfReach (proposed DEV-45).

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2256 (Player Core pg. 138, Reactive Strike)

#### Scenario: A move that starts outside reach does not trigger

- **GIVEN** Alive CombatantId(1) at (0, 0) with melee reach 5 and Alive CombatantId(2) at (0, 2), both Medium, Activity::Move, ReactiveStrike::Available, a ready reaction, no path argument, and a scripted natural 10 after initiative
- **WHEN** CombatantId(1) attempts the Reactive Strike
- **THEN** the error is OutOfReach
- **AND** the reaction is still ready
- **AND** the scripted 10 is still unused

### Requirement: The reaction is spent through encounter turns

A resolved Strike SHALL spend the reactor's reaction only through spend_reaction. A second call SHALL return NoReaction. The spend SHALL NOT change either creature's actions and SHALL NOT end the turn. It SHALL be legal before the first turn and on the reactor's own turn.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2432 (Player Core pg. 436, Reactions in Encounters)

#### Scenario: A second reactive strike is NoReaction

- **GIVEN** a successful manipulate Reactive Strike that consumed a natural 10 and a damage die of 3, and a further scripted natural 4
- **WHEN** CombatantId(1) attempts that Reactive Strike again
- **THEN** the error is NoReaction
- **AND** the scripted 4 is still unused
- **AND** CombatantId(2) still has 13 Hit Points

#### Scenario: Actor actions stay unchanged

- **GIVEN** CombatantId(1) with Perception +0 and a scripted initiative natural 8, CombatantId(2) with Perception +0 and a scripted initiative natural 10, CombatantId(2)'s turn open with 3 actions, both Medium, CombatantId(1) at (0, 0) with reach 5 and a ready reaction, CombatantId(2) at (0, 1), Activity::Manipulate, ReactiveStrike::Available, attack bonus +8, Armor Class 18, damage 1d8+4, and a scripted natural 10 then a damage die of 3
- **WHEN** CombatantId(1) makes the Reactive Strike
- **THEN** the spend succeeds
- **AND** CombatantId(2) still has 3 actions
- **AND** the open turn is still CombatantId(2)
- **AND** CombatantId(1) still has 0 actions
- **AND** CombatantId(1)'s reaction is not ready

#### Scenario: A reactive strike on your turn spends no action

- **GIVEN** CombatantId(1)'s turn is open with 3 actions and a ready reaction, CombatantId(2) is Alive at (0, 1), CombatantId(1) is at (0, 0) with reach 5, both Medium, Activity::Manipulate, ReactiveStrike::Available, and a script of initiative 10, then 8, then natural 10, then damage 3
- **WHEN** CombatantId(1) makes the Reactive Strike
- **THEN** CombatantId(1) still has 3 actions
- **AND** the open turn is still CombatantId(1)
- **AND** CombatantId(1)'s reaction is not ready

### Requirement: The penalty stays 0 and the count stays

This Strike SHALL use penalty 0 whether or not it is agile, on the reactor's turn and off it. It SHALL NOT change the stored attack count. The reactor's next own-turn Strike SHALL use the count from before this reaction. A caller attack penalty SHALL still apply.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2256 (Player Core pg. 138, Reactive Strike)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2289 (Player Core pg. 402, Multiple Attack Penalty)

#### Scenario: Reactive strike does not use MAP

- **GIVEN** CombatantId(1)'s turn is open, two non-agile Strikes have already resolved against Armor Class 18 with attack bonus +8 and naturals 2 then 2, the stored count is 2, 1 action remains, and the Reactive Strike is Agility::Agile with attack bonus +8, a caller circumstance penalty of -2 on Attack, natural 18, and damage 1d8+4 showing 3
- **WHEN** CombatantId(1) makes that Reactive Strike against CombatantId(2) in reach with Activity::Manipulate and ReactiveStrike::Available
- **THEN** the penalty is 0
- **AND** the total is 24
- **AND** the total is not 16 and not 14
- **AND** the degree is success and the damage is 7
- **AND** the stored count is still 2
- **AND** 1 action remains

#### Scenario: Reactive strike does not count toward MAP

- **GIVEN** CombatantId(1)'s turn is open, one non-agile Strike has resolved with natural 2, attack bonus +8, and Armor Class 18, the stored count is 1, and 2 actions remain
- **WHEN** CombatantId(1) makes a Reactive Strike with natural 10, damage 1d8+4 showing 3, Activity::Manipulate, and ReactiveStrike::Available, and then Strikes with natural 12
- **THEN** the Reactive Strike penalty is 0 and its damage is 7
- **AND** the stored count after the Reactive Strike is still 1
- **AND** the following Strike has penalty -5 and total 15
- **AND** that penalty is not -10 and that total is not 10

### Requirement: A critical hit disrupts a manipulate action

A CriticalSuccess on Manipulate SHALL return Disruption::Disrupted. Every other degree SHALL return NotDisrupted, as SHALL a critical hit on Move or RangedAttack. This function SHALL NOT spend or refund the actor's action.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2256 (Player Core pg. 138, Reactive Strike)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2342 (Player Core pg. 415, Disrupting Actions)

#### Scenario: A critical hit disrupts a manipulate action

- **GIVEN** no turn is open, CombatantId(2) has 20 Hit Points and 0 actions, attack bonus +10, Armor Class 15, Activity::Manipulate, ReactiveStrike::Available, damage 1d8+4, and a script of initiative 10, then 8, then natural 15, then damage 3
- **WHEN** CombatantId(1) at reach 5 makes the Reactive Strike against the adjacent CombatantId(2)
- **THEN** the total is 25 and the degree is critical success
- **AND** the damage is 14
- **AND** the disruption is Disrupted
- **AND** CombatantId(2) has 6 Hit Points and still has 0 actions
- **AND** the reaction is not ready

#### Scenario: A critical hit does not disrupt a move

- **GIVEN** no turn is open, CombatantId(2) has 20 Hit Points, attack bonus +10, Armor Class 15, Activity::Move, ReactiveStrike::Available, damage 1d8+4, reach 5, adjacent squares, and a script of 10, then 8, then natural 15, then damage 3
- **WHEN** the Reactive Strike is resolved
- **THEN** the degree is critical success and the damage is 14
- **AND** the disruption is NotDisrupted
- **AND** the reaction is not ready

#### Scenario: A critical hit does not disrupt a ranged attack

- **GIVEN** no turn is open, CombatantId(2) has 20 Hit Points, attack bonus +10, Armor Class 15, Activity::RangedAttack, ReactiveStrike::Available, damage 1d8+4, reach 5, adjacent squares, and a script of 10, then 8, then natural 15, then damage 3
- **WHEN** the Reactive Strike is resolved
- **THEN** the degree is critical success and the damage is 14
- **AND** the disruption is NotDisrupted

#### Scenario: A natural 20 that is only a success does not disrupt

- **GIVEN** attack bonus +0, Armor Class 25, Activity::Manipulate, ReactiveStrike::Available, damage 1d8+4, and a script of initiative 10, then 8, then natural 20, then damage 3
- **WHEN** the Reactive Strike is resolved against a creature at 20 Hit Points
- **THEN** the total is 20 and the degree is success
- **AND** the damage is 7
- **AND** the damage is not 14
- **AND** the disruption is NotDisrupted
- **AND** the creature has 13 Hit Points

### Requirement: A reactor that is not Alive cannot react

A reactor whose vital is not Alive SHALL return NotConscious. The reaction SHALL stay as it was, and no attack die SHALL be drawn. Dying, Unconscious, and Dead SHALL each do this. NotConscious SHALL win over Unavailable and over SelfTarget.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2429 (Player Core pg. 436, Step 2: Act)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=95 (Player Core pg. 446, Unconscious)

#### Scenario: A dying creature cannot react

- **GIVEN** CombatantId(1) is Dying(1) at 0 Hit Points with the reaction ready, CombatantId(2) is Alive and in reach, Activity::Manipulate, ReactiveStrike::Available, and a scripted natural 10 after initiative
- **WHEN** CombatantId(1) attempts the Reactive Strike
- **THEN** the error is NotConscious
- **AND** the reaction is still ready
- **AND** the scripted 10 is still unused

#### Scenario: An unconscious creature cannot react

- **GIVEN** CombatantId(1) is Unconscious at 0 Hit Points with the reaction ready, CombatantId(2) is Alive and in reach, Activity::Manipulate, ReactiveStrike::Available, and a scripted natural 10 after initiative
- **WHEN** CombatantId(1) attempts the Reactive Strike
- **THEN** the error is NotConscious
- **AND** the reaction is still ready
- **AND** the scripted 10 is still unused

#### Scenario: A dead creature cannot react

- **GIVEN** CombatantId(1) is Dead with the reaction forced ready, CombatantId(2) is Alive and in reach, Activity::Manipulate, ReactiveStrike::Available, and a scripted natural 10 after initiative
- **WHEN** CombatantId(1) attempts the Reactive Strike
- **THEN** the error is NotConscious
- **AND** the error is not NoReaction
- **AND** the reaction is still ready
- **AND** the scripted 10 is still unused

#### Scenario: Not conscious wins over unavailable

- **GIVEN** CombatantId(1) is Dying(1) at 0 Hit Points with the reaction ready, and the call passes ReactiveStrike::Unavailable and Activity::Manipulate in reach
- **WHEN** CombatantId(1) attempts the Reactive Strike
- **THEN** the error is NotConscious
- **AND** the error is not Unavailable
- **AND** the reaction is still ready
- **AND** no attack die is drawn

#### Scenario: A dying self target is not conscious

- **GIVEN** CombatantId(1) is Dying(1) at 0 Hit Points with the reaction ready, and the actor is CombatantId(1)
- **WHEN** that Reactive Strike is attempted with ReactiveStrike::Available and Activity::Manipulate
- **THEN** the error is NotConscious
- **AND** the error is not SelfTarget
- **AND** the reaction is still ready
- **AND** no attack die is drawn

### Requirement: Unavailable spends nothing

ReactiveStrike::Unavailable SHALL return Unavailable for an Alive reactor and SHALL spend nothing (proposed DEV-44). Unavailable SHALL win over NotATrigger and over InvalidReach.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2256 (Player Core pg. 138, Reactive Strike)

#### Scenario: Unavailable spends nothing

- **GIVEN** an Alive CombatantId(1) with a ready reaction, melee reach 7, size Large, Activity::Step, and ReactiveStrike::Unavailable, and a scripted natural 10 after initiative
- **WHEN** CombatantId(1) attempts the Reactive Strike against an adjacent creature
- **THEN** the error is Unavailable
- **AND** the error is not NotATrigger and not InvalidReach
- **AND** the reaction is still ready
- **AND** the scripted 10 is still unused

### Requirement: Damage follows Strike

A hit SHALL apply damage through the same function Encounter::strike uses. Temporary Hit Points SHALL be removed first. A miss SHALL spend the reaction, deal 0, and draw no damage die. This change SHALL NOT define knockout or dying.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2306 (Player Core pg. 418, Strike)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2321 (Player Core pg. 410, Temporary Hit Points)

#### Scenario: Temporary Hit Points are spent first

- **GIVEN** CombatantId(2) has 20 current Hit Points, 5 temporary Hit Points, and wounded 0, attack bonus +8, Armor Class 18, Activity::Manipulate, ReactiveStrike::Available, damage 1d8+4, and a script of initiative 10, then 8, then natural 10, then damage 3
- **WHEN** the Reactive Strike is resolved
- **THEN** the damage is 7
- **AND** temporary Hit Points are 0
- **AND** current Hit Points are 18
- **AND** wounded stays 0
- **AND** vital stays Alive

#### Scenario: A failure spends the reaction and deals no damage

- **GIVEN** CombatantId(2) has 20 Hit Points, attack bonus +0, Armor Class 18, Activity::Manipulate, ReactiveStrike::Available, a script of initiative 10, then 8, then natural 9, and a scripted damage die of 8
- **WHEN** the Reactive Strike is resolved
- **THEN** the total is 9 and the degree is failure
- **AND** the damage is 0 and the disruption is NotDisrupted
- **AND** the scripted 8 is still unused
- **AND** Hit Points stay 20
- **AND** the reaction is not ready
- **AND** the penalty is 0

#### Scenario: A critical failure spends the reaction and deals no damage

- **GIVEN** CombatantId(2) has 20 Hit Points, attack bonus +0, Armor Class 18, Activity::Manipulate, ReactiveStrike::Available, a script of initiative 10, then 8, then natural 5, and a scripted damage die of 8
- **WHEN** the Reactive Strike is resolved
- **THEN** the degree is critical failure
- **AND** the damage is 0 and the disruption is NotDisrupted
- **AND** the scripted 8 is still unused
- **AND** Hit Points stay 20
- **AND** the reaction is not ready

### Requirement: Errors have one order

Checks SHALL run in this order: UnknownCombatant for the reactor, then the actor, SelfTarget, NotConscious, Unavailable, NotATrigger, UnsupportedSize, InvalidReach, Overflow, OutOfReach, TargetDead, then NoReaction. An earlier failure SHALL win, draw no attack die, and leave the reaction unchanged. AttackOverflow and DamageOverflow SHALL spend nothing and leave Hit Points unchanged.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2256 (Player Core pg. 138, Reactive Strike)

#### Scenario: Striking oneself rolls nothing

- **GIVEN** Alive CombatantId(1) with a ready reaction and a scripted natural 10 after initiative
- **WHEN** CombatantId(1) attempts a Reactive Strike against CombatantId(1) with Activity::Manipulate and ReactiveStrike::Available
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

- **GIVEN** Alive CombatantId(1) at (0, 0) with reach 5 and a ready reaction, Dead CombatantId(2) at (0, 2), both Medium, Activity::Manipulate, ReactiveStrike::Available, and a scripted natural 10 after initiative
- **WHEN** CombatantId(1) attempts the Reactive Strike
- **THEN** the error is OutOfReach
- **AND** the error is not TargetDead
- **AND** the reaction is still ready
- **AND** the scripted 10 is still unused

#### Scenario: A dead target in reach is TargetDead

- **GIVEN** Alive CombatantId(1) at (0, 0) with reach 5 and a reaction that is not ready, Dead CombatantId(2) at (0, 1), both Medium, Activity::Manipulate, ReactiveStrike::Available, and a scripted natural 10 after initiative
- **WHEN** CombatantId(1) attempts the Reactive Strike
- **THEN** the error is TargetDead
- **AND** the error is not NoReaction
- **AND** the reaction is still not ready
- **AND** the scripted 10 is still unused

#### Scenario: An attack bonus that does not fit spends nothing

- **GIVEN** a legal in-reach manipulate trigger, ReactiveStrike::Available, a ready reaction, attack bonus 2147483647, and a scripted natural 10 after initiative
- **WHEN** the Reactive Strike is attempted
- **THEN** the error is AttackOverflow
- **AND** the reaction is still ready
- **AND** the scripted 10 is still unused

#### Scenario: A damage total that does not fit spends nothing

- **GIVEN** attack bonus +8, Armor Class 18, Activity::Manipulate, ReactiveStrike::Available, a ready reaction, current Hit Points 20, damage 1d8 with bonus 2147483647, and a script of initiative 10, then 8, then natural 10, then damage 1
- **WHEN** the Reactive Strike is resolved
- **THEN** the error is DamageOverflow
- **AND** the reaction is still ready
- **AND** Hit Points stay 20
