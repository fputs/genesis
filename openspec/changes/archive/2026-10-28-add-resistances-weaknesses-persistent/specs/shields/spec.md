# shields

## MODIFIED Requirements

### Requirement: Persistent damage is not Shield Block

Shield Block SHALL trigger only on bludgeoning, piercing, or slashing damage from an attack, and only on the amount remaining after immunity, weakness, and resistance. A non-physical type, or immunity to the attack's type, SHALL NOT trigger it and SHALL NOT spend the reaction. Persistent damage SHALL NOT be passed to the attack blow. It SHALL go through one ordinary blow and SHALL leave the shield and the reaction unchanged.

**Source**: https://2e.aonprd.com/Feats.aspx?ID=5212 (Player Core pg. 262, Shield Block)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2309 (Player Core pg. 407, Step 3)

#### Scenario: Persistent damage leaves the shield and the reaction

- **GIVEN** an Alive combatant at 30 current Hit Points, maximum 30, a ready reaction, and a raised steel shield at 20 Hit Points
- **WHEN** persistent damage of 5 is applied with apply_resolved_blow at Success
- **THEN** current Hit Points are 25, shield Hit Points stay 20, and the reaction stays ready

#### Scenario: A fire Strike does not block

- **GIVEN** a target at 20 Hit Points with a steel shield that is not raised, a ready reaction, and Armor Class 11
- **WHEN** a Fire Strike hits for a damage die of 8 with Shield Block requested
- **THEN** Hit Points are 12, shield Hit Points stay unchanged, and the reaction stays ready

#### Scenario: Immunity to slashing does not block

- **GIVEN** a target immune to Slashing, at 20 Hit Points, with a raised steel shield, a ready reaction, and Armor Class 11
- **WHEN** a Slashing Strike hits for a damage die of 8 with Shield Block requested
- **THEN** Hit Points stay 20, shield Hit Points stay unchanged, and the reaction stays ready

#### Scenario: Resistance to 0 does not spend the reaction

- **GIVEN** a target at 20 Hit Points with resistance 10 to Slashing, a raised steel shield at 20 Hit Points, a ready reaction, and Armor Class 11
- **WHEN** a Slashing Strike hits for a damage die of 8 with Shield Block requested
- **THEN** Hit Points stay 20, shield Hit Points stay 20, and the reaction stays ready

#### Scenario: Resistance is applied before hardness

- **GIVEN** a target at 30 Hit Points with resistance 5 to Slashing, a raised steel shield of hardness 5 at 20 Hit Points, and Armor Class 11
- **WHEN** a Slashing Strike hits for 12 damage with Shield Block requested
- **THEN** current Hit Points are 28 and shield Hit Points are 18
