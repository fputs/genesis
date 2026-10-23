# ready

## REMOVED Requirements

### Requirement: Hidden targets stay with the caller

**Reason**: A readied Strike now calls resolve_targeting. The old requirement forbade the flat check.
**Migration**: Use "A readied Strike resolves targeting before the attack roll". An observed target that is not concealed still draws no flat-check die.

## ADDED Requirements

### Requirement: A readied Strike resolves targeting before the attack roll

After the checks in Fire rejects before it rolls, except CannotTarget, a readied Strike SHALL call resolve_targeting with area_effect false and SHALL NOT reimplement it. The flat check is drawn before the attack d20, and only when the target is concealed or hidden. The attack d20 is drawn only when the target is affected. An observed target that is not concealed draws no flat-check die. The relation SHALL stay unchanged. The Strike SHALL use baked Armor Class.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2299 (Player Core pg. 417, Ready)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=62 (Player Core pg. 442, Concealed)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=79 (Player Core pg. 444, Hidden)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2278 (Player Core pg. 400, Checks (Flat Checks))

#### Scenario: An observed readied Strike draws the attack die first

- **GIVEN** Alive CombatantId(1) at (0, 0), Medium, reach 5, with a stored non-agile Strike, snapshot 0, attack bonus +8, and Alive CombatantId(2) at (0, 1), Medium, Armor Class 18, with no stored relation, and a script of 10 then 5
- **WHEN** the Strike fires
- **THEN** the total is 18 and the degree is success
- **AND** the penalty is 0
- **AND** the scripted 5 is still unused

#### Scenario: A natural 11 at DC 11 then the readied attack

- **GIVEN** Alive CombatantId(1) at (0, 0), Medium, reach 5, with a stored non-agile Strike, snapshot 0, attack bonus +8, and Alive CombatantId(2) at (0, 1), Medium, Armor Class 18, hidden and not concealed from CombatantId(1) with Stealth DC 17, and scripted naturals 11 then 10
- **WHEN** the Strike fires
- **THEN** the flat check DC is 11 and the Strike is Rolled
- **AND** the penalty is 0, the total is 18, and the degree is success
- **AND** the relation is still hidden and not concealed
- **AND** the reaction is spent and the ready is cleared

#### Scenario: A natural 5 at DC 5 keeps the frozen penalty

- **GIVEN** Alive CombatantId(1) at (0, 0), Medium, reach 5, with a stored non-agile Strike, snapshot 1, attack bonus +8, and Alive CombatantId(2) at (0, 1), Medium, Armor Class 18, observed and concealed from CombatantId(1), and scripted naturals 5 then 15
- **WHEN** the Strike fires
- **THEN** the flat check DC is 5 and the Strike is Rolled
- **AND** the penalty is -5, the total is 18, and the degree is success
- **AND** the attack count stays 1

### Requirement: A lost readied Strike spends the reaction only

When the target is not affected, a readied Strike SHALL return LostStrike, spend the reaction, and clear the ready. It SHALL NOT increment the attack count, spend an action, or draw the attack d20 or a damage die. Hit Points SHALL stay unchanged. The creature's attack count SHALL stay the count stored at Ready. After Ok(LostStrike) the caller SHALL still call note_strike (DEV-43).

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2299 (Player Core pg. 417, Ready)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2416 (Player Core pg. 434, Hidden)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2289 (Player Core pg. 402, Multiple Attack Penalty)
**Source**: R-05 in docs/designer/rules-rulings.md
**Source**: R-12 in docs/designer/rules-rulings.md

#### Scenario: A natural 4 at DC 5 loses the readied Strike

- **GIVEN** Alive CombatantId(1) at (0, 0), Medium, reach 5, attack count 1, actions 0, with a stored non-agile Strike, snapshot 1, attack bonus +8, a ready reaction, and Alive CombatantId(2) at (0, 1), Medium, Armor Class 18, 20 Hit Points, observed and concealed from CombatantId(1), and scripted naturals 4, then 15, then a damage die of 1
- **WHEN** the Strike fires
- **THEN** the result is LostStrike, the flat natural is 4, the DC is 5, and the degree is failure
- **AND** the reaction is spent and the ready is cleared
- **AND** the attack count stays 1 and actions stay 0
- **AND** Hit Points stay 20
- **AND** the scripted 15 and the damage die of 1 are still unused
- **WHEN** CombatantId(1)'s next turn starts and its attack-trait penalty is read
- **THEN** the penalty is 0

### Requirement: An undetected readied Strike is CannotTarget

An undetected target SHALL return ReadyError::CannotTarget after TargetDead, an attack total past the bounds, and NoReaction. The reaction SHALL stay ready, the ready SHALL stay stored, and no die SHALL be drawn.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=96 (Player Core pg. 447, Undetected)
**Source**: https://2e.aonprd.com/Actions.aspx?ID=2299 (Player Core pg. 417, Ready)

#### Scenario: An undetected readied Strike spends nothing

- **GIVEN** Alive CombatantId(1) at (0, 0), Medium, reach 5, reaction ready, with a stored non-agile Strike on CombatantId(2) at (0, 1), Medium, who is undetected and not concealed with Stealth DC 17, and a scripted natural 11
- **WHEN** the Strike fires
- **THEN** the error is CannotTarget
- **AND** the reaction stays ready and the ready stays stored
- **AND** the scripted 11 is still unused

#### Scenario: A dead undetected readied target is TargetDead

- **GIVEN** Alive CombatantId(1) at (0, 0), Medium, reach 5, reaction ready, with a stored non-agile Strike on Dead CombatantId(2) at (0, 1), Medium, who is undetected with Stealth DC 17, and a scripted natural 11
- **WHEN** the Strike fires
- **THEN** the error is TargetDead
- **AND** the error is not CannotTarget
- **AND** the reaction stays ready and the ready stays stored
- **AND** the scripted 11 is still unused

## MODIFIED Requirements

### Requirement: Fire rejects before it rolls

Fire SHALL reject, in order, an unknown readier, an unknown subject, no stored ready, the wrong action, a readier who is not Alive, a delayed readier, a trigger mismatch, a begun Strike out of reach, a Dead target, then no reaction, then CannotTarget. An illegal path, PositioningError from Dismiss, and an attack total past the bounds SHALL fail before the reaction is spent or the d20 is drawn. PositioningError SHALL be returned. No earlier check writes, and no die is rolled.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2299 (Player Core pg. 417, Ready)

#### Scenario: No stored ready is rejected before a dead target

- **GIVEN** Alive CombatantId(1) at (0, 0), Medium, has no stored ready, and Dead CombatantId(2) at (0, 1), Medium, and a scripted natural 10
- **WHEN** CombatantId(2) begins a Strike and resolve_reactions runs
- **THEN** the result is empty and the error is not TargetDead
- **AND** the natural 10 is unused

#### Scenario: The wrong action is rejected before a delayed readier

- **GIVEN** Delayed CombatantId(1) at (0, 0), Medium, whose stored ready is a non-agile Strike on CombatantId(2) at (0, 1), Medium, reaction ready, and a scripted natural 10
- **WHEN** that stored Strike is checked as a Stride
- **THEN** the error is WrongAction
- **AND** the ready stays stored
- **AND** the natural 10 is unused

#### Scenario: A delayed readier is rejected before the trigger is matched

- **GIVEN** Delayed CombatantId(1) at (0, 0), Medium, reach 5, with a stored non-agile Strike on CombatantId(2) beginning a Strike, Alive CombatantId(2) at (0, 1), Medium, reaction ready, and a scripted natural 10
- **WHEN** CombatantId(2) begins a Strike and resolve_reactions runs
- **THEN** the error is Delayed
- **AND** the ready stays stored
- **AND** the natural 10 is unused

#### Scenario: A dying readier is rejected before a dead target

- **GIVEN** Dying(1) CombatantId(1) at (0, 0), Medium, 0 Hit Points, with a stored non-agile Strike, reaction ready, and Dead CombatantId(2) at (0, 1), Medium, and a scripted natural 10
- **WHEN** CombatantId(2) begins a Strike and resolve_reactions runs
- **THEN** the error is NotConscious
- **AND** the reaction flag is unchanged
- **AND** the ready stays stored
- **AND** the natural 10 is unused

#### Scenario: A dead target is rejected before a missing reaction

- **GIVEN** Alive CombatantId(1) at (0, 0), Medium, reach 5, with a stored non-agile Strike and no reaction, and Dead CombatantId(2) at (0, 1), Medium, and a scripted natural 10
- **WHEN** CombatantId(2) begins a Strike and resolve_reactions runs
- **THEN** the error is TargetDead
- **AND** the ready stays stored
- **AND** the natural 10 is unused
