# seek-and-hidden Specification

## Purpose
Records whether one creature is observed, hidden, or undetected by another, records concealment beside that detection, and resolves Seek, Hide, and the targeting flat checks.

## Requirements

### Requirement: The same detection twice matches

The same inputs SHALL return the same result. A call SHALL NOT read a file, the network, a clock, or a global generator. A fallible call SHALL return a typed error and SHALL NOT panic. The rules crate SHALL gain no dependency. A call SHALL NOT write Hit Points, actions, or the Creature.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2414 (Player Core pg. 434, Detecting Creatures)

#### Scenario: The same relation twice matches

- **GIVEN** CombatantId(2) hidden from CombatantId(1), not concealed, and a scripted natural 8
- **WHEN** the relation is read twice
- **THEN** both results are hidden and not concealed
- **AND** the scripted 8 is still unused

#### Scenario: The same Seek twice matches

- **GIVEN** two scripts of natural 12, Perception modifier +5, an area of 30 by 10 by 10 feet, and CombatantId(2) undetected by CombatantId(1) with Stealth DC 17
- **WHEN** Seek is resolved on each script
- **THEN** both totals are 17, both degrees are success, and both relations are hidden
- **AND** each script is fully consumed

#### Scenario: The same Hide twice matches

- **GIVEN** two scripts of natural 6, Stealth DC 17, standard cover, Perception modifier +5, and CombatantId(2) observed by CombatantId(1) and not concealed
- **WHEN** Hide is resolved on each script
- **THEN** both totals are 15, both degrees are success, and both relations are hidden
- **AND** each script is fully consumed

#### Scenario: The same flat check twice matches

- **GIVEN** two scripts of natural 5, and CombatantId(2) observed and concealed from CombatantId(1)
- **WHEN** a non-area targeting check is resolved on each script
- **THEN** both degrees are success, both DCs are 5, and both targets are affected
- **AND** each script is fully consumed

### Requirement: Detection is one relation per observer

Detections SHALL store one Relation per observer CombatantId and subject CombatantId. A missing pair SHALL be observed and Concealment::NotConcealed. store SHALL reject SameCreature when the ids match and SHALL write nothing. store SHALL NOT read Vital. A Dead observer or a Dead subject SHALL be stored.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2414 (Player Core pg. 434, Detecting Creatures)

#### Scenario: One subject is hidden from one observer and observed by another

- **GIVEN** CombatantId(2) is hidden from CombatantId(1) and no relation is stored for CombatantId(3)
- **WHEN** each observer reads CombatantId(2)
- **THEN** CombatantId(1) reads hidden
- **AND** CombatantId(3) reads observed and not concealed

#### Scenario: A missing relation is observed

- **GIVEN** no stored relation from CombatantId(1) to CombatantId(2)
- **WHEN** the relation is read
- **THEN** the detection is observed and the concealment is NotConcealed

#### Scenario: The observer and the subject cannot be the same

- **GIVEN** CombatantId(1) as both observer and subject, and a scripted natural 8
- **WHEN** a relation is stored
- **THEN** the error is SameCreature
- **AND** no relation is stored and the scripted 8 is still unused

#### Scenario: A dead subject or observer is stored

- **GIVEN** a Dead observer and a Dead subject, Stealth DC 17, and a scripted natural 8
- **WHEN** hidden and NotConcealed is stored
- **THEN** the store succeeds and the detection is hidden
- **AND** the scripted 8 is still unused

### Requirement: Side does not choose observers

Seek, Hide, store, and get SHALL NOT take Side and SHALL NOT reject a pair because of it. A Pc SHALL be able to observe an Enemy, and an Enemy SHALL be able to observe a Pc.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2414 (Player Core pg. 434, Detecting Creatures)

#### Scenario: A Pc and an Enemy observe each other

- **GIVEN** CombatantId(1) is Side::Pc and CombatantId(2) is Side::Enemy, and each is hidden from the other with Stealth DC 17
- **WHEN** both relations are read
- **THEN** each detection is hidden

### Requirement: Concealed does not change the detection category

Concealment SHALL be Concealed or NotConcealed on the relation. It SHALL NOT replace observed, hidden, or undetected. Observed SHALL stay observed while concealment is Concealed.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2414 (Player Core pg. 434, Detecting Creatures)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=62 (Player Core pg. 442, Concealed)

#### Scenario: An observed creature can be concealed

- **GIVEN** CombatantId(2) observed and concealed from CombatantId(1)
- **WHEN** the relation is read
- **THEN** the detection is observed
- **AND** the concealment is Concealed

#### Scenario: A hidden creature can be concealed

- **GIVEN** CombatantId(2) hidden and concealed from CombatantId(1), with Stealth DC 17
- **WHEN** the relation is read
- **THEN** the detection is hidden
- **AND** the concealment is Concealed

### Requirement: Hidden and undetected need a Stealth DC

Hidden or undetected SHALL be rejected when the subject has no Stealth DC, and the relation SHALL stay unchanged. Observed, with either concealment, SHALL be stored with no Stealth DC.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2285 (Player Core pg. 401, Getting a DC from a Modifier)

#### Scenario: Hidden without a Stealth DC is rejected

- **GIVEN** CombatantId(2) has no Stealth DC and is observed by CombatantId(1)
- **WHEN** hidden is stored
- **THEN** the error is NoStealthDc
- **AND** the relation is still observed

#### Scenario: Undetected without a Stealth DC is rejected

- **GIVEN** CombatantId(2) has no Stealth DC and is observed by CombatantId(1)
- **WHEN** undetected is stored
- **THEN** the error is NoStealthDc
- **AND** the relation is still observed

#### Scenario: Concealed without a Stealth DC is stored

- **GIVEN** CombatantId(2) has no Stealth DC
- **WHEN** observed and concealed is stored for CombatantId(1)
- **THEN** the detection is observed and the concealment is Concealed

### Requirement: A concealed target takes a DC 5 flat check

An observed and concealed creature SHALL take one flat check at DC 5 before the effect. A failure or critical failure SHALL report the action spent and the attack counting for MAP. A success or critical success SHALL affect the target and report neither. counts_for_map describes the attack case; the caller SHALL apply it to the multiple attack penalty only when the action has the attack trait (Rules 2289). resolve_targeting SHALL NOT call spend_actions or strike. No modifier SHALL be added.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=62 (Player Core pg. 442, Concealed)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2416 (Player Core pg. 434, Hidden)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2289 (Player Core pg. 402, Multiple Attack Penalty)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2278 (Player Core pg. 400, Checks (Flat Checks))
**Source**: Ruling R-12 in docs/designer/rules-rulings.md

#### Scenario: A natural 5 at DC 5 affects the target

- **GIVEN** CombatantId(2) observed and concealed from CombatantId(1), and a scripted natural 5
- **WHEN** a non-area targeting check is resolved
- **THEN** the roll is consumed, the DC is 5, the degree is success, and the target is affected
- **AND** the result reports neither the action spent nor the attack counting for MAP

#### Scenario: A natural 4 at DC 5 does not affect the target

- **GIVEN** CombatantId(2) observed and concealed from CombatantId(1), and a scripted natural 4
- **WHEN** a non-area targeting check is resolved
- **THEN** the roll is consumed, the DC is 5, the degree is failure, and the target is not affected
- **AND** the result reports the action spent and the attack counting for MAP

#### Scenario: A natural 1 at DC 5 does not affect the target

- **GIVEN** CombatantId(2) observed and concealed from CombatantId(1), and a scripted natural 1
- **WHEN** a non-area targeting check is resolved
- **THEN** the numeric degree is critical failure
- **AND** the effect resolves as a failure and the target is not affected
- **AND** the result reports the action spent and the attack counting for MAP

#### Scenario: A natural 20 at DC 5 affects the target

- **GIVEN** CombatantId(2) observed and concealed from CombatantId(1), and a scripted natural 20
- **WHEN** a non-area targeting check is resolved
- **THEN** the numeric degree is critical success
- **AND** the effect resolves as a success and the target is affected
- **AND** the result reports neither the action spent nor the attack counting for MAP

### Requirement: A hidden target takes a DC 11 flat check

A creature that is hidden and not concealed SHALL take one flat check at DC 11. A critical success SHALL affect the target. A failure SHALL report the action spent and the attack counting for MAP, and SHALL NOT call spend_actions or strike.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=79 (Player Core pg. 444, Hidden)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2416 (Player Core pg. 434, Hidden)

#### Scenario: A natural 11 at DC 11 affects the target

- **GIVEN** CombatantId(2) hidden and not concealed from CombatantId(1), with Stealth DC 17, and a scripted natural 11
- **WHEN** a non-area targeting check is resolved
- **THEN** the roll is consumed, the DC is 11, the degree is success, and the target is affected

#### Scenario: A natural 10 at DC 11 does not affect the target

- **GIVEN** CombatantId(2) hidden and not concealed from CombatantId(1), with Stealth DC 17, and a scripted natural 10
- **WHEN** a non-area targeting check is resolved
- **THEN** the roll is consumed, the DC is 11, the degree is failure, and the target is not affected
- **AND** the result reports the action spent and the attack counting for MAP
- **AND** the relation is still hidden and strike is not called

#### Scenario: A natural 20 at DC 11 affects the target

- **GIVEN** CombatantId(2) hidden and not concealed from CombatantId(1), with Stealth DC 17, and a scripted natural 20
- **WHEN** a non-area targeting check is resolved
- **THEN** the numeric degree is critical success
- **AND** the effect resolves as a success and the target is affected

### Requirement: Hidden and concealed roll once at DC 11

When the subject is hidden and concealed, the targeting flat checks SHALL roll once at DC 11, the higher DC. A second die SHALL NOT be drawn. The reported DC SHALL be 11.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2278 (Player Core pg. 400, Checks (Flat Checks))

#### Scenario: Hidden and concealed fail one DC 11 roll

- **GIVEN** CombatantId(2) hidden and concealed from CombatantId(1), with Stealth DC 17, and scripted naturals 10 and then 20
- **WHEN** a non-area targeting check is resolved
- **THEN** one roll of 10 is compared with DC 11, the degree is failure, and the target is not affected
- **AND** the scripted 20 is still unused

#### Scenario: Hidden and concealed succeed at the reported DC 11

- **GIVEN** CombatantId(2) hidden and concealed from CombatantId(1), with Stealth DC 17, and a scripted natural 11
- **WHEN** a non-area targeting check is resolved
- **THEN** one roll of 11 is compared with DC 11, the degree is success, and the target is affected
- **AND** the reported DC is 11

### Requirement: An observed target or an area effect rolls nothing

An observed creature that is not concealed SHALL be affected and SHALL draw no die. An area effect SHALL be affected and SHALL draw no die, including when the subject is hidden, concealed, or undetected.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2415 (Player Core pg. 434, Observed)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=79 (Player Core pg. 444, Hidden)

#### Scenario: An observed target rolls nothing

- **GIVEN** CombatantId(2) observed and not concealed from CombatantId(1), and a scripted natural 4
- **WHEN** a non-area targeting check is resolved
- **THEN** the target is affected
- **AND** the scripted 4 is still unused

#### Scenario: An area effect rolls nothing

- **GIVEN** CombatantId(2) hidden and concealed from CombatantId(1), with Stealth DC 17, and a scripted natural 4
- **WHEN** an area effect is resolved
- **THEN** the target is affected
- **AND** the scripted 4 is still unused

### Requirement: An undetected target cannot be chosen

A non-area effect against an undetected subject SHALL return CannotTarget, draw no die, and leave the relation unchanged.

**Rule**: Guess a square, then a secret DC 11 (Player Core pg. 447).
**Why not**: No hidden-information UI.
**Substitute**: The target cannot be chosen. No square and no flat check.
**Intent**: Seek learns the space. Proposed DEV-41.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=96 (Player Core pg. 447, Undetected)

#### Scenario: An undetected target cannot be chosen

- **GIVEN** CombatantId(2) undetected and not concealed from CombatantId(1), with Stealth DC 17, and a scripted natural 11
- **WHEN** a non-area targeting check is resolved
- **THEN** the error is CannotTarget
- **AND** the relation is still undetected and the scripted 11 is still unused

### Requirement: Hidden and undetected reuse OffGuard

Hidden or undetected SHALL make the observer off-guard to the subject, as OffGuard::Yes from positioning. This change SHALL NOT declare OffGuard and SHALL NOT add a second −2. Concealed on an observed creature SHALL be OffGuard::No. Armor Class with that one Yes, no cover, and no unconscious penalty SHALL be −2.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=79 (Player Core pg. 444, Hidden)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=58 (Player Core pg. 445, Off-Guard)

#### Scenario: Hidden is off-guard at minus 2

- **GIVEN** CombatantId(2) hidden and not concealed from CombatantId(1), with Stealth DC 17, and no other off-guard
- **WHEN** off-guard is resolved and Armor Class is totaled once
- **THEN** the result is OffGuard::Yes
- **AND** the Armor Class total is −2

#### Scenario: Undetected is off-guard at minus 2

- **GIVEN** CombatantId(2) undetected from CombatantId(1), with Stealth DC 17, and no other off-guard
- **WHEN** off-guard is resolved and Armor Class is totaled once
- **THEN** the result is OffGuard::Yes
- **AND** the Armor Class total is −2

#### Scenario: Concealed is not off-guard

- **GIVEN** CombatantId(2) observed and concealed from CombatantId(1), and no other off-guard
- **WHEN** off-guard is resolved and Armor Class is totaled once
- **THEN** the result is OffGuard::No
- **AND** the Armor Class total is 0

#### Scenario: Hidden and concealed is still off-guard

- **GIVEN** CombatantId(2) hidden and concealed from CombatantId(1), with Stealth DC 17, and no other off-guard
- **WHEN** off-guard is resolved and Armor Class is totaled once
- **THEN** the result is OffGuard::Yes
- **AND** the Armor Class total is −2

#### Scenario: Hidden plus another off-guard is still minus 2

- **GIVEN** CombatantId(2) hidden from CombatantId(1), with Stealth DC 17, and OffGuard::Yes from another cause
- **WHEN** off-guard is resolved and Armor Class is totaled once
- **THEN** the result is OffGuard::Yes
- **AND** the Armor Class total is −2
- **AND** the total is not −4

### Requirement: A seeker or hider who is not Alive rolls nothing

A seeker or hider whose vital is not Alive SHALL return NotConscious, draw no die, and leave every relation unchanged. NotConscious SHALL be checked first. Scenarios that do not name a vital use Alive.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2429 (Player Core pg. 436, Step 2: Act)

#### Scenario: A dying seeker rolls nothing

- **GIVEN** the seeker is Vital::Dying(1), an area of 35 by 5 by 5 feet, Perception modifier +5, a scripted natural 12, and CombatantId(2) undetected with Stealth DC 17
- **WHEN** Seek is resolved
- **THEN** the error is NotConscious
- **AND** the error is not AreaTooLarge
- **AND** the relation is still undetected and the scripted 12 is still unused

#### Scenario: An unconscious hider rolls nothing

- **GIVEN** the hider is Vital::Unconscious, no Stealth DC, lesser cover, Perception modifier +5, a scripted natural 15, and CombatantId(2) observed by CombatantId(1)
- **WHEN** Hide is resolved
- **THEN** the error is NotConscious
- **AND** the error is not NoStealthDc
- **AND** the relation is still observed and the scripted 15 is still unused

### Requirement: Seek is one action with concentrate

Seek SHALL cost DetectionCost::Seek, which is 1 action with the concentrate trait. The caller SHALL spend that 1 action through encounter-turns spend_actions. This module SHALL NOT call spend_actions. The result SHALL expose the natural roll, the total, and the degree.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2301 (Player Core pg. 417, Seek)

#### Scenario: Seek reports one concentrate action

- **GIVEN** Perception modifier +5, a scripted natural 12, an area of 30 by 10 by 10 feet, and CombatantId(2) undetected by CombatantId(1) with Stealth DC 17
- **WHEN** Seek is resolved
- **THEN** the cost is DetectionCost::Seek, 1 action with the concentrate trait
- **AND** the natural is 12, the total is 17, and the degree is success

### Requirement: Seek degrees move undetected and hidden

A critical success SHALL move hidden or undetected to observed. A success SHALL move undetected to hidden and hidden to observed. Failure SHALL leave the relation unchanged. A critical failure has no entry and SHALL leave it unchanged. Invisibility or an imprecise sense SHALL NOT keep the creature hidden (proposed DEV-42).

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2301 (Player Core pg. 417, Seek)

#### Scenario: A critical success moves hidden to observed

- **GIVEN** Perception modifier +5, a scripted natural 20, an area of 30 by 10 by 10 feet, and CombatantId(2) hidden from CombatantId(1) with Stealth DC 17 and not concealed
- **WHEN** Seek is resolved
- **THEN** the total is 25 and the numeric degree is critical success
- **AND** the detection is observed

#### Scenario: A critical success moves undetected to observed

- **GIVEN** Perception modifier +5, a scripted natural 20, an area of 30 by 10 by 10 feet, and CombatantId(2) undetected by CombatantId(1) with Stealth DC 17 and not concealed
- **WHEN** Seek is resolved
- **THEN** the total is 25 and the numeric degree is critical success
- **AND** the detection is observed

#### Scenario: A success moves hidden to observed

- **GIVEN** Perception modifier +5, a scripted natural 12, an area of 30 by 10 by 10 feet, and CombatantId(2) hidden from CombatantId(1) with Stealth DC 17 and not concealed
- **WHEN** Seek is resolved
- **THEN** the total is 17 and the degree is success
- **AND** the detection is observed

#### Scenario: A success moves undetected to hidden

- **GIVEN** Perception modifier +5, a scripted natural 12, an area of 30 by 10 by 10 feet, and CombatantId(2) undetected by CombatantId(1) with Stealth DC 17 and not concealed
- **WHEN** Seek is resolved
- **THEN** the natural is 12, the total is 17, and the degree is success
- **AND** the detection is hidden

#### Scenario: A failure leaves undetected

- **GIVEN** Perception modifier +5, a scripted natural 11, an area of 30 by 10 by 10 feet, and CombatantId(2) undetected by CombatantId(1) with Stealth DC 17
- **WHEN** Seek is resolved
- **THEN** the total is 16 and the degree is failure
- **AND** the detection is still undetected

#### Scenario: A critical failure leaves hidden

- **GIVEN** Perception modifier +0, a scripted natural 1, an area of 30 by 10 by 10 feet, and CombatantId(2) hidden from CombatantId(1) with Stealth DC 17 and not concealed
- **WHEN** Seek is resolved
- **THEN** the total is 1 and the numeric degree is critical failure
- **AND** the effect resolves as a failure and the detection is still hidden

#### Scenario: A critical failure leaves undetected

- **GIVEN** Perception modifier +0, a scripted natural 1, an area of 30 by 10 by 10 feet, and CombatantId(2) undetected by CombatantId(1) with Stealth DC 17
- **WHEN** Seek is resolved
- **THEN** the total is 1 and the numeric degree is critical failure
- **AND** the effect resolves as a failure and the detection is still undetected

#### Scenario: A natural 20 moves hidden to observed

- **GIVEN** Perception modifier +0, a scripted natural 20, an area of 30 by 10 by 10 feet, and CombatantId(2) hidden from CombatantId(1) with Stealth DC 25 and not concealed
- **WHEN** Seek is resolved
- **THEN** the total is 20 and the degree is success
- **AND** the detection is observed

#### Scenario: A success leaves concealment set

- **GIVEN** Perception modifier +5, a scripted natural 12, an area of 30 by 10 by 10 feet, and CombatantId(2) hidden and concealed from CombatantId(1) with Stealth DC 17
- **WHEN** Seek is resolved
- **THEN** the total is 17 and the degree is success
- **AND** the detection is observed and the concealment is Concealed

### Requirement: Seek rolls once and skips everyone else

One Perception check SHALL be compared with each hidden or undetected subject that has a Stealth DC. Observed subjects and subjects with no Stealth DC SHALL NOT be compared. When none qualify, Seek SHALL succeed and SHALL draw no die.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2301 (Player Core pg. 417, Seek)

#### Scenario: One Perception roll meets one Stealth DC and misses another

- **GIVEN** Perception modifier +5, a scripted natural 13, an area of 30 by 10 by 10 feet, CombatantId(2) undetected with Stealth DC 17, and CombatantId(4) undetected with Stealth DC 27
- **WHEN** Seek is resolved
- **THEN** one total of 18 is a success against DC 17 and a failure against DC 27
- **AND** CombatantId(2) is hidden and CombatantId(4) is still undetected

#### Scenario: Observed and a missing Stealth DC are not compared

- **GIVEN** Perception modifier +5, a scripted natural 12, an area of 30 by 10 by 10 feet, CombatantId(2) observed with Stealth DC 17, CombatantId(3) observed with no Stealth DC, and CombatantId(4) hidden with Stealth DC 17
- **WHEN** Seek is resolved
- **THEN** one total of 17 moves CombatantId(4) to observed
- **AND** CombatantId(2) and CombatantId(3) stay observed

#### Scenario: Only an observed subject rolls nothing

- **GIVEN** Perception modifier +5, a scripted natural 12, an area of 30 by 10 by 10 feet, and CombatantId(2) observed by CombatantId(1) with Stealth DC 17
- **WHEN** Seek is resolved
- **THEN** the call succeeds and the relation stays observed
- **AND** the scripted 12 is still unused

### Requirement: Seek rejects before it rolls

Before any die, Seek SHALL reject in this order: NotConscious, DuplicateId, SameCreature, EmptyArea, AreaTooLarge, then PerceptionOutOfRange. PerceptionOutOfRange SHALL be a failed checked_add(20). Any dimension of 0 SHALL be EmptyArea. A modifier that passes the gate SHALL be rolled.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2301 (Player Core pg. 417, Seek)

#### Scenario: An area beyond 30 feet rolls nothing

- **GIVEN** an area of 35 by 5 by 5 feet, Perception modifier +5, a scripted natural 12, and CombatantId(2) undetected by CombatantId(1) with Stealth DC 17
- **WHEN** Seek is resolved
- **THEN** the error is AreaTooLarge
- **AND** the relation is still undetected and the scripted 12 is still unused

#### Scenario: A zero foot area rolls nothing

- **GIVEN** an area of 0 by 0 by 0 feet, Perception modifier +5, a scripted natural 12, and CombatantId(2) undetected by CombatantId(1) with Stealth DC 17
- **WHEN** Seek is resolved
- **THEN** the error is EmptyArea
- **AND** the relation is still undetected and the scripted 12 is still unused

#### Scenario: A duplicate subject rolls nothing

- **GIVEN** Perception modifier +5, a scripted natural 12, an area of 35 by 5 by 5 feet, and CombatantId(2) listed twice, undetected, with Stealth DC 17
- **WHEN** Seek is resolved
- **THEN** the error is DuplicateId
- **AND** the relation is still undetected and the scripted 12 is still unused
- **AND** the error is not AreaTooLarge

#### Scenario: The seeker as a subject rolls nothing

- **GIVEN** the seeker is CombatantId(1), the only subject is CombatantId(1), Perception modifier +5, a scripted natural 12, an area of 35 by 5 by 5 feet, and Stealth DC 17
- **WHEN** Seek is resolved
- **THEN** the error is SameCreature
- **AND** the scripted 12 is still unused
- **AND** the error is not AreaTooLarge

#### Scenario: The seeker listed twice rolls nothing

- **GIVEN** the seeker is CombatantId(1), the subject list is CombatantId(1) twice, Perception modifier +5, a scripted natural 12, an area of 30 by 10 by 10 feet, and Stealth DC 17
- **WHEN** Seek is resolved
- **THEN** the error is DuplicateId
- **AND** the scripted 12 is still unused
- **AND** the error is not SameCreature

#### Scenario: A Seek area with a zero dimension rolls nothing

- **GIVEN** an area of 30 by 0 by 10 feet, Perception modifier +5, a scripted natural 12, and CombatantId(2) undetected by CombatantId(1) with Stealth DC 17
- **WHEN** Seek is resolved
- **THEN** the error is EmptyArea
- **AND** the relation is still undetected and the scripted 12 is still unused

#### Scenario: A Seek at Perception 2147483628 rolls nothing

- **GIVEN** Perception modifier 2147483628, a scripted natural 12, an area of 30 by 10 by 10 feet, and CombatantId(2) hidden from CombatantId(1) with Stealth DC 17
- **WHEN** Seek is resolved
- **THEN** the error is PerceptionOutOfRange
- **AND** the relation is still hidden and the scripted 12 is still unused

#### Scenario: A Seek at Perception 2147483627 is a critical success

- **GIVEN** Perception modifier 2147483627, a scripted natural 20, an area of 30 by 10 by 10 feet, and CombatantId(2) hidden from CombatantId(1) with Stealth DC 17 and not concealed
- **WHEN** Seek is resolved
- **THEN** the natural is 20, the total is 2147483647, and the numeric degree is critical success
- **AND** the detection is observed

#### Scenario: A Seek at Perception i32 minimum stays hidden

- **GIVEN** Perception modifier i32::MIN, a scripted natural 20, an area of 30 by 10 by 10 feet, and CombatantId(2) hidden from CombatantId(1) with Stealth DC 17 and not concealed
- **WHEN** Seek is resolved
- **THEN** the natural is 20, the total is -2147483628, and the degree is failure
- **AND** the detection is still hidden

#### Scenario: Perception 2147483500 is a critical success

- **GIVEN** Perception modifier 2147483500, a scripted natural 12, an area of 30 by 10 by 10 feet, and CombatantId(2) hidden from CombatantId(1) with Stealth DC 17 and not concealed
- **WHEN** Seek is resolved
- **THEN** the natural is 12, the total is 2147483512, and the numeric degree is critical success
- **AND** the detection is observed

### Requirement: Hide needs cover or concealment

Hide SHALL compare an observer who has standard cover, greater cover, or Concealment::Concealed. Lesser cover SHALL NOT qualify. An observer who does not qualify SHALL stay unchanged when another does, and that call SHALL NOT be RequirementNotMet.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2404 (Player Core pg. 244, Hide)

#### Scenario: One observer without cover is left observed

- **GIVEN** Stealth DC 17, a scripted natural 6, CombatantId(2) observed and not concealed by CombatantId(1) with standard cover and Perception +5, and by CombatantId(5) with no cover and Perception +5
- **WHEN** Hide is resolved
- **THEN** one total of 15 hides CombatantId(2) from CombatantId(1)
- **AND** CombatantId(5) still observes CombatantId(2)

#### Scenario: Concealment from one observer hides from that observer

- **GIVEN** Stealth DC 17, a scripted natural 8, no cover, Perception modifier +5, CombatantId(2) observed and Concealed from CombatantId(1), and observed and NotConcealed from CombatantId(5)
- **WHEN** Hide is resolved
- **THEN** the error is not RequirementNotMet
- **AND** the modifier is +7, the DC is 15, the total is 15, and CombatantId(2) is hidden and Concealed from CombatantId(1)
- **AND** CombatantId(5) still observes CombatantId(2) and that concealment is NotConcealed

### Requirement: Hide rejects before it rolls

After NotConscious, Hide SHALL reject in this order: DuplicateId, SameCreature, NoStealthDc, RequirementNotMet, then Overflow. No Stealth DC SHALL beat a cover failure. A Stealth DC of i32::MIN with standard cover, and a Stealth DC of i32::MAX with greater cover, SHALL be Overflow.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2404 (Player Core pg. 244, Hide)

#### Scenario: Lesser cover is not enough

- **GIVEN** lesser cover, not concealed, Stealth DC 17, Perception modifier +5, CombatantId(2) observed by CombatantId(1), and a scripted natural 15
- **WHEN** Hide is resolved
- **THEN** the error is RequirementNotMet
- **AND** the relation is still observed and the scripted 15 is still unused

#### Scenario: No cover and not concealed is not enough

- **GIVEN** no cover, not concealed, Stealth DC 17, Perception modifier +5, CombatantId(2) observed by CombatantId(1), and a scripted natural 15
- **WHEN** Hide is resolved
- **THEN** the error is RequirementNotMet
- **AND** the relation is still observed and the scripted 15 is still unused

#### Scenario: Hide without a Stealth DC rolls nothing

- **GIVEN** no Stealth DC, lesser cover, not concealed, Perception modifier +5, CombatantId(2) observed by CombatantId(1), and a scripted natural 15
- **WHEN** Hide is resolved
- **THEN** the error is NoStealthDc
- **AND** the relation is still observed and the scripted 15 is still unused
- **AND** the error is not RequirementNotMet

#### Scenario: Hiding from yourself rolls nothing

- **GIVEN** Stealth DC 17, standard cover, Perception modifier +5, a scripted natural 6, and CombatantId(2) as both the subject and the only observer
- **WHEN** Hide is resolved
- **THEN** the error is SameCreature
- **AND** the scripted 6 is still unused

#### Scenario: Hiding from yourself twice rolls nothing

- **GIVEN** Stealth DC 17, standard cover, Perception modifier +5, a scripted natural 6, and CombatantId(2) listed twice as an observer of CombatantId(2)
- **WHEN** Hide is resolved
- **THEN** the error is DuplicateId
- **AND** the scripted 6 is still unused
- **AND** the error is not SameCreature

#### Scenario: A duplicate observer rolls nothing

- **GIVEN** Stealth DC 17, standard cover, Perception modifier +5, a scripted natural 6, and CombatantId(1) listed twice as an observer of CombatantId(2)
- **WHEN** Hide is resolved
- **THEN** the error is DuplicateId
- **AND** the relation is still observed and the scripted 6 is still unused

#### Scenario: A Perception DC that does not fit rolls nothing

- **GIVEN** Stealth DC 17, standard cover, Perception modifier 2147483638, a scripted natural 10, and CombatantId(2) observed by CombatantId(1)
- **WHEN** Hide is resolved
- **THEN** the error is Overflow
- **AND** the relation is still observed and the scripted 10 is still unused

#### Scenario: The lowest Stealth DC with standard cover rolls nothing

- **GIVEN** Stealth DC i32::MIN, standard cover, Perception modifier +5, a scripted natural 6, and CombatantId(2) observed by CombatantId(1) and not concealed
- **WHEN** Hide is resolved
- **THEN** the error is Overflow
- **AND** the relation is still observed and the scripted 6 is still unused

#### Scenario: The highest Stealth DC with greater cover rolls nothing

- **GIVEN** Stealth DC i32::MAX, greater cover, Perception modifier +5, a scripted natural 6, and CombatantId(2) observed by CombatantId(1) and not concealed
- **WHEN** Hide is resolved
- **THEN** the error is Overflow
- **AND** the relation is still observed and the scripted 6 is still unused

### Requirement: The Hide modifier is the Stealth DC minus 10

The Stealth modifier SHALL be the Stealth DC minus 10, plus 2 for standard cover, 4 for greater cover, or 0 if only concealed. The DC SHALL be 10 plus the Perception modifier.

**Rule**: Secret Stealth roll (Player Core pg. 244).
**Why not**: No skills. DEV-10 omitted Hide.
**Substitute**: Modifier is that DC minus 10. The roll is open.
**Intent**: One Stealth number. Proposed DEV-40.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2404 (Player Core pg. 244, Hide)

#### Scenario: Standard cover and a natural 6 hides

- **GIVEN** Stealth DC 17, standard cover, Perception modifier +5, a scripted natural 6, and CombatantId(2) observed by CombatantId(1) and not concealed
- **WHEN** Hide is resolved
- **THEN** the modifier is +9, the DC is 15, the total is 15, the degree is success, and the relation is hidden

#### Scenario: Standard cover and a natural 5 stays observed

- **GIVEN** Stealth DC 17, standard cover, Perception modifier +5, a scripted natural 5, and CombatantId(2) observed by CombatantId(1) and not concealed
- **WHEN** Hide is resolved
- **THEN** the modifier is +9, the DC is 15, the total is 14, and the degree is failure
- **AND** the relation is still observed

#### Scenario: Greater cover adds 4

- **GIVEN** Stealth DC 17, greater cover, Perception modifier +5, a scripted natural 4, and CombatantId(2) observed by CombatantId(1) and not concealed
- **WHEN** Hide is resolved
- **THEN** the modifier is +11, the DC is 15, the total is 15, and the degree is success
- **AND** the relation is hidden

#### Scenario: Concealment alone and a natural 8 hides

- **GIVEN** Stealth DC 17, no cover, concealed, Perception modifier +5, a scripted natural 8, and CombatantId(2) observed by CombatantId(1)
- **WHEN** Hide is resolved
- **THEN** the modifier is +7, the DC is 15, the total is 15, and the degree is success
- **AND** the relation is hidden and still concealed

#### Scenario: Concealment alone and a natural 7 stays observed

- **GIVEN** Stealth DC 17, no cover, concealed, Perception modifier +5, a scripted natural 7, and CombatantId(2) observed by CombatantId(1)
- **WHEN** Hide is resolved
- **THEN** the modifier is +7, the DC is 15, the total is 14, and the degree is failure
- **AND** the relation is still observed and still concealed

#### Scenario: One Stealth roll meets one Perception DC and misses another

- **GIVEN** Stealth DC 17, standard cover, a scripted natural 10, CombatantId(2) observed and not concealed by CombatantId(1) at Perception +5 and by CombatantId(5) at Perception +10
- **WHEN** Hide is resolved
- **THEN** one total of 19 is a success against DC 15 and a failure against DC 20
- **AND** CombatantId(2) is hidden from CombatantId(1) and still observed by CombatantId(5)

### Requirement: Hide uses the caller's Perception modifier

perception_modifier SHALL be the caller's final value, and this module SHALL NOT read the observer's Vital.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2404 (Player Core pg. 244, Hide)

#### Scenario: The caller passes the final Perception modifier

- **GIVEN** Stealth DC 17, standard cover, an Unconscious observer, Perception modifier +1 from +5 minus 4, a scripted natural 2, and CombatantId(2) observed by CombatantId(1) and not concealed
- **WHEN** Hide is resolved
- **THEN** the modifier is +9, the DC is 11, the total is 11, and the degree is success
- **AND** the relation is hidden

### Requirement: Hide costs one action and follows the success entry

Hide SHALL cost DetectionCost::Hide: 1 action, with none of the concentrate, move, or manipulate traits. The caller SHALL spend that 1 action through spend_actions. This module SHALL NOT call it. Success and critical success SHALL hide an observed subject. Failure SHALL leave that subject observed. An observer who already has the subject hidden or undetected SHALL be skipped and SHALL NOT be rolled.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2404 (Player Core pg. 244, Hide)

#### Scenario: Hide reports one action

- **GIVEN** Stealth DC 17, standard cover, Perception modifier +5, a scripted natural 6, and CombatantId(2) observed by CombatantId(1) and not concealed
- **WHEN** Hide is resolved
- **THEN** the cost is DetectionCost::Hide, 1 action, with none of the concentrate, move, or manipulate traits

#### Scenario: A critical success on Hide hides

- **GIVEN** Stealth DC 17, standard cover, Perception modifier +5, a scripted natural 16, and CombatantId(2) observed by CombatantId(1) and not concealed
- **WHEN** Hide is resolved
- **THEN** the total is 25 and the numeric degree is critical success
- **AND** the effect resolves as a success and the relation is hidden

#### Scenario: A natural 1 on Hide stays observed

- **GIVEN** Stealth DC 17, standard cover, Perception modifier +5, a scripted natural 1, and CombatantId(2) observed by CombatantId(1) and not concealed
- **WHEN** Hide is resolved
- **THEN** the modifier is +9, the DC is 15, the total is 10, and the numeric degree is critical failure
- **AND** the effect resolves as a failure and the relation is still observed

#### Scenario: A natural 20 on Hide hides

- **GIVEN** Stealth DC 17, standard cover, Perception modifier +25, a scripted natural 20, and CombatantId(2) observed by CombatantId(1) and not concealed
- **WHEN** Hide is resolved
- **THEN** the modifier is +9, the DC is 35, the total is 29, and the degree is success
- **AND** the relation is hidden

#### Scenario: An observer who already has you hidden is skipped

- **GIVEN** Stealth DC 17, standard cover, Perception modifier +5, a scripted natural 6, and CombatantId(2) already hidden from CombatantId(1)
- **WHEN** Hide is resolved
- **THEN** the call succeeds and the relation is still hidden
- **AND** the scripted 6 is still unused

#### Scenario: An observer who already has you undetected is skipped

- **GIVEN** Stealth DC 17, standard cover, Perception modifier +5, a scripted natural 6, and CombatantId(2) already undetected by CombatantId(1)
- **WHEN** Hide is resolved
- **THEN** the call succeeds and the relation is still undetected
- **AND** the scripted 6 is still unused

### Requirement: Losing cover and concealment ends hidden

A hidden subject that has neither standard nor greater cover, and is not concealed, SHALL become observed when the caller calls refresh. Lesser cover SHALL NOT keep the subject hidden. Undetected SHALL stay undetected. Concealment SHALL keep a hidden subject hidden. refresh SHALL NOT run unless the caller calls it.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2404 (Player Core pg. 244, Hide)

#### Scenario: Losing cover and concealment ends hidden

- **GIVEN** CombatantId(2) hidden and not concealed from CombatantId(1), with Stealth DC 17, and no cover
- **WHEN** the caller calls refresh
- **THEN** the detection is observed

#### Scenario: Lesser cover does not keep a creature hidden

- **GIVEN** CombatantId(2) hidden and not concealed from CombatantId(1), with Stealth DC 17, and lesser cover
- **WHEN** the caller calls refresh
- **THEN** the detection is observed

#### Scenario: Concealment keeps a hidden creature hidden

- **GIVEN** CombatantId(2) hidden and concealed from CombatantId(1), with Stealth DC 17, and no cover
- **WHEN** the caller calls refresh
- **THEN** the detection is still hidden
- **AND** the concealment is Concealed

#### Scenario: Losing cover does not end undetected

- **GIVEN** CombatantId(2) undetected and not concealed from CombatantId(1), with Stealth DC 17, and no cover
- **WHEN** the caller calls refresh
- **THEN** the detection is still undetected

### Requirement: A hidden Strike is off-guard, then every observer observes

A hidden Strike SHALL read OffGuard::Yes first. Each observer who had the striker hidden or undetected SHALL then observe them. Armor Class SHALL be −2 with no cover. Concealment SHALL stay, and so SHALL every other pair.

**Rule**: A Strike ends hidden for every observer (PC 244).
**Why not**: Sneak is deferred.
**Substitute**: Every observer observes the striker.
**Intent**: Off-guard, then observed. DEV-43.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2404 (Player Core pg. 244, Hide)

#### Scenario: A hidden Strike is off-guard, then every observer observes

- **GIVEN** CombatantId(2) hidden and not concealed from CombatantId(1) and from CombatantId(3), with Stealth DC 17, and a scripted natural 8
- **WHEN** note_strike is called with striker CombatantId(2) and target CombatantId(1), and Armor Class is totaled once with no cover
- **THEN** the off-guard result is OffGuard::Yes and the Armor Class total is −2
- **AND** CombatantId(1) and CombatantId(3) read CombatantId(2) as observed and not concealed
- **AND** the scripted 8 is still unused

#### Scenario: A Strike does not flip the other direction

- **GIVEN** observer 1 has subject 2 and subject 3 hidden, observer 3 has subject 2 and subject 1 hidden, each NotConcealed with Stealth DC 17, and a scripted natural 8
- **WHEN** note_strike is called with striker 2 and target 1
- **THEN** observer 1 and observer 3 read subject 2 as observed and NotConcealed
- **AND** observer 1 still reads subject 3 as hidden, and observer 3 still reads subject 1 as hidden
- **AND** the scripted 8 is still unused

### Requirement: Other actions do not refresh hidden

note_action SHALL NOT call refresh. Stride SHALL leave the hider hidden, including when cover and concealment are already gone, until the caller calls refresh.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2404 (Player Core pg. 244, Hide)

#### Scenario: A Stride does not refresh hidden

- **GIVEN** CombatantId(2) hidden and not concealed from CombatantId(1), with Stealth DC 17, no cover, and a scripted natural 8
- **WHEN** note_action is called with Stride and refresh is not called
- **THEN** the relation is still hidden
- **AND** the scripted 8 is still unused

#### Scenario: Refresh after a Stride ends hidden

- **GIVEN** CombatantId(2) hidden and not concealed from CombatantId(1), with Stealth DC 17, no cover, and a scripted natural 8
- **WHEN** note_action is called with Stride and the caller then calls refresh with no cover
- **THEN** the detection is observed
- **AND** the scripted 8 is still unused

### Requirement: An undetected Strike becomes observed

A Strike while undetected SHALL read the target's OffGuard::Yes before any relation changes. With no cover and no unconscious penalty, Armor Class SHALL be −2. Every observer who had the striker hidden or undetected SHALL observe them.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2405 (Player Core pg. 245, Sneak)

#### Scenario: An undetected Strike is off-guard, then the striker is observed

- **GIVEN** CombatantId(2) undetected and not concealed from CombatantId(1) and from CombatantId(3), with Stealth DC 17
- **WHEN** note_strike is called with striker CombatantId(2) and target CombatantId(1), and Armor Class is totaled once with no cover
- **THEN** the off-guard result is OffGuard::Yes and the Armor Class total is −2
- **AND** CombatantId(1) and CombatantId(3) read CombatantId(2) as observed and not concealed

### Requirement: note_strike reports detection off-guard only

note_strike's OffGuard SHALL cover detection only. The caller SHALL combine it with other OffGuard sources through off_guard_against. An observed striker SHALL give OffGuard::No, and the relations SHALL stay unchanged.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=58 (Player Core pg. 445, Off-Guard)

#### Scenario: An observed striker is not off-guard from detection

- **GIVEN** CombatantId(2) observed and not concealed from CombatantId(1), and a scripted natural 8
- **WHEN** note_strike is called with striker CombatantId(2) and target CombatantId(1)
- **THEN** the off-guard result is OffGuard::No
- **AND** off_guard_against of that observed relation with OffGuard::Yes is OffGuard::Yes
- **AND** the relation is still observed and the scripted 8 is still unused
