# dying-and-victory

## MODIFIED Requirements

### Requirement: A blow to 0 Hit Points sets dying

A creature with uses_dying_rules that drops from above 0 Hit Points to 0 SHALL gain dying 1, or dying 2 when that blow is critical, plus its wounded value, and SHALL set current Hit Points to 0 in that same step. That blow SHALL NOT also apply damage-while-dying. The Creature SHALL stay unchanged. All damage is lethal (DEV-29): there is no nonlethal result, and the blow's critical flag is the only critical signal.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2324 (Player Core pg. 410, Getting Knocked Out)
**Source**: DEV-29. Nonlethal, death effects, save critical-failure dying 2, and regeneration are not modeled. Persistent damage is modeled by damage-defenses.

#### Scenario: A non-critical blow to 0 sets dying 1

- **GIVEN** uses_dying_rules, maximum Hit Points 9, current Hit Points 9, wounded 0, and a scripted d8 of 8 for non-critical damage 1d8+1
- **WHEN** that blow is applied
- **THEN** current Hit Points are 0, vital is Dying(1), wounded is 0, vital is not Dead, and the Creature still has 9 Hit Points

#### Scenario: A critical blow to 0 sets dying 2

- **GIVEN** uses_dying_rules, maximum Hit Points 16, current Hit Points 16, wounded 0, and a scripted d8 of 8 for critical damage 1d8
- **WHEN** that blow is applied
- **THEN** current Hit Points are 0 and vital is Dying(2)
- **AND** wounded is 0 and vital is not Dead

#### Scenario: The dropping blow does not also increase dying

- **GIVEN** uses_dying_rules, maximum Hit Points 16, current Hit Points 16, wounded 0, and a scripted d8 of 8 for critical damage 1d8
- **WHEN** that blow is applied
- **THEN** vital is Dying(2), not Dead from a second increase

#### Scenario: Damage above 0 does not start dying

- **GIVEN** uses_dying_rules, maximum Hit Points 9, current Hit Points 9, and a scripted d8 of 1 for non-critical damage 1d8+1
- **WHEN** that blow is applied
- **THEN** current Hit Points are 7, vital is Alive, and there is no dying value

#### Scenario: Damage past 0 sets dying 1

- **GIVEN** uses_dying_rules, maximum Hit Points 10, current Hit Points 3, wounded 0, and a scripted d8 rolling 7 with bonus 8 for non-critical damage 1d8+8
- **WHEN** that blow is applied
- **THEN** current Hit Points are 0, vital is Dying(1), and wounded is 0
