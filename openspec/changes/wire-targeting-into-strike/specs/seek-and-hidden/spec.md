# seek-and-hidden

## ADDED Requirements

### Requirement: The targeting result reports the flat natural

When resolve_targeting draws a flat-check die, the result SHALL include that face. When it draws no die, the face SHALL be absent. The DC, the degree, affected, and the spend flags SHALL stay as they are. This function SHALL NOT call spend_actions or strike.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2278 (Player Core pg. 400, Checks (Flat Checks))
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=62 (Player Core pg. 442, Concealed)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=96 (Player Core pg. 447, Undetected)

#### Scenario: A natural 4 at DC 5 reports that face

- **GIVEN** CombatantId(2) observed and concealed from CombatantId(1), and a scripted natural 4
- **WHEN** a non-area targeting check is resolved
- **THEN** the face is 4, the DC is 5, the degree is failure, and the target is not affected
- **AND** the result reports the action spent and the attack counting for MAP

#### Scenario: A natural 5 at DC 5 reports that face

- **GIVEN** CombatantId(2) observed and concealed from CombatantId(1), and a scripted natural 5
- **WHEN** a non-area targeting check is resolved
- **THEN** the face is 5, the DC is 5, the degree is success, and the target is affected
- **AND** the result reports neither the action spent nor the attack counting for MAP

#### Scenario: An observed target reports no face

- **GIVEN** CombatantId(2) observed and not concealed from CombatantId(1), and a scripted natural 4
- **WHEN** a non-area targeting check is resolved
- **THEN** the target is affected and the face is absent
- **AND** the scripted 4 is still unused

#### Scenario: An undetected target reports no face

- **GIVEN** CombatantId(2) undetected and not concealed from CombatantId(1), with Stealth DC 17, and a scripted natural 11
- **WHEN** a non-area targeting check is resolved
- **THEN** the error is CannotTarget
- **AND** the scripted 11 is still unused
