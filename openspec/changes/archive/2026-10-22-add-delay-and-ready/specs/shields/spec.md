# shields

## MODIFIED Requirements

### Requirement: An illegal block writes nothing

The checks SHALL run in this order: NotAHit, NegativeAmount, Declined, then Requested checks for NotConscious, Delayed, NoShield, NotRaised, Broken, and NoReaction, then amount 0, then the remainder. Declined SHALL skip those Requested checks and SHALL pass the full amount to apply_resolved_blow. A failure SHALL return that error, write nothing, and spend nothing. Broken SHALL cover a destroyed shield. A second requested block in the same round SHALL return NoReaction. A direct blow skips it.

**Source**: https://2e.aonprd.com/Feats.aspx?ID=5212 (Player Core pg. 262, Shield Block)
**Source**: https://2e.aonprd.com/Actions.aspx?ID=2294 (Player Core pg. 416, Delay)

#### Scenario: Declining the block leaves the shield intact

- **GIVEN** an Alive combatant at 30 current Hit Points, a ready reaction, and a raised steel shield at 20 Hit Points
- **WHEN** an attack blow of 12 at Success is ShieldBlock::Declined
- **THEN** current Hit Points are 18, shield Hit Points stay 20, and the reaction stays ready

#### Scenario: Declining with no shield deals the blow

- **GIVEN** an Alive combatant at 30 current Hit Points and no shield
- **WHEN** an attack blow of 12 at Success is ShieldBlock::Declined
- **THEN** the result is Ok and current Hit Points are 18

#### Scenario: A shield that is not raised does not block

- **GIVEN** an Alive combatant at 30 current Hit Points, a ready reaction, and a steel shield at 20 Hit Points that is not raised
- **WHEN** an attack blow of 12 at Success is ShieldBlock::Requested
- **THEN** the result is NotRaised, current Hit Points stay 30, shield Hit Points stay 20, and the reaction stays ready

#### Scenario: A creature with no shield does not block

- **GIVEN** an Alive combatant at 30 current Hit Points, a ready reaction, and no shield
- **WHEN** an attack blow of 12 at Success is ShieldBlock::Requested
- **THEN** the result is NoShield, current Hit Points stay 30, and the reaction stays ready

#### Scenario: A spent reaction does not block

- **GIVEN** an Alive combatant at 30 current Hit Points, a reaction that is not ready, and a raised steel shield at 20 Hit Points
- **WHEN** an attack blow of 12 at Success is ShieldBlock::Requested
- **THEN** the result is NoReaction, current Hit Points stay 30, shield Hit Points stay 20, and the reaction stays not ready

#### Scenario: A broken shield does not block

- **GIVEN** an Alive combatant at 30 current Hit Points, a ready reaction, and a raised steel shield at 10 Hit Points, broken threshold 10
- **WHEN** an attack blow of 12 at Success is ShieldBlock::Requested
- **THEN** the result is Broken, current Hit Points stay 30, shield Hit Points stay 10, and the reaction stays ready

#### Scenario: A destroyed shield does not block

- **GIVEN** an Alive combatant at 30 current Hit Points, a ready reaction, and a raised steel shield at 0 Hit Points
- **WHEN** an attack blow of 12 at Success is ShieldBlock::Requested
- **THEN** the result is Broken, current Hit Points stay 30, shield Hit Points stay 0, and the reaction stays ready

#### Scenario: Dying on the direct state does not block

- **GIVEN** vital Dying(1), current Hit Points 0, maximum 9, uses_dying_rules, a ready reaction, and a raised steel shield at 20 Hit Points
- **WHEN** an attack blow of 8 at Success is ShieldBlock::Requested on that state
- **THEN** the result is NotConscious, vital stays Dying(1), current Hit Points stay 0, shield Hit Points stay 20, and the reaction stays ready

#### Scenario: A strike against a dying creature does not block

- **GIVEN** uses_dying_rules, CombatantId(1) at 9 current Hit Points, maximum 9, baked Armor Class 11, a raised steel shield at 20 Hit Points, and a ready reaction, and CombatantId(2) with attack bonus +0, both Perception +0, scripted initiative naturals 10 then 8
- **WHEN** CombatantId(1) has raised the shield and ended the turn, CombatantId(2) Strikes with a natural 11 and damage 1d8+1 with a damage die of 8 at ShieldBlock::Declined, then Strikes with ShieldBlock::Requested and a damage die of 8
- **THEN** the second result is EncounterError::TargetNotConscious
- **AND** vital stays Dying(1), current Hit Points stay 0, and shield Hit Points stay 20
- **AND** CombatantId(1)'s reaction stays ready
- **AND** CombatantId(2) still has 2 actions
- **AND** the second natural and the second damage die are unused

#### Scenario: A delayed creature is rejected before the shield is read

- **GIVEN** Side::Pc CombatantId(1) at (0, 0), Delayed, Alive at 30 Hit Points, Armor Class 18, reaction ready, and no shield, and Side::Enemy CombatantId(2) at (0, 1) whose turn is open with 3 actions, Perception +0, scripted initiative naturals 15 then 10, and a scripted natural 10 and a damage die of 5
- **WHEN** CombatantId(2) Strikes CombatantId(1) with attack bonus +8 and ShieldBlock::Requested
- **THEN** the result is Delayed
- **AND** current Hit Points stay 30 and the reaction stays ready
- **AND** CombatantId(2) still has 3 actions
- **AND** the natural 10 and the damage die are unused
