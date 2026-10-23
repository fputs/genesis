# modifiers

## ADDED Requirements

### Requirement: Fortitude and Will are separate statistics

Fortitude and Will SHALL each be their own statistic. Neither SHALL share a stack with the other, with Reflex, with attack rolls, with Armor Class, with Perception, or with damage. Unconscious SHALL NOT add a penalty to Fortitude or to Will. Reflex SHALL keep the existing status penalty of -4. No Fortitude or Will producer SHALL be added.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2296 (Player Core pg. 404, Saving Throws)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=95 (Player Core pg. 446, Unconscious)

#### Scenario: Fortitude and Will do not share a stack

- **GIVEN** a circumstance bonus of +2 on Fortitude and a circumstance bonus of +1 on Will

- **WHEN** the two statistics are totaled separately

- **THEN** Fortitude is +2 and Will is +1

#### Scenario: Unconscious adds nothing to Fortitude

- **GIVEN** an unconscious creature and a circumstance bonus of +1 on Fortitude

- **WHEN** Fortitude is totaled

- **THEN** the total is +1

- **AND** Fortitude has no status penalty

#### Scenario: Unconscious adds nothing to Will

- **GIVEN** an unconscious creature and a circumstance bonus of +1 on Will

- **WHEN** Will is totaled

- **THEN** the total is +1

- **AND** Will has no status penalty

#### Scenario: Reflex keeps the unconscious penalty

- **GIVEN** an unconscious creature and no other Reflex modifier

- **WHEN** reflex_modifiers(true) is totaled

- **THEN** Reflex includes a status penalty of -4 at sequence 10

- **AND** Fortitude does not include that penalty
