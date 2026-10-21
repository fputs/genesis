# dice-and-checks Specification

## Purpose
Resolves caller-injected d20 checks, flat checks, and the untyped multiple attack penalty so later encounter actions can roll without a hidden generator.

## Requirements

### Requirement: Callers inject dice

Callers SHALL pass the die source into every roll. No roll SHALL read a process-wide generator. One seed MUST reproduce the same sequence from a new seeded source. A scripted source MUST panic after its values are consumed, and MUST panic when a value is outside 1 through the requested number of faces.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2278 (Player Core pg. 400, Checks)

#### Scenario: The same seed repeats

- **GIVEN** two seeded sources created with seed 42
- **WHEN** each rolls a d20, a d8, and a d6 in that order
- **THEN** the two sequences are equal

#### Scenario: Scripted dice panic when exhausted

- **GIVEN** a scripted source holding only 8
- **WHEN** it rolls a d20 and then another d20 is requested
- **THEN** the first roll is 8 and the second request panics

#### Scenario: Scripted dice panic out of range

- **GIVEN** a scripted source holding 0, then one holding 21, then one holding 6
- **WHEN** those values are rolled on a d20, a d20, and a d4
- **THEN** each request panics

#### Scenario: Scripted bounds are legal

- **GIVEN** a scripted source holding 1 and then 20
- **WHEN** both values are rolled on a d20
- **THEN** the rolls are 1 and then 20

### Requirement: Check degrees follow the DC bands

A d20 check SHALL be a success when the total is at least the DC, a critical success when the total is at least the DC plus 10, and a critical failure when the total is at most the DC minus 10. Every other total SHALL be a failure.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2286 (Player Core pg. 401, Step 4: Degree of Success)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2278 (Player Core pg. 400, Checks)

#### Scenario: Total equal to the DC is a success

- **GIVEN** a scripted natural 11, a modifier of +4, and DC 15
- **WHEN** the check is resolved
- **THEN** the total is 15 and the degree is success

#### Scenario: Beating the DC by 9 is a success

- **GIVEN** a scripted natural 14, a modifier of +10, and DC 15
- **WHEN** the check is resolved
- **THEN** the total is 24 and the degree is success

#### Scenario: Beating the DC by 10 is a critical success

- **GIVEN** a scripted natural 15, a modifier of +10, and DC 15
- **WHEN** the check is resolved
- **THEN** the total is 25 and the degree is critical success

#### Scenario: Missing the DC by 9 is a failure

- **GIVEN** a scripted natural 6, a modifier of +0, and DC 15
- **WHEN** the check is resolved
- **THEN** the total is 6 and the degree is failure

#### Scenario: Missing the DC by 10 is a critical failure

- **GIVEN** a scripted natural 5, a modifier of +0, and DC 15
- **WHEN** the check is resolved
- **THEN** the total is 5 and the degree is critical failure

### Requirement: Natural 20 and natural 1 shift one degree first

A natural 20 SHALL improve the numeric degree by one step, and a natural 1 SHALL worsen it by one step. A critical success SHALL NOT improve further. A critical failure SHALL NOT worsen further. This shift SHALL be applied before any other effect that changes the degree.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2287 (Player Core pg. 401, Natural 1 and Natural 20)

#### Scenario: Natural 20 turns a failure into a success

- **GIVEN** a scripted natural 20, a modifier of +0, and DC 25
- **WHEN** the check is resolved
- **THEN** the total is 20, the numeric degree is failure, and the final degree is success

#### Scenario: Natural 20 turns a success into a critical success

- **GIVEN** a scripted natural 20, a modifier of +0, and DC 18
- **WHEN** the check is resolved
- **THEN** the total is 20 and the degree is critical success

#### Scenario: Natural 20 does not raise a critical success

- **GIVEN** a scripted natural 20, a modifier of +8, and DC 18
- **WHEN** the check is resolved
- **THEN** the total is 28 and the degree stays critical success

#### Scenario: Natural 20 turns a critical failure into a failure

- **GIVEN** a scripted natural 20, a modifier of +0, and DC 30
- **WHEN** the check is resolved
- **THEN** the total is 20, the numeric degree is critical failure, and the final degree is failure

#### Scenario: Natural 1 turns a success into a failure

- **GIVEN** a scripted natural 1, a modifier of +17, and DC 15
- **WHEN** the check is resolved
- **THEN** the total is 18, the numeric degree is success, and the final degree is failure

#### Scenario: Natural 1 turns a critical success into a success

- **GIVEN** a scripted natural 1, a modifier of +24, and DC 15
- **WHEN** the check is resolved
- **THEN** the total is 25 and the degree is success

#### Scenario: Natural 1 does not lower a critical failure

- **GIVEN** a scripted natural 1, a modifier of +0, and DC 15
- **WHEN** the check is resolved
- **THEN** the total is 1 and the degree stays critical failure

#### Scenario: A later upgrade sees the degree after the natural 20

- **GIVEN** a scripted natural 20, a modifier of +0, and DC 25, and a later effect that upgrades a success to a critical success
- **WHEN** the degrees are resolved
- **THEN** the natural 20 first changes failure to success, and the later effect then changes that success to a critical success

#### Scenario: A later upgrade does not see the degree from before the natural 1

- **GIVEN** a scripted natural 1, a modifier of +17, and DC 15, and a later effect that upgrades a success to a critical success
- **WHEN** the degrees are resolved
- **THEN** the natural 1 first changes success to failure, and the later effect leaves the degree a failure

### Requirement: A missing critical entry falls back

When an effect lists no critical success, a critical success SHALL be resolved as a success. When an effect lists no critical failure, a critical failure SHALL be resolved as a failure. An effect that lists the critical entry SHALL keep that degree.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2286 (Player Core pg. 401, Step 4: Degree of Success)

#### Scenario: No critical success entry

- **GIVEN** an effect that lists no critical success, a scripted natural 15, a modifier of +10, and DC 15
- **WHEN** the effect is resolved
- **THEN** the numeric degree is critical success and the effect resolves as a success

#### Scenario: No critical failure entry

- **GIVEN** an effect that lists no critical failure, a scripted natural 5, a modifier of +0, and DC 15
- **WHEN** the effect is resolved
- **THEN** the numeric degree is critical failure and the effect resolves as a failure

#### Scenario: Listed critical entries stay

- **GIVEN** an effect that lists both critical entries, a scripted natural 15 with modifier +10 against DC 15, and a scripted natural 5 with modifier +0 against DC 15
- **WHEN** both effects are resolved
- **THEN** the first stays a critical success and the second stays a critical failure

### Requirement: DC from a modifier

A DC taken from a modifier SHALL equal 10 plus that modifier.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2285 (Player Core pg. 401, Getting a DC from a Modifier)

#### Scenario: Positive modifier

- **GIVEN** a modifier of +7
- **WHEN** the DC is taken from that modifier
- **THEN** the DC is 17

#### Scenario: Zero modifier

- **GIVEN** a modifier of +0
- **WHEN** the DC is taken from that modifier
- **THEN** the DC is 10

#### Scenario: Negative modifier

- **GIVEN** a modifier of -3
- **WHEN** the DC is taken from that modifier
- **THEN** the DC is 7

#### Scenario: A check uses the derived DC

- **GIVEN** a Stealth modifier of +7, so the DC is 17, and a scripted natural 12 with a Perception modifier of +5
- **WHEN** the check is resolved against that DC
- **THEN** the total is 17 and the degree is success

### Requirement: Flat checks roll one bare d20

A flat check SHALL roll one d20 and SHALL NOT add a modifier. A DC of 1 or lower SHALL succeed without a roll. A DC of 21 or higher SHALL fail without a roll. An automatic result SHALL be an ordinary success or an ordinary failure. Several flat checks that would cause or prevent the same thing SHALL roll once, against the highest of those DCs. Flat checks for different things SHALL roll separately.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2278 (Player Core pg. 400, Flat Checks)

#### Scenario: Modifiers are ignored

- **GIVEN** a flat check at DC 11, a +9 modifier that a normal check would add, and a scripted natural 10
- **WHEN** the flat check is resolved
- **THEN** the total is 10 and the degree is failure

#### Scenario: DC 1 succeeds without rolling

- **GIVEN** a flat check at DC 1 and a scripted natural 4
- **WHEN** the flat check is resolved
- **THEN** the degree is success and the scripted 4 is still unused

#### Scenario: DC 0 succeeds without rolling

- **GIVEN** a flat check at DC 0 and a scripted natural 4
- **WHEN** the flat check is resolved
- **THEN** the degree is success and the scripted 4 is still unused

#### Scenario: DC 21 fails without rolling

- **GIVEN** a flat check at DC 21 and a scripted natural 20
- **WHEN** the flat check is resolved
- **THEN** the degree is failure and the scripted 20 is still unused

#### Scenario: DC 20 still rolls

- **GIVEN** a flat check at DC 20 and a scripted natural 20
- **WHEN** the flat check is resolved
- **THEN** the roll is consumed, the total is 20, and the degree is success

#### Scenario: The same thing rolls once at the highest DC

- **GIVEN** two flat checks for the same thing, at DC 5 and DC 15, and scripted naturals 10 and then 20
- **WHEN** they are resolved together
- **THEN** one roll of 10 is compared with DC 15, the degree is failure, and the scripted 20 is still unused

#### Scenario: Different things roll separately

- **GIVEN** a flat check at DC 5 for one thing and a flat check at DC 11 for another, with scripted naturals 10 and then 4
- **WHEN** each thing is resolved
- **THEN** the first total is 10 and succeeds, and the second total is 4 and fails

### Requirement: Multiple attack penalty on your turn

On the creature's own turn, the second attack-trait check SHALL take -5 and the third and later SHALL take -10. An agile current weapon SHALL make those penalties -4 and -8. That current weapon SHALL set the penalty. A reaction on another creature's turn SHALL take no penalty and SHALL NOT add to this creature's own-turn count. Every attack-trait check on the creature's own turn SHALL count.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2289 (Player Core pg. 402, Multiple Attack Penalty)

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

- **GIVEN** a reaction attack-trait check while it is another creature's turn, after the reactor already made two attack-trait checks on its own earlier turn, attack bonus +8, DC 18, and a scripted natural 10
- **WHEN** the reaction check is resolved, and the reactor's next own turn then makes its first attack-trait check with a scripted natural 11, attack bonus +8, and DC 19
- **THEN** the reaction penalty is 0 and its total is 18, and the first check of the next own turn also has penalty 0 and total 19

#### Scenario: An attack-trait check that is not a Strike still counts

- **GIVEN** an attack-trait check that is not a Strike as the first check on the creature's own turn, then a non-agile Strike with attack bonus +8, DC 19, and a scripted natural 15
- **WHEN** the Strike is resolved
- **THEN** the Strike's penalty is -5, the total is 18, and the degree is failure

### Requirement: Multiple attack penalty is untyped

The multiple attack penalty SHALL be an untyped penalty. It SHALL be added to every other untyped penalty on the same check. It SHALL also be added in full to a circumstance penalty on that check. It SHALL NOT be dropped by keeping only the worse of the multiple attack penalty and a circumstance penalty.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2282 (Player Core pg. 400, Penalties)

#### Scenario: The penalty stacks with a circumstance penalty

- **GIVEN** the creature's second attack-trait check on its own turn, a non-agile weapon, attack bonus +9, a circumstance penalty of -2, DC 16, and a scripted natural 12
- **WHEN** the check is resolved
- **THEN** the total is 14, because 12 + 9 - 5 - 2 are all included, and the degree is failure

#### Scenario: The penalty stacks with another untyped penalty

- **GIVEN** the creature's second attack-trait check on its own turn, a non-agile weapon, attack bonus +9, another untyped penalty of -2, no circumstance penalty, DC 16, and a scripted natural 12
- **WHEN** the check is resolved
- **THEN** the total is 14, because 12 + 9 - 5 - 2 are all included, and the degree is failure
