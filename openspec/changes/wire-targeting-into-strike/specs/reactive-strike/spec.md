# reactive-strike

## ADDED Requirements

### Requirement: A Reactive Strike resolves targeting before the attack roll

After the checks in Errors have one order, and after AttackOverflow, a Reactive Strike SHALL call resolve_targeting with area_effect false and SHALL NOT reimplement it. The flat check is drawn before the attack d20. Only a concealed or hidden actor draws it. The attack d20 is drawn only when the actor is affected. An observed actor who is not concealed draws no flat-check die. The relation SHALL stay unchanged.

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

### Requirement: A lost Reactive Strike spends the reaction only

When the actor is not affected, the result SHALL be Ok with LostStrike and Disruption::NotDisrupted. The reaction SHALL be spent. Actions and the attack count SHALL stay unchanged. No attack d20 and no damage die SHALL be drawn. Hit Points SHALL stay unchanged.

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

- **GIVEN** CombatantId(1)'s turn is open, one non-agile Strike has resolved, the stored count is 1, 2 actions remain, CombatantId(2) is observed and concealed and in reach, and scripted naturals 4, then 12, then a damage die of 8
- **WHEN** CombatantId(1) makes a Reactive Strike with an ActionCost of 1 action with Manipulate and ReactiveStrike::Available, and then Strikes with attack bonus +8 against Armor Class 18
- **THEN** the Reactive Strike is LostStrike and the stored count is still 1
- **AND** the following Strike has penalty -5 and total 15
- **AND** that penalty is not -10 and that total is not 10
- **AND** the damage die of 8 is still unused
- **AND** 1 action remains

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
