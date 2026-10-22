# creature-template

## MODIFIED Requirements

### Requirement: Aid modifier against DC 15

When an Aid modifier is on the build, derive SHALL copy that modifier and an AidRank onto the Creature. Omitting the rank SHALL store AidRank::Trained. An Aid check SHALL add that modifier to one d20 against DC 15 and SHALL NOT change the Creature. The DC SHALL stay 15. Success SHALL grant AidCircumstance::Plus1. Failure SHALL grant AidCircumstance::Zero. Critical failure SHALL grant AidCircumstance::Minus1. Those three results SHALL NOT read the rank. A critical success SHALL grant Plus1 at Untrained, Plus2 at Trained, Plus3 at Expert, and Plus4 at Master. A modifier that cannot be added to every face in 0..=255 SHALL return no roll and SHALL draw no die. The build SHALL NOT store a prepared-to-help flag.

**Rule**: Player Core Aid grants +2 circumstance on a critical success, +3 if you are a master with the check you attempted, and +4 if you are legendary. https://2e.aonprd.com/Actions.aspx?ID=2292 (Player Core pg. 416)
**Why not**: Phase 1 has no skill to be master or legendary in. The critical-success bonus is selected from four typed ranks.
**Substitute**: AidRank Untrained, Trained, Expert, and Master grant +1, +2, +3, and +4 on a critical success. There is no separate legendary rank. Success stays +1, failure stays 0, and a critical failure stays -1. (DEV-50, proposed)
**Intent**: A higher rank still grants a higher critical-success bonus, and that bonus is data on the creature.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2292 (Player Core pg. 416, Aid)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2286 (Player Core pg. 401, Step 4: Degree of Success)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2287 (Player Core pg. 401, natural 20 and natural 1)
**Source**: DEV-04. The typical DC is 15, and this check does not adjust it. https://2e.aonprd.com/Actions.aspx?ID=2292 (Player Core pg. 416)
**Source**: DEV-02. The roll uses the baked aid modifier. https://2e.aonprd.com/Actions.aspx?ID=2292 (Player Core pg. 416)

#### Scenario: Success grants +1

- **GIVEN** an aid modifier of +7, AidRank::Master, and a scripted natural 10
- **WHEN** the Aid check is resolved against DC 15
- **THEN** the total is 17, the degree is success, the circumstance is Plus1, and the Creature is unchanged

#### Scenario: Critical success grants +2

- **GIVEN** an aid modifier of +7, AidRank::Trained, and a scripted natural 18
- **WHEN** the Aid check is resolved against DC 15
- **THEN** the total is 25, the degree is critical success, and the circumstance is Plus2

#### Scenario: Critical success at untrained grants plus 1

- **GIVEN** an aid modifier of +7, AidRank::Untrained, and a scripted natural 18
- **WHEN** the Aid check is resolved against DC 15
- **THEN** the total is 25, the degree is critical success, and the circumstance is Plus1

#### Scenario: Critical success at expert grants plus 3

- **GIVEN** an aid modifier of +7, AidRank::Expert, and a scripted natural 18
- **WHEN** the Aid check is resolved against DC 15
- **THEN** the total is 25, the degree is critical success, and the circumstance is Plus3

#### Scenario: Critical success at master grants plus 4

- **GIVEN** an aid modifier of +7, AidRank::Master, and a scripted natural 18
- **WHEN** the Aid check is resolved against DC 15
- **THEN** the total is 25, the degree is critical success, and the circumstance is Plus4

#### Scenario: Failure grants 0

- **GIVEN** an aid modifier of +7, AidRank::Untrained, and a scripted natural 4
- **WHEN** the Aid check is resolved against DC 15
- **THEN** the total is 11, the degree is failure, and the circumstance is Zero

#### Scenario: Critical failure grants -1

- **GIVEN** an aid modifier of +4, AidRank::Master, and a scripted natural 1
- **WHEN** the Aid check is resolved against DC 15
- **THEN** the total is 5, the degree is critical failure, and the circumstance is Minus1

#### Scenario: A natural 20 uses the fixed critical bonus

- **GIVEN** an aid modifier of +0, AidRank::Trained, and a scripted natural 20
- **WHEN** the Aid check is resolved against DC 15
- **THEN** the numeric total 20 is a success, the natural 20 makes it a critical success, and the circumstance is Plus2

#### Scenario: A natural 20 at expert grants plus 3

- **GIVEN** an aid modifier of +0, AidRank::Expert, and a scripted natural 20
- **WHEN** the Aid check is resolved against DC 15
- **THEN** the total is 20, that total is a success before the natural 20, the degree is critical success, and the circumstance is Plus3

#### Scenario: No prepared flag is required

- **GIVEN** a build with aid modifier +6, no rank, and no prepared-to-help value
- **WHEN** derive is called
- **THEN** the Creature has aid modifier +6, AidRank::Trained, and does not record a prepared-to-help state

#### Scenario: An aid modifier past i32 is not rolled

- **GIVEN** an aid modifier of 2147483647, AidRank::Trained, and a scripted natural 10
- **WHEN** the Aid check is resolved against DC 15
- **THEN** no roll is returned and the scripted 10 is still unused
