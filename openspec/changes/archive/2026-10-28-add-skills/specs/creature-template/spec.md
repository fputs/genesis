# creature-template

## ADDED Requirements

### Requirement: derive copies skill training

derive SHALL copy optional skill training and SHALL NOT roll. Authored Armor Class, Perception, the strike, and saves SHALL stay authored. None SHALL stay none. A later contribution SHALL replace training. Skill attributes SHALL NOT be read from saves (DEV-59, proposed).

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2134 (Player Core pg. 225, Key Attribute)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2300 (Player Core pg. 405, Skill Checks)

#### Scenario: Skill training leaves Armor Class authored

- **GIVEN** a build with Armor Class 18, Perception modifier +5, attack bonus +7, and Athletics trained at level 1 with Strength modifier +2
- **WHEN** derive is called
- **THEN** Armor Class is 18, Perception is +5, and the strike attack bonus is +7
- **AND** the Athletics training is level 1, Strength modifier +2, and trained

#### Scenario: Omitted skill training stays omitted

- **GIVEN** a build with no skill training and a scripted natural 10
- **WHEN** derive is called
- **THEN** the Creature has no skill training and the scripted 10 is unused

#### Scenario: The same skill build twice matches

- **GIVEN** two equal builds, each with Acrobatics expert at level 5 and Dexterity modifier -1
- **WHEN** derive is called on each
- **THEN** the two Creatures' skill training are equal

#### Scenario: A later contribution replaces skill training

- **GIVEN** a base contribution with untrained Athletics at level 1 and Strength modifier +0, and a later contribution with trained Athletics at level 1 and Strength modifier +2
- **WHEN** the contributions are folded in that order
- **THEN** the Creature's Athletics training is trained, level 1, and Strength modifier +2

#### Scenario: Skill Dexterity is not the save Dexterity

- **GIVEN** a build whose saves have Dexterity modifier +1 and whose skills have Dexterity modifier +4, with Acrobatics untrained at level 1
- **WHEN** derive is called
- **THEN** the save Dexterity modifier is +1 and the skill Dexterity modifier is +4
