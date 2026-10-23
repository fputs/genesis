# encounter-turns

## MODIFIED Requirements

### Requirement: Strike spends one action against baked Armor Class

On an open Alive turn, a Strike SHALL spend 1 action. It SHALL roll one d20 once the target is affected, add the strike's attack bonus and the multiple attack penalty, and compare that total with the target's baked Armor Class. A lost flat check rolls no attack d20. No square and no reach SHALL be required. The degree SHALL use that baked Armor Class. A target whose vital is neither Alive nor Dead keeps that baked Armor Class until wire-dying-into-turns replaces this result (DEV-36).

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

## ADDED Requirements

### Requirement: Strike resolves targeting before the attack roll

Strike SHALL call resolve_targeting, area_effect false, after this no-die order, before the attack d20: TurnNotOpen, NotYourTurn, UnknownCombatant, NotConscious, InsufficientActions, UnknownCombatant, SelfTarget, TargetDead, AttackCountOverflow, Requested TargetNotConscious, Delayed, NoShield, NotRaised, Broken, NoReaction, term AttackOverflow. It SHALL NOT reimplement it. No relation: no flat die. Attack d20 only if affected. Summed AttackOverflow follows it. The relation SHALL stay unchanged.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=62 (Player Core pg. 442, Concealed)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=79 (Player Core pg. 444, Hidden)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2278 (Player Core pg. 400, Checks (Flat Checks))
**Source**: https://2e.aonprd.com/Actions.aspx?ID=2306 (Player Core pg. 418, Strike)

#### Scenario: An observed target draws the attack die first

- **GIVEN** an open turn, attack bonus +8, Armor Class 18, 0 damage dice, no stored relation, and scripted naturals 10 then 4
- **WHEN** the active combatant Strikes the other
- **THEN** the total is 18 and the degree is success
- **AND** the scripted 4 is still unused
- **AND** 2 actions remain

#### Scenario: A natural 5 at DC 5 then the attack roll

- **GIVEN** an open turn, attack bonus +8, Armor Class 18, damage 1d8+4, CombatantId(2) observed and concealed from the striker, and scripted naturals 5, then 10, then a damage die of 3
- **WHEN** the active combatant Strikes CombatantId(2)
- **THEN** the flat check DC is 5 and the Strike is Rolled
- **AND** the attack total is 18, the degree is success, and the damage is 7
- **AND** the relation is still observed and concealed
- **AND** 2 actions remain

#### Scenario: A natural 11 at DC 11 then the attack roll

- **GIVEN** an open turn, attack bonus +8, Armor Class 18, damage 1d8+4, CombatantId(2) hidden and not concealed from the striker with Stealth DC 17, and scripted naturals 11, then 10, then a damage die of 3
- **WHEN** the active combatant Strikes CombatantId(2)
- **THEN** the flat check DC is 11 and the Strike is Rolled
- **AND** the attack total is 18, the degree is success, and the damage is 7
- **AND** the relation is still hidden and not concealed

#### Scenario: Hidden and concealed succeed on one DC 11 roll

- **GIVEN** an open turn, attack bonus +8, Armor Class 18, damage 1d8+4, CombatantId(2) hidden and concealed from the striker with Stealth DC 17, and scripted naturals 11, then 10, then a damage die of 3
- **WHEN** the active combatant Strikes CombatantId(2)
- **THEN** one flat check of 11 is compared with DC 11
- **AND** the attack total is 18 and the damage is 7
- **AND** the relation is still hidden and concealed

#### Scenario: The same lost Strike twice matches

- **GIVEN** two encounters, each with an open turn, attack bonus +8, Armor Class 18, CombatantId(2) observed and concealed, and the script 4 then 10
- **WHEN** each active combatant Strikes CombatantId(2)
- **THEN** both results are LostStrike with flat natural 4, DC 5, and degree failure
- **AND** each scripted 10 is still unused
- **AND** each attack count is 1

### Requirement: A lost Strike spends the action and counts for MAP

When the target is not affected, Strike SHALL return Ok(LostStrike) and SHALL NOT draw the attack d20 or a damage die. It SHALL spend 1 action and increment the attack count. Hit Points, temporary Hit Points, wounded, vital, and shield Hit Points SHALL stay unchanged, and the target's reaction SHALL stay ready. The next Strike on that turn SHALL use the new count. After Ok(LostStrike) the caller SHALL still call note_strike (DEV-43).

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2416 (Player Core pg. 434, Hidden)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2289 (Player Core pg. 402, Multiple Attack Penalty)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=62 (Player Core pg. 442, Concealed)
**Source**: R-12 in docs/designer/rules-rulings.md

#### Scenario: A natural 4 at DC 5 loses the Strike

- **GIVEN** an open turn, 3 actions, attack count 0, attack bonus +8, Armor Class 18, the target at 20 Hit Points with a ready reaction and a shield at 20 Hit Points, the target observed and concealed, and scripted naturals 4, then 10, then a damage die of 3
- **WHEN** the active combatant Strikes that target
- **THEN** the result is LostStrike, the flat natural is 4, the DC is 5, and the degree is failure
- **AND** 2 actions remain and the attack count is 1
- **AND** the scripted 10 and the damage die of 3 are still unused
- **AND** Hit Points stay 20, shield Hit Points stay 20, and the target's reaction stays ready
- **AND** the relation is still observed and concealed

#### Scenario: A natural 1 at DC 5 loses the Strike and counts

- **GIVEN** an open turn, attack count 0, the target observed and concealed, and scripted naturals 1 then 10
- **WHEN** the active combatant Strikes that target
- **THEN** the result is LostStrike, the flat natural is 1, the DC is 5, and the degree is critical failure
- **AND** the attack count is 1
- **AND** the scripted 10 is still unused

#### Scenario: A natural 10 at DC 11 makes the next Strike take -5

- **GIVEN** an open turn, a non-agile strike, attack bonus +8, Armor Class 18, 0 damage dice, CombatantId(2) hidden and not concealed with Stealth DC 17, CombatantId(3) observed and not concealed, and scripted naturals 10 then 12
- **WHEN** the active combatant Strikes CombatantId(2) and then Strikes CombatantId(3)
- **THEN** the first result is LostStrike, the DC is 11, and the degree is failure
- **AND** the second multiple attack penalty is -5 and the second total is 15
- **AND** the second degree is failure
- **AND** 1 action remains

#### Scenario: A lost Strike makes the next agile Strike take -4

- **GIVEN** an open turn, an agile strike, attack bonus +8, Armor Class 18, damage 1d8+4, CombatantId(2) observed and concealed, CombatantId(3) observed and not concealed, and scripted naturals 4, then 14, then a damage die of 3
- **WHEN** the active combatant Strikes CombatantId(2) and then Strikes CombatantId(3)
- **THEN** the first result is LostStrike
- **AND** the second multiple attack penalty is -4, the second total is 18, and the second damage is 7
- **AND** the second penalty is not -5

#### Scenario: Two lost Strikes make the third take -10

- **GIVEN** an open turn, a non-agile strike, attack bonus +8, Armor Class 18, 0 damage dice, CombatantId(2) observed and concealed, CombatantId(3) observed and not concealed, and scripted naturals 4, then 4, then 15
- **WHEN** the active combatant Strikes CombatantId(2) twice and then Strikes CombatantId(3)
- **THEN** the third multiple attack penalty is -10 and the third total is 13
- **AND** the third degree is failure
- **AND** 0 actions remain
- **AND** the turn stays open

#### Scenario: Hidden and concealed lose on one DC 11 roll

- **GIVEN** an open turn, CombatantId(2) hidden and concealed with Stealth DC 17, and scripted naturals 10, then 20, then 6
- **WHEN** the active combatant Strikes CombatantId(2)
- **THEN** one flat check of 10 is compared with DC 11 and the result is LostStrike
- **AND** the scripted 20 and the scripted 6 are still unused
- **AND** the attack count is 1
- **AND** the relation is still hidden and concealed

#### Scenario: A lost Strike still reveals the hidden striker

- **GIVEN** an open turn, CombatantId(1) hidden and not concealed from CombatantId(2) and from CombatantId(3), each with Stealth DC 17, CombatantId(2) observed and concealed from CombatantId(1), attack bonus +8, Armor Class 18, and scripted naturals 4 then 10
- **WHEN** CombatantId(1) Strikes CombatantId(2) and, after Ok(LostStrike), the caller calls note_strike
- **THEN** the flat natural is 4, the DC is 5, and the degree is failure
- **AND** the scripted 10 is still unused
- **AND** every observer reads CombatantId(1) as observed, including CombatantId(2) and CombatantId(3)
- **AND** CombatantId(2) is still observed and concealed from CombatantId(1)

### Requirement: An undetected Strike is CannotTarget

When resolve_targeting returns CannotTarget, Strike SHALL return EncounterError::CannotTarget, draw no die, and leave actions and the attack count unchanged. Each gate in that no-die order SHALL win and SHALL NOT call resolve_targeting.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=96 (Player Core pg. 447, Undetected)
**Source**: https://2e.aonprd.com/Actions.aspx?ID=2306 (Player Core pg. 418, Strike)

#### Scenario: An undetected target cannot be struck

- **GIVEN** an open turn, 3 actions, attack count 0, CombatantId(2) undetected and not concealed with Stealth DC 17, and a scripted natural 11
- **WHEN** the active combatant Strikes CombatantId(2)
- **THEN** the error is CannotTarget
- **AND** 3 actions remain and the attack count stays 0
- **AND** the scripted 11 is still unused
- **AND** the relation is still undetected

#### Scenario: A dead undetected target is TargetDead

- **GIVEN** an open turn, 3 actions, a dead CombatantId(2) who is undetected with Stealth DC 17, and a scripted natural 11
- **WHEN** the active combatant Strikes CombatantId(2)
- **THEN** the error is TargetDead
- **AND** the error is not CannotTarget
- **AND** 3 actions remain
- **AND** the scripted 11 is still unused

#### Scenario: No actions beats a concealed flat check

- **GIVEN** an open Alive turn with 0 actions, the target observed and concealed, and a scripted natural 4
- **WHEN** the active combatant Strikes that target
- **THEN** the error is InsufficientActions
- **AND** the scripted 4 is still unused
- **AND** the turn stays open

#### Scenario: An attack count at the maximum draws no flat check

- **GIVEN** the active creature's stored attack count is 255, 3 actions remain, the target is observed and concealed, and a scripted natural 4
- **WHEN** that creature Strikes
- **THEN** the error is AttackCountOverflow
- **AND** 3 actions remain and the stored count is 255
- **AND** the scripted 4 is still unused

#### Scenario: A shield that is not raised draws no flat check

- **GIVEN** an open turn, 3 actions, a concealed target whose shield is not raised, ShieldBlock::Requested, and a scripted natural 4
- **WHEN** the active combatant Strikes that target
- **THEN** the error is NotRaised
- **AND** 3 actions remain
- **AND** the scripted 4 is still unused

#### Scenario: A flat check success can still overflow the attack

- **GIVEN** an open turn with 3 actions, attack bonus +0, Armor Class 11, 0 damage dice, the target observed and concealed, scripted naturals 5, then 11, then 8, and two caller untyped penalties of -2147483648 from different sources
- **WHEN** the Strike is attempted
- **THEN** the error is AttackOverflow of the modifier overflow
- **AND** the scripted 5 and the scripted 11 were drawn
- **AND** the scripted 8 is still unused
- **AND** 3 actions remain
- **AND** the next Strike on that turn has multiple attack penalty 0
