# creature-template

## MODIFIED Requirements

### Requirement: Derived statistics and one melee strike

derive SHALL put the authored Hit Points, Armor Class, Perception modifier, and Speed in feet on the Creature, and SHALL include exactly one melee strike. The strike SHALL record attack bonus, damage dice, die faces, damage bonus, reach in feet, an agile flag, and one damage type. Those numbers SHALL NOT be recomputed from level or from roster size.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2319 (Player Core pg. 410, Hit Points, Healing, and Dying)
**Source**: https://2e.aonprd.com/Actions.aspx?ID=2306 (Player Core pg. 418, Strike)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2301 (Player Core pg. 406, Damage Rolls)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2308 (Player Core pg. 407, Damage Types)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2289 (Player Core pg. 402, Multiple Attack Penalty)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2345 (Player Core pg. 420, Speed)

#### Scenario: Authored numbers are kept

- **GIVEN** a build with 20 Hit Points, Armor Class 18, Perception modifier +5, and Speed 25
- **WHEN** derive is called
- **THEN** the Creature has those four numbers

#### Scenario: Strike fields are kept

- **GIVEN** a melee strike with attack bonus +7, damage 1d8+4, reach 5 feet, agile false, and damage type Slashing
- **WHEN** derive is called
- **THEN** the Creature's strike has attack bonus +7, 1 die of 8 faces, bonus +4, reach 5, agile false, and damage type Slashing

#### Scenario: The agile flag is kept

- **GIVEN** a melee strike with agile true, attack bonus +6, damage 1d6+2, reach 5 feet, and damage type Piercing
- **WHEN** derive is called
- **THEN** the agile flag is true, the damage type is Piercing, and the other strike numbers are unchanged

### Requirement: Penalties leave at least 1 damage

When penalties on an attack reduce a damage total to 0 or below, the roll SHALL deal 1 damage. That floor SHALL apply only to penalties on an attack and SHALL be applied before critical doubling, so the smallest critical hit SHALL deal 2 damage. RAW does not state that order; this change applies DEV-14. An overflowing total SHALL return a typed error and SHALL NOT change Hit Points. Resistance is applied after that roll and MAY reduce the result to 0.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2302 (Player Core pg. 406, Step 1: Roll Damage Dice)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2307 (Player Core pg. 407, Doubling and Halving)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2318 (Player Core pg. 408, resistance floor of 0)
**Source**: DEV-14. RAW does not state whether the floor is before or after doubling; this change applies it first.

#### Scenario: Penalties reduce damage to 1

- **GIVEN** penalties on an attack, a rolled 4, and a penalty of -6
- **WHEN** the damage is resolved and the hit is not critical
- **THEN** the damage dealt is 1

#### Scenario: Minimum critical damage is 2

- **GIVEN** penalties on an attack, a rolled 4, and a penalty of -6
- **WHEN** the hit is critical
- **THEN** the damage dealt is 2

#### Scenario: An overflowing damage total returns an error

- **GIVEN** a rolled 1, a bonus of 2147483647, and a state at 9 current Hit Points
- **WHEN** that damage is applied
- **THEN** the result is a typed overflow error
- **AND** current Hit Points stay 9
