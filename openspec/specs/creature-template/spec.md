# creature-template Specification

## Purpose
Builds a character by composition: immutable inputs derive a read-only creature, and combat keeps its own state.

## Requirements

### Requirement: CharacterBuild holds immutable inputs

A CharacterBuild SHALL be the immutable inputs for one character. Phase 1 inputs SHALL be the baked stat block plus stealth DC, Aid modifier, shield, and uses_dying_rules. The build SHALL be able to gain ancestry, class, feat, attribute, and proficiency later as further sources. Derivation and combat SHALL NOT modify the build.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2319 (Player Core, Hit Points)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2285 (Player Core pg. 401, DC from a modifier)

#### Scenario: The build is unchanged by derive

- **GIVEN** a CharacterBuild with 20 Hit Points, Armor Class 18, Perception +5, and Speed 25
- **WHEN** derive is called
- **THEN** the build still compares equal to its value from before the call

#### Scenario: Combat does not write the build

- **GIVEN** that build, its derived Creature, and a CombatantState for CombatantId(0)
- **WHEN** the state's current Hit Points change
- **THEN** the CharacterBuild still compares equal to its original value

### Requirement: derive is pure and deterministic

derive SHALL be a pure function of the CharacterBuild. It SHALL NOT roll Dice, perform I/O, or read global state. Two calls on one build SHALL return equal Creatures. A clone of that build, and any other build that compares equal to it, SHALL derive an equal Creature.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2319 (Player Core, Hit Points are fixed by the creature, not by a roll)

#### Scenario: Calling derive twice matches

- **GIVEN** one CharacterBuild and a scripted d20 holding 8
- **WHEN** derive is called twice
- **THEN** the two Creatures are equal and the scripted 8 is still unused

#### Scenario: A clone and an equal build match

- **GIVEN** a CharacterBuild, a clone of it, and a second build with the same inputs so the two compare equal
- **WHEN** derive is called on each
- **THEN** all three Creatures are equal

### Requirement: Derivation folds ordered contributions

derive SHALL fold an ordered list of contributions taken from the CharacterBuild. Phase 1 SHALL supply only the base contribution, which carries the authored statistics. A later contribution's numbers SHALL be applied after the base, in list order. Callers SHALL still call derive with only the build.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2319 (Player Core, Hit Points are authored on the creature, then later sources add to them)

#### Scenario: Phase 1 folds only the base

- **GIVEN** a Phase 1 CharacterBuild with Armor Class 18
- **WHEN** derive is called with that build alone
- **THEN** the contribution list is the base only and the Creature's Armor Class is 18

#### Scenario: A later contribution is folded after the base

- **GIVEN** an ordered list whose base contribution has Armor Class 18 and whose next contribution adds +1 Armor Class
- **WHEN** those contributions are folded
- **THEN** Armor Class is 19, the base was applied first, and derive's caller still passes only a CharacterBuild

### Requirement: Derived statistics and one melee strike

derive SHALL put the authored Hit Points, Armor Class, Perception modifier, and Speed in feet on the Creature, and SHALL include exactly one melee strike. The strike SHALL record attack bonus, damage dice, die faces, damage bonus, reach in feet, and an agile flag. Those numbers SHALL NOT be recomputed from level or from roster size.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2319 (Player Core, Hit Points)
**Source**: https://2e.aonprd.com/Actions.aspx?ID=2306 (Player Core pg. 418, Strike)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2301 (Player Core pg. 406, Damage Rolls)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2289 (Player Core pg. 402, agile)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2345 (Player Core pg. 420, Speed)

#### Scenario: Authored numbers are kept

- **GIVEN** a build with 20 Hit Points, Armor Class 18, Perception modifier +5, and Speed 25
- **WHEN** derive is called
- **THEN** the Creature has those four numbers

#### Scenario: Strike fields are kept

- **GIVEN** a melee strike with attack bonus +7, damage 1d8+4, reach 5 feet, and agile false
- **WHEN** derive is called
- **THEN** the Creature's strike has attack bonus +7, 1 die of 8 faces, bonus +4, reach 5, and agile false

#### Scenario: The agile flag is kept

- **GIVEN** a melee strike with agile true, attack bonus +6, damage 1d6+2, and reach 5 feet
- **WHEN** derive is called
- **THEN** the agile flag is true and the other strike numbers are unchanged

### Requirement: Shield bonus is stored per shield

An optional shield on the build SHALL carry hardness, Hit Points, broken threshold, and that shield's own AC bonus through derive onto the Creature. The AC bonus SHALL remain the stored value. derive SHALL NOT replace it with a flat +2. Shield Hit Points on the Creature SHALL be the maximum, not a runtime value.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2181 (Player Core pg. 274, Shield Statistics)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2281 (Player Core pg. 400, a buckler grants +1)

#### Scenario: A buckler bonus stays +1

- **GIVEN** a shield with hardness 3, Hit Points 8, broken threshold 4, and AC bonus +1
- **WHEN** derive is called
- **THEN** the Creature's shield numbers are 3, 8, 4, and +1

#### Scenario: A second shield keeps its own bonus

- **GIVEN** another build whose shield has hardness 5, Hit Points 20, broken threshold 10, and AC bonus +2
- **WHEN** derive is called on each build
- **THEN** the bonuses are +1 and +2

#### Scenario: No shield is omitted

- **GIVEN** a build with no shield
- **WHEN** derive is called
- **THEN** the Creature has no shield and no AC bonus is invented

### Requirement: Optional stealth DC

An optional stealth DC SHALL equal 10 plus the creature's Stealth modifier. derive SHALL copy that DC onto the Creature. A Seek check SHALL use the Creature's DC and SHALL NOT change the Creature. Omitting the field SHALL leave no Stealth DC.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2301 (Player Core pg. 417, Seek)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2285 (Player Core pg. 401, DC = 10 + modifier)

#### Scenario: Stealth modifier plus 10

- **GIVEN** a Stealth modifier of +7
- **WHEN** the stealth DC is placed on the build and derive is called
- **THEN** the Creature's stealth DC is 17

#### Scenario: Seek uses the derived DC

- **GIVEN** a Creature with stealth DC 17, Perception modifier +5, and a scripted natural 12
- **WHEN** the Seek check is resolved against that DC
- **THEN** the total is 17, the degree is success, and the Creature is unchanged

#### Scenario: Omitted stealth DC stores nothing

- **GIVEN** a build that omits stealth DC
- **WHEN** derive is called
- **THEN** the Creature has no Stealth DC

### Requirement: Aid modifier against DC 15

When an Aid modifier is on the build, derive SHALL copy it onto the Creature. An Aid check SHALL add that modifier to a d20 against DC 15 and SHALL NOT change the Creature. Success SHALL grant +1 circumstance. Failure SHALL grant 0. Critical failure SHALL grant -1 circumstance. Critical success SHALL grant +2 circumstance. The build SHALL NOT store a prepared-to-help flag.

**Rule**: Player Core Aid gives +2 circumstance on a critical success, +3 at master, and +4 at legendary. https://2e.aonprd.com/Actions.aspx?ID=2292
**Why**: The baked modifier has no proficiency rank, so master and legendary cannot be chosen.
**Substitute**: Critical success on this Aid check always grants +2 circumstance.
**Intent**: A critical success still helps more than a success, at the usual trained value.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2292 (Player Core pg. 416, Aid)

#### Scenario: Success grants +1

- **GIVEN** an aid modifier of +7 and a scripted natural 10
- **WHEN** the Aid check is resolved against DC 15
- **THEN** the total is 17, the degree is success, the circumstance bonus is +1, and the Creature is unchanged

#### Scenario: Critical success grants +2

- **GIVEN** an aid modifier of +7 and a scripted natural 18
- **WHEN** the Aid check is resolved against DC 15
- **THEN** the total is 25, the degree is critical success, and the circumstance bonus is +2

#### Scenario: Failure grants 0

- **GIVEN** an aid modifier of +7 and a scripted natural 4
- **WHEN** the Aid check is resolved against DC 15
- **THEN** the total is 11, the degree is failure, and the circumstance bonus is 0

#### Scenario: Critical failure grants -1

- **GIVEN** an aid modifier of +4 and a scripted natural 1
- **WHEN** the Aid check is resolved against DC 15
- **THEN** the total is 5, the degree is critical failure, and the circumstance penalty is -1

#### Scenario: A natural 20 uses the fixed critical bonus

- **GIVEN** an aid modifier of +0 and a scripted natural 20
- **WHEN** the Aid check is resolved against DC 15
- **THEN** the numeric total 20 is a success, the natural 20 makes it a critical success, and the circumstance bonus is +2

#### Scenario: No prepared flag is required

- **GIVEN** a build with aid modifier +6 and no prepared-to-help value
- **WHEN** derive is called
- **THEN** the Creature has aid modifier +6 and does not record a prepared-to-help state

### Requirement: Validation rejects impossible statistics

Validation of a CharacterBuild SHALL reject Hit Points of 0 or less and SHALL reject a negative Speed. Speed 0 SHALL be accepted. A rejected build SHALL NOT be derived.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2324 (Player Core pg. 410, creatures are not reduced below 0 Hit Points)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2345 (Player Core pg. 420, Speed)

#### Scenario: Zero Hit Points is rejected

- **GIVEN** inputs whose other numbers are valid and whose Hit Points are 0
- **WHEN** the build is validated
- **THEN** validation rejects it

#### Scenario: Negative Hit Points are rejected

- **GIVEN** inputs whose Hit Points are -1
- **WHEN** the build is validated
- **THEN** validation rejects it

#### Scenario: Negative Speed is rejected

- **GIVEN** inputs whose Speed is -5 and whose Hit Points are 12
- **WHEN** the build is validated
- **THEN** validation rejects it

#### Scenario: Speed zero is accepted

- **GIVEN** inputs with 12 Hit Points and Speed 0
- **WHEN** the build is validated and derived
- **THEN** the Creature's Speed is 0

### Requirement: CombatantState is separate from the creature

Current Hit Points, temporary Hit Points, conditions, dying, wounded, shield Hit Points, raised shield, actions, and the reaction SHALL live on CombatantState. That state SHALL be keyed by CombatantId(u32) and SHALL be initialized from a Creature. Fresh states for the same Creature and CombatantId SHALL be equal. Mutating a state SHALL leave the Creature and the CharacterBuild unchanged. A roster SHALL have no fixed size, and an id SHALL NOT be limited to four slots.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2421 (Player Core, Encounter Mode)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2324 (Player Core pg. 410, Getting Knocked Out)

#### Scenario: Fresh states from one creature match

- **GIVEN** one Creature and CombatantId(7)
- **WHEN** two CombatantStates are initialized from that pair
- **THEN** the two states are equal, current Hit Points equal the Creature's Hit Points, temporary Hit Points are 0, and the shield is not raised

#### Scenario: Mutating state leaves the creature unchanged

- **GIVEN** a Creature with 20 Hit Points and a CombatantState initialized from it
- **WHEN** the state's current Hit Points become 12
- **THEN** the Creature still has 20 Hit Points and compares equal to its value from before the change

#### Scenario: Two ids can share one creature

- **GIVEN** one Creature
- **WHEN** states are created for CombatantId(0) and CombatantId(4000000000)
- **THEN** both states exist, their ids are those values, and the Creature is unchanged

#### Scenario: More than four states are valid

- **GIVEN** one Creature
- **WHEN** six states are created with CombatantId(0) through CombatantId(5)
- **THEN** the roster length is 6

### Requirement: uses_dying_rules defaults to death at 0 Hit Points

uses_dying_rules SHALL be a build input and SHALL default to false when omitted. derive SHALL copy it onto the Creature and SHALL NOT roll Dice. At 0 current Hit Points, a false flag SHALL mark that CombatantState dead with no dying. A true flag SHALL record that dying applies, as encounter-core specifies, and SHALL NOT mark the state dead only for reaching 0. The Creature SHALL stay unchanged. This capability SHALL NOT set the dying value.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2324 (Player Core pg. 410, Getting Knocked Out)

#### Scenario: Omitting the flag defaults to false

- **GIVEN** a build that does not set uses_dying_rules
- **WHEN** derive is called
- **THEN** the Creature's uses_dying_rules is false

#### Scenario: Without the flag, 0 Hit Points is death on the state

- **GIVEN** a Creature with 9 Hit Points and uses_dying_rules false, a state at 9 current Hit Points, and a scripted d8 of 8 for damage 1d8+1
- **WHEN** that damage is applied to the state
- **THEN** the state has 0 current Hit Points, is dead, and has no dying condition, and the Creature still has 9 Hit Points

#### Scenario: With the flag, 0 Hit Points enters dying on the state

- **GIVEN** a Creature with 9 Hit Points and uses_dying_rules true, a state at 9 current Hit Points, and a scripted d8 of 8 for damage 1d8+1
- **WHEN** that damage is applied to the state
- **THEN** the state has 0 current Hit Points, is not dead, and gains dying as encounter-core specifies, and the Creature is unchanged

#### Scenario: Damage above 0 does not kill

- **GIVEN** a Creature with 9 Hit Points and uses_dying_rules false, a state at 9 current Hit Points, and a scripted d8 of 1 for damage 1d8+1
- **WHEN** that damage is applied to the state
- **THEN** the state has 7 current Hit Points and is not dead, and the Creature still has 9 Hit Points
