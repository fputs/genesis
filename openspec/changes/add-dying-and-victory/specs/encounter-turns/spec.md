# encounter-turns

## REMOVED Requirements

### Requirement: Reaching 0 Hit Points floors at 0

**Reason**: Strike damage goes through apply_resolved_blow, so massive damage and the dying value are dying-and-victory.
**Migration**: Requirement "Strike damage uses resolved blows".

## ADDED Requirements

### Requirement: Strike damage uses resolved blows

A Strike whose degree is Success or CriticalSuccess SHALL pass damage_dealt's result and that degree to apply_resolved_blow. CriticalSuccess SHALL be a critical blow and Success an ordinary blow. The Strike SHALL NOT call note_hit_points. Dying, massive damage, temporary Hit Points, and the initiative report SHALL follow dying-and-victory. The Strike SHALL keep the target's baked Armor Class. The Strike SHALL NOT reorder combatants. Shield Hit Points SHALL stay unchanged.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2324 (Player Core pg. 410, Getting Knocked Out)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2332 (Player Core pg. 412, Massive Damage)

#### Scenario: Without dying rules, 0 Hit Points is death

- **GIVEN** a creature with 9 Hit Points, uses_dying_rules false, attack bonus +0, Armor Class 11, a scripted natural 11, and damage 1d8+1 with a scripted damage die of 8
- **WHEN** the Strike is resolved
- **THEN** current Hit Points are 0
- **AND** the combatant is dead
- **AND** wounded stays 0
- **AND** the Creature still has 9 Hit Points

#### Scenario: With dying rules, a Strike to 0 sets Dying(1) and a pending report

- **GIVEN** a creature with maximum Hit Points 9, current Hit Points 9, and uses_dying_rules true, acting after the striker, and a Strike of attack bonus +0 against Armor Class 11, natural 11, and damage 1d8+1 with a damage die of 8
- **WHEN** that Strike is resolved and the report is taken
- **THEN** vital is Dying(1)
- **AND** wounded stays 0
- **AND** the initiative order is unchanged
- **AND** the take is Some(InitiativeMove::Pending)

#### Scenario: A massive Strike leaves no dying value

- **GIVEN** uses_dying_rules true, maximum Hit Points 10, current Hit Points 10, no pending report, attack bonus +0, Armor Class 11, a scripted natural 11, and damage 1d8+12 with a scripted damage die of 8
- **WHEN** the Strike is resolved and the report is taken
- **THEN** vital is Dead
- **AND** no dying value is stored
- **AND** the take is None
- **AND** wounded is unchanged

#### Scenario: A Strike does not change the shield

- **GIVEN** a target with a shield of 20 Hit Points that is not raised, attack bonus +0, Armor Class 11, a scripted natural 11, and damage 1d4 with a scripted damage die of 4
- **WHEN** that Strike is resolved
- **THEN** the shield still has 20 Hit Points
- **AND** the shield is not raised

## MODIFIED Requirements

### Requirement: Strike damage removes temporary Hit Points first

Damage from a Strike SHALL remove temporary Hit Points first. Only the remainder SHALL reduce current Hit Points. Damage of 0 SHALL leave both unchanged. One helper SHALL apply that amount. apply_dice_damage SHALL stay unchanged. When the temporary amount reaches 0, its rounds SHALL be 0.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2321 (Player Core pg. 410, Temporary Hit Points)

#### Scenario: Damage removes temporary Hit Points, then Hit Points

- **GIVEN** 20 current Hit Points, 5 temporary Hit Points for 3 rounds, attack bonus +0, Armor Class 11, a scripted natural 11, and damage 1d8+4 with a scripted damage die of 3
- **WHEN** the Strike is resolved
- **THEN** the damage dealt is 7
- **AND** temporary Hit Points are 0
- **AND** rounds are 0
- **AND** current Hit Points are 18

#### Scenario: Damage within temporary Hit Points leaves Hit Points

- **GIVEN** 20 current Hit Points, 5 temporary Hit Points, attack bonus +0, Armor Class 11, a scripted natural 11, and damage 1d4 with a scripted damage die of 3
- **WHEN** the Strike is resolved
- **THEN** temporary Hit Points are 2
- **AND** current Hit Points stay 20

#### Scenario: Zero damage leaves temporary Hit Points

- **GIVEN** 5 temporary Hit Points, attack bonus +0, Armor Class 11, a scripted natural 11, and damage 1d4-5 with a scripted damage die of 1
- **WHEN** the Strike is resolved
- **THEN** temporary Hit Points stay 5
- **AND** current Hit Points are unchanged
