# skills Specification

## Purpose
Resolves the sixteen Player Core skills from a key attribute and a proficiency bonus, then rolls a skill check or Recall Knowledge with the existing degree of success.

## Requirements

### Requirement: A skill modifier adds the key attribute and proficiency

The modifier SHALL be that skill's key attribute modifier, plus the proficiency bonus, plus modifiers on that skill, plus the frightened and sickened status penalty copied onto that skill. Another skill's own modifier SHALL NOT apply. Untrained SHALL be +0. Proficiency SHALL NOT be a bonus type. A repeated modifier SHALL be totaled once.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2300 (Player Core pg. 405, Skill Checks)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2281 (Player Core pg. 400, Bonuses)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2282 (Player Core pg. 400, Penalties)

#### Scenario: Athletics is Strength plus trained at level 1

- **GIVEN** Athletics, level 1, Strength modifier +2, trained, no other modifier, a scripted natural 10, and DC 15
- **WHEN** the skill check is resolved
- **THEN** the modifier is +5, the total is 15, and the degree is success

#### Scenario: Acrobatics is Dexterity plus expert at level 5

- **GIVEN** Acrobatics, level 5, Dexterity modifier -1, expert, no other modifier, a scripted natural 10, and DC 18
- **WHEN** the skill check is resolved
- **THEN** the modifier is +8, the total is 18, and the degree is success

#### Scenario: Arcana is Intelligence plus master at level 10

- **GIVEN** Arcana, level 10, Intelligence modifier +1, master, no other modifier, a scripted natural 10, and DC 27
- **WHEN** the skill check is resolved
- **THEN** the modifier is +17, the total is 27, and the degree is success

#### Scenario: Medicine is Wisdom plus legendary at level 20

- **GIVEN** Medicine, level 20, Wisdom modifier +4, legendary, no other modifier, a scripted natural 10, and DC 42
- **WHEN** the skill check is resolved
- **THEN** the modifier is +32, the total is 42, and the degree is success

#### Scenario: Diplomacy is Charisma plus untrained

- **GIVEN** Diplomacy, level 20, Charisma modifier +3, untrained, no other modifier, a scripted natural 10, and DC 13
- **WHEN** the skill check is resolved
- **THEN** the modifier is +3, the total is 13, and the degree is success

#### Scenario: An item bonus adds when untrained

- **GIVEN** Athletics, level 1, Strength modifier +0, untrained, an item bonus of +1 on Athletics, a scripted natural 10, and DC 11
- **WHEN** the skill check is resolved
- **THEN** the modifier is +1 and the total is 11

#### Scenario: Two item bonuses keep the highest

- **GIVEN** Athletics, level 1, Strength modifier +0, untrained, an item bonus of +2 and an item bonus of +1 on Athletics from different sources, and a scripted natural 10
- **WHEN** the skill check is resolved
- **THEN** the modifier is +2 and the total is 12

#### Scenario: A circumstance bonus adds beside an item bonus

- **GIVEN** Athletics, level 1, Strength modifier +0, trained, an item bonus of +1 and a circumstance bonus of +1, both on Athletics, and a scripted natural 10
- **WHEN** the skill check is resolved
- **THEN** the modifier is +5 and the total is 15

#### Scenario: A status penalty applies

- **GIVEN** Athletics, level 1, Strength modifier +0, trained, a status penalty of -2 on Athletics, and a scripted natural 10
- **WHEN** the skill check is resolved
- **THEN** the modifier is +1 and the total is 11

#### Scenario: An Acrobatics bonus does not change Athletics

- **GIVEN** Athletics, level 1, Strength modifier +0, untrained, an item bonus of +2 on Acrobatics, and a scripted natural 10
- **WHEN** the Athletics check is resolved
- **THEN** the modifier is +0 and the total is 10

#### Scenario: An attack bonus does not change Athletics

- **GIVEN** Athletics, level 1, Strength modifier +0, untrained, an item bonus of +2 on an attack roll, and a scripted natural 10
- **WHEN** the Athletics check is resolved
- **THEN** the modifier is +0 and the total is 10

#### Scenario: An item bonus and an item penalty both apply

- **GIVEN** Athletics, level 1, Strength modifier +0, untrained, an item bonus of +1 and an item penalty of -1, both on Athletics, and a scripted natural 10
- **WHEN** the skill check is resolved
- **THEN** the modifier is +0 and the total is 10

#### Scenario: The same skill modifier twice matches

- **GIVEN** Athletics, level 1, Strength modifier +0, untrained, and the same item bonus of +1 on Athletics supplied twice
- **WHEN** the skill check is resolved on a scripted natural 10
- **THEN** the modifier is +1 and the total is 11
- **AND** the repeated bonus is totaled once (PC 398, Duplicate Effects, Rules 2266, https://2e.aonprd.com/Rules.aspx?ID=2266)

#### Scenario: Frightened 2 on trained Athletics is plus 1 and DC 11

- **GIVEN** Athletics, level 1, Strength modifier +0, trained, and frightened 2 attached to the attack roll
- **WHEN** the skill modifier and the skill DC are taken
- **THEN** the modifier is +1 and the DC is 11
- **AND** frightened 2 is a status penalty (https://2e.aonprd.com/Conditions.aspx?ID=76)

#### Scenario: Frightened 2 and sickened 1 give minus 2

- **GIVEN** Athletics, level 1, Strength modifier +0, trained, frightened 2 attached to the attack roll, and sickened 1 attached to Fortitude
- **WHEN** the skill modifier is taken
- **THEN** the status penalty is -2 and the modifier is +1
- **AND** both are status penalties, so only the worst applies (https://2e.aonprd.com/Conditions.aspx?ID=91, https://2e.aonprd.com/Rules.aspx?ID=2282)

#### Scenario: Two untyped penalties of minus 1 on Athletics give minus 2

- **GIVEN** Athletics, level 1, Strength modifier +0, untrained, and two untyped penalties of -1 on Athletics from different sources
- **WHEN** the skill check is resolved on a scripted natural 10
- **THEN** the modifier is -2 and the total is 8
- **AND** untyped penalties stack (Player Core pg. 400, Penalties, https://2e.aonprd.com/Rules.aspx?ID=2282)

#### Scenario: Two status penalties keep only the worst

- **GIVEN** Athletics, level 1, Strength modifier +0, untrained, a status penalty of -2 and a status penalty of -1 on Athletics from different sources
- **WHEN** the skill check is resolved on a scripted natural 10
- **THEN** the modifier is -2 and the total is 8
- **AND** only the worst status penalty applies (Player Core pg. 400, Penalties, https://2e.aonprd.com/Rules.aspx?ID=2282)

### Requirement: The sixteen skills use their key attributes

Each skill SHALL add the key attribute named in the scenario. Lore SHALL NOT be a skill. Constitution SHALL NOT be stored on the skill block.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2134 (Player Core pg. 225, Key Attribute)

#### Scenario: Each skill uses its key attribute

- **GIVEN** level 1, Strength, Dexterity, Intelligence, Wisdom, and Charisma modifiers +1, every rank untrained, and no other modifier
- **WHEN** each skill modifier is taken
- **THEN** each modifier is +1
- **AND** the skills are Acrobatics (Dexterity, https://2e.aonprd.com/Skills.aspx?ID=34), Arcana (Intelligence, https://2e.aonprd.com/Skills.aspx?ID=35), Athletics (Strength, https://2e.aonprd.com/Skills.aspx?ID=36), Crafting (Intelligence, https://2e.aonprd.com/Skills.aspx?ID=37), Deception (Charisma, https://2e.aonprd.com/Skills.aspx?ID=38), Diplomacy (Charisma, https://2e.aonprd.com/Skills.aspx?ID=39), Intimidation (Charisma, https://2e.aonprd.com/Skills.aspx?ID=40), Medicine (Wisdom, https://2e.aonprd.com/Skills.aspx?ID=42), Nature (Wisdom, https://2e.aonprd.com/Skills.aspx?ID=43), Occultism (Intelligence, https://2e.aonprd.com/Skills.aspx?ID=44), Performance (Charisma, https://2e.aonprd.com/Skills.aspx?ID=45), Religion (Wisdom, https://2e.aonprd.com/Skills.aspx?ID=46), Society (Intelligence, https://2e.aonprd.com/Skills.aspx?ID=47), Stealth (Dexterity, https://2e.aonprd.com/Skills.aspx?ID=48), Survival (Wisdom, https://2e.aonprd.com/Skills.aspx?ID=49), and Thievery (Dexterity, https://2e.aonprd.com/Skills.aspx?ID=50)
- **AND** Lore is not among them

### Requirement: Proficiency uses the existing rank table

Trained, expert, master, and legendary SHALL use the existing proficiency bonus: level plus 2, 4, 6, and 8. Untrained SHALL stay 0. A level outside 1 through 25 SHALL be rejected.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2281 (Player Core pg. 400, Proficiency Bonus)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2300 (Player Core pg. 405, Skill Checks)

#### Scenario: Trained at level 2 with Strength 0 is plus 4

- **GIVEN** Athletics, level 2, Strength modifier +0, trained, and no other modifier
- **WHEN** the modifier is taken
- **THEN** the modifier is +4

#### Scenario: Legendary Athletics at level 25 is plus 33

- **GIVEN** Athletics, level 25, Strength modifier +0, legendary, and no other modifier
- **WHEN** the modifier is taken
- **THEN** the modifier is +33

#### Scenario: A skill level of 0 cannot be built

- **GIVEN** a level of 0
- **WHEN** a skill block is built
- **THEN** the level is rejected

#### Scenario: A skill level of 26 cannot be built

- **GIVEN** a level of 26
- **WHEN** a skill block is built
- **THEN** the level is rejected

#### Scenario: A skill level of 25 can be built

- **GIVEN** a level of 25
- **WHEN** a skill block is built
- **THEN** the level is accepted

### Requirement: A skill degree uses the existing check

The degree SHALL be the existing check, including the natural 1 and natural 20 shift. This change SHALL NOT add another degree adjustment.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2286 (Player Core pg. 401, Step 4: Degree of Success)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2287 (Player Core pg. 401, Natural 1 and Natural 20)

#### Scenario: Total DC minus 1 on a skill check is a failure

- **GIVEN** Athletics, level 1, Strength modifier +0, untrained, a scripted natural 14, and DC 15
- **WHEN** the skill check is resolved
- **THEN** the total is 14 and the degree is failure

#### Scenario: Natural 1 on a skill check at a total equal to the DC is a failure

- **GIVEN** Athletics, level 1, Strength modifier +14, untrained, a scripted natural 1, and DC 15
- **WHEN** the skill check is resolved
- **THEN** the total is 15, the numeric degree is success, and the final degree is failure

#### Scenario: Natural 20 on a skill check at DC minus 11 is a failure

- **GIVEN** Athletics, level 1, Strength modifier +0, untrained, a scripted natural 20, and DC 31
- **WHEN** the skill check is resolved
- **THEN** the total is 20, the numeric degree is critical failure, and the final degree is failure

#### Scenario: Beating a skill DC by 10 is a critical success

- **GIVEN** Athletics, level 1, Strength modifier +0, untrained, a scripted natural 15, and DC 5
- **WHEN** the skill check is resolved
- **THEN** the total is 15 and the degree is critical success

#### Scenario: A negative skill total against DC 5 is a failure

- **GIVEN** Athletics, level 1, Strength modifier -5, untrained, a scripted natural 4, and DC 5
- **WHEN** the skill check is resolved
- **THEN** the total is -1 and the degree is failure

#### Scenario: Natural 20 on a skill check at the DC is a critical success

- **GIVEN** Athletics, level 1, Strength modifier +0, untrained, a scripted natural 20, and DC 20
- **WHEN** the skill check is resolved
- **THEN** the total is 20, the numeric degree is success, and the final degree is critical success
- **AND** a natural 20 improves the degree by one step (PC 401, Rules 2287, https://2e.aonprd.com/Rules.aspx?ID=2287)

#### Scenario: Natural 20 on a skill check at DC minus 10 is a failure

- **GIVEN** Athletics, level 1, Strength modifier +0, untrained, a scripted natural 20, and DC 30
- **WHEN** the skill check is resolved
- **THEN** the total is 20, the numeric degree is critical failure, and the final degree is failure

#### Scenario: Natural 1 on a skill check at DC plus 10 is a success

- **GIVEN** Athletics, level 1, Strength modifier +14, untrained, a scripted natural 1, and DC 5
- **WHEN** the skill check is resolved
- **THEN** the total is 15, the numeric degree is critical success, and the final degree is success

#### Scenario: A total of exactly DC minus 10 is a critical failure

- **GIVEN** Athletics, level 1, Strength modifier +0, untrained, a scripted natural 5, and DC 15
- **WHEN** the skill check is resolved
- **THEN** the total is 5 and the degree is critical failure

### Requirement: A skill DC is 10 plus the modifier

A skill DC SHALL be 10 plus the skill modifier.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2300 (Player Core pg. 405, Skill Checks)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2285 (Player Core pg. 401, Getting a DC from a Modifier)

#### Scenario: A skill modifier of plus 7 has DC 17

- **GIVEN** Athletics, level 1, Strength modifier +4, trained, and no other modifier
- **WHEN** the skill DC is taken
- **THEN** the modifier is +7 and the DC is 17

### Requirement: Recall Knowledge returns the degree

Recall Knowledge SHALL roll the named skill against the caller DC and SHALL return that degree and the natural result. It SHALL NOT return a fact. Any of the sixteen skills SHALL be legal while untrained. Its concentrate trait SHALL NOT be stored.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2367 (Player Core pg. 231, Recall Knowledge)
**Source**: https://2e.aonprd.com/Skills.aspx?General=true&ID=24 (Player Core pg. 231, Recall Knowledge)

#### Scenario: Recall Knowledge success returns that degree

- **GIVEN** the actor's turn is open, Society, level 1, Intelligence modifier +0, untrained, a scripted natural 15, and DC 15
- **WHEN** Recall Knowledge is resolved
- **THEN** the total is 15, the degree is success, and the natural result is 15

#### Scenario: Recall Knowledge critical success returns that degree

- **GIVEN** the actor's turn is open, Society, level 1, Intelligence modifier +0, untrained, a scripted natural 15, and DC 5
- **WHEN** Recall Knowledge is resolved
- **THEN** the total is 15 and the degree is critical success

#### Scenario: Recall Knowledge failure returns that degree

- **GIVEN** the actor's turn is open, Society, level 1, Intelligence modifier +0, untrained, a scripted natural 14, and DC 15
- **WHEN** Recall Knowledge is resolved
- **THEN** the total is 14 and the degree is failure

#### Scenario: Recall Knowledge critical failure returns that degree

- **GIVEN** the actor's turn is open, Society, level 1, Intelligence modifier +0, untrained, a scripted natural 5, and DC 15
- **WHEN** Recall Knowledge is resolved
- **THEN** the total is 5 and the degree is critical failure

#### Scenario: Untrained Recall Knowledge is legal

- **GIVEN** the actor's turn is open, Society untrained, level 1, Intelligence modifier +0, a scripted natural 15, and DC 15
- **WHEN** Recall Knowledge is resolved
- **THEN** the result is success and not an error

#### Scenario: The same Recall twice matches

- **GIVEN** two equal actors, each with Society untrained, level 1, Intelligence modifier +0, and a scripted natural 15 against DC 15
- **WHEN** each resolves Recall Knowledge
- **THEN** the two results are equal

### Requirement: An overflowing modifier rolls nothing

A sum that does not fit a check modifier SHALL be rejected before any die is drawn. Actions SHALL stay unchanged.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2300 (Player Core pg. 405, Skill Checks)

#### Scenario: A modifier that does not fit rolls nothing

- **GIVEN** an open turn with 3 actions, Athletics, level 1, Strength modifier 2147483647, untrained, no other modifier, and a scripted natural 10
- **WHEN** the skill check is resolved, and Recall Knowledge is resolved
- **THEN** both errors are SkillOverflow, 3 actions remain, and the scripted 10 is unused

### Requirement: New skill errors keep their own match arms

`ready_from_encounter` SHALL map NoSkills and SkillOverflow to ReadyEncounter variants of the same name, keeping the overflow value. `from_spend` SHALL map each to NoReaction. `as_damage_overflow` SHALL map each to DamageOverflow of ModifierError::Overflow. Each variant SHALL have its own arm.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2300 (Player Core pg. 405, Skill Checks)

#### Scenario: No skills keeps its own arms

- **GIVEN** EncounterError::NoSkills
- **WHEN** ready_from_encounter, from_spend, and as_damage_overflow each receive it
- **THEN** Ready is ReadyError::Encounter of ReadyEncounter::NoSkills
- **AND** from_spend is ReactiveError::NoReaction
- **AND** as_damage_overflow is ReactiveError::DamageOverflow of ModifierError::Overflow

#### Scenario: Skill overflow keeps its own arms

- **GIVEN** EncounterError::SkillOverflow of ArithmeticOverflow::SumDoesNotFit
- **WHEN** ready_from_encounter, from_spend, and as_damage_overflow each receive it
- **THEN** Ready is ReadyError::Encounter of ReadyEncounter::SkillOverflow of that overflow
- **AND** from_spend is ReactiveError::NoReaction
- **AND** as_damage_overflow is ReactiveError::DamageOverflow of ModifierError::Overflow
