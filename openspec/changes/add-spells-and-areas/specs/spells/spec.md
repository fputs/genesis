# spells

## Purpose

Resolves Cast a Spell on the existing 5-foot grid: one spell attack modifier, the spell DC, burst, cone, line, and emanation membership, and two reference spells.

## ADDED Requirements

### Requirement: The same cast twice matches

The same caster, squares, and scripted dice SHALL return the same spell result. A call SHALL NOT read a clock, a file, or a global generator. A fallible call SHALL return a typed error and SHALL NOT panic. The rules crate SHALL gain no dependency.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2734 (Player Core pg. 299, Cast a Spell)

#### Scenario: The same projectile twice matches

- **GIVEN** two encounters, each with an open Alive caster at (0, 0) and an Alive target at (1, 0), both Medium, spellcasting level 1, attribute modifier +4, trained, baked Armor Class 16, empty modifiers, empty detection, and the script 10, 3, 4
- **WHEN** each casts Telekinetic Projectile at that target
- **THEN** both results are equal
- **AND** both totals are 17, both degrees are success, and both damages are 7
- **AND** each script is fully consumed

#### Scenario: The same cone twice matches

- **GIVEN** two encounters, each with an open Alive caster at (0, 0) and Alive CombatantId(2) at (1, 0), both Medium, the same trained spellcasting, Reflex +0, and the script 3, 4, 10
- **WHEN** each casts Breathe Fire east
- **THEN** both results are equal
- **AND** both rolled damages are 7 and both save degrees are failure
- **AND** each script is fully consumed

### Requirement: Cast a Spell spends two actions

On the caster's open Alive turn, each reference spell SHALL spend 2 actions. The cast SHALL NOT resolve reactions or spend a reaction, a slot, or a focus point.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2734 (Player Core pg. 299, Cast a Spell)

#### Scenario: Breathe Fire leaves one action

- **GIVEN** an open Alive turn with 3 actions, a cone with no creature inside it, and a scripted 3
- **WHEN** the caster casts Breathe Fire
- **THEN** 1 action remains
- **AND** the cost is 2 actions with concentrate, fire, and manipulate
- **AND** the cost does not name attack or ranged
- **AND** the reaction is still ready
- **AND** the scripted 3 is unused

#### Scenario: The projectile leaves one action

- **GIVEN** an open Alive turn with 3 actions, a target in range, and a scripted natural 10 that misses
- **WHEN** the caster casts Telekinetic Projectile
- **THEN** 1 action remains
- **AND** the cost is 2 actions with attack, cantrip, concentrate, and manipulate
- **AND** the reaction is still ready

### Requirement: The two spells name their traits

Breathe Fire's cost SHALL name concentrate, fire, and manipulate, and SHALL NOT name attack or ranged. Telekinetic Projectile's cost SHALL name attack, cantrip, concentrate, and manipulate, and SHALL NOT name ranged.

**Source**: https://2e.aonprd.com/Spells.aspx?ID=1457 (Player Core pg. 319, Breathe Fire)
**Source**: https://2e.aonprd.com/Spells.aspx?ID=1718 (Player Core pg. 363, Telekinetic Projectile)

#### Scenario: Breathe Fire has no attack trait

- **GIVEN** an open Alive turn and a cone with no creature inside it
- **WHEN** Breathe Fire is cast
- **THEN** the cost names concentrate, fire, and manipulate
- **AND** the cost does not name attack or ranged

#### Scenario: The projectile has the attack trait

- **GIVEN** an open Alive turn and a target in range
- **WHEN** Telekinetic Projectile is cast and misses
- **THEN** the cost names attack, cantrip, concentrate, and manipulate
- **AND** the cost does not name ranged

### Requirement: A cast off the open turn rolls nothing

A cast with no open turn SHALL return TurnNotOpen. A cast by a creature who is not the active combatant SHALL return NotYourTurn. A cast by a creature who is not Alive SHALL return NotConscious. An unknown caster SHALL return UnknownCombatant. Each SHALL spend nothing and draw no die.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2734 (Player Core pg. 299, Cast a Spell)

#### Scenario: Breathe Fire before the turn starts rolls nothing

- **GIVEN** initiative has been rolled, no turn is open, and a scripted 3
- **WHEN** Breathe Fire is attempted
- **THEN** the error is TurnNotOpen
- **AND** the scripted 3 is unused

#### Scenario: A projectile off the active turn rolls nothing

- **GIVEN** it is another combatant's open turn, and a scripted natural 10
- **WHEN** a combatant who is not active casts Telekinetic Projectile
- **THEN** the error is NotYourTurn
- **AND** the scripted 10 is unused

#### Scenario: An unconscious caster rolls nothing

- **GIVEN** the open creature is unconscious, with 3 actions and a scripted natural 10
- **WHEN** that creature casts Telekinetic Projectile
- **THEN** the error is NotConscious
- **AND** 3 actions remain
- **AND** the scripted 10 is unused

#### Scenario: An unknown caster rolls nothing

- **GIVEN** an open turn and a scripted 3
- **WHEN** a CombatantId that is not in the encounter casts Breathe Fire
- **THEN** the error is UnknownCombatant
- **AND** the scripted 3 is unused

### Requirement: Too few actions roll nothing

A cast that costs more actions than the caster has remaining SHALL return InsufficientActions. It SHALL spend nothing and draw no die. The turn SHALL stay open.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2734 (Player Core pg. 299, Cast a Spell)

#### Scenario: One action cannot cast either spell

- **GIVEN** an open Alive turn with 1 action, and a scripted natural 10
- **WHEN** Breathe Fire is attempted, and Telekinetic Projectile is attempted on a second encounter in the same state
- **THEN** each error is InsufficientActions
- **AND** each caster still has 1 action
- **AND** each scripted 10 is unused

### Requirement: Missing spellcasting rolls nothing

A creature with no spellcasting entry SHALL return NoSpellcasting. The cast SHALL spend nothing and draw no die. An omitted entry SHALL stay omitted.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2292 (Player Core pg. 403, Spell Attack Rolls)

#### Scenario: No spellcasting rolls nothing

- **GIVEN** an open Alive turn with 3 actions, no spellcasting entry, and a scripted natural 10
- **WHEN** Telekinetic Projectile is attempted
- **THEN** the error is NoSpellcasting
- **AND** 3 actions remain
- **AND** the scripted 10 is unused

### Requirement: The spell attack modifier and the spell DC

The modifier SHALL be the attribute modifier, plus that level's proficiency bonus, plus SpellAttack modifiers. Untrained SHALL be +0. The other ranks SHALL be level plus 2, 4, 6, or 8. The DC SHALL be 10 plus the modifier. The attribute SHALL be a modifier, not a score. The same inputs SHALL match and draw no die.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2292 (Player Core pg. 403, Spell Attack Rolls)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2293 (Player Core pg. 403, Spell DC)

#### Scenario: Trained at level 1 with plus 4 is plus 7 and DC 17

- **GIVEN** level 1, attribute modifier +4, trained, and no other modifier
- **WHEN** the spell attack modifier and spell DC are resolved twice
- **THEN** both modifiers are +7 and both DCs are 17
- **AND** no die is drawn

#### Scenario: Expert at level 5 with plus 3 is plus 12 and DC 22

- **GIVEN** level 5, attribute modifier +3, expert, and no other modifier
- **WHEN** the spell attack modifier and spell DC are resolved
- **THEN** the modifier is +12 and the DC is 22

#### Scenario: Untrained at level 20 with plus 4 is plus 4 and DC 14

- **GIVEN** level 20, attribute modifier +4, untrained, and no other modifier
- **WHEN** the spell attack modifier and spell DC are resolved
- **THEN** the modifier is +4 and the DC is 14

#### Scenario: A negative attribute lowers the DC

- **GIVEN** level 1, attribute modifier -1, trained, and no other modifier
- **WHEN** the spell attack modifier and spell DC are resolved
- **THEN** the modifier is +2 and the DC is 12

### Requirement: SpellAttack does not share the Attack stack

A modifier on Attack SHALL NOT change the spell attack modifier or the spell DC. A modifier on SpellAttack SHALL change both. Two circumstance bonuses on SpellAttack SHALL keep the highest. A sum that does not fit SHALL return an overflow and draw no die.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2292 (Player Core pg. 403, Spell Attack Rolls)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2281 (Player Core pg. 400, Bonuses)

#### Scenario: An Attack bonus does not raise the spell DC

- **GIVEN** level 1, attribute modifier +4, trained, and a circumstance bonus of +2 on Attack
- **WHEN** the spell attack modifier and spell DC are resolved
- **THEN** the modifier is +7 and the DC is 17

#### Scenario: A SpellAttack status bonus raises the roll and the DC

- **GIVEN** level 1, attribute modifier +4, trained, and a status bonus of +1 on SpellAttack
- **WHEN** the spell attack modifier and spell DC are resolved
- **THEN** the modifier is +8 and the DC is 18

#### Scenario: Two circumstance bonuses keep the highest

- **GIVEN** level 1, attribute modifier +4, trained, and circumstance bonuses of +2 and +1 on SpellAttack
- **WHEN** the spell attack modifier and spell DC are resolved
- **THEN** the modifier is +9 and the DC is 19

#### Scenario: A modifier that does not fit draws nothing

- **GIVEN** level 1, attribute modifier 2147483647, trained, and a scripted natural 10
- **WHEN** the spell attack modifier is resolved
- **THEN** the error is an overflow
- **AND** the scripted 10 is unused

### Requirement: The multiple attack penalty hits the roll only

The attack trait SHALL apply the multiple attack penalty, and the spell SHALL NOT be agile. The spell DC SHALL NOT include that penalty. Breathe Fire SHALL NOT take the penalty and SHALL NOT change the attack count.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2289 (Player Core pg. 402, Multiple Attack Penalty)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2292 (Player Core pg. 403, Spell Attack Rolls)

#### Scenario: The second attack takes minus 5 and the DC stays 17

- **GIVEN** an open turn, attack count 1, a non-agile spell, modifier +7 before the penalty, target Armor Class 18, and a scripted natural 10
- **WHEN** Telekinetic Projectile is cast
- **THEN** the penalty is -5 and the total is 12
- **AND** the degree is failure
- **AND** the spell DC is 17
- **AND** the attack count is 2

#### Scenario: An agile Strike does not make the spell agile

- **GIVEN** an open turn, attack count 1, an agile Strike on the same creature, modifier +7 before the penalty, and a scripted natural 10
- **WHEN** Telekinetic Projectile is cast
- **THEN** the penalty is -5
- **AND** the total is 12

#### Scenario: Breathe Fire leaves the attack count

- **GIVEN** an open turn, attack count 1, a cone with no creature inside it, and a scripted 3
- **WHEN** Breathe Fire is cast
- **THEN** the attack count is still 1
- **AND** the scripted 3 is unused

### Requirement: A spell attack uses baked Armor Class

The degree SHALL compare the total with the target's baked Armor Class. A caller modifier on Armor Class SHALL NOT change that degree. A miss or a critical failure SHALL roll no damage die and SHALL leave Hit Points unchanged.

**Source**: https://2e.aonprd.com/Spells.aspx?ID=1718 (Player Core pg. 363, Telekinetic Projectile)

#### Scenario: A natural 10 against Armor Class 16 is a success

- **GIVEN** modifier +7, baked Armor Class 16, an empty modifier list, and the script 10, 3, 4
- **WHEN** Telekinetic Projectile is cast
- **THEN** the total is 17 and the degree is success
- **AND** the script is fully consumed

#### Scenario: Armor Class in the caller list does not change the result

- **GIVEN** modifier +0, baked Armor Class 12, a caller circumstance penalty of -2 on Armor Class, and a scripted natural 10
- **WHEN** Telekinetic Projectile is cast
- **THEN** the total is 10
- **AND** the degree is failure
- **AND** the damage die is unused

#### Scenario: A natural 10 against Armor Class 18 rolls no damage die

- **GIVEN** modifier +7, baked Armor Class 18, 20 Hit Points, and the script 10, 3
- **WHEN** Telekinetic Projectile is cast
- **THEN** the total is 17 and the degree is failure
- **AND** the target still has 20 Hit Points
- **AND** the scripted 3 is unused

### Requirement: A hit rolls 2d6 and a critical hit doubles once

A hit or a critical hit SHALL roll 2d6 once and SHALL NOT add the attribute modifier. A hit SHALL apply that total as an ordinary blow. A critical hit SHALL double that total once and apply it as a critical blow.

**Source**: https://2e.aonprd.com/Spells.aspx?ID=1718 (Player Core pg. 363, Telekinetic Projectile)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2307 (Player Core pg. 407, Doubling and Halving Damage)

#### Scenario: A natural 10 against Armor Class 16 hits for 7

- **GIVEN** modifier +7, baked Armor Class 16, 20 Hit Points, and the script 10, 3, 4
- **WHEN** Telekinetic Projectile hits
- **THEN** the damage is 7
- **AND** the target has 13 Hit Points and is Alive

#### Scenario: A natural 20 doubles 7 to 14 once

- **GIVEN** modifier +7, baked Armor Class 18, 10 Hit Points, uses_dying_rules, and the script 20, 3, 4
- **WHEN** Telekinetic Projectile is cast
- **THEN** the total is 27 and the degree is critical success
- **AND** the damage is 14
- **AND** the target has 0 Hit Points and dying 2

### Requirement: Spell attack degrees use the one-step shift

A natural 20 at Armor Class minus 1 SHALL be a success and SHALL deal 2d6 once. A natural 1 at Armor Class plus 10 SHALL be a success and SHALL deal 2d6 once. A natural 1 equal to Armor Class SHALL be a failure and SHALL roll no damage die. A natural 1 one below Armor Class SHALL be a critical failure and SHALL roll no damage die.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2287 (Player Core pg. 401, Natural 1 and Natural 20)

#### Scenario: A natural 20 at Armor Class minus 1 is a success

- **GIVEN** modifier +0, baked Armor Class 21, 20 Hit Points, and the script 20, 3, 4
- **WHEN** Telekinetic Projectile is cast
- **THEN** the total is 20 and the degree is success
- **AND** the damage is 7
- **AND** the target has 13 Hit Points

#### Scenario: A natural 1 at Armor Class plus 10 is a success

- **GIVEN** modifier +29, baked Armor Class 20, 20 Hit Points, and the script 1, 3, 4
- **WHEN** Telekinetic Projectile is cast
- **THEN** the total is 30 and the degree is success
- **AND** the damage is 7
- **AND** the target has 13 Hit Points

#### Scenario: A natural 1 at Armor Class is a failure

- **GIVEN** modifier +19, baked Armor Class 20, 20 Hit Points, and the script 1, 3
- **WHEN** Telekinetic Projectile is cast
- **THEN** the total is 20 and the degree is failure
- **AND** the target still has 20 Hit Points
- **AND** the scripted 3 is unused

#### Scenario: A natural 1 one below Armor Class is a critical failure

- **GIVEN** modifier +18, baked Armor Class 20, 20 Hit Points, and the script 1, 3
- **WHEN** Telekinetic Projectile is cast
- **THEN** the total is 19 and the degree is critical failure
- **AND** the target still has 20 Hit Points
- **AND** the scripted 3 is unused

### Requirement: Both spells stay at rank 1

Each cast SHALL deal 2d6. A higher caster level SHALL NOT add a die. There SHALL be no rank argument.

**Source**: https://2e.aonprd.com/Spells.aspx?ID=1457 (Player Core pg. 319, Breathe Fire)
**Source**: https://2e.aonprd.com/Spells.aspx?ID=1718 (Player Core pg. 363, Telekinetic Projectile)

#### Scenario: A level 5 projectile still deals 7

- **GIVEN** level 5, attribute modifier +3, expert, baked Armor Class 16, 20 Hit Points, and the script 10, 3, 4
- **WHEN** Telekinetic Projectile is cast
- **THEN** the modifier is +12 and the damage is 7
- **AND** the target has 13 Hit Points
- **AND** the script is fully consumed

#### Scenario: A level 5 cone still deals 7

- **GIVEN** level 5, attribute modifier +3, expert, one creature in the cone with Reflex +0 and 20 Hit Points, and the script 3, 4, 13
- **WHEN** Breathe Fire is cast
- **THEN** the spell DC is 22 and the rolled damage is 7
- **AND** the degree is failure and the creature has 13 Hit Points
- **AND** the script is fully consumed

### Requirement: Range is 30 feet and draws nothing beyond it

Distance SHALL be the existing grid distance. 30 feet or less SHALL be in range, including the caster. One creature SHALL be legal, including an ally, and Side SHALL NOT be read. Farther SHALL return OutOfSpellRange and draw no die. There SHALL be no range increment and no line-of-effect test.

**Source**: https://2e.aonprd.com/Spells.aspx?ID=1718 (Player Core pg. 363, Telekinetic Projectile)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2379 (Player Core pg. 426, Range and Reach)

#### Scenario: Six orthogonal squares are in range

- **GIVEN** the caster at (0, 0) and the target at (6, 0), both Medium, modifier +7, Armor Class 18, and a scripted natural 10
- **WHEN** Telekinetic Projectile is cast
- **THEN** the cast is in range
- **AND** the total is 17
- **AND** there is no range penalty

#### Scenario: Seven orthogonal squares spend nothing

- **GIVEN** the caster at (0, 0) and the target at (7, 0), both Medium, 3 actions, and a scripted natural 10
- **WHEN** Telekinetic Projectile is attempted
- **THEN** the error is OutOfSpellRange
- **AND** 3 actions remain
- **AND** the scripted 10 is unused

#### Scenario: The caster may target their own square

- **GIVEN** the caster at (0, 0) targeting their own id, modifier +7, baked Armor Class 16, 20 Hit Points, and the script 10, 3, 4
- **WHEN** Telekinetic Projectile is cast
- **THEN** the degree is success and the damage is 7
- **AND** the caster has 13 Hit Points
- **AND** the target is that one creature

#### Scenario: An ally is a legal target

- **GIVEN** a Pc caster at (0, 0) and a Pc ally at (1, 0), modifier +7, baked Armor Class 16, 20 Hit Points, and the script 10, 3, 4
- **WHEN** Telekinetic Projectile is cast at that ally
- **THEN** the degree is success and the damage is 7
- **AND** the ally has 13 Hit Points

### Requirement: A dead or unknown target spends nothing

A dead target SHALL return TargetDead. An unknown target SHALL return UnknownCombatant. Each SHALL spend nothing and draw no die.

**Source**: https://2e.aonprd.com/Spells.aspx?ID=1718 (Player Core pg. 363, Telekinetic Projectile)

#### Scenario: A dead target spends nothing

- **GIVEN** an open turn with 3 actions, a dead target in range, and a scripted natural 10
- **WHEN** Telekinetic Projectile is attempted
- **THEN** the error is TargetDead
- **AND** 3 actions remain
- **AND** the scripted 10 is unused

#### Scenario: An unknown target spends nothing

- **GIVEN** an open turn with 3 actions and a scripted natural 10
- **WHEN** Telekinetic Projectile targets a CombatantId that is not in the encounter
- **THEN** the error is UnknownCombatant
- **AND** 3 actions remain
- **AND** the scripted 10 is unused

### Requirement: Concealment uses the existing flat check

A concealed or hidden target SHALL take the existing targeting flat check before the attack d20. A lost check SHALL spend the 2 actions, increment the attack count, and roll no attack d20 and no damage die. An undetected target SHALL return CannotTarget and spend nothing. Empty detection SHALL draw no flat-check die.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=62 (Player Core pg. 442, Concealed)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=79 (Player Core pg. 444, Hidden)

#### Scenario: A lost concealed check spends the cast

- **GIVEN** the target observed and concealed from the caster, 3 actions, attack count 0, and the script 4, 10
- **WHEN** Telekinetic Projectile is cast
- **THEN** 1 action remains and the attack count is 1
- **AND** no attack total is produced
- **AND** the scripted 10 is unused

#### Scenario: A concealed success then rolls the attack

- **GIVEN** the target observed and concealed from the caster, modifier +7, Armor Class 16, and the script 5, 10, 3, 4
- **WHEN** Telekinetic Projectile is cast
- **THEN** the attack total is 17 and the damage is 7
- **AND** the script is fully consumed

#### Scenario: Empty detection draws no flat check

- **GIVEN** an empty detection map, modifier +7, Armor Class 16, and the script 10, 3, 4
- **WHEN** Telekinetic Projectile is cast
- **THEN** the attack total is 17 and the damage is 7
- **AND** the script is fully consumed

#### Scenario: An undetected target spends nothing

- **GIVEN** the target undetected by the caster, 3 actions, and a scripted natural 10
- **WHEN** Telekinetic Projectile is attempted
- **THEN** the error is CannotTarget
- **AND** 3 actions remain
- **AND** the scripted 10 is unused

### Requirement: An area spell draws no flat check

Breathe Fire SHALL draw no flat-check die, including when a creature in the cone is concealed.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=62 (Player Core pg. 442, Concealed)

#### Scenario: A concealed creature in the cone draws no flat check

- **GIVEN** CombatantId(2) in the east cone, observed and concealed from the caster, Reflex +0, spell DC 17, and the script 3, 4, 10
- **WHEN** Breathe Fire is cast east
- **THEN** the rolled damage is 7 and the save total is 10
- **AND** the script is fully consumed

### Requirement: Areas use the grid and ignore terrain

x SHALL increase east and y SHALL increase south. A corner SHALL be the northwest corner of the named square. Distance SHALL use the existing 5-then-10 alternation, starting fresh. Difficult terrain and the turn's diagonal counter SHALL NOT change membership. The same query twice SHALL match and draw no die.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2384 (Player Core pg. 428, Area)

#### Scenario: The same burst twice matches

- **GIVEN** a 10-foot burst from the corner of square (0, 0) and a scripted d20 holding 8
- **WHEN** membership is resolved twice
- **THEN** both results are equal
- **AND** the scripted 8 is unused

### Requirement: Only Small and Medium spaces are accepted

An area query SHALL accept Small and Medium only. Any other size SHALL return the existing unsupported-size error and no squares.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2359 (Player Core pg. 421, Size, Space, and Reach)

#### Scenario: A Large creature is rejected

- **GIVEN** a Large creature at (1, 0) and a 15-foot east cone from a Medium caster at (0, 0)
- **WHEN** membership is resolved
- **THEN** the error is unsupported size
- **AND** no square list is returned

### Requirement: A burst is measured from a corner

A burst radius SHALL be a positive multiple of 5 feet. Any other radius SHALL be rejected. A square SHALL be inside when the grid distance from the origin corner to that square's farthest corner is at most the radius.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2385 (Player Core pg. 428, Burst)

#### Scenario: A 5-foot burst is the four touching squares

- **GIVEN** a 5-foot burst from the corner of square (0, 0)
- **WHEN** membership is listed
- **THEN** the squares are (-1, -1), (-1, 0), (0, -1), and (0, 0)

#### Scenario: A 10-foot burst is twelve squares

- **GIVEN** a 10-foot burst from the corner of square (0, 0)
- **WHEN** membership is listed
- **THEN** the squares are (-2, -1), (-2, 0), (-1, -2), (-1, -1), (-1, 0), (-1, 1), (0, -2), (0, -1), (0, 0), (0, 1), (1, -1), and (1, 0)
- **AND** (1, 1) is not included

#### Scenario: A 20-foot burst reaches the far-corner boundary

- **GIVEN** a 20-foot burst from the corner of square (0, 0)
- **WHEN** membership is listed
- **THEN** there are 44 squares
- **AND** (3, 0) and (2, 2) are included
- **AND** (4, 0) and (3, 1) are not included

#### Scenario: A radius of 0, 4, or negative 5 is rejected

- **GIVEN** burst radii 0, 4, and -5
- **WHEN** each membership is resolved
- **THEN** each result is rejected
- **AND** none returns a square

### Requirement: A 15-foot cardinal cone is the seven-square template

The origin square SHALL NOT be included. The two squares diagonally adjacent to the origin, beside the first square, SHALL NOT be included. This 15-foot template SHALL be ruling R-13 and SHALL NOT be a deviation. Any cone length other than 15 feet SHALL be rejected and SHALL return no squares.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2386 (Player Core pg. 428, Cone)
**Source**: docs/designer/rules-rulings.md (R-13)

#### Scenario: East is the seven squares

- **GIVEN** a Medium caster at (0, 0) and a 15-foot cone aimed east
- **WHEN** membership is listed
- **THEN** the squares are (1, 0), (2, -1), (2, 0), (2, 1), (3, -1), (3, 0), and (3, 1)
- **AND** (0, 0), (1, -1), and (1, 1) are not included
- **AND** the template is ruling R-13

#### Scenario: North is that template turned

- **GIVEN** a Medium caster at (0, 0) and a 15-foot cone aimed north
- **WHEN** membership is listed
- **THEN** the squares are (0, -1), (-1, -2), (0, -2), (1, -2), (-1, -3), (0, -3), and (1, -3)

#### Scenario: South is that template turned

- **GIVEN** a Medium caster at (0, 0) and a 15-foot cone aimed south
- **WHEN** membership is listed
- **THEN** the squares are (0, 1), (-1, 2), (0, 2), (1, 2), (-1, 3), (0, 3), and (1, 3)

#### Scenario: West is that template turned

- **GIVEN** a Medium caster at (0, 0) and a 15-foot cone aimed west
- **WHEN** membership is listed
- **THEN** the squares are (-1, 0), (-2, -1), (-2, 0), (-2, 1), (-3, -1), (-3, 0), and (-3, 1)

#### Scenario: A 30-foot cone is rejected

- **GIVEN** a Medium caster at (0, 0) and a 30-foot cone aimed east
- **WHEN** membership is resolved
- **THEN** the error is UnsupportedCone
- **AND** no square is returned

### Requirement: A 15-foot diagonal cone is one burst quadrant

The cone SHALL be the squares of one 15-foot burst quadrant from the chosen corner of the origin square. The origin square SHALL NOT be included. This template SHALL be ruling R-13 and SHALL NOT be a deviation.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2386 (Player Core pg. 428, Cone)
**Source**: docs/designer/rules-rulings.md (R-13)

#### Scenario: Northeast is the six squares

- **GIVEN** a Medium caster at (0, 0) and a 15-foot cone aimed northeast
- **WHEN** membership is listed
- **THEN** the squares are (1, -1), (2, -1), (3, -1), (1, -2), (2, -2), and (1, -3)
- **AND** (0, 0) is not included
- **AND** the template is ruling R-13

#### Scenario: Southeast is the six squares

- **GIVEN** a Medium caster at (0, 0) and a 15-foot cone aimed southeast
- **WHEN** membership is listed
- **THEN** the squares are (1, 1), (1, 2), (1, 3), (2, 1), (2, 2), and (3, 1)

#### Scenario: Southwest is the six squares

- **GIVEN** a Medium caster at (0, 0) and a 15-foot cone aimed southwest
- **WHEN** membership is listed
- **THEN** the squares are (-3, 1), (-2, 1), (-2, 2), (-1, 1), (-1, 2), and (-1, 3)

#### Scenario: Northwest is the six squares

- **GIVEN** a Medium caster at (0, 0) and a 15-foot cone aimed northwest
- **WHEN** membership is listed
- **THEN** the squares are (-3, -1), (-2, -2), (-2, -1), (-1, -3), (-1, -2), and (-1, -1)

### Requirement: An emanation is distance from the origin square

A radius SHALL be a positive multiple of 5 feet. A square SHALL be inside when the existing distance from the origin square is at most the radius. The origin square SHALL be included only when the caller includes it.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2387 (Player Core pg. 428, Emanation)

#### Scenario: A 5-foot emanation is the eight adjacent squares

- **GIVEN** a Medium creature at (0, 0) and a 5-foot emanation that excludes the origin
- **WHEN** membership is listed
- **THEN** the squares are the eight squares adjacent to (0, 0)
- **AND** (0, 0) is not included

#### Scenario: The caller can include the origin

- **GIVEN** a Medium creature at (0, 0) and a 5-foot emanation that includes the origin
- **WHEN** membership is listed
- **THEN** (0, 0) is included
- **AND** the eight adjacent squares are included

#### Scenario: A 10-foot emanation includes 2,1 and excludes 2,2

- **GIVEN** a Medium creature at (0, 0) and a 10-foot emanation that excludes the origin
- **WHEN** membership is listed
- **THEN** there are 20 squares
- **AND** (2, 1) is included
- **AND** (2, 2) and (0, 0) are not included

### Requirement: A line is one square wide

A length SHALL be a positive multiple of 5 feet. A cardinal line SHALL be that many squares, starting at the neighbor. A diagonal line SHALL use the existing 5-10-5-10 distance. Four squares at 30 feet SHALL be that rule, not a deviation. The origin SHALL be excluded. Any other length SHALL be rejected.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2384 (Player Core pg. 428, Line)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2357 (Player Core pg. 421, Diagonal Movement)

#### Scenario: A 30-foot east line is six squares

- **GIVEN** a Medium creature at (0, 0) and a 30-foot line aimed east
- **WHEN** membership is listed
- **THEN** the squares are (1, 0), (2, 0), (3, 0), (4, 0), (5, 0), and (6, 0)
- **AND** (0, 0) is not included

#### Scenario: A 30-foot northeast line is four squares

- **GIVEN** a Medium creature at (0, 0) and a 30-foot line aimed northeast
- **WHEN** membership is listed
- **THEN** the squares are (1, -1), (2, -2), (3, -3), and (4, -4)
- **AND** (5, -5) is not included

#### Scenario: A length of 0 is rejected

- **GIVEN** a line of length 0 aimed east
- **WHEN** membership is resolved
- **THEN** the result is rejected
- **AND** no square is returned

### Requirement: Breathe Fire is one basic Reflex save for the cone

When any living creature is in the cone, the spell SHALL roll 2d6 once and then call the existing basic Reflex save for each such creature, in ascending CombatantId order, for that one total, against the caster's spell DC. Side SHALL NOT be read.

**Source**: https://2e.aonprd.com/Spells.aspx?ID=1457 (Player Core pg. 319, Breathe Fire)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2297 (Player Core pg. 404, Basic Saving Throws)

#### Scenario: Failure deals 7 and success deals 3

- **GIVEN** spell DC 17, CombatantId(2) and CombatantId(5) both in the east cone, both Reflex +0, both at 20 Hit Points, one of them an Enemy and one a Pc, and the script 3, 4, 10, 17
- **WHEN** Breathe Fire is cast east
- **THEN** the rolled damage is 7
- **AND** CombatantId(2) fails and has 13 Hit Points
- **AND** CombatantId(5) succeeds and has 17 Hit Points
- **AND** 1 action remains and the attack count is 0

#### Scenario: A critical success deals 0

- **GIVEN** spell DC 17, one creature in the cone with Reflex +0 and 20 Hit Points, and the script 3, 4, 20
- **WHEN** Breathe Fire is cast
- **THEN** the save degree is critical success and the damage is 0
- **AND** the creature still has 20 Hit Points

#### Scenario: A critical failure doubles 7 to 14

- **GIVEN** spell DC 17, one creature in the cone with Reflex +0 and 20 Hit Points, and the script 3, 4, 1
- **WHEN** Breathe Fire is cast
- **THEN** the save degree is critical failure and the damage is 14
- **AND** the creature has 6 Hit Points and is Alive

#### Scenario: A natural 1 turns a success into a failure

- **GIVEN** spell DC 17, one creature in the cone with Reflex +16 and 20 Hit Points, and the script 3, 4, 1
- **WHEN** Breathe Fire is cast
- **THEN** the total is 17, the numeric degree is success, and the final degree is failure
- **AND** the damage is 3
- **AND** the creature has 17 Hit Points

### Requirement: A dying creature in the cone takes the unconscious penalty

A dying creature SHALL take the existing −4 status penalty to Reflex. A failure SHALL be an ordinary blow and SHALL raise dying by 1. A critical success SHALL deal 0, SHALL NOT apply a blow, and SHALL leave dying unchanged.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=95 (Player Core pg. 446, Unconscious)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2327 (Player Core pg. 411, Damage and Dying)
**Source**: docs/designer/rules-rulings.md (R-08, R-09, R-10)

#### Scenario: A dying creature who fails goes to dying 2

- **GIVEN** spell DC 17, one creature in the cone at dying 1 with 0 Hit Points and Reflex +0, and the script 3, 4, 13
- **WHEN** Breathe Fire is cast
- **THEN** the total is 9 and the degree is failure
- **AND** the damage is 7
- **AND** the creature is dying 2

#### Scenario: A critical success leaves a dying creature at dying 1

- **GIVEN** spell DC 17, one creature in the cone at dying 1 with 0 Hit Points and Reflex +1, and the script 3, 4, 20
- **WHEN** Breathe Fire is cast
- **THEN** the total is 17 and the degree is critical success
- **AND** the damage is 0
- **AND** the creature stays dying 1
- **AND** Hit Points stay 0

### Requirement: Area saves do not apply cover

Standard or greater cover would apply against an area, and lesser cover would not. This cast SHALL NOT add a cover bonus to the Reflex save. Cover on Armor Class SHALL stay the positioning model.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2384 (Player Core pg. 428, Areas)

#### Scenario: Standard cover does not raise the Reflex save

- **GIVEN** spell DC 17, one creature in the east cone with standard cover, Reflex +0, an empty modifier slice, and the script 3, 4, 10
- **WHEN** Breathe Fire is cast east
- **THEN** the save total is 10
- **AND** the degree is failure

### Requirement: Creatures outside the cone are unchanged

A creature whose square is outside the cone SHALL be unchanged and SHALL draw no die. A dead creature inside the cone SHALL be skipped and SHALL draw no die. The caster's own square SHALL NOT be in the cone.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2386 (Player Core pg. 428, Cone)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2301 (Player Core pg. 406, Damage Rolls)

#### Scenario: The excluded adjacent squares are safe

- **GIVEN** the caster at (0, 0), creatures at (1, -1) and (1, 1), and a scripted 3
- **WHEN** Breathe Fire is cast east
- **THEN** both creatures are unchanged
- **AND** the scripted 3 is unused

#### Scenario: A dead creature in the cone draws no save

- **GIVEN** dead CombatantId(2) at (1, 0) and Alive CombatantId(5) at (2, 0) with Reflex +0 and 20 Hit Points, spell DC 17, and the script 3, 4, 10
- **WHEN** Breathe Fire is cast east
- **THEN** CombatantId(2) draws no die and stays dead
- **AND** CombatantId(5) fails and has 13 Hit Points

### Requirement: An area target that cannot save spends nothing

A duplicate id SHALL return DuplicateAreaTarget. An unknown id SHALL return UnknownCombatant. A creature in the cone with no save training SHALL return NoSavingThrows. Each SHALL spend nothing and draw no die. A creature outside the cone with no save training SHALL NOT cause that error.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2296 (Player Core pg. 404, Saving Throws)

#### Scenario: A duplicate id rolls nothing

- **GIVEN** CombatantId(2) listed twice, 3 actions, and a scripted 3
- **WHEN** Breathe Fire is attempted
- **THEN** the error is DuplicateAreaTarget
- **AND** 3 actions remain
- **AND** the scripted 3 is unused

#### Scenario: No save training in the cone rolls nothing

- **GIVEN** a creature in the east cone with no save training, 3 actions, and a scripted 3
- **WHEN** Breathe Fire is attempted
- **THEN** the error is NoSavingThrows
- **AND** 3 actions remain
- **AND** the scripted 3 is unused

#### Scenario: No save training outside the cone still casts

- **GIVEN** a creature at (1, -1) with no save training, an empty cone otherwise, and a scripted 3
- **WHEN** Breathe Fire is cast east
- **THEN** the cast spends its actions
- **AND** the scripted 3 is unused
