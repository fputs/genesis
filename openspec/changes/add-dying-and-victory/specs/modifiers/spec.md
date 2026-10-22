# modifiers

## ADDED Requirements

### Requirement: Reflex takes the unconscious penalty

Reflex SHALL be its own statistic. It SHALL NOT share a stack with attack rolls, Armor Class, Perception, or damage. Unconscious SHALL add a status penalty of -4 to Reflex. When the creature is not unconscious, that producer SHALL add nothing.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=95 (Player Core pg. 446, Unconscious)

#### Scenario: Unconscious is -4 status to Reflex

- **GIVEN** an unconscious creature and no other Reflex modifier
- **WHEN** Reflex modifiers are totaled
- **THEN** the total includes a status penalty of -4

#### Scenario: A conscious creature adds no Reflex penalty

- **GIVEN** a creature that is not unconscious
- **WHEN** Reflex modifiers are totaled
- **THEN** the total from this producer is 0

#### Scenario: Reflex and Armor Class both keep the penalty

- **GIVEN** an unconscious creature and no other modifiers
- **WHEN** Reflex and Armor Class are totaled separately
- **THEN** each total includes a status penalty of -4
- **AND** neither total drops the other's penalty
