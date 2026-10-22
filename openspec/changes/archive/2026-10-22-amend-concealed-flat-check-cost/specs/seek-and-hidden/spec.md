# seek-and-hidden

## MODIFIED Requirements

### Requirement: A concealed target takes a DC 5 flat check

An observed and concealed creature SHALL take one flat check at DC 5 before the effect. A failure or critical failure SHALL report the action spent and the attack counting for MAP. A success or critical success SHALL affect the target and report neither. counts_for_map describes the attack case; the caller SHALL apply it to the multiple attack penalty only when the action has the attack trait (Rules 2289). resolve_targeting SHALL NOT call spend_actions or strike. No modifier SHALL be added.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=62 (Player Core pg. 442, Concealed)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2416 (Player Core pg. 434, Hidden)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2289 (Player Core pg. 402, Multiple Attack Penalty)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2278 (Player Core pg. 400, Checks (Flat Checks))
**Source**: Ruling R-12 in docs/designer/rules-rulings.md

#### Scenario: A natural 5 at DC 5 affects the target

- **GIVEN** CombatantId(2) observed and concealed from CombatantId(1), and a scripted natural 5
- **WHEN** a non-area targeting check is resolved
- **THEN** the roll is consumed, the DC is 5, the degree is success, and the target is affected
- **AND** the result reports neither the action spent nor the attack counting for MAP

#### Scenario: A natural 4 at DC 5 does not affect the target

- **GIVEN** CombatantId(2) observed and concealed from CombatantId(1), and a scripted natural 4
- **WHEN** a non-area targeting check is resolved
- **THEN** the roll is consumed, the DC is 5, the degree is failure, and the target is not affected
- **AND** the result reports the action spent and the attack counting for MAP

#### Scenario: A natural 1 at DC 5 does not affect the target

- **GIVEN** CombatantId(2) observed and concealed from CombatantId(1), and a scripted natural 1
- **WHEN** a non-area targeting check is resolved
- **THEN** the numeric degree is critical failure
- **AND** the effect resolves as a failure and the target is not affected
- **AND** the result reports the action spent and the attack counting for MAP

#### Scenario: A natural 20 at DC 5 affects the target

- **GIVEN** CombatantId(2) observed and concealed from CombatantId(1), and a scripted natural 20
- **WHEN** a non-area targeting check is resolved
- **THEN** the numeric degree is critical success
- **AND** the effect resolves as a success and the target is affected
- **AND** the result reports neither the action spent nor the attack counting for MAP
