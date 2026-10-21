# creature-template

## MODIFIED Requirements

### Requirement: Validation rejects impossible statistics

Validation of a CharacterBuild SHALL reject Hit Points of 0 or less, a negative Speed, and a Speed that is not a multiple of 5, each as a typed error. Speed 0 SHALL be accepted. A rejected build SHALL NOT be derived. Folding contributions SHALL use those same checks and SHALL return a FoldError instead of a Creature when they fail. Current Hit Points on the damage path and the heal path SHALL stay from 0 through the maximum, and an overflow on either path SHALL return a typed error.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2324 (Player Core pg. 410, creatures are not reduced below 0 Hit Points)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2320 (Player Core pg. 410, Healing)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2346 (Player Core pg. 420, Speed is at least 0)
**Source**: validation (DEV-21). https://2e.aonprd.com/Rules.aspx?ID=2347 (Player Core pg. 420, Speed). Player Core has no explicit multiple-of-5 rule.
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2356 (Player Core pg. 421, 5-ft squares)

#### Scenario: Zero Hit Points is rejected

- **GIVEN** inputs whose other numbers are valid and whose Hit Points are 0
- **WHEN** the build is validated
- **THEN** validation returns the non-positive Hit Points error and does not derive a Creature

#### Scenario: Negative Hit Points are rejected

- **GIVEN** inputs whose Hit Points are -1
- **WHEN** the build is validated
- **THEN** validation returns the non-positive Hit Points error and does not derive a Creature

#### Scenario: Negative Speed is rejected

- **GIVEN** inputs whose Speed is -5 and whose Hit Points are 12
- **WHEN** the build is validated
- **THEN** validation returns the negative Speed error and does not derive a Creature

#### Scenario: Speed zero is accepted

- **GIVEN** inputs with 12 Hit Points and Speed 0
- **WHEN** the build is validated and derived
- **THEN** the Creature's Speed is 0

#### Scenario: Speed 7 is rejected at load

- **GIVEN** inputs with 12 Hit Points and Speed 7
- **WHEN** the build is validated
- **THEN** validation returns the non-multiple-of-5 Speed error and does not derive a Creature

#### Scenario: A folded zero Hit Points is rejected

- **GIVEN** contributions whose other numbers can be stored and whose Hit Points sum to 0
- **WHEN** they are folded
- **THEN** the result is a FoldError for a statistic the template would reject
- **AND** the result is not a Creature

#### Scenario: A folded negative Speed is rejected

- **GIVEN** contributions whose Hit Points sum to 12, whose Speed sums to -5, and whose other numbers can be stored
- **WHEN** they are folded
- **THEN** the result is a FoldError for a statistic the template would reject
- **AND** the result is not a Creature

#### Scenario: Folded Speed zero is accepted

- **GIVEN** contributions with 12 Hit Points, Speed 0, and other numbers that can be stored
- **WHEN** they are folded
- **THEN** the Creature's Speed is 0

#### Scenario: A folded Speed 7 is rejected

- **GIVEN** contributions whose Hit Points sum to 12, whose Speed sums to 7, and whose other numbers can be stored
- **WHEN** they are folded
- **THEN** the result is a FoldError for a statistic the template would reject
- **AND** the result is not a Creature

#### Scenario: Damage and healing keep current Hit Points in range

- **GIVEN** a Creature with 9 Hit Points and a state at 2 current Hit Points
- **WHEN** 5 damage is applied, and then healing of 4 is applied to a state at 9 current Hit Points
- **THEN** current Hit Points are 0 after the damage
- **AND** current Hit Points are 9 after the healing

#### Scenario: Healing that does not fit returns an error

- **GIVEN** a Creature whose maximum Hit Points are 2147483647 and a state at that current value
- **WHEN** healing of 1 is applied
- **THEN** the result is a typed overflow error
- **AND** current Hit Points stay 2147483647

## ADDED Requirements

### Requirement: Penalties leave at least 1 damage

When penalties on an attack reduce a damage total to 0 or below, the roll SHALL deal 1 damage. That floor SHALL apply only to penalties on an attack and SHALL be applied before critical doubling, so the smallest critical hit SHALL deal 2 damage. RAW does not state that order; this change applies DEV-14. An overflowing total SHALL return a typed error and SHALL NOT change Hit Points. Resistance has a separate floor of 0, stays out of scope, and SHALL NOT be applied.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2302 (Player Core pg. 406, penalties on an attack)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2307 (Player Core pg. 407, Doubling and Halving)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2318 (Player Core pg. 408, resistance floor of 0)
**Source**: DEV-14. RAW does not state whether the floor is before or after doubling; this change applies it first.

#### Scenario: Penalties reduce damage to 1

- **GIVEN** penalties on an attack, a rolled 4, and a penalty of -6
- **WHEN** the damage is resolved and the hit is not critical
- **THEN** the damage dealt is 1

#### Scenario: Minimum critical damage is 2

- **GIVEN** penalties on an attack, a rolled 4, and a penalty of -6
- **WHEN** the hit is critical
- **THEN** the damage dealt is 2

#### Scenario: An overflowing damage total returns an error

- **GIVEN** a rolled 1, a bonus of 2147483647, and a state at 9 current Hit Points
- **WHEN** that damage is applied
- **THEN** the result is a typed overflow error
- **AND** current Hit Points stay 9
