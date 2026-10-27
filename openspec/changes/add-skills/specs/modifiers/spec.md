# modifiers

## ADDED Requirements

### Requirement: A skill statistic does not share a stack

Each of the sixteen skills SHALL be its own statistic. An item, circumstance, or untyped modifier on one skill SHALL NOT change another skill. The frightened and sickened producer SHALL put a status penalty equal to its value on all sixteen skill statistics, including when that condition names another statistic. Unconscious SHALL NOT penalize a skill.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2282 (Player Core pg. 400, Penalties)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=95 (Player Core pg. 446, Unconscious)

#### Scenario: Athletics and Acrobatics do not share a stack

- **GIVEN** an item bonus of +2 on Athletics and an item bonus of +1 on Acrobatics
- **WHEN** the two statistics are totaled separately
- **THEN** Athletics is +2 and Acrobatics is +1

#### Scenario: Unconscious adds nothing to Athletics

- **GIVEN** an unconscious creature and an item bonus of +1 on Athletics
- **WHEN** Athletics is totaled
- **THEN** the total is +1
- **AND** Athletics has no status penalty

#### Scenario: A skill item bonus does not change Fortitude

- **GIVEN** an item bonus of +1 on Athletics and no Fortitude modifier
- **WHEN** Fortitude is totaled
- **THEN** Fortitude is +0

#### Scenario: Frightened 2 reaches every skill statistic

- **GIVEN** frightened 2 attached to the attack roll and no skill modifier
- **WHEN** each of the sixteen skill statistics is totaled
- **THEN** each total is a status penalty of -2
- **AND** the value is the status penalty to all checks and DCs (https://2e.aonprd.com/Conditions.aspx?ID=76)

#### Scenario: Sickened 1 reaches every skill statistic

- **GIVEN** sickened 1 attached to Fortitude and no skill modifier
- **WHEN** each of the sixteen skill statistics is totaled
- **THEN** each total is a status penalty of -1
- **AND** sickened uses that same status penalty (https://2e.aonprd.com/Conditions.aspx?ID=91)
