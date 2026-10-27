# modifiers

## ADDED Requirements

### Requirement: SpellAttack is its own statistic

SpellAttack SHALL be its own statistic. It SHALL NOT share a stack with Attack, Armor Class, Perception, damage, Fortitude, Reflex, or Will. Unconscious SHALL NOT add a penalty to it. Proficiency SHALL NOT be a bonus type.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2292 (Player Core pg. 403, Spell Attack Rolls)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2281 (Player Core pg. 400, Bonuses)

#### Scenario: SpellAttack and Attack do not share a stack

- **GIVEN** a circumstance bonus of +2 on SpellAttack and a circumstance bonus of +1 on Attack
- **WHEN** the two statistics are totaled separately
- **THEN** SpellAttack is +2 and Attack is +1

#### Scenario: Unconscious adds nothing to SpellAttack

- **GIVEN** an unconscious creature and a circumstance bonus of +1 on SpellAttack
- **WHEN** SpellAttack is totaled
- **THEN** the total is +1
- **AND** SpellAttack has no status penalty
