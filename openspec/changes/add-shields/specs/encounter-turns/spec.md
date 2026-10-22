# encounter-turns

## MODIFIED Requirements

### Requirement: Strike damage uses resolved blows

A Strike whose degree is Success or CriticalSuccess SHALL pass damage_dealt's result and that degree to apply_resolved_blow. CriticalSuccess SHALL be a critical blow and Success an ordinary blow. The Strike SHALL NOT call note_hit_points. Dying, massive damage, temporary Hit Points, and the initiative report SHALL follow dying-and-victory. The Strike SHALL keep the target's baked Armor Class. The Strike SHALL NOT reorder combatants. Shield Hit Points SHALL stay unchanged unless a legal Shield Block is requested.

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

#### Scenario: A legal Shield Block changes shield Hit Points

- **GIVEN** CombatantId(1) with 30 current Hit Points, maximum 30, baked Armor Class 11, and a raised steel shield at 20 Hit Points, and CombatantId(2) with attack bonus +0, both Perception +0, scripted initiative naturals 10 then 8
- **WHEN** CombatantId(1) has raised that shield, and CombatantId(2) Strikes with a natural 11, damage 1d8+4 with a damage die of 8, and ShieldBlock::Requested
- **THEN** the degree is success
- **AND** CombatantId(1)'s current Hit Points are 23
- **AND** shield Hit Points are 13
