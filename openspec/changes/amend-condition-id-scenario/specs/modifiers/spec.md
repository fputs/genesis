# modifiers

## MODIFIED Requirements

### Requirement: Duplicate key comparison is a total order

Within a duplicate key the kept modifier SHALL follow a total order: higher level, a level over none, higher rank, a rank over none, then the greater sequence. If those tie, ruling R-01 (an extension consistent with RAW via Rules 2282, not a deviation) SHALL keep the higher bonus or the more negative penalty. If the value ties, the lower condition id SHALL win and a missing id SHALL lose, then the lower source id. Both steps SHALL be engineering tiebreaks with no rules effect, not a deviation.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2266 (Player Core pg. 398, Duplicate Effects)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2282 (Player Core pg. 400, Penalties)

#### Scenario: An equal sequence keeps the higher bonus

- **GIVEN** source 1 on the attack roll, circumstance bonuses of +1 and +4, both at sequence 4, with no level
- **WHEN** they are totaled with either modifier first
- **THEN** one bonus is applied
- **AND** that bonus is +4
- **AND** the attack-roll total is +4 in either order

#### Scenario: An equal sequence keeps the worse penalty

- **GIVEN** source 2 on the attack roll, untyped penalties of -1 and -8, both at sequence 7, with no level
- **WHEN** they are totaled with either modifier first
- **THEN** one penalty is applied
- **AND** that penalty is -8
- **AND** the attack-roll total is -8 in either order

#### Scenario: The lower condition id wins an equal value

- **GIVEN** source 9 on the attack roll, two circumstance bonuses of +2, both level 2, rank 1, and sequence 5, with condition ids 8 and 4
- **WHEN** they are totaled with either modifier first
- **THEN** one modifier is applied
- **AND** its condition id is 4
- **AND** the attack-roll total is +2 in either order

#### Scenario: A three-way tie is the same in all six orders

- **GIVEN** source 11 on the attack roll, three circumstance bonuses of +2, all level 1, rank 1, and sequence 8, with condition ids 6, 1, and 4
- **WHEN** they are totaled in all 6 orders
- **THEN** every result is the same
- **AND** one modifier is applied
- **AND** its condition id is 1
- **AND** the attack-roll total is +2
