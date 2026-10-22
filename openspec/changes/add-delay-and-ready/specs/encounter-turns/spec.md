# encounter-turns

## MODIFIED Requirements

### Requirement: A reaction attack penalty stays 0

An attack-trait check while it is not that creature's open turn SHALL take penalty 0 and SHALL NOT change that creature's attack count, except a readied attack. A readied attack SHALL read the attack count snapshotted when Ready resolved, SHALL take that count's penalty, and SHALL NOT change the attack count or the actions remaining. The snapshot SHALL remain until that creature's next start. The turn SHALL be taken from the encounter.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2289 (Player Core pg. 402, Multiple Attack Penalty)
**Source**: https://2e.aonprd.com/Actions.aspx?ID=2299 (Player Core pg. 417, Ready)
**Source**: R-05 in docs/designer/rules-rulings.md. The snapshot is the attack count. A reaction that is not readied stays at 0.

#### Scenario: A reaction after two Strikes keeps penalty 0

- **GIVEN** a non-agile creature that has resolved two Strikes this turn, and whose turn has since ended
- **WHEN** an attack-trait penalty is read for that creature while the turn is not open
- **THEN** the penalty is 0
- **AND** the read rolls no die
- **WHEN** that creature's next turn starts and the penalty is read
- **THEN** the penalty is 0

#### Scenario: A readied attack keeps the stored count off the open turn

- **GIVEN** a non-agile creature with a stored ready whose attack count is 2, and whose turn has since ended
- **WHEN** that readied attack's penalty is read while the turn is not open
- **THEN** the penalty is -10
- **AND** the read rolls no die
- **AND** the stored count stays 2
- **AND** that creature's actions stay 0
