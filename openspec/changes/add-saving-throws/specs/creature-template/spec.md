# creature-template

## ADDED Requirements

### Requirement: derive copies save training

derive SHALL copy optional save training and SHALL NOT roll. Authored Armor Class, Perception, and the strike SHALL stay authored. None SHALL stay none. A later contribution SHALL replace training. The same build SHALL match. Only Constitution, Dexterity, and Wisdom are stored (DEV-54, proposed).

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2296 (Player Core pg. 404, Saving Throws)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2295 (Player Core pg. 404, Armor Class)

#### Scenario: Authored Armor Class stays authored

- **GIVEN** a build with Armor Class 18, Perception modifier +5, attack bonus +7, and Fortitude trained at level 1 with Constitution modifier +2
- **WHEN** derive is called
- **THEN** Armor Class is 18, Perception is +5, and the strike attack bonus is +7
- **AND** the Fortitude training is level 1, Constitution modifier +2, and trained

#### Scenario: Omitted training stays omitted

- **GIVEN** a build with no save training and a scripted natural 10
- **WHEN** derive is called
- **THEN** the Creature has no save training and the scripted 10 is unused

#### Scenario: The same build twice matches

- **GIVEN** two equal builds, each with Reflex expert at level 5 and Dexterity modifier -1
- **WHEN** derive is called on each
- **THEN** the two Creatures' save training are equal

#### Scenario: A later contribution replaces training

- **GIVEN** a base contribution with untrained Fortitude at level 1 and Constitution modifier +0, and a later contribution with trained Fortitude at level 1 and Constitution modifier +2
- **WHEN** the contributions are folded in that order
- **THEN** the Creature's Fortitude training is trained, level 1, and Constitution modifier +2
