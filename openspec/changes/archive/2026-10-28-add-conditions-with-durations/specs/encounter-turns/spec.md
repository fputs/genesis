# encounter-turns

## REMOVED Requirements

### Requirement: The start step changes nothing for an Alive creature

**Reason**: Round durations now decrease at the start of the clock creature's turn.
**Migration**: Requirement "The start step ticks round durations". The old alive-start scenario moves there unchanged.

## ADDED Requirements

### Requirement: The start step ticks round durations

The start step SHALL run before actions are regained. It SHALL decrease round durations clocked to that creature, including a dead creature whose slot is skipped. It SHALL NOT roll a die or change Hit Points, temporary Hit Points, or wounded. With no such duration it SHALL NOT change conditions.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2428 (Player Core pg. 435, Step 1: Start Your Turn)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2242 (Player Core pg. 302, Durations)

#### Scenario: An alive start rolls nothing

- **GIVEN** an Alive creature at 12 Hit Points and a scripted natural 8
- **WHEN** that creature's turn starts
- **THEN** the scripted 8 is still unused
- **AND** Hit Points stay 12
- **AND** wounded stays 0
- **AND** the creature has 3 actions

#### Scenario: A 3-round duration decreases on the clock's next start

- **GIVEN** CombatantId(1) and CombatantId(2), both Alive, Perception +0, scripted initiative naturals 15 then 10, and sickened 1 for 3 rounds on CombatantId(2) clocked to CombatantId(1), applied before any turn
- **WHEN** CombatantId(1)'s turn starts
- **THEN** 2 rounds remain and sickened 1 is still present
- **AND** Hit Points are unchanged
- **AND** no condition die is rolled

#### Scenario: The bearer's start does not decrease another clock

- **GIVEN** the same pair, with sickened 1 for 3 rounds on CombatantId(2) clocked to CombatantId(1), and CombatantId(1)'s turn has ended
- **WHEN** CombatantId(2)'s turn starts
- **THEN** 3 rounds remain

#### Scenario: A skipped dead slot still ticks its round clock

- **GIVEN** the order CombatantId(1), CombatantId(2), CombatantId(3), CombatantId(2) already dead, and sickened 1 for 2 rounds on CombatantId(3) clocked to CombatantId(2)
- **WHEN** CombatantId(1)'s turn starts and ends, and the next turn starts
- **THEN** the open turn is CombatantId(3)
- **AND** CombatantId(2) did not start a turn
- **AND** 1 round remains

### Requirement: Apply marks until the end of the next turn

Applying until the end of a creature's next turn while that creature's turn is open SHALL leave the condition in place when that turn ends, and SHALL remove it when that creature's following turn ends. Applying it while that turn is not open SHALL remove it at the end of that creature's next turn.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2430 (Player Core pg. 436, Step 3: End Your Turn)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2455 (Player Core pg. 442, Appendix entry, Redundant Conditions)

#### Scenario: An open turn's end is not the next end

- **GIVEN** CombatantId(1)'s turn is open, and CombatantId(2) has no conditions
- **WHEN** off-guard is applied to CombatantId(2) until the end of CombatantId(1)'s next turn, and CombatantId(1)'s turn ends
- **THEN** off-guard remains
- **WHEN** CombatantId(1)'s following turn ends
- **THEN** off-guard is absent

#### Scenario: A closed turn's next end removes it

- **GIVEN** CombatantId(2)'s turn is open, and CombatantId(1)'s turn is not
- **WHEN** off-guard is applied to CombatantId(2) until the end of CombatantId(1)'s next turn, and CombatantId(1)'s next turn ends
- **THEN** off-guard is absent

### Requirement: The attacker's conditions adjust the attack and damage

A Strike SHALL add the attacker's modifiers for its attack basis to the attack, and for its damage basis to the damage. Enfeebled SHALL penalize a Strength melee attack and Strength damage. A Dexterity melee attack SHALL NOT take enfeebled. Strength damage on that attack SHALL. A ranged attack SHALL include clumsy and SHALL NOT include enfeebled. Damage SHALL NOT include frightened or clumsy.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=71 (Player Core pg. 443, Enfeebled)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=61 (Player Core pg. 442, Clumsy)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=76 (Player Core pg. 444, Frightened)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2282 (Player Core pg. 400, Penalties)

#### Scenario: Frightened 2 lowers the attack total by 2

- **GIVEN** an open turn, Strength melee, attack bonus +6, Armor Class 16, 0 damage dice, the attacker frightened 2, and a scripted natural 10
- **WHEN** the Strike is resolved
- **THEN** the total is 14
- **AND** the degree is failure

#### Scenario: Frightened 2 and enfeebled 2 stay at minus 2

- **GIVEN** an open turn, Strength melee, attack bonus +8, Armor Class 16, 0 damage dice, the attacker frightened 2 and enfeebled 2, and a scripted natural 10
- **WHEN** the Strike is resolved
- **THEN** the total is 16
- **AND** the degree is success

#### Scenario: Clumsy 2 does not lower a Strength melee attack

- **GIVEN** an open turn, Strength melee, attack bonus +6, Armor Class 16, 0 damage dice, the attacker clumsy 2, and a scripted natural 10
- **WHEN** the Strike is resolved
- **THEN** the total is 16
- **AND** the degree is success

#### Scenario: Clumsy 2 lowers a Dexterity melee attack

- **GIVEN** an open turn, Dexterity melee, attack bonus +6, Armor Class 16, 0 damage dice, the attacker clumsy 2, and a scripted natural 10
- **WHEN** the Strike is resolved
- **THEN** the total is 14
- **AND** the degree is failure

#### Scenario: Enfeebled 2 does not lower a Dexterity melee attack

- **GIVEN** an open turn, a Dexterity melee attack, Strength damage, attack bonus +6, Armor Class 16, 0 damage dice, the attacker enfeebled 2, and a scripted natural 10
- **WHEN** the Strike is resolved
- **THEN** the total is 16
- **AND** the degree is success

#### Scenario: Enfeebled 2 lowers Dexterity melee damage

- **GIVEN** an open turn, a Dexterity melee attack, Strength damage, attack bonus +10, Armor Class 11, the attacker enfeebled 2, a scripted natural 10, and damage 1d8+0 showing 4
- **WHEN** the Strike is resolved
- **THEN** the total is 20
- **AND** the degree is success
- **AND** the damage is 2

#### Scenario: Clumsy 2 lowers a ranged attack

- **GIVEN** an open turn, a ranged strike, attack bonus +6, Armor Class 16, 0 damage dice, the attacker clumsy 2, and a scripted natural 10
- **WHEN** the Strike is resolved
- **THEN** the total is 14
- **AND** the degree is failure

#### Scenario: Enfeebled 2 lowers Strength melee damage

- **GIVEN** an open turn, Strength melee, attack bonus +10, Armor Class 11, the attacker enfeebled 2, a scripted natural 10, and damage 1d8+0 showing 4
- **WHEN** the Strike is resolved
- **THEN** the degree is success
- **AND** the damage is 2

#### Scenario: Enfeebled 2 does not lower ranged damage

- **GIVEN** an open turn, a ranged strike, attack bonus +10, Armor Class 11, the attacker enfeebled 2, a scripted natural 10, and damage 1d8+0 showing 4
- **WHEN** the Strike is resolved
- **THEN** the degree is success
- **AND** the damage is 4

### Requirement: The returned turn decrements once

The end of the turn a creature returns to SHALL decrement frightened and SHALL expire a due negative until-end condition, once. It SHALL NOT run the persistent hook again. The next ordinary end SHALL decrement again.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2294 (Player Core pg. 416, Delay)
**Source**: R-14 in docs/designer/rules-rulings.md. Delay, Player Core pg. 416.

#### Scenario: Frightened drops at the end of the returned turn

- **GIVEN** CombatantId(1) frightened 2, and CombatantId(2), both Alive, Perception +0, scripted initiative naturals 15 then 10
- **WHEN** CombatantId(1)'s turn starts and CombatantId(1) Delays
- **THEN** frightened is 2
- **WHEN** CombatantId(2)'s turn ends, CombatantId(1) returns, and that turn ends
- **THEN** frightened is 1
- **WHEN** CombatantId(1)'s next ordinary turn ends
- **THEN** frightened is absent

### Requirement: Death clears condition instances

Becoming Dead SHALL clear that creature's condition instances, as persistent damage is cleared. A skipped dead slot SHALL NOT run the end decrement. A skipped dead slot SHALL still tick round durations at the start.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2329 (Player Core pg. 411, Death)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2430 (Player Core pg. 436, Step 3: End Your Turn)

#### Scenario: Death clears frightened

- **GIVEN** uses_dying_rules false, CombatantId(2) Alive at 4 Hit Points with frightened 2, CombatantId(1)'s turn open, a Strength melee attack, Strength damage, attack bonus +10, Armor Class 11, a scripted natural 10, and damage 1d8+0 showing 4
- **WHEN** CombatantId(1) Strikes CombatantId(2)
- **THEN** CombatantId(2) is Dead
- **AND** frightened is absent

#### Scenario: A skipped dead slot does not decrement

- **GIVEN** the order CombatantId(1), CombatantId(2), CombatantId(3), CombatantId(2) already Dead with no condition instances, CombatantId(1) frightened 2, and a scripted natural 8
- **WHEN** CombatantId(1)'s turn ends and the next turn starts
- **THEN** the open turn is CombatantId(3)
- **AND** CombatantId(1) is frightened 1
- **AND** CombatantId(2) did not end a turn
- **AND** the scripted 8 is unused

## MODIFIED Requirements

### Requirement: Initiative is one Perception total

Each combatant SHALL add one d20 to that creature's Perception modifier, including tracked penalties on Perception. The ranking SHALL use that total. A natural 20 or a natural 1 SHALL NOT change the total. With no such penalty the total is the d20 plus Perception. Dice SHALL be drawn in ascending CombatantId order.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2423 (Player Core pg. 435, Step 1: Roll Initiative)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=76 (Player Core pg. 444, Frightened)

#### Scenario: Perception is added to the d20

- **GIVEN** one combatant with Perception modifier +5 and a scripted natural 10
- **WHEN** initiative is rolled
- **THEN** that combatant's total is 15
- **AND** one d20 was rolled

#### Scenario: A natural 20 does not add 10

- **GIVEN** CombatantId(1) with Perception +0 and CombatantId(2) with Perception +5, and scripted naturals 20 then 14
- **WHEN** initiative is rolled
- **THEN** CombatantId(1) has total 20 and CombatantId(2) has total 19
- **AND** CombatantId(1) acts earlier

#### Scenario: A natural 1 does not subtract 10

- **GIVEN** one combatant with Perception modifier +19 and a scripted natural 1
- **WHEN** initiative is rolled
- **THEN** the total is 20

#### Scenario: A negative Perception modifier is added

- **GIVEN** one combatant with Perception modifier -3 and a scripted natural 8
- **WHEN** initiative is rolled
- **THEN** the total is 5

#### Scenario: Roster order does not choose the die

- **GIVEN** CombatantId(5) listed before CombatantId(1), Perception +0 and +5, and scripted naturals 10 then 14
- **WHEN** initiative is rolled
- **THEN** CombatantId(1) has total 15 and CombatantId(5) has total 14

#### Scenario: Frightened 2 and sickened 1 lower initiative by 2

- **GIVEN** one combatant with Perception modifier +5, frightened 2, sickened 1, and a scripted natural 10
- **WHEN** initiative is rolled
- **THEN** that combatant's total is 13

### Requirement: Ending a turn drops actions and keeps the reaction

Ending a turn SHALL set actions to 0 and SHALL keep an unspent reaction. `end_turn` SHALL keep its dice argument. An ordinary end SHALL deal persistent damage, then roll its flat check, then decrement. Rules 2430 allows expiries in any order; this change removes a due until-end instance before the decrement. With no persistent damage, that hook SHALL NOT roll or change Hit Points. The decrement SHALL NOT roll. No open turn SHALL return TurnNotOpen and SHALL NOT roll.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2430 (Player Core pg. 436, Step 3: End Your Turn)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=76 (Player Core pg. 444, Frightened)

#### Scenario: Ending early drops leftover actions and keeps the reaction

- **GIVEN** an open turn with 2 actions left and the reaction ready, current Hit Points 12, no persistent damage, and a scripted natural 8
- **WHEN** the turn ends with that die
- **THEN** actions are 0
- **AND** the reaction is still ready
- **AND** Hit Points stay 12
- **AND** temporary Hit Points stay 0
- **AND** wounded stays 0
- **AND** the scripted 8 is still unused

#### Scenario: End before a start returns an error

- **GIVEN** initiative has been rolled and no turn has started, and a scripted natural 8 is next
- **WHEN** a turn end is requested with that die
- **THEN** the error is TurnNotOpen
- **AND** every combatant still has 0 actions
- **AND** the scripted 8 is unused

#### Scenario: Ending the turn lowers frightened and leaves Hit Points

- **GIVEN** an open turn, the active creature frightened 2 and sickened 1, no persistent damage, current Hit Points 12, and a scripted natural 8
- **WHEN** the turn ends with that die
- **THEN** frightened is 1
- **AND** sickened is 1
- **AND** Hit Points stay 12
- **AND** the scripted 8 is still unused

#### Scenario: The persistent hook runs before frightened decreases

- **GIVEN** an open turn, Hit Points 20, empty defenses, persistent Fire 5, frightened 2, and a scripted natural 10
- **WHEN** the turn ends with that die
- **THEN** Hit Points are 15
- **AND** frightened is 1
- **AND** persistent Fire 5 remains

### Requirement: Strike spends one action against baked Armor Class

On an open Alive turn, a Strike SHALL spend 1 action. It SHALL roll one d20 once the target is affected, add the strike's attack bonus and the multiple attack penalty, and compare that total with Armor Class. A lost flat check rolls no attack d20. No square and no reach SHALL be required. Armor Class SHALL be the baked value plus the target's condition modifiers. A caller Armor Class modifier SHALL NOT apply. A not-alive target keeps that result until wire-dying-into-turns (DEV-36).

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2306 (Player Core pg. 418, Strike)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=76 (Player Core pg. 444, Frightened)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=58 (Player Core pg. 445, Off-Guard)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2282 (Player Core pg. 400, Penalties)

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

#### Scenario: Frightened 2 on the target changes the degree

- **GIVEN** an open turn, attack bonus +0, baked Armor Class 16, 0 damage dice, the target frightened 2, and a scripted natural 14
- **WHEN** the Strike is resolved
- **THEN** the total is 14
- **AND** the degree is success

#### Scenario: Frightened 2 and sickened 1 do not sum on Armor Class

- **GIVEN** an open turn, attack bonus +0, baked Armor Class 15, 0 damage dice, the target frightened 2 and sickened 1, and a scripted natural 12
- **WHEN** the Strike is resolved
- **THEN** the total is 12
- **AND** the degree is failure

#### Scenario: Frightened 2 and off-guard both lower Armor Class

- **GIVEN** an open turn, attack bonus +0, baked Armor Class 18, 0 damage dice, the target frightened 2 and off-guard, and a scripted natural 14
- **WHEN** the Strike is resolved
- **THEN** the total is 14
- **AND** the degree is success
