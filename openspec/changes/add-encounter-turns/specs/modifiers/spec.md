# modifiers

## REMOVED Requirements

### Requirement: Attack check internal terms use reserved sources

**Reason**: The requirement totaled a caller penalty with the internal terms and never named a public argument, so the caller list was removed and the tests read a private helper.
**Migration**: Use "Attack check total takes caller modifiers" and "Internal attack terms keep reserved sources". The same cases are now results of `attack_check_total`.

## ADDED Requirements

### Requirement: Attack check total takes caller modifiers

`attack_check_total` SHALL take the caller modifiers as a public list and fold that list with its internal terms by the stacking rules. A success SHALL be that total. An overflowing fold SHALL return a typed error and SHALL NOT panic.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2282 (Player Core pg. 400, Penalties)

#### Scenario: An empty caller list keeps the stacked total

- **GIVEN** a natural 12, attack bonus +9, multiple attack penalty -5, circumstance penalty -2, range penalty 0, and an empty caller list
- **WHEN** `attack_check_total` is called
- **THEN** the total is 14

#### Scenario: A caller untyped penalty stacks with the multiple attack penalty

- **GIVEN** a natural 12, attack bonus +9, multiple attack penalty -5, and a caller untyped penalty of -2 from source 1
- **WHEN** `attack_check_total` is called
- **THEN** the total is 14
- **AND** the total is not 16

#### Scenario: Two caller circumstance bonuses keep the higher

- **GIVEN** a natural 10, attack bonus +0, and caller circumstance bonuses of +4 and +1 from different sources
- **WHEN** `attack_check_total` is called
- **THEN** the total is 14
- **AND** the total is not 15

#### Scenario: A caller circumstance bonus and an internal circumstance penalty both apply

- **GIVEN** a natural 10, attack bonus +0, an internal circumstance penalty of -2, and a caller circumstance bonus of +2 from source 1
- **WHEN** `attack_check_total` is called
- **THEN** the total is 10

#### Scenario: An Armor Class modifier does not change the attack total

- **GIVEN** a natural 10, attack bonus +0, and a caller circumstance penalty of -2 on Armor Class from source 1
- **WHEN** `attack_check_total` is called
- **THEN** the total is 10
- **AND** the total is not 8

#### Scenario: An overflowing caller list returns an error

- **GIVEN** a natural 12 and two caller untyped penalties of -2147483648 from different sources
- **WHEN** `attack_check_total` is called
- **THEN** the result is a typed overflow error
- **AND** the result is not a total of 12
- **AND** the call does not panic

#### Scenario: The same caller list totals twice

- **GIVEN** a natural 10, attack bonus +4, multiple attack penalty -5, and a caller circumstance penalty of -2 from source 3
- **WHEN** `attack_check_total` is called twice with that list
- **THEN** the two totals are equal

### Requirement: Internal attack terms keep reserved sources

The multiple attack penalty SHALL apply once per check through its reserved source. The range penalty SHALL be the other untyped internal term. A reserved source SHALL NOT skip same-type stacking. A caller SHALL NOT construct a reserved source.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2289 (Player Core pg. 402, Multiple Attack Penalty)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2290 (Player Core pg. 403, Range Penalty)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2266 (Player Core pg. 398, Rules Overview, section Duplicate Effects)

#### Scenario: A caller source 1 does not collapse an internal term

- **GIVEN** a natural 10, attack bonus +0, multiple attack penalty -5, circumstance penalty -2, range penalty -2, and a caller untyped penalty of -3 from source 1
- **WHEN** `attack_check_total` is called
- **THEN** the caller's source id is 1
- **AND** the total is -2
- **AND** the total is not 1

#### Scenario: A second attack with no caller MAP gets -5 once

- **GIVEN** a natural 10, attack bonus +0, the multiple attack penalty of -5 for a second non-agile attack on the creature's own turn, and an empty caller list
- **WHEN** `attack_check_total` is called
- **THEN** the total is 5
- **AND** the total is not 0

#### Scenario: An internal circumstance penalty and a caller circumstance penalty total -3

- **GIVEN** a natural 10, attack bonus +0, an internal circumstance penalty of -2, and a caller circumstance penalty of -3 from source 1
- **WHEN** `attack_check_total` is called
- **THEN** the total is 7
- **AND** the total is not 5
