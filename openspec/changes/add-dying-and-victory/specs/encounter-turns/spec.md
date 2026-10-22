# encounter-turns

## REMOVED Requirements

### Requirement: Reaching 0 Hit Points does not apply the later rules

**Reason**: Strike damage goes through apply_resolved_blow, so massive damage and the dying value are dying-and-victory.
**Migration**: Requirement "Strike damage uses resolved blows".

## ADDED Requirements

### Requirement: Strike damage uses resolved blows

A Strike whose degree is Success or CriticalSuccess SHALL pass damage_dealt's result and that degree to apply_resolved_blow. CriticalSuccess SHALL be a critical blow and Success an ordinary blow. The Strike SHALL NOT call note_hit_points. Dying, massive damage, temporary Hit Points, and the initiative report SHALL follow dying-and-victory. The Strike SHALL keep the target's baked Armor Class. The Strike SHALL NOT reorder combatants. Shield Hit Points SHALL stay unchanged.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2324 (Player Core pg. 410, Getting Knocked Out)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2332 (Player Core pg. 412, Massive Damage)

#### Scenario: Without dying rules, 0 Hit Points is death

- **GIVEN** a creature with 9 Hit Points, `uses_dying_rules` false, attack bonus +0, Armor Class 11, a scripted natural 11, and damage 1d8+1 with a scripted damage die of 8
- **WHEN** the Strike is resolved
- **THEN** current Hit Points are 0
- **AND** the combatant is dead
- **AND** wounded stays 0
- **AND** the Creature still has 9 Hit Points

#### Scenario: With dying rules, 0 Hit Points does not move initiative

- **GIVEN** a creature with maximum Hit Points 9, current Hit Points 9, and `uses_dying_rules` true, acting after the striker, and a Strike of attack bonus +0 against Armor Class 11, natural 11, and damage 1d8+1 with a damage die of 8
- **WHEN** that Strike is resolved and the report is taken
- **THEN** vital is Dying(1)
- **AND** wounded stays 0
- **AND** the initiative order is unchanged
- **AND** the take is Some(InitiativeMove::Pending)

#### Scenario: A massive Strike leaves no dying value

- **GIVEN** `uses_dying_rules` true, maximum Hit Points 10, current Hit Points 10, no pending report, attack bonus +0, Armor Class 11, a scripted natural 11, and damage 1d8+12 with a scripted damage die of 8
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

### Requirement: The start step changes nothing

The start step SHALL run before actions are regained. It SHALL NOT roll a die and SHALL NOT change Hit Points, temporary Hit Points, or wounded.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2428 (Player Core pg. 435, Step 1: Start Your Turn)

#### Scenario: A start with dying entered rolls nothing

- **GIVEN** a creature at vital Dying(1), at 0 Hit Points, and a scripted natural 8
- **WHEN** that creature's turn starts
- **THEN** the scripted 8 is still unused
- **AND** Hit Points stay 0
- **AND** wounded stays 0
- **AND** the creature has 3 actions

#### Scenario: An unconscious creature's start rolls nothing

- **GIVEN** a creature at vital Unconscious, at 0 Hit Points, and a scripted natural 8
- **WHEN** that creature's turn starts
- **THEN** the scripted 8 is still unused
- **AND** Hit Points stay 0
- **AND** wounded stays 0
- **AND** the creature has 3 actions

### Requirement: A dead combatant is skipped

A dead combatant SHALL remain in the order and SHALL NOT start a turn. The next living combatant SHALL start instead. A creature at Dying(1) or Unconscious SHALL still start a turn.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2428 (Player Core pg. 435, Step 1: Start Your Turn)

#### Scenario: A dead combatant in the middle is skipped

- **GIVEN** the order CombatantId(1), CombatantId(2), CombatantId(3), with CombatantId(2) already dead
- **WHEN** the first turn starts and ends, and the next turn starts
- **THEN** the first start is CombatantId(1)
- **AND** the next start is CombatantId(3)
- **AND** the round is still 1
- **AND** CombatantId(2) has 0 actions

#### Scenario: The only living combatant starts the next round

- **GIVEN** CombatantId(1) alive and CombatantId(2) dead, with CombatantId(1) earlier in the order
- **WHEN** CombatantId(1) starts and ends a turn, and a start is requested
- **THEN** the round is 2
- **AND** the started combatant is CombatantId(1)

#### Scenario: No living combatant returns an error

- **GIVEN** every combatant is already dead and the round is 1
- **WHEN** a start is requested
- **THEN** the result is a typed error
- **AND** the round stays 1
- **AND** the order is unchanged

#### Scenario: Dying and unconscious creatures still start a turn

- **GIVEN** the order CombatantId(1) at Dying(1), CombatantId(2) Dead, and CombatantId(3) Unconscious
- **WHEN** each living turn starts
- **THEN** CombatantId(1) starts with 3 actions
- **AND** CombatantId(3) starts with 3 actions
- **AND** CombatantId(2) does not start

### Requirement: Strike damage removes temporary Hit Points first

Damage from a Strike SHALL remove temporary Hit Points first. Only the remainder SHALL reduce current Hit Points. Damage of 0 SHALL leave both unchanged. When the temporary amount reaches 0, its rounds SHALL be 0.

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
