# seek-and-hidden

## Purpose

Records whether one creature is observed, hidden, or undetected by another, including a concealed flag, and resolves Seek, Hide, and the targeting flat checks.

## ADDED Requirements

### Requirement: The same detection twice matches

The same inputs SHALL return the same result. A call SHALL NOT read a file, the network, a clock, or a global generator. A fallible call SHALL return a typed error and SHALL NOT panic. The rules crate SHALL gain no dependency. A call SHALL NOT write Hit Points, actions, or the Creature.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2414 (Player Core pg. 444, Detecting Creatures)

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

Observed, hidden, and undetected SHALL be a relation from one observer CombatantId to one subject CombatantId. A missing relation SHALL be observed and not concealed. The observer and the subject SHALL NOT be the same id, and that rejection SHALL write nothing.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2414 (Player Core pg. 444, Detecting Creatures)

#### Scenario: One subject is hidden from one observer and observed by another

- **GIVEN** CombatantId(2) is hidden from CombatantId(1) and no relation is stored for CombatantId(3)
- **WHEN** each observer reads CombatantId(2)
- **THEN** CombatantId(1) reads hidden
- **AND** CombatantId(3) reads observed and not concealed

#### Scenario: A missing relation is observed

- **GIVEN** no stored relation from CombatantId(1) to CombatantId(2)
- **WHEN** the relation is read
- **THEN** the detection is observed and the subject is not concealed

#### Scenario: The observer and the subject cannot be the same

- **GIVEN** CombatantId(1) as both observer and subject, and a scripted natural 8
- **WHEN** a relation is stored
- **THEN** the error is SameCreature
- **AND** no relation is stored and the scripted 8 is still unused

### Requirement: Concealed does not change the detection category

Concealed SHALL be a flag on the relation. It SHALL NOT replace observed, hidden, or undetected. Observed SHALL stay observed while the flag is set.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2414 (Player Core pg. 444, Detecting Creatures)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=62 (Player Core pg. 442, Concealed)

#### Scenario: An observed creature can be concealed

- **GIVEN** CombatantId(2) observed and concealed from CombatantId(1)
- **WHEN** the relation is read
- **THEN** the detection is observed
- **AND** the subject is concealed

#### Scenario: A hidden creature can be concealed

- **GIVEN** CombatantId(2) hidden and concealed from CombatantId(1), with Stealth DC 17
- **WHEN** the relation is read
- **THEN** the detection is hidden
- **AND** the subject is concealed

### Requirement: Hidden and undetected need a Stealth DC

Hidden or undetected SHALL be rejected when the subject has no Stealth DC, and the relation SHALL stay unchanged. Observed, with or without concealed, SHALL be stored with no Stealth DC.

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
- **THEN** the detection is observed and the subject is concealed

### Requirement: A concealed target takes a DC 5 flat check

A creature that is observed and concealed SHALL take one flat check at DC 5 before the effect. A critical success SHALL affect the target. A critical failure SHALL NOT. No modifier SHALL be added.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=62 (Player Core pg. 442, Concealed)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2278 (Player Core pg. 400, Flat Checks)

#### Scenario: A natural 5 at DC 5 affects the target

- **GIVEN** CombatantId(2) observed and concealed from CombatantId(1), and a scripted natural 5
- **WHEN** a non-area targeting check is resolved
- **THEN** the roll is consumed, the DC is 5, the degree is success, and the target is affected

#### Scenario: A natural 4 at DC 5 does not affect the target

- **GIVEN** CombatantId(2) observed and concealed from CombatantId(1), and a scripted natural 4
- **WHEN** a non-area targeting check is resolved
- **THEN** the roll is consumed, the DC is 5, the degree is failure, and the target is not affected

#### Scenario: A natural 1 at DC 5 does not affect the target

- **GIVEN** CombatantId(2) observed and concealed from CombatantId(1), and a scripted natural 1
- **WHEN** a non-area targeting check is resolved
- **THEN** the numeric degree is critical failure
- **AND** the effect resolves as a failure and the target is not affected

#### Scenario: A natural 20 at DC 5 affects the target

- **GIVEN** CombatantId(2) observed and concealed from CombatantId(1), and a scripted natural 20
- **WHEN** a non-area targeting check is resolved
- **THEN** the numeric degree is critical success
- **AND** the effect resolves as a success and the target is affected

### Requirement: A hidden target takes a DC 11 flat check

A creature that is hidden and not concealed SHALL take one flat check at DC 11. A critical success SHALL affect the target. The actions of the effect are not refunded by this result.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=79 (Player Core pg. 444, Hidden)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2416 (Player Core pg. 444, Hidden)

#### Scenario: A natural 11 at DC 11 affects the target

- **GIVEN** CombatantId(2) hidden and not concealed from CombatantId(1), with Stealth DC 17, and a scripted natural 11
- **WHEN** a non-area targeting check is resolved
- **THEN** the roll is consumed, the DC is 11, the degree is success, and the target is affected

#### Scenario: A natural 10 at DC 11 does not affect the target

- **GIVEN** CombatantId(2) hidden and not concealed from CombatantId(1), with Stealth DC 17, and a scripted natural 10
- **WHEN** a non-area targeting check is resolved
- **THEN** the roll is consumed, the DC is 11, the degree is failure, and the target is not affected

#### Scenario: A natural 20 at DC 11 affects the target

- **GIVEN** CombatantId(2) hidden and not concealed from CombatantId(1), with Stealth DC 17, and a scripted natural 20
- **WHEN** a non-area targeting check is resolved
- **THEN** the numeric degree is critical success
- **AND** the effect resolves as a success and the target is affected

### Requirement: Hidden and concealed roll once at DC 11

When the subject is hidden and concealed, the targeting flat checks SHALL roll once at DC 11, the higher DC. A second die SHALL NOT be drawn.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2278 (Player Core pg. 400, Flat Checks)

#### Scenario: Hidden and concealed fail one DC 11 roll

- **GIVEN** CombatantId(2) hidden and concealed from CombatantId(1), with Stealth DC 17, and scripted naturals 10 and then 20
- **WHEN** a non-area targeting check is resolved
- **THEN** one roll of 10 is compared with DC 11, the degree is failure, and the target is not affected
- **AND** the scripted 20 is still unused

### Requirement: An observed target or an area effect rolls nothing

An observed creature that is not concealed SHALL be affected and SHALL draw no die. An area effect SHALL be affected and SHALL draw no die, including when the subject is hidden, concealed, or undetected.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2415 (Player Core pg. 444, Observed)
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
**Intent**: Seek learns the space. Proposed DEV-39.

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

### Requirement: Seek is one action inside 30 feet

Seek SHALL cost 1 action with the concentrate trait. Any dimension above 30 feet SHALL return AreaTooLarge and SHALL draw no die (DEV-08). The degree SHALL be in the result.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2301 (Player Core pg. 417, Seek)

#### Scenario: Seek reports one concentrate action

- **GIVEN** Perception modifier +5, a scripted natural 12, an area of 30 by 10 by 10 feet, and CombatantId(2) undetected by CombatantId(1) with Stealth DC 17
- **WHEN** Seek is resolved
- **THEN** the cost is 1 action with the concentrate trait
- **AND** the total is 17, the degree is success, and the relation is hidden

#### Scenario: An area beyond 30 feet rolls nothing

- **GIVEN** an area of 35 by 5 by 5 feet, Perception modifier +5, a scripted natural 12, and CombatantId(2) undetected by CombatantId(1) with Stealth DC 17
- **WHEN** Seek is resolved
- **THEN** the error is AreaTooLarge
- **AND** the relation is still undetected and the scripted 12 is still unused

### Requirement: Seek degrees move undetected and hidden

A critical success SHALL move hidden or undetected to observed. A success SHALL move undetected to hidden and hidden to observed. Failure SHALL leave the relation unchanged. A critical failure has no entry and SHALL leave it unchanged. Invisibility or an imprecise sense SHALL NOT keep the creature hidden (proposed DEV-40).

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

#### Scenario: A natural 20 moves hidden to observed

- **GIVEN** Perception modifier +0, a scripted natural 20, an area of 30 by 10 by 10 feet, and CombatantId(2) hidden from CombatantId(1) with Stealth DC 25 and not concealed
- **WHEN** Seek is resolved
- **THEN** the total is 20 and the degree is success
- **AND** the detection is observed

#### Scenario: A success leaves the concealed flag set

- **GIVEN** Perception modifier +5, a scripted natural 12, an area of 30 by 10 by 10 feet, and CombatantId(2) hidden and concealed from CombatantId(1) with Stealth DC 17
- **WHEN** Seek is resolved
- **THEN** the total is 17 and the degree is success
- **AND** the detection is observed and the subject is still concealed

### Requirement: Seek rolls once and skips everyone else

One Perception check SHALL be compared with each hidden or undetected subject that has a Stealth DC. Observed subjects and subjects with no Stealth DC SHALL NOT be compared. A duplicate list SHALL return DuplicateId and draw no die, even if the area is too large. The seeker as a subject SHALL return SameCreature and draw no die. A Perception modifier that does not fit a d20 SHALL return Overflow and draw no die.

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

#### Scenario: A Perception modifier that does not fit rolls nothing

- **GIVEN** Perception modifier 2147483647, a scripted natural 12, an area of 30 by 10 by 10 feet, and CombatantId(2) hidden from CombatantId(1) with Stealth DC 17
- **WHEN** Seek is resolved
- **THEN** the error is Overflow
- **AND** the relation is still hidden and the scripted 12 is still unused

### Requirement: Hide needs cover or concealment

Hide SHALL be rejected, draw no die, and leave the relation unchanged when the subject lacks standard cover, greater cover, and the concealed flag. Lesser cover SHALL NOT qualify. No Stealth DC SHALL return NoStealthDc even when cover also fails. An observer id equal to the subject SHALL return SameCreature and draw no die. An observer without cover or concealment SHALL stay unchanged when another qualifies.

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

#### Scenario: One observer without cover is left observed

- **GIVEN** Stealth DC 17, a scripted natural 6, CombatantId(2) observed and not concealed by CombatantId(1) with standard cover and Perception +5, and by CombatantId(5) with no cover and Perception +5
- **WHEN** Hide is resolved
- **THEN** one total of 15 hides CombatantId(2) from CombatantId(1)
- **AND** CombatantId(5) still observes CombatantId(2)

### Requirement: The Hide modifier is the Stealth DC minus 10

The Stealth modifier SHALL be the Stealth DC minus 10, plus 2 for standard cover, 4 for greater cover, or 0 if only concealed. The DC SHALL be 10 plus the Perception modifier.

**Rule**: Secret Stealth roll (Player Core pg. 244).
**Why not**: No skills. DEV-10 omitted Hide.
**Substitute**: Modifier is that DC minus 10. The roll is open.
**Intent**: One Stealth number. Proposed DEV-38.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2404 (Player Core pg. 244, Hide)

#### Scenario: Standard cover and a natural 6 hides

- **GIVEN** Stealth DC 17, standard cover, Perception modifier +5, a scripted natural 6, and CombatantId(2) observed by CombatantId(1) and not concealed
- **WHEN** Hide is resolved
- **THEN** the cost is 1 action with no concentrate, move, or manipulate trait
- **AND** the modifier is +9, the DC is 15, the total is 15, the degree is success, and the relation is hidden

### Requirement: Hide costs one action and follows the success entry

Hide SHALL cost 1 action and SHALL have none of the concentrate, move, or manipulate traits. Success and critical success SHALL hide an observed subject. A subject already hidden or undetected SHALL keep that detection. Failure SHALL leave the relation unchanged.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2404 (Player Core pg. 244, Hide)

#### Scenario: Standard cover and a natural 5 stays observed

- **GIVEN** Stealth DC 17, standard cover, Perception modifier +5, a scripted natural 5, and CombatantId(2) observed by CombatantId(1) and not concealed
- **WHEN** Hide is resolved
- **THEN** the modifier is +9, the DC is 15, the total is 14, and the degree is failure
- **AND** the relation is still observed

#### Scenario: A critical success on Hide hides

- **GIVEN** Stealth DC 17, standard cover, Perception modifier +5, a scripted natural 16, and CombatantId(2) observed by CombatantId(1) and not concealed
- **WHEN** Hide is resolved
- **THEN** the total is 25 and the numeric degree is critical success
- **AND** the effect resolves as a success and the relation is hidden

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

#### Scenario: A success keeps hidden

- **GIVEN** Stealth DC 17, standard cover, Perception modifier +5, a scripted natural 6, and CombatantId(2) already hidden from CombatantId(1)
- **WHEN** Hide is resolved
- **THEN** the total is 15 and the degree is success
- **AND** the relation is still hidden

#### Scenario: A success keeps undetected

- **GIVEN** Stealth DC 17, standard cover, Perception modifier +5, a scripted natural 6, and CombatantId(2) already undetected by CombatantId(1)
- **WHEN** Hide is resolved
- **THEN** the total is 15 and the degree is success
- **AND** the relation is still undetected

#### Scenario: A Perception DC that does not fit rolls nothing

- **GIVEN** Stealth DC 17, standard cover, Perception modifier 2147483638, a scripted natural 10, and CombatantId(2) observed by CombatantId(1)
- **WHEN** Hide is resolved
- **THEN** the error is Overflow
- **AND** the relation is still observed and the scripted 10 is still unused

#### Scenario: A duplicate observer rolls nothing

- **GIVEN** Stealth DC 17, standard cover, Perception modifier +5, a scripted natural 6, and CombatantId(1) listed twice as an observer of CombatantId(2)
- **WHEN** Hide is resolved
- **THEN** the error is DuplicateId
- **AND** the relation is still observed and the scripted 6 is still unused

### Requirement: Losing cover and concealment ends hidden

A hidden subject that has neither standard nor greater cover, and is not concealed, SHALL become observed. Lesser cover SHALL NOT keep the subject hidden. Undetected SHALL stay undetected. The concealed flag SHALL keep a hidden subject hidden.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2404 (Player Core pg. 244, Hide)

#### Scenario: Losing cover and concealment ends hidden

- **GIVEN** CombatantId(2) hidden and not concealed from CombatantId(1), with Stealth DC 17, and no cover
- **WHEN** the cover and concealment are refreshed
- **THEN** the detection is observed

#### Scenario: Lesser cover does not keep a creature hidden

- **GIVEN** CombatantId(2) hidden and not concealed from CombatantId(1), with Stealth DC 17, and lesser cover
- **WHEN** the cover and concealment are refreshed
- **THEN** the detection is observed

#### Scenario: Concealment keeps a hidden creature hidden

- **GIVEN** CombatantId(2) hidden and concealed from CombatantId(1), with Stealth DC 17, and no cover
- **WHEN** the cover and concealment are refreshed
- **THEN** the detection is still hidden
- **AND** the subject is still concealed

#### Scenario: Losing cover does not end undetected

- **GIVEN** CombatantId(2) undetected and not concealed from CombatantId(1), with Stealth DC 17, and no cover
- **WHEN** the cover and concealment are refreshed
- **THEN** the detection is still undetected

### Requirement: A hidden Strike is off-guard, then that observer observes

A Strike while hidden SHALL read OffGuard::Yes first, then set only that observer to observed. The concealed flag SHALL stay.

**Rule**: Any action but Hide, Sneak, or Step ends hidden (Player Core pg. 244).
**Why not**: Sneak is deferred.
**Substitute**: Only this Strike clears hidden, and only for the struck observer.
**Intent**: The off-guard Strike stays. Proposed DEV-41.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2404 (Player Core pg. 244, Hide)

#### Scenario: A hidden Strike is off-guard, then that observer observes

- **GIVEN** CombatantId(2) hidden and not concealed from CombatantId(1) and from CombatantId(3), with Stealth DC 17, and a scripted natural 8
- **WHEN** a Strike against CombatantId(1) is noted and Armor Class is totaled once with no cover
- **THEN** the off-guard result is OffGuard::Yes and the Armor Class total is −2
- **AND** CombatantId(1) reads observed and not concealed, CombatantId(3) reads hidden, and the scripted 8 is still unused

### Requirement: Other actions do not end hidden

An action other than that Strike, including Stride, SHALL leave the hider's detection unchanged.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2404 (Player Core pg. 244, Hide)

#### Scenario: A Stride does not end hidden

- **GIVEN** CombatantId(2) hidden and not concealed from CombatantId(1), with Stealth DC 17, and a scripted natural 8
- **WHEN** a Stride is noted
- **THEN** the relation is still hidden
- **AND** the scripted 8 is still unused

### Requirement: An undetected Strike stays undetected

An undetected striker SHALL stay undetected, and the struck observer SHALL be OffGuard::Yes.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=96 (Player Core pg. 447, Undetected)

#### Scenario: An undetected Strike stays undetected and off-guard

- **GIVEN** CombatantId(2) undetected and not concealed from CombatantId(1), with Stealth DC 17
- **WHEN** a Strike against CombatantId(1) is noted and Armor Class is totaled once with no cover
- **THEN** the off-guard result is OffGuard::Yes and the Armor Class total is −2
- **AND** the relation is still undetected
