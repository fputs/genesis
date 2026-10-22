# creature-template

## MODIFIED Requirements

### Requirement: uses_dying_rules defaults to death at 0 Hit Points

uses_dying_rules SHALL be a build input and SHALL default to false when omitted. derive SHALL copy it onto the Creature and SHALL NOT roll Dice. At 0 Hit Points, a false flag SHALL mark that CombatantState dead with no dying. A true flag SHALL record the dying value from dying-and-victory and SHALL NOT mark the state dead only for reaching 0. Massive damage SHALL kill instead. The Creature SHALL stay unchanged.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2324 (Player Core pg. 410)

#### Scenario: Omitting the flag defaults to false

- **GIVEN** a build that does not set uses_dying_rules
- **WHEN** derive is called
- **THEN** the Creature's uses_dying_rules is false

#### Scenario: Without the flag, 0 Hit Points is death on the state

- **GIVEN** a Creature with 9 Hit Points and uses_dying_rules false, a state at 9 current Hit Points, and a scripted d8 of 8 for damage 1d8+1
- **WHEN** that damage is applied to the state
- **THEN** the state has 0 current Hit Points, is dead, and has no dying condition, and the Creature still has 9 Hit Points

#### Scenario: With the flag, 0 Hit Points enters dying on the state

- **GIVEN** a Creature with 9 Hit Points and uses_dying_rules true, a state at 9 current Hit Points, and a scripted d8 of 8 for damage 1d8+1
- **WHEN** that damage is applied to the state
- **THEN** the state has 0 current Hit Points, dying value 1, wounded 0, is unconscious and not dead, and the Creature is unchanged

#### Scenario: Damage above 0 does not kill

- **GIVEN** a Creature with 9 Hit Points and uses_dying_rules false, a state at 9 current Hit Points, and a scripted d8 of 1 for damage 1d8+1
- **WHEN** that damage is applied to the state
- **THEN** the state has 7 current Hit Points and is not dead, and the Creature still has 9 Hit Points
