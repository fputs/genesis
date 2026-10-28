# saving-throws

## ADDED Requirements

### Requirement: A basic save includes tracked conditions

The defender's tracked modifiers on that save SHALL be included with the caller modifiers. Same-type stacking SHALL keep the worst penalty. Frightened and sickened SHALL apply to Fortitude, Reflex, and Will. Clumsy SHALL apply to Reflex only. Enfeebled SHALL apply to none of the three.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2296 (Player Core pg. 404, Saving Throws)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=76 (Player Core pg. 444, Frightened)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=61 (Player Core pg. 442, Clumsy)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=71 (Player Core pg. 443, Enfeebled)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2282 (Player Core pg. 400, Penalties)

#### Scenario: Frightened 2 lowers Fortitude by 2

- **GIVEN** Fortitude, level 1, Constitution modifier +0, untrained, frightened 2, a scripted natural 10, and DC 15
- **WHEN** the save is resolved
- **THEN** the total is 8 and the degree is failure

#### Scenario: Frightened 2 and sickened 1 lower Will by 2

- **GIVEN** Will, level 1, Wisdom modifier +0, untrained, frightened 2, sickened 1, a scripted natural 10, and DC 15
- **WHEN** the save is resolved
- **THEN** the total is 8 and the degree is failure

#### Scenario: Clumsy 2 lowers Reflex and not Fortitude

- **GIVEN** clumsy 2, level 1, Dexterity and Constitution modifiers +0, both saves untrained, and scripted naturals 10 then 10
- **WHEN** Reflex and Fortitude are resolved against DC 15
- **THEN** Reflex totals 8 and Fortitude totals 10

#### Scenario: Enfeebled 2 does not lower Fortitude

- **GIVEN** Fortitude, level 1, Constitution modifier +0, untrained, enfeebled 2, a scripted natural 10, and DC 15
- **WHEN** the save is resolved
- **THEN** the total is 10 and the degree is failure

#### Scenario: Frightened 1 and unconscious Reflex stay at minus 4

- **GIVEN** an unconscious defender, Reflex, level 1, Dexterity modifier +0, untrained, frightened 1, a scripted natural 15, and DC 15
- **WHEN** the save is resolved
- **THEN** the total is 11 and the degree is failure
