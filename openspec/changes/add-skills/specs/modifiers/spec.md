# modifiers

## ADDED Requirements

### Requirement: A skill statistic does not share a stack

Each of the sixteen skills SHALL be its own statistic. A modifier on one SHALL NOT change another statistic. Unconscious SHALL NOT penalize a skill. No skill producer SHALL be added.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2281 (Player Core pg. 400, Bonuses)
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
