# reactive-strike

## MODIFIED Requirements

### Requirement: Errors have one order

Checks SHALL run in this order: UnknownCombatant for the reactor, then the actor, NotConscious, SelfTarget, Unavailable, NotATrigger, UnsupportedSize, Overflow, InvalidReach, OutOfReach, TargetNotConscious, NoReaction, AttackOverflow, then CannotTarget. A failure before the d20 SHALL draw no die and leave the reaction unchanged. DamageOverflow SHALL consume both dice and leave the reaction ready and Hit Points unchanged. AttackOverflow SHALL reject bonus + modifiers + 20 over i32 before the d20.

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

## ADDED Requirements

### Requirement: A Reactive Strike resolves targeting before the attack roll

After every check in Errors have one order through AttackOverflow, a Reactive Strike SHALL call resolve_targeting with area_effect false and SHALL NOT reimplement it. The flat check is drawn before the attack d20. Only a concealed or hidden actor draws it. The attack d20 is drawn only when the actor is affected. An observed actor who is not concealed draws no flat-check die. The relation SHALL stay unchanged. After Ok(LostStrike) the caller SHALL still call note_strike (DEV-43).

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2256 (Player Core pg. 138, Reactive Strike)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=62 (Player Core pg. 442, Concealed)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=79 (Player Core pg. 444, Hidden)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2278 (Player Core pg. 400, Checks (Flat Checks))

#### Scenario: A natural 5 at DC 5 then the reactive attack

- **GIVEN** Alive CombatantId(1) at (0, 0) with melee reach 5 and Alive CombatantId(2) at (0, 1), both Medium and at 20 Hit Points, CombatantId(2) observed and concealed from CombatantId(1), no turn open, an ActionCost of 1 action with Manipulate, ReactiveStrike::Available, attack bonus +8, Armor Class 18, damage 1d8+4, and a script of 10, then 8, then 5, then 10, then 3
- **WHEN** CombatantId(1) makes the Reactive Strike
- **THEN** the flat check DC is 5 and the Strike is Rolled
- **AND** the degree is success, the total is 18, the penalty is 0, and the damage is 7
- **AND** the disruption is NotDisrupted
- **AND** CombatantId(2) has 13 Hit Points
- **AND** the reaction is not ready and the attack count stays 0
- **AND** the relation is still observed and concealed

#### Scenario: Hidden and concealed use one DC 11 roll

- **GIVEN** Alive CombatantId(1) at (0, 0) with melee reach 5 and a ready reaction, Alive CombatantId(2) at (0, 1) with 20 Hit Points, both Medium, CombatantId(2) hidden and concealed from CombatantId(1) with Stealth DC 17, an ActionCost of 1 action with Manipulate, ReactiveStrike::Available, and scripted naturals 10, then 8, then 10, then 20
- **WHEN** CombatantId(1) makes the Reactive Strike
- **THEN** one flat check of 10 is compared with DC 11 and the result is LostStrike
- **AND** the scripted 20 is still unused
- **AND** the reaction is not ready
- **AND** Hit Points stay 20

#### Scenario: Each reaction draws its own flat check

- **GIVEN** the order is Side::Enemy CombatantId(1) at (0, 0) then Side::Pc CombatantId(2) at (1, 0) then Alive CombatantId(3) at (0, 1), all Medium, reach 5, CombatantId(3) has 20 Hit Points, Armor Class 18, and 3 actions, and is observed and concealed from both, CombatantId(1) has attack bonus +8, and CombatantId(2) has a stored non-agile Strike, snapshot 0, attack bonus +8, on CombatantId(3) beginning a Stride
- **WHEN** CombatantId(3) begins a Stride and resolve_reactions is called once with one caller entry, CombatantId(1) only: ReactiveStrike::Available, attack bonus +8, no modifiers, both sizes Medium, and square (0, 0), then CombatantId(1)'s flat natural is 4, and CombatantId(2)'s flat natural is 5, attack natural 10, and damage 1
- **THEN** CombatantId(1)'s result is LostStrike and that reaction is spent
- **AND** CombatantId(2)'s penalty is 0, the total is 18, and the degree is success
- **AND** CombatantId(3) is at 19 Hit Points and still has 3 actions
- **AND** CombatantId(2)'s reaction is spent after CombatantId(1)'s reaction

### Requirement: A lost Reactive Strike spends the reaction only

When the actor is not affected, the result SHALL be Ok with LostStrike and Disruption::NotDisrupted. The reaction SHALL be spent. Actions and the attack count SHALL stay unchanged. No attack d20 and no damage die SHALL be drawn. Hit Points SHALL stay unchanged. After Ok(LostStrike) the caller SHALL still call note_strike (DEV-43).

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2256 (Player Core pg. 138, Reactive Strike)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2416 (Player Core pg. 434, Hidden)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2289 (Player Core pg. 402, Multiple Attack Penalty)
**Source**: R-12 in docs/designer/rules-rulings.md

#### Scenario: A natural 4 at DC 5 loses the Reactive Strike

- **GIVEN** Alive CombatantId(1) at (0, 0) with melee reach 5, 0 actions, attack count 0, and a ready reaction, Alive CombatantId(2) at (0, 1) with 20 Hit Points, both Medium, CombatantId(2) observed and concealed from CombatantId(1), an ActionCost of 1 action with Manipulate, ReactiveStrike::Available, and a script of 10, then 8, then 4, then 10, then 3
- **WHEN** CombatantId(1) makes the Reactive Strike
- **THEN** the result is LostStrike, the flat natural is 4, the DC is 5, and the degree is failure
- **AND** the disruption is NotDisrupted
- **AND** the reaction is not ready
- **AND** CombatantId(1) still has 0 actions and the attack count stays 0
- **AND** Hit Points stay 20
- **AND** the scripted 10 after the flat check, and the damage die of 3, are still unused

#### Scenario: A natural 10 at DC 11 loses the Reactive Strike

- **GIVEN** Alive CombatantId(1) at (0, 0) with melee reach 5 and a ready reaction, Alive CombatantId(2) at (0, 1) with 20 Hit Points, both Medium, CombatantId(2) hidden and not concealed from CombatantId(1) with Stealth DC 17, an ActionCost of 1 action with Manipulate, ReactiveStrike::Available, and a script of 10, then 8, then 10, then 12
- **WHEN** CombatantId(1) makes the Reactive Strike
- **THEN** the result is LostStrike, the flat natural is 10, the DC is 11, and the degree is failure
- **AND** the reaction is not ready and the attack count stays 0
- **AND** the scripted 12 is still unused
- **AND** Hit Points stay 20

#### Scenario: A lost Reactive Strike does not count toward MAP

- **GIVEN** CombatantId(1)'s turn is open and initiative is already rolled, one non-agile Strike has resolved, the stored count is 1, 2 actions remain, and the reaction is ready. CombatantId(1) is at (0, 0), Medium, melee reach 5. Alive CombatantId(2) is at (0, 1), Medium, observed and concealed from CombatantId(1). Alive CombatantId(3) is observed and not concealed, Armor Class 18. The script is 4, then 12, then 8
- **WHEN** CombatantId(1) makes a Reactive Strike against CombatantId(2) with an ActionCost of 1 action with Manipulate, ReactiveStrike::Available, and attack bonus +8, and then Strikes CombatantId(3) with a non-agile attack bonus +8 and 0 damage dice
- **THEN** the Reactive Strike is LostStrike and the attack count is still 1
- **AND** the Strike against CombatantId(3) draws no flat-check die
- **AND** that Strike uses attack natural 12, penalty -5, and total 15
- **AND** that penalty is not -10 and that total is not 11
- **AND** the degree is failure and the scripted 8 is still unused
- **AND** 1 action remains
- **AND** the attack count is 2 after that Strike

### Requirement: An undetected Reactive Strike is CannotTarget

An undetected actor SHALL return ReactiveError::CannotTarget after NoReaction and AttackOverflow. The reaction SHALL stay as it was, and no die SHALL be drawn.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=96 (Player Core pg. 447, Undetected)
**Source**: https://2e.aonprd.com/Actions.aspx?ID=2256 (Player Core pg. 138, Reactive Strike)

#### Scenario: An undetected actor cannot be the target

- **GIVEN** Alive CombatantId(1) at (0, 0) with melee reach 5 and a ready reaction, Alive CombatantId(2) at (0, 1), both Medium, CombatantId(2) undetected and not concealed from CombatantId(1) with Stealth DC 17, an ActionCost of 1 action with Manipulate, ReactiveStrike::Available, and a script of 10, then 8, then 11
- **WHEN** CombatantId(1) attempts the Reactive Strike
- **THEN** the error is CannotTarget
- **AND** the error is not NoReaction
- **AND** the reaction is still ready
- **AND** the scripted 11 is still unused

#### Scenario: No reaction beats an undetected actor

- **GIVEN** Alive CombatantId(1) at (0, 0) with melee reach 5 and a reaction that is not ready, Alive CombatantId(2) at (0, 1) who is undetected with Stealth DC 17, both Medium, an ActionCost of 1 action with Manipulate, ReactiveStrike::Available, and a scripted natural 11 after initiative
- **WHEN** CombatantId(1) attempts the Reactive Strike
- **THEN** the error is NoReaction
- **AND** the error is not CannotTarget
- **AND** the scripted 11 is still unused
