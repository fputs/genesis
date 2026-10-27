# encounter-turns

## MODIFIED Requirements

### Requirement: Ending a turn drops actions and keeps the reaction

Ending a turn SHALL set that combatant's actions to 0 and SHALL keep an unspent reaction ready. The end step SHALL run once. `end_turn` SHALL take the injected dice. With no persistent damage, the end step SHALL NOT roll a die and SHALL NOT change Hit Points, temporary Hit Points, or wounded. `end_turn` with no open turn SHALL return TurnNotOpen and SHALL NOT roll.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2430 (Player Core pg. 436, Step 3: End Your Turn)

#### Scenario: Ending early drops leftover actions and keeps the reaction

- **GIVEN** an open turn with 2 actions left and the reaction ready, current Hit Points 12, no persistent damage, and a scripted natural 8
- **WHEN** the turn ends with that die
- **THEN** actions are 0
- **AND** the reaction is still ready
- **AND** Hit Points stay 12
- **AND** temporary Hit Points stay 0
- **AND** wounded stays 0
- **AND** the scripted 8 is still unused

#### Scenario: End before a start returns an error

- **GIVEN** initiative has been rolled and no turn has started, and a scripted natural 8 is next
- **WHEN** a turn end is requested with that die
- **THEN** the error is TurnNotOpen
- **AND** every combatant still has 0 actions
- **AND** the scripted 8 is unused
