# ready

## REMOVED Requirements

### Requirement: Hidden targets stay with the caller

**Reason**: A readied Strike now calls resolve_targeting. The old requirement forbade the flat check.
**Migration**: Use "A readied Strike resolves targeting before the attack roll". An observed target that is not concealed still draws no flat-check die.

## ADDED Requirements

### Requirement: A readied Strike resolves targeting before the attack roll

After the checks in Fire rejects before it rolls, a readied Strike SHALL call resolve_targeting with area_effect false and SHALL NOT reimplement it. The flat check is drawn before the attack d20, and only when the target is concealed or hidden. The attack d20 is drawn only when the target is affected. An observed target that is not concealed draws no flat-check die. The relation SHALL stay unchanged.

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

#### Scenario: Each reaction draws its own flat check

- **GIVEN** the order is Side::Enemy CombatantId(1) at (0, 0) then Side::Pc CombatantId(2) at (1, 0) then Alive CombatantId(3) at (0, 1), all Medium, reach 5, CombatantId(3) has 20 Hit Points, Armor Class 18, and 3 actions, and is observed and concealed from both, CombatantId(1) has attack bonus +8, and CombatantId(2) has a stored non-agile Strike, snapshot 0, attack bonus +8, on CombatantId(3) beginning a Stride
- **WHEN** CombatantId(3) begins a Stride and resolve_reactions is called once with one caller entry, CombatantId(1) only: ReactiveStrike::Available, attack bonus +8, no modifiers, both sizes Medium, and square (0, 0), then CombatantId(1)'s flat natural is 4, and CombatantId(2)'s flat natural is 5, attack natural 10, and damage 1
- **THEN** CombatantId(1)'s result is LostStrike and that reaction is spent
- **AND** CombatantId(2)'s penalty is 0, the total is 18, and the degree is success
- **AND** CombatantId(3) is at 19 Hit Points and still has 3 actions
- **AND** CombatantId(2)'s reaction is spent after CombatantId(1)'s reaction

### Requirement: A lost readied Strike spends the reaction only

When the target is not affected, a readied Strike SHALL return LostStrike, spend the reaction, and clear the ready. It SHALL NOT increment the attack count, spend an action, or draw the attack d20 or a damage die. Hit Points SHALL stay unchanged. The creature's attack count SHALL stay the count stored at Ready.

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

An undetected target SHALL return ReadyError::CannotTarget after TargetDead and NoReaction. The reaction SHALL stay ready, the ready SHALL stay stored, and no die SHALL be drawn.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=96 (Player Core pg. 447, Undetected)
**Source**: https://2e.aonprd.com/Actions.aspx?ID=2299 (Player Core pg. 417, Ready)

#### Scenario: An undetected readied Strike spends nothing

- **GIVEN** Alive CombatantId(1) at (0, 0), Medium, reach 5, reaction ready, with a stored non-agile Strike on CombatantId(2) at (0, 1), Medium, who is undetected and not concealed with Stealth DC 17, and a scripted natural 11
- **WHEN** the Strike fires
- **THEN** the error is CannotTarget
- **AND** the reaction stays ready and the ready stays stored
- **AND** the scripted 11 is still unused

#### Scenario: A dead undetected target is TargetDead

- **GIVEN** Alive CombatantId(1) at (0, 0), Medium, reach 5, reaction ready, with a stored non-agile Strike on Dead CombatantId(2) at (0, 1), Medium, who is undetected with Stealth DC 17, and a scripted natural 11
- **WHEN** the Strike fires
- **THEN** the error is TargetDead
- **AND** the error is not CannotTarget
- **AND** the reaction stays ready and the ready stays stored
- **AND** the scripted 11 is still unused
