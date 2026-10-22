# creature-template

## ADDED Requirements

### Requirement: A shield is validated at build

A shield SHALL be rejected as InvalidShield when hardness is below 0, Hit Points are below 1, the AC bonus is below 1, the broken threshold is below 0, or the broken threshold is not below the shield's Hit Points. That build SHALL NOT be derived. Folding the same shield SHALL return a FoldError and SHALL NOT return a Creature. The stored fields stay signed i32, their type on main. Validation is the gate.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2181 (Player Core pg. 274, Shield Statistics)

#### Scenario: Negative hardness is rejected

- **GIVEN** a build whose other numbers are valid and whose shield has hardness -1, Hit Points 20, broken threshold 10, and AC bonus +2
- **WHEN** the build is validated
- **THEN** the result is InvalidShield and no Creature is derived

#### Scenario: A zero shield bonus is rejected

- **GIVEN** a build whose shield has hardness 5, Hit Points 20, broken threshold 10, and AC bonus 0
- **WHEN** the build is validated
- **THEN** the result is InvalidShield and no Creature is derived

#### Scenario: A negative shield bonus is rejected

- **GIVEN** a build whose shield has hardness 5, Hit Points 20, broken threshold 10, and AC bonus -1
- **WHEN** the build is validated
- **THEN** the result is InvalidShield and no Creature is derived

#### Scenario: A broken threshold at maximum is rejected

- **GIVEN** a build whose shield has hardness 5, Hit Points 20, broken threshold 20, and AC bonus +2
- **WHEN** the build is validated
- **THEN** the result is InvalidShield and no Creature is derived

#### Scenario: Shield hit points of 0 are rejected

- **GIVEN** a build whose shield has hardness 5, Hit Points 0, broken threshold -1, and AC bonus +2
- **WHEN** the build is validated
- **THEN** the result is InvalidShield and no Creature is derived

#### Scenario: A negative broken threshold is rejected

- **GIVEN** a build whose shield has hardness 5, Hit Points 20, broken threshold -1, and AC bonus +2
- **WHEN** the build is validated
- **THEN** the result is InvalidShield and no Creature is derived

#### Scenario: A folded invalid shield is rejected

- **GIVEN** contributions whose other numbers can be stored and whose shield has hardness -1, Hit Points 20, broken threshold 10, and AC bonus +2
- **WHEN** they are folded
- **THEN** the result is a FoldError
- **AND** the result is not a Creature

#### Scenario: A valid steel shield still derives

- **GIVEN** a build whose other numbers are valid and whose shield has hardness 5, Hit Points 20, broken threshold 10, and AC bonus +2
- **WHEN** the build is validated and derived
- **THEN** the Creature's shield numbers are 5, 20, 10, and +2

#### Scenario: The authored buckler still derives

- **GIVEN** a build whose shield has hardness 3, Hit Points 8, broken threshold 4, and AC bonus +1
- **WHEN** the build is validated and derived
- **THEN** the Creature's shield numbers are 3, 8, 4, and +1
