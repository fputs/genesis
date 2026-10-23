# modifiers

## ADDED Requirements

### Requirement: Fortitude and Will are separate statistics

Fortitude and Will SHALL each be their own statistic. Neither SHALL share a stack with the other, with Reflex, with attack rolls, with Armor Class, with Perception, or with damage. Unconscious SHALL NOT add a penalty to Fortitude or to Will. Reflex SHALL keep the existing status penalty of -4.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2296 (Player Core pg. 404, Saving Throws)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=95 (Player Core pg. 446, Unconscious)

#### Scenario: Fortitude and Will do not share a stack

- **GIVEN** a circumstance bonus of +2 on Fortitude and a circumstance bonus of +1 on Will
- **WHEN** the two statistics are totaled separately
- **THEN** Fortitude is +2 and Will is +1

#### Scenario: Unconscious adds nothing to Fortitude

- **GIVEN** an unconscious creature and no other Fortitude modifier
- **WHEN** Fortitude modifiers from the unconscious producer are totaled
- **THEN** the total is 0

#### Scenario: Unconscious adds nothing to Will

- **GIVEN** an unconscious creature and no other Will modifier
- **WHEN** Will modifiers from the unconscious producer are totaled
- **THEN** the total is 0

#### Scenario: Reflex keeps the unconscious penalty

- **GIVEN** an unconscious creature and no other Reflex modifier
- **WHEN** Reflex and Fortitude are totaled separately
- **THEN** Reflex includes a status penalty of -4 at sequence 10
- **AND** Fortitude does not include that penalty
