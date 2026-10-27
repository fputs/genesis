# creature-template

## ADDED Requirements

### Requirement: derive copies optional spellcasting

derive SHALL copy a present spellcasting entry and SHALL store none when it is omitted. It SHALL NOT roll. It SHALL NOT change Armor Class, the strike, Perception, or saves from that entry. The same build twice SHALL match.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2292 (Player Core, Spell Attack Rolls)

#### Scenario: Omitted spellcasting stays omitted

- **GIVEN** a build with no spellcasting entry and a scripted d20 holding 8
- **WHEN** derive is called
- **THEN** the creature has no spellcasting entry
- **AND** the scripted 8 is unused

#### Scenario: A present entry is copied

- **GIVEN** a build with level 1, attribute modifier +4, and trained spellcasting
- **WHEN** derive is called
- **THEN** the creature's spellcasting is level 1, attribute modifier +4, and trained
- **AND** Armor Class, the strike, and Perception stay the authored values

#### Scenario: The same spellcasting build twice matches

- **GIVEN** one build with that spellcasting entry
- **WHEN** derive is called twice
- **THEN** the two creatures are equal
