# creature-template

## MODIFIED Requirements

### Requirement: Aid modifier against DC 15

When an Aid modifier is on the build, derive SHALL copy that modifier and an AidRank onto the Creature. Omitting the rank SHALL store AidRank::BelowMaster. An Aid check SHALL add that modifier to one d20 against DC 15 and SHALL NOT change the Creature. The DC SHALL stay 15. Success SHALL grant AidCircumstance::Plus1. Failure SHALL grant AidCircumstance::Zero. Critical failure SHALL grant AidCircumstance::Minus1. Those three results SHALL NOT read the rank. A critical success SHALL grant Plus2 at BelowMaster, Plus3 at Master, and Plus4 at Legendary. A natural 1 SHALL lower the degree one step before the circumstance is chosen. The roll SHALL NOT use CheckModifier. A modifier for which modifier plus 20 does not fit in i32 SHALL return no roll and SHALL draw no die. The build SHALL NOT store a prepared-to-help flag.

**Rule**: Player Core Aid grants +2 circumstance on a critical success, +3 if you are a master in the check, and +4 if you are legendary. https://2e.aonprd.com/Actions.aspx?ID=2292 (Player Core pg. 416)
**Why not**: DEV-03 substituted a fixed +2 because the baked modifier had no rank. This change stores the rank, so that substitute does not apply.
**Substitute**: This change supersedes DEV-03. BelowMaster grants Plus2, Master grants Plus3, and Legendary grants Plus4 on a critical success. Success stays Plus1, failure stays Zero, and a critical failure stays Minus1. The ledger row is updated in a later docs pull request. This change does not edit docs/designer.
**Intent**: The critical-success bonus matches the book.

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

- **GIVEN** an aid modifier of +7, AidRank::BelowMaster, and a scripted natural 18
- **WHEN** the Aid check is resolved against DC 15
- **THEN** the total is 25, the degree is critical success, and the circumstance is Plus2

#### Scenario: Critical success at master grants plus 3

- **GIVEN** an aid modifier of +7, AidRank::Master, and a scripted natural 18
- **WHEN** the Aid check is resolved against DC 15
- **THEN** the total is 25, the degree is critical success, and the circumstance is Plus3

#### Scenario: Critical success at legendary grants plus 4

- **GIVEN** an aid modifier of +7, AidRank::Legendary, and a scripted natural 18
- **WHEN** the Aid check is resolved against DC 15
- **THEN** the total is 25, the degree is critical success, and the circumstance is Plus4

#### Scenario: Failure grants 0

- **GIVEN** an aid modifier of +7, AidRank::Legendary, and a scripted natural 4
- **WHEN** the Aid check is resolved against DC 15
- **THEN** the total is 11, the degree is failure, and the circumstance is Zero

#### Scenario: Critical failure grants -1

- **GIVEN** an aid modifier of +4, AidRank::Master, and a scripted natural 1
- **WHEN** the Aid check is resolved against DC 15
- **THEN** the total is 5, the degree is critical failure, and the circumstance is Minus1

#### Scenario: A natural 20 uses the fixed critical bonus

- **GIVEN** an aid modifier of +0, AidRank::BelowMaster, and a scripted natural 20
- **WHEN** the Aid check is resolved against DC 15
- **THEN** the numeric total 20 is a success, the natural 20 makes it a critical success, and the circumstance is Plus2

#### Scenario: A total of exactly 15 succeeds

- **GIVEN** an aid modifier of +5, AidRank::BelowMaster, and a scripted natural 10
- **WHEN** the Aid check is resolved against DC 15
- **THEN** the total is 15, the degree is success, and the circumstance is Plus1

#### Scenario: A natural 1 at plus 14 drops to failure

- **GIVEN** an aid modifier of +14, AidRank::Master, and a scripted natural 1
- **WHEN** the Aid check is resolved against DC 15
- **THEN** the numeric total 15 is a success before the natural 1, the natural 1 makes it a failure, and the circumstance is Zero

#### Scenario: The Aid check at modifier 2147483628 rolls nothing

- **GIVEN** an aid modifier of 2147483628, AidRank::BelowMaster, and a scripted natural 20
- **WHEN** the Aid check is resolved against DC 15
- **THEN** no roll is returned and the scripted 20 is still unused

#### Scenario: No prepared flag is required

- **GIVEN** a build with aid modifier +6, no rank, and no prepared-to-help value
- **WHEN** derive is called
- **THEN** the Creature has aid modifier +6, AidRank::BelowMaster, and does not record a prepared-to-help state
