# positioning

## MODIFIED Requirements

### Requirement: Posture and cover are caller values

The Posture value and a Take Cover benefit SHALL be values the caller keeps. Drop Prone and Stand SHALL NOT write combatant state. A prone condition SHALL NOT be that Posture and SHALL NOT change the posture result. A positioning call SHALL NOT change Hit Points, actions remaining, or the reaction.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=88 (Player Core pg. 445, Prone)

#### Scenario: Drop Prone leaves the combatant state

- **GIVEN** a standing creature with Mobility::Free, a combatant state with 2 actions remaining and the reaction not ready, and square (3, 4)
- **WHEN** Drop Prone succeeds without that state as an argument
- **THEN** the returned posture is prone on square (3, 4)
- **AND** the state still has 2 actions remaining and the reaction is still not ready
- **AND** the condition list is empty

#### Scenario: A prone condition does not change Drop Prone

- **GIVEN** a standing caller at square (3, 4) with Mobility::Free, and a combatant state that already has prone
- **WHEN** Drop Prone succeeds without that state as an argument
- **THEN** the returned posture is prone on square (3, 4)
- **AND** the condition list still has prone
