# damage-defenses Specification

## Purpose
Types each damage instance, applies immunity then weakness then resistance, and deals persistent damage at the end of the creature's turn.

## Requirements

### Requirement: A damage instance has one type

A damage instance SHALL be a non-negative amount and exactly one damage type. The types SHALL be Bludgeoning, Piercing, Slashing, Bleed, Acid, Cold, Electricity, Fire, Sonic, Force, Spirit, Mental, and Poison.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2308 (Player Core pg. 407, Damage Types)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2301 (Player Core pg. 406, Damage Rolls)

#### Scenario: Fire and slashing are different instances

- **GIVEN** an amount of 5 with type Fire and an amount of 5 with type Slashing
- **WHEN** the two instances are compared
- **THEN** the types differ and neither amount is changed

### Requirement: Immunity is applied first

If the creature is immune to the instance's type, the adjusted amount SHALL be 0. Weakness and resistance SHALL NOT be applied.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2309 (Player Core pg. 407, Step 3)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2313 (Player Core pg. 408, Immunity)

#### Scenario: Immunity to fire ignores 10 fire

- **GIVEN** immunity to Fire and an instance of 10 Fire
- **WHEN** the instance is adjusted
- **THEN** the amount is 0

#### Scenario: Immunity beats weakness to the same type

- **GIVEN** immunity to Fire, weakness 5 to Fire, and an instance of 10 Fire
- **WHEN** the instance is adjusted
- **THEN** the amount is 0

#### Scenario: Immunity to fire does not affect slashing

- **GIVEN** immunity to Fire and an instance of 8 Slashing
- **WHEN** the instance is adjusted
- **THEN** the amount is 8

### Requirement: Weakness is the highest value of that type

If the creature is not immune, the highest weakness of the instance's type SHALL be added once. A weakness of another type SHALL NOT be added. Two weakness values of the same type SHALL NOT be summed. A weakness that applies without a damage instance is out of scope.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2317 (Player Core pg. 408, Weakness)
**Source**: DEV-62. The book adds each weakness once and does not say two values of one type sum.

#### Scenario: Weakness 5 fire raises 7 fire to 12

- **GIVEN** weakness 5 to Fire and an instance of 7 Fire
- **WHEN** the instance is adjusted
- **THEN** the amount is 12

#### Scenario: Two fire weaknesses use 10 once

- **GIVEN** weakness 5 to Fire, weakness 10 to Fire, and an instance of 4 Fire
- **WHEN** the instance is adjusted
- **THEN** the amount is 14

#### Scenario: A slashing weakness does not raise fire

- **GIVEN** weakness 5 to Slashing and an instance of 7 Fire
- **WHEN** the instance is adjusted
- **THEN** the amount is 7

### Requirement: Resistance follows weakness and stops at 0

After weakness, the highest resistance of that type SHALL be subtracted. The result SHALL NOT be below 0. Two resistance values of the same type SHALL NOT be summed. A resistance of another type SHALL NOT be subtracted.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2318 (Player Core pg. 408, Resistance)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2309 (Player Core pg. 407, Step 3)
**Source**: DEV-63. The book says usually the highest. This engine always uses the highest.

#### Scenario: Weakness then resistance leaves 3

- **GIVEN** weakness 5 to Fire, resistance 10 to Fire, and an instance of 8 Fire
- **WHEN** the instance is adjusted
- **THEN** the amount is 3

#### Scenario: Resistance 10 fire reduces 8 fire to 0

- **GIVEN** resistance 10 to Fire and an instance of 8 Fire
- **WHEN** the instance is adjusted
- **THEN** the amount is 0

#### Scenario: Two fire resistances use 10

- **GIVEN** resistance 5 to Fire, resistance 10 to Fire, and an instance of 12 Fire
- **WHEN** the instance is adjusted
- **THEN** the amount is 2

#### Scenario: Slashing resistance does not reduce fire

- **GIVEN** resistance 10 to Slashing and an instance of 8 Fire
- **WHEN** the instance is adjusted
- **THEN** the amount is 8

### Requirement: The roll is adjusted after the floor and the double

The penalty floor of 1 and critical doubling SHALL stay in the existing damage roll, before this adjustment. Resistance MAY reduce that result to 0. Weakness and resistance SHALL NOT themselves be doubled.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2302 (Player Core pg. 406, Step 1: Roll Damage Dice)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2307 (Player Core pg. 407, Doubling and Halving)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2318 (Player Core pg. 408, Resistance)

#### Scenario: A floored 1 is resisted to 0

- **GIVEN** a damage roll whose penalties reduce 4 to the floor of 1, and resistance 5 to Slashing
- **WHEN** that non-critical Slashing result is adjusted
- **THEN** the amount is 0

#### Scenario: A critical 6 fire with weakness 5 deals 17

- **GIVEN** a Fire damage roll of 6 that doubles to 12, and weakness 5 to Fire
- **WHEN** the doubled result is adjusted
- **THEN** the amount is 17

#### Scenario: A critical 6 fire with resistance 5 deals 7

- **GIVEN** a Fire damage roll of 6 that doubles to 12, and resistance 5 to Fire
- **WHEN** the doubled result is adjusted
- **THEN** the amount is 7

### Requirement: An overflowing adjustment writes nothing

An adjustment that does not fit in i32 SHALL return DamageOverflow. Hit Points, shield Hit Points, the reaction, actions, and conditions SHALL stay unchanged. Dice already drawn SHALL stay drawn.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2317 (Player Core pg. 408, Weakness)

#### Scenario: Weakness on the maximum amount overflows

- **GIVEN** weakness 1 to Fire, an instance of 2147483647 Fire, current Hit Points 10, and a scripted natural 8
- **WHEN** the instance is adjusted
- **THEN** the error is DamageOverflow
- **AND** current Hit Points stay 10 and the scripted 8 is unused

#### Scenario: Two maximum persistent amounts overflow the end step

- **GIVEN** persistent Fire 2147483647 and persistent Bleed 2147483647, current Hit Points 10, and a scripted natural 8
- **WHEN** that creature's turn ends
- **THEN** the error is DamageOverflow
- **AND** Hit Points stay 10, both conditions remain, the turn stays open, and the scripted 8 is unused

### Requirement: The same adjustment matches

The same amount, type, and defenses SHALL adjust to the same result on a second call. No die SHALL be drawn.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2309 (Player Core pg. 407, Step 3)

#### Scenario: The same fire adjustment matches

- **GIVEN** weakness 5 to Fire, resistance 3 to Fire, and an instance of 8 Fire
- **WHEN** the instance is adjusted twice
- **THEN** both results are 10

### Requirement: A hit uses the profile type

A hit SHALL adjust the damage roll by the Strike profile's damage type and the target's defenses before Shield Block and before the blow. A miss SHALL NOT adjust and SHALL NOT roll a damage die. Reactive Strike and a readied Strike SHALL use that same adjustment. Empty defenses SHALL leave the rolled total unchanged.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2301 (Player Core pg. 406, Damage Rolls)
**Source**: https://2e.aonprd.com/Actions.aspx?ID=2306 (Player Core pg. 418, Strike)

#### Scenario: A fire Strike adds weakness 5

- **GIVEN** a target at 20 Hit Points with weakness 5 to Fire, Armor Class 11, and an attacker whose Fire profile is 1d8 with bonus 0
- **WHEN** the Strike hits on a scripted natural 11 and a damage die of 4
- **THEN** the target has 11 Hit Points

#### Scenario: Empty defenses deal the rolled total

- **GIVEN** a target at 20 Hit Points with no defenses, Armor Class 11, and a Slashing profile of 1d8 with bonus 0
- **WHEN** the Strike hits on a scripted natural 11 and a damage die of 8
- **THEN** the target has 12 Hit Points

#### Scenario: A miss does not apply weakness

- **GIVEN** a target at 20 Hit Points with weakness 5 to Fire and Armor Class 11
- **WHEN** the Strike misses on a scripted natural 2, with a damage die of 4 next in the script
- **THEN** the target stays at 20 Hit Points and the damage die is unused

#### Scenario: A readied fire hit adds weakness 5

- **GIVEN** a readied Fire Strike whose damage roll is 6, and a target at 20 Hit Points with weakness 5 to Fire
- **WHEN** that readied Strike hits
- **THEN** the target has 9 Hit Points

#### Scenario: A reactive fire hit subtracts resistance 5

- **GIVEN** a Reactive Strike whose Fire damage roll is 8, and a target at 20 Hit Points with resistance 5 to Fire
- **WHEN** that Reactive Strike hits
- **THEN** the target has 17 Hit Points

### Requirement: A zero Strike skips the blow

The Strike path SHALL skip the blow when the adjusted amount is 0, as a basic save of 0 does. Hit Points and dying SHALL stay unchanged.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2327 (Player Core pg. 411, Taking Damage)

#### Scenario: Dying 1 and immune fire 8 stays dying 1

- **GIVEN** uses_dying_rules, dying 1, 0 Hit Points, immunity to Fire, Armor Class 11, and a Fire Strike of 1d8 with bonus 0
- **WHEN** the Strike hits on a scripted natural 11 and a damage die of 8
- **THEN** Hit Points stay 0 and the vital stays Dying(1)

#### Scenario: Dying 1 and resisted fire 8 stays dying 1

- **GIVEN** uses_dying_rules, dying 1, 0 Hit Points, resistance 10 to Fire, Armor Class 11, and a Fire Strike of 1d8 with bonus 0
- **WHEN** the Strike hits on a scripted natural 11 and a damage die of 8
- **THEN** Hit Points stay 0 and the vital stays Dying(1)

### Requirement: A basic save is not a typed instance

`basic_save` SHALL NOT take a damage type and SHALL NOT apply immunity, weakness, or resistance. The listed amount SHALL be the amount the blow receives.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2297 (Player Core pg. 404, Basic Saving Throws)

#### Scenario: Fire resistance does not reduce a basic save

- **GIVEN** a defender at 20 Hit Points with resistance 10 to Fire, Fortitude, level 1, Constitution modifier +0, and untrained
- **WHEN** a basic Fortitude save fails on a scripted natural 6 against DC 15 for listed damage 5
- **THEN** the damage is 5 and Hit Points are 15

### Requirement: This change owns the persistent container

This change owns the condition container. Persistent damage SHALL be one entry of a damage type and a positive amount. The end-of-turn hook SHALL run before any decrease. An entry SHALL NOT end by counting rounds.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=86 (Player Core pg. 445, Persistent Damage)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2430 (Player Core pg. 436, Step 3: End Your Turn)

#### Scenario: A fresh creature has no persistent damage

- **GIVEN** a creature derived with empty defenses and a new combatant state
- **WHEN** the state is read
- **THEN** it has no persistent damage condition

### Requirement: Gaining persistent damage stores one entry per type

The amount SHALL be at least 1. A non-positive amount SHALL return NonPositivePersistent and SHALL NOT change conditions. An unknown id SHALL return UnknownCombatant. A dead combatant SHALL return TargetDead. The call SHALL NOT spend an action or a reaction and SHALL NOT roll.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=86 (Player Core pg. 445, Persistent Damage)

#### Scenario: Fire 5 is stored

- **GIVEN** an Alive combatant with no persistent damage
- **WHEN** persistent Fire 5 is gained
- **THEN** the condition is Fire 5, actions stay 3, and the reaction stays ready

#### Scenario: A non-positive amount is rejected

- **GIVEN** an Alive combatant with no persistent damage and a scripted natural 8
- **WHEN** persistent Fire 0 is gained
- **THEN** the error is NonPositivePersistent
- **AND** no persistent damage is stored and the scripted 8 is unused

#### Scenario: An unknown combatant is rejected

- **GIVEN** an encounter that has no CombatantId(9)
- **WHEN** persistent Fire 5 is gained for CombatantId(9)
- **THEN** the error is UnknownCombatant

#### Scenario: A dead combatant cannot gain it

- **GIVEN** a dead combatant
- **WHEN** persistent Fire 5 is gained
- **THEN** the error is TargetDead
- **AND** no persistent damage is stored

### Requirement: A caller doubles persistent damage on a critical hit

On a critical hit the caller SHALL pass twice the listed persistent amount. The gain SHALL store that amount and SHALL NOT double it. Twice 2 SHALL be stored as 4, not as 6.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=86 (Player Core pg. 445, Persistent Damage)

#### Scenario: A critical persistent fire 2 is stored as 4

- **GIVEN** an Alive combatant with no persistent damage and a critical hit listing persistent Fire 2
- **WHEN** the caller gains persistent Fire 4
- **THEN** the stored amount is 4

### Requirement: The higher amount of one type replaces the lower

A second gain of the same type SHALL replace the stored amount only when the new amount is strictly higher. A lower or equal amount SHALL leave the stored amount. A different type SHALL be stored as well.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=86 (Player Core pg. 445, Multiple Persistent Damage Conditions)

#### Scenario: Fire 8 replaces fire 5

- **GIVEN** persistent Fire 5
- **WHEN** persistent Fire 8 is gained
- **THEN** the only Fire condition is 8

#### Scenario: Fire 5 does not replace fire 8

- **GIVEN** persistent Fire 8
- **WHEN** persistent Fire 5 is gained
- **THEN** the Fire condition stays 8

#### Scenario: An equal fire amount stays

- **GIVEN** persistent Fire 5
- **WHEN** persistent Fire 5 is gained again
- **THEN** the Fire condition stays 5

#### Scenario: Fire and bleed are both stored

- **GIVEN** persistent Fire 5
- **WHEN** persistent Bleed 3 is gained
- **THEN** Fire 5 and Bleed 3 are both stored

### Requirement: End of turn deals persistent damage as one blow

The end step SHALL adjust the active creature's persistent conditions and, when the sum is positive, apply one ordinary blow of that sum. The blow SHALL NOT be a critical hit. Several types SHALL increase dying once. The initiative order SHALL stay unchanged.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2430 (Player Core pg. 436, End Your Turn)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2327 (Player Core pg. 411, damage while dying)

#### Scenario: Fire 5 deals 5 then waits on the flat check

- **GIVEN** an Alive creature at 20 Hit Points with persistent Fire 5, no defenses, and a scripted natural 14
- **WHEN** that creature's turn ends
- **THEN** Hit Points are 15, Fire 5 remains, and actions are 0

#### Scenario: Fire and bleed deal 8 as one blow

- **GIVEN** an Alive creature at 20 Hit Points with persistent Fire 5 and persistent Bleed 3, and scripted naturals 10 then 10
- **WHEN** that creature's turn ends
- **THEN** Hit Points are 12
- **AND** both conditions are still stored

#### Scenario: Two types while dying 1 increase dying once

- **GIVEN** uses_dying_rules, dying 1, 0 Hit Points, persistent Fire 5, and persistent Bleed 5, with scripted naturals 10 then 10
- **WHEN** that creature's turn ends
- **THEN** the vital is Dying(2), not Dying(3) or Dead

#### Scenario: Persistent damage to 0 sets dying 1

- **GIVEN** uses_dying_rules, Alive at 5 Hit Points, maximum 20, persistent Fire 5, and a scripted natural 10
- **WHEN** that creature's turn ends
- **THEN** Hit Points are 0, the vital is Dying(1), and the order is unchanged

#### Scenario: Without dying rules, 0 Hit Points is death

- **GIVEN** uses_dying_rules false, Alive at 5 Hit Points, persistent Fire 5, and a scripted natural 10
- **WHEN** that creature's turn ends
- **THEN** the creature is Dead and no persistent damage remains
- **AND** the scripted 10 is unused

#### Scenario: Temporary Hit Points are spent once

- **GIVEN** Alive at 10 Hit Points with 3 temporary Hit Points, persistent Fire 2, and persistent Bleed 2, with scripted naturals 10 then 10
- **WHEN** that creature's turn ends
- **THEN** temporary Hit Points are 0 and current Hit Points are 9

### Requirement: Persistent damage does not use Shield Block

The persistent blow SHALL NOT use Shield Block. Another creature's persistent damage SHALL NOT tick on this end step. A knockout report SHALL be cleared.

**Source**: https://2e.aonprd.com/Feats.aspx?ID=5212 (Player Core pg. 262, Shield Block)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2430 (Player Core pg. 436, Step 3: End Your Turn)

#### Scenario: Persistent slashing leaves the shield and the reaction

- **GIVEN** Alive at 30 Hit Points, a ready reaction, a raised steel shield at 20 Hit Points, persistent Slashing 5, and a scripted natural 10
- **WHEN** that creature's turn ends
- **THEN** Hit Points are 25, shield Hit Points stay 20, and the reaction stays ready

#### Scenario: The other creature's fire does not tick

- **GIVEN** CombatantId(1)'s turn is open, CombatantId(2) has persistent Fire 5 at 20 Hit Points, and a scripted natural 10
- **WHEN** CombatantId(1)'s turn ends
- **THEN** CombatantId(2) stays at 20 Hit Points with Fire 5, and the scripted 10 is unused

### Requirement: A zero sum still rolls the flat checks

A sum of 0 SHALL NOT apply a blow and SHALL NOT change Hit Points or dying. Each remaining condition SHALL still roll its flat check. The GM call that negating the initial damage also negates persistent damage is out of scope.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=86 (Player Core pg. 445, Persistent Damage)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2318 (Player Core pg. 408, Resistance)

#### Scenario: Resistance 10 leaves Hit Points unchanged

- **GIVEN** Alive at 20 Hit Points, resistance 10 to Fire, persistent Fire 5, and a scripted natural 14
- **WHEN** that creature's turn ends
- **THEN** Hit Points stay 20 and Fire 5 remains

#### Scenario: Immune fire 5 still rolls the flat check

- **GIVEN** Alive at 20 Hit Points, immunity to Fire, persistent Fire 5, and a scripted natural 15
- **WHEN** that creature's turn ends
- **THEN** Hit Points stay 20 and no persistent damage remains

### Requirement: Each condition then rolls DC 15

After the blow, each remaining condition on a creature that is not Dead SHALL roll the existing DC 15 flat check. Success and critical success SHALL end that condition. Failure and critical failure SHALL keep the amount.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=86 (Player Core pg. 445, Persistent Damage)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2306 (Player Core pg. 406, Persistent Damage)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2278 (Player Core pg. 400, Flat Checks)

#### Scenario: A natural 15 ends fire after the damage

- **GIVEN** Alive at 20 Hit Points with persistent Fire 5 and a scripted natural 15
- **WHEN** that creature's turn ends
- **THEN** Hit Points are 15 and no persistent damage remains

#### Scenario: A natural 14 keeps fire

- **GIVEN** Alive at 20 Hit Points with persistent Fire 5 and a scripted natural 14
- **WHEN** that creature's turn ends
- **THEN** Hit Points are 15 and Fire 5 remains

#### Scenario: A natural 20 ends fire

- **GIVEN** Alive at 20 Hit Points with persistent Fire 5 and a scripted natural 20
- **WHEN** that creature's turn ends
- **THEN** Hit Points are 15 and no persistent damage remains

#### Scenario: A natural 1 keeps fire

- **GIVEN** Alive at 20 Hit Points with persistent Fire 5 and a scripted natural 1
- **WHEN** that creature's turn ends
- **THEN** Hit Points are 15 and Fire 5 remains

### Requirement: Flat checks follow the damage type order

Present conditions SHALL roll in this order: Bludgeoning, Piercing, Slashing, Bleed, Acid, Cold, Electricity, Fire, Sonic, Force, Spirit, Mental, Poison. An absent type SHALL NOT roll.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=86 (Player Core pg. 445, Multiple Persistent Damage Conditions)

#### Scenario: Bleed is checked before fire

- **GIVEN** Alive at 20 Hit Points with persistent Fire 5 and persistent Bleed 3, and scripted naturals 15 then 10
- **WHEN** that creature's turn ends
- **THEN** Hit Points are 12, Bleed is gone, and Fire 5 remains

### Requirement: Death clears persistent damage before the flat check

If the creature is already Dead when the end step starts, persistent damage SHALL be cleared and no die SHALL be drawn. If the blow leaves the creature Dead, persistent damage SHALL be cleared and no flat-check die SHALL be drawn. A tick that raises dying 3 SHALL leave the creature Dead.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2329 (Player Core pg. 411, Death)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=86 (Player Core pg. 445, Persistent Damage)

#### Scenario: Massive persistent damage draws no flat check

- **GIVEN** Alive at 9 Hit Points, maximum 9, persistent Fire 20, and a scripted natural 15
- **WHEN** that creature's turn ends
- **THEN** the creature is Dead, no persistent damage remains, and the scripted 15 is unused

#### Scenario: Dying 3 and persistent bleed 2 draws no flat check

- **GIVEN** uses_dying_rules, dying 3, 0 Hit Points, persistent Bleed 2, and a scripted natural 15
- **WHEN** that creature's turn ends
- **THEN** the creature is Dead, no persistent damage remains, and the scripted 15 is unused

#### Scenario: A dead creature's end step draws nothing

- **GIVEN** the active creature is already Dead, had persistent Fire 5, and a scripted natural 15 is next
- **WHEN** that turn ends
- **THEN** no persistent damage remains and the scripted 15 is unused

### Requirement: Bleed ends at full Hit Points

Healing that sets current Hit Points to the maximum SHALL remove persistent bleed and SHALL leave every other persistent condition. A heal that leaves current Hit Points below the maximum SHALL leave bleed.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2308 (Player Core pg. 407, Bleed Damage)

#### Scenario: A heal to maximum removes bleed and keeps fire

- **GIVEN** maximum Hit Points 20, current Hit Points 10, persistent Bleed 3, and persistent Fire 5
- **WHEN** the creature is healed by 10
- **THEN** current Hit Points are 20, Bleed is gone, and Fire 5 remains

#### Scenario: A short heal leaves bleed

- **GIVEN** maximum Hit Points 20, current Hit Points 10, and persistent Bleed 3
- **WHEN** the creature is healed by 4
- **THEN** current Hit Points are 14 and Bleed 3 remains

### Requirement: Assisted recovery is not a check

This capability SHALL NOT roll any flat check except the end-of-turn DC 15 check. It SHALL NOT use DC 10 and SHALL NOT spend actions to remove persistent damage.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=86 (Player Core pg. 445, Assisted Recovery)

#### Scenario: The end step rolls one d20 and spends no action

- **GIVEN** an open turn with 0 actions, persistent Fire 5, Hit Points 20, and a script holding 14 then 6
- **WHEN** the turn ends
- **THEN** Hit Points are 15, Fire 5 remains, actions stay 0, and the scripted 6 is unused

### Requirement: Defenses are validated and copied

A weakness or resistance value below 1 SHALL be InvalidDefenses, and that build SHALL NOT be derived. An empty defense list SHALL be valid. derive SHALL copy the strike's damage type and the defense lists, SHALL NOT roll, and SHALL NOT recompute them. The same build twice SHALL match.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2312 (Player Core pg. 408, Immunity, Weakness, and Resistance)

#### Scenario: Weakness 0 is rejected

- **GIVEN** a build whose other numbers are valid and whose Fire weakness is 0
- **WHEN** the build is validated
- **THEN** the result is InvalidDefenses and no Creature is derived

#### Scenario: A negative resistance is rejected

- **GIVEN** a build whose other numbers are valid and whose Fire resistance is -1
- **WHEN** the build is validated
- **THEN** the result is InvalidDefenses and no Creature is derived

#### Scenario: The strike type and the defenses are kept

- **GIVEN** a melee strike of type Slashing and resistance 5 to Slashing
- **WHEN** derive is called twice
- **THEN** both Creatures have a Slashing strike and resistance 5 to Slashing, and they match
