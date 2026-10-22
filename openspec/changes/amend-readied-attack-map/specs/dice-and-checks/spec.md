# dice-and-checks

## MODIFIED Requirements

### Requirement: Multiple attack penalty on your turn

On the creature's own turn, the second attack-trait check SHALL take -5 and the third and later SHALL take -10. An agile current weapon SHALL make those penalties -4 and -8. That current weapon SHALL set the penalty. A reaction on another creature's turn SHALL NOT add to this creature's own-turn count. It SHALL take no penalty, except a readied attack, which SHALL use the penalty for the attack count frozen when Ready resolved. Every attack-trait check on the creature's own turn SHALL count.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2289 (Player Core pg. 402, Multiple Attack Penalty)
**Source**: https://2e.aonprd.com/Actions.aspx?ID=2299 (Player Core pg. 417, Ready)

#### Scenario: The first check on your turn has no penalty

- **GIVEN** the creature's first attack-trait check on its own turn, a non-agile weapon, attack bonus +8, DC 18, and a scripted natural 10
- **WHEN** the check is resolved
- **THEN** the multiple attack penalty is 0, the total is 18, and the degree is success

#### Scenario: The second non-agile check takes -5

- **GIVEN** the creature's second attack-trait check on its own turn, a non-agile weapon, attack bonus +8, DC 18, and a scripted natural 15
- **WHEN** the check is resolved
- **THEN** the multiple attack penalty is -5, the total is 18, and the degree is success

#### Scenario: The third and later non-agile checks take -10

- **GIVEN** the creature's third attack-trait check on its own turn, a non-agile weapon, attack bonus +8, DC 18, and a scripted natural 15
- **WHEN** the check is resolved
- **THEN** the multiple attack penalty is -10, the total is 13, and the degree is failure

#### Scenario: A fourth non-agile check stays at -10

- **GIVEN** the creature's fourth attack-trait check on its own turn, a non-agile weapon, attack bonus +8, DC 18, and a scripted natural 15
- **WHEN** the check is resolved
- **THEN** the multiple attack penalty is -10 and the total is 13

#### Scenario: The second agile check takes -4

- **GIVEN** the creature's second attack-trait check on its own turn, an agile weapon, attack bonus +8, DC 18, and a scripted natural 14
- **WHEN** the check is resolved
- **THEN** the multiple attack penalty is -4, the total is 18, and the degree is success

#### Scenario: The third agile check takes -8

- **GIVEN** the creature's third attack-trait check on its own turn, an agile weapon, attack bonus +8, DC 18, and a scripted natural 18
- **WHEN** the check is resolved
- **THEN** the multiple attack penalty is -8, the total is 18, and the degree is success

#### Scenario: The current weapon sets the penalty

- **GIVEN** one prior attack-trait check on this turn with a non-agile weapon, then an agile weapon, attack bonus +8, DC 16, and a scripted natural 12
- **WHEN** the agile check is resolved
- **THEN** the penalty is -4, the total is 16, and the degree is success

#### Scenario: A non-agile weapon after an agile weapon uses -5

- **GIVEN** one prior attack-trait check on this turn with an agile weapon, then a non-agile weapon, attack bonus +8, DC 16, and a scripted natural 12
- **WHEN** the non-agile check is resolved
- **THEN** the penalty is -5, the total is 15, and the degree is failure

#### Scenario: A reaction on another turn takes no penalty and does not count

- **GIVEN** a reaction attack-trait check that is not a readied attack, while it is another creature's turn, after the reactor already made two attack-trait checks on its own earlier turn, attack bonus +8, DC 18, and a scripted natural 10
- **WHEN** the reaction check is resolved, and the reactor's next own turn then makes its first attack-trait check with a scripted natural 11, attack bonus +8, and DC 19
- **THEN** the reaction penalty is 0 and its total is 18, and the first check of the next own turn also has penalty 0 and total 19

#### Scenario: An attack-trait check that is not a Strike still counts

- **GIVEN** an attack-trait check that is not a Strike as the first check on the creature's own turn, then a non-agile Strike with attack bonus +8, DC 19, and a scripted natural 15
- **WHEN** the Strike is resolved
- **THEN** the Strike's penalty is -5, the total is 18, and the degree is failure

#### Scenario: A readied non-agile attack after one check takes -5

- **GIVEN** one prior attack-trait check on the creature's turn with an agile weapon, a readied attack-trait check on another creature's turn whose frozen attack count is 1, a non-agile weapon for that readied check, attack bonus +8, DC 18, and a scripted natural 15
- **WHEN** the readied check is resolved
- **THEN** the multiple attack penalty is -5, the total is 18, and the degree is success
- **AND** the penalty is not -4

#### Scenario: A readied attack with no prior check takes no penalty

- **GIVEN** no prior attack-trait check on the creature's turn, a readied attack-trait check on another creature's turn whose frozen attack count is 0, a non-agile weapon, attack bonus +8, DC 18, and a scripted natural 10
- **WHEN** the readied check is resolved
- **THEN** the multiple attack penalty is 0, the total is 18, and the degree is success

#### Scenario: A readied agile attack after two checks takes -8

- **GIVEN** two prior attack-trait checks on the creature's turn with a non-agile weapon, a readied attack-trait check on another creature's turn whose frozen attack count is 2, an agile weapon for that readied check, attack bonus +8, DC 18, and a scripted natural 18
- **WHEN** the readied check is resolved
- **THEN** the multiple attack penalty is -8, the total is 18, and the degree is success
- **AND** the penalty is not -10

#### Scenario: A readied non-agile attack after two checks takes -10

- **GIVEN** two prior attack-trait checks on the creature's turn, a readied attack-trait check on another creature's turn whose frozen attack count is 2, a non-agile weapon for that readied check, attack bonus +8, DC 18, and a scripted natural 15
- **WHEN** the readied check is resolved
- **THEN** the multiple attack penalty is -10, the total is 13, and the degree is failure

#### Scenario: A readied attack leaves the next own turn at no penalty

- **GIVEN** a readied non-agile attack-trait check on another creature's turn whose frozen attack count is 1, attack bonus +8, DC 18, and a scripted natural 15, and that check does not add to the own-turn count, so the creature's next own turn's first attack-trait check is a non-agile weapon with prior count 0, attack bonus +8, DC 18, and a scripted natural 10
- **WHEN** the readied check is resolved and then that next own-turn check is resolved
- **THEN** the readied penalty is -5 and its total is 18, and the first check of the next own turn has penalty 0 and total 18

#### Scenario: A Reactive Strike on the same round takes no penalty

- **GIVEN** two attack-trait checks already made on the creature's own turn, then a Reactive Strike that is not a readied attack while it is another creature's turn, attack bonus +8, DC 18, and a scripted natural 10
- **WHEN** the Reactive Strike is resolved
- **THEN** the penalty is 0 and the total is 18
- **AND** the creature's own-turn attack count is unchanged
