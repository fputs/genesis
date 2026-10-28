# positioning Specification

## Purpose
Decides prone posture, flanking on a 5-foot grid, off-guard to an attack, and Take Cover, and folds those bonuses and penalties through the existing modifier total.

## Requirements

### Requirement: Positioning is pure

The same positioning inputs SHALL return the same result. A call SHALL NOT roll dice, read a clock, or read a file. A fallible call SHALL return a typed error and SHALL NOT panic. The rules crate SHALL gain no dependency.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2375 (Player Core pg. 425, Flanking)

#### Scenario: The same flanking query twice matches

- **GIVEN** Enemy id 1 at (0, 0) and Pc ids 2 and 3 at (0, 1) and (0, -1), all Medium, Able, Standing, and reach 5, and a scripted d20 holding 8
- **WHEN** flanking is resolved twice
- **THEN** both results are equal and both say the target is flanked
- **AND** the scripted 8 is still unused

#### Scenario: The same take cover twice matches

- **GIVEN** a standing creature at (3, 4) with standard cover, no stored benefit, and a scripted d20 holding 8
- **WHEN** Take Cover is resolved twice
- **THEN** both results are equal and both store greater cover
- **AND** the scripted 8 is still unused

#### Scenario: The same drop prone twice matches

- **GIVEN** a standing creature with Mobility::Free, square (3, 4), no stored benefit, and a scripted d20 holding 8
- **WHEN** Drop Prone is resolved twice
- **THEN** both results are equal and both are prone on (3, 4)
- **AND** the scripted 8 is still unused

#### Scenario: The same off-guard query twice matches

- **GIVEN** Enemy id 1 at (0, 0) and Pc ids 2 and 3 at (0, 1) and (0, -1), all Medium, Able, Standing, and reach 5, a melee attack, and a scripted d20 holding 8
- **WHEN** off-guard is resolved twice
- **THEN** both results are equal and both are off-guard
- **AND** the scripted 8 is still unused

### Requirement: Posture and cover are caller values

The Posture value and a Take Cover benefit SHALL be values the caller keeps. Drop Prone and Stand SHALL NOT write combatant state. A prone condition SHALL NOT be that Posture and SHALL NOT change the posture result. A positioning call SHALL NOT change Hit Points, actions remaining, or the reaction.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=88 (Player Core pg. 445, Prone)

#### Scenario: Drop Prone leaves the combatant state

- **GIVEN** a standing creature with Mobility::Free, a combatant state with 2 actions remaining and the reaction not ready, and square (3, 4)
- **WHEN** Drop Prone succeeds without that state as an argument
- **THEN** the returned posture is prone on square (3, 4)
- **AND** the state still has 2 actions remaining and the reaction is still not ready
- **AND** the condition list is empty

#### Scenario: A prone condition does not change Drop Prone

- **GIVEN** a standing caller at square (3, 4) with Mobility::Free, and a combatant state that already has prone
- **WHEN** Drop Prone succeeds without that state as an argument
- **THEN** the returned posture is prone on square (3, 4)
- **AND** the condition list still has prone

### Requirement: Drop Prone and Stand name the move trait

Drop Prone and Stand SHALL each name a cost of 1 action with the move trait. They SHALL NOT spend that action. They SHALL NOT resolve Reactive Strike. The result SHALL NOT include a trigger flag. Change 8 treats any PostureChange whose cost includes the move trait as its reactive trigger (FIX-06).

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2295 (Player Core pg. 416, Drop Prone)

#### Scenario: Drop Prone reports one move action

- **GIVEN** a standing creature with Mobility::Free
- **WHEN** Drop Prone succeeds
- **THEN** the cost is 1 action with the move trait
- **AND** the result has no trigger flag

#### Scenario: Stand reports one move action

- **GIVEN** a prone creature with Mobility::Free
- **WHEN** Stand succeeds
- **THEN** the cost is 1 action with the move trait
- **AND** the result has no trigger flag

### Requirement: Immobilized blocks Drop Prone and Stand

Drop Prone, Stand, Leap, and prone movement SHALL take Mobility, not a bool. Mobility::Immobilized SHALL return Immobilized and SHALL change nothing. That error SHALL be reported before NotProne and before ProneRestrictsMovement. A standing creature with Mobility::Free SHALL get Ok from prone movement for Step, and the caller's diagonal counter SHALL stay as it was.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=81 (Player Core pg. 444, Immobilized)

#### Scenario: An immobilized creature cannot drop prone or stand

- **GIVEN** a creature at (3, 4) with Mobility::Immobilized, once standing and once prone, with a stored benefit
- **WHEN** Drop Prone and Stand are each attempted
- **THEN** each result is Immobilized
- **AND** posture, square, and the stored benefit stay as they were

#### Scenario: Standing cannot stand

- **GIVEN** a standing creature with Mobility::Free
- **WHEN** Stand is attempted
- **THEN** the result is NotProne

#### Scenario: Immobilized wins over not prone

- **GIVEN** a standing creature with Mobility::Immobilized
- **WHEN** Stand is attempted
- **THEN** the result is Immobilized

#### Scenario: Immobilized wins over a prone move

- **GIVEN** a prone creature with Mobility::Immobilized and a diagonal counter whose count is 1
- **WHEN** prone movement is attempted with Step
- **THEN** the result is Immobilized
- **AND** the counter still counts 1 diagonal

#### Scenario: Immobilized wins over leap while prone

- **GIVEN** a prone creature with Mobility::Immobilized
- **WHEN** Leap is attempted
- **THEN** the result is Immobilized
- **AND** the result contains no distance

#### Scenario: Standing prone move step is ok

- **GIVEN** a standing creature with Mobility::Free and a diagonal counter whose count is 1
- **WHEN** prone movement is attempted with Step
- **THEN** the result is Ok
- **AND** the counter still counts 1 diagonal

### Requirement: Only Stand is legal while prone

While prone, Stand SHALL be the only move this change allows. This change SHALL NOT implement Stride or Step; they stay in grid-movement. Naming them, or Drop Prone, while prone SHALL return ProneRestrictsMovement and no distance. Leap while prone SHALL return the same. A standing Leap SHALL return Unsupported and no distance. Crawl and standing Leap distance belong to add-crawl-and-leap.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=88 (Player Core pg. 445, Prone)

#### Scenario: Only Stand is legal while prone

- **GIVEN** a prone creature with Mobility::Free, on square (3, 4)
- **WHEN** Stand, Drop Prone, Stride, and Step are each attempted
- **THEN** Stand succeeds on (3, 4)
- **AND** Drop Prone, Stride, and Step each return ProneRestrictsMovement

#### Scenario: No prone move variant returns a distance

- **GIVEN** a prone creature with Mobility::Free and a diagonal counter whose count is 1
- **WHEN** Stride, Step, and Drop Prone are each passed to prone movement
- **THEN** each result is ProneRestrictsMovement
- **AND** none of the results contains a distance
- **AND** the counter still counts 1 diagonal

#### Scenario: Leap while prone is rejected

- **GIVEN** a prone creature with Mobility::Free
- **WHEN** Leap is attempted
- **THEN** the result is ProneRestrictsMovement
- **AND** the result contains no distance

#### Scenario: A standing leap is not resolved

- **GIVEN** a standing creature with Mobility::Free
- **WHEN** Leap is attempted
- **THEN** the result is Unsupported
- **AND** the result contains no distance

#### Scenario: Move actions are stride step and drop prone

- **GIVEN** the move actions this change accepts
- **WHEN** they are listed
- **THEN** they are Stride, Step, and Drop Prone
- **AND** Leap and Crawl are not among them
- **AND** Stride and Step are not implemented by this change

### Requirement: The creature stays in its square

Drop Prone, Stand, and Take Cover SHALL NOT move the creature. The returned square SHALL equal the input square. An occupant SHALL NOT block Stand (DEV-27). PostureChange SHALL have exactly the fields posture, square, benefit, and cost. Another creature ending a move in the square is grid-movement's OccupiedEnd.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2361 (Player Core pg. 422, Sharing Space)

#### Scenario: The returned square equals the input square

- **GIVEN** a creature at (3, 4) with caller standard cover, standing for Drop Prone and Take Cover, and prone for Stand
- **WHEN** Drop Prone, Stand, and Take Cover each succeed
- **THEN** each returned square is (3, 4)

#### Scenario: Stand ignores an occupant

- **GIVEN** a prone creature at (3, 4) and another creature in that square
- **WHEN** Stand is attempted
- **THEN** Stand succeeds on square (3, 4)
- **AND** the result has exactly the fields posture, square, benefit, and cost

#### Scenario: Drop Prone keeps the square and the cover benefit

- **GIVEN** a standing creature at (3, 4) with stored standard cover
- **WHEN** Drop Prone succeeds
- **THEN** the posture is prone, the square is (3, 4), and the benefit is still standard

### Requirement: Prone uses the existing attack penalty and off-guard

A prone creature SHALL be off-guard, and that penalty SHALL be the existing circumstance minus 2 to Armor Class, for melee and for ranged. The attack penalty SHALL take Posture. Posture::Prone SHALL be the existing circumstance minus 2. Posture::Standing SHALL add no prone penalty. This change SHALL NOT build a second copy of either penalty.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=88 (Player Core pg. 445, Prone)

#### Scenario: Prone off-guard and the attack penalty

- **GIVEN** Posture::Prone for the attacker and OffGuard::Yes for the defender, with no cover
- **WHEN** the attack penalty and the Armor Class total are resolved for melee and for ranged
- **THEN** the attack total is minus 2
- **AND** each Armor Class total is minus 2

#### Scenario: Prone off-guard applies to melee and ranged

- **GIVEN** a prone defender who is not flanked
- **WHEN** off-guard is queried for melee and for ranged
- **THEN** both results are off-guard

#### Scenario: A standing attacker has no prone penalty

- **GIVEN** Posture::Standing
- **WHEN** the attack penalty is resolved
- **THEN** the attack total has no prone penalty

#### Scenario: Prone penalty and an untyped penalty total minus 7

- **GIVEN** Posture::Prone and a caller untyped penalty of minus 5 to the attack
- **WHEN** the prone penalty and that untyped penalty are totaled
- **THEN** the attack total is minus 7

### Requirement: Cover and off-guard both apply

A cover bonus and off-guard SHALL both apply. Standard cover and off-guard SHALL total 0. Greater cover and off-guard SHALL total plus 2. The cover used for an attack SHALL be the higher of the stored benefit and the caller cover for that attack, and it SHALL be passed once to the existing Armor Class modifiers.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2372 (Player Core pg. 424, Cover)

#### Scenario: Standard cover and off-guard total 0

- **GIVEN** caller standard cover, no stored benefit, and OffGuard::Yes
- **WHEN** Armor Class is totaled for melee and for ranged
- **THEN** both totals are 0

#### Scenario: Greater cover and off-guard total plus 2

- **GIVEN** caller greater cover, no stored benefit, and OffGuard::Yes
- **WHEN** Armor Class is totaled for melee and for ranged
- **THEN** both totals are plus 2

#### Scenario: Stored standard and caller greater total plus 4

- **GIVEN** stored standard cover, caller greater cover, and OffGuard::No
- **WHEN** Armor Class is totaled for melee and for ranged
- **THEN** both totals are plus 4
- **AND** the plus 4 is one cover bonus

### Requirement: One off-guard penalty is enough

Off-guard from two causes SHALL be one circumstance minus 2, passed as OffGuard. A circumstance penalty of minus 1 SHALL leave that total at minus 2. Grabbed is OffGuard::Yes, not a second penalty. A caller status of minus 4 plus OffGuard::Yes SHALL total minus 6, not minus 8.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2282 (Player Core pg. 400, Penalties)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=58 (Player Core pg. 445, Off-Guard)

#### Scenario: Off-guard and a circumstance penalty total minus 2

- **GIVEN** OffGuard::Yes and a caller circumstance penalty of minus 1 to Armor Class, with no cover
- **WHEN** Armor Class is totaled
- **THEN** the total is minus 2

#### Scenario: Unconscious status and off-guard total minus 6

- **GIVEN** OffGuard::Yes, no cover, and a caller status penalty of minus 4, with no second off-guard
- **WHEN** Armor Class is totaled
- **THEN** the total is minus 6
- **AND** the total is not minus 8

#### Scenario: Grabbed and flanked are off-guard once

- **GIVEN** Enemy id 1 at (0, 0) and Pc ids 2 and 3 at (0, 1) and (0, -1), all Medium, Able, Standing, and reach 5, and OffGuard::Yes for the other cause
- **WHEN** melee off-guard is queried and Armor Class is totaled once from that result
- **THEN** the query is off-guard
- **AND** the Armor Class total is minus 2

#### Scenario: Prone and flanking are off-guard once

- **GIVEN** prone Enemy id 1 at (0, 0) and Pc ids 2 and 3 at (0, 1) and (0, -1), all Medium, Able, and reach 5
- **WHEN** melee off-guard is queried and Armor Class is totaled once from that result
- **THEN** the query is off-guard
- **AND** the Armor Class total is minus 2

#### Scenario: A prone attacker and an off-guard defender

- **GIVEN** prone Pc id 2 at (0, 1), Pc id 3 at (0, -1), and standing Enemy id 1 at (0, 0), all Medium, Able, and reach 5
- **WHEN** attack_penalty is called with Posture::Prone and the defender's Armor Class is totaled once with OffGuard::Yes
- **THEN** the attack total is minus 2
- **AND** the Armor Class total is minus 2

### Requirement: A corner counts as both edges

A flanking line SHALL be the segment between two allies' square centers. A corner SHALL count as touching both edges that meet there (DEV-23). The line SHALL flank when those edges include an opposite pair, or when it passes through two opposite corners. Otherwise it SHALL NOT flank. There SHALL be no not-yet-ruled result.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2375 (Player Core pg. 425, Flanking)

#### Scenario: Opposite sides at reach 5

- **GIVEN** Enemy id 1 at (0, 0), Pc id 2 at (0, 1), and Pc id 3 at (0, -1), all Medium, Able, Standing, and reach 5
- **WHEN** flanking is resolved
- **THEN** the target is flanked

#### Scenario: Opposite corners at reach 5

- **GIVEN** Enemy id 1 at (0, 0), Pc id 2 at (1, 1), and Pc id 3 at (-1, -1), all Medium, Able, Standing, and reach 5
- **WHEN** flanking is resolved
- **THEN** the target is flanked

#### Scenario: The same side does not flank

- **GIVEN** Enemy id 1 at (0, 0), Pc id 2 at (-1, 1), and Pc id 3 at (0, 1), all Medium, Able, Standing, and reach 5
- **WHEN** flanking is resolved
- **THEN** the target is not flanked

#### Scenario: One corner does not flank

- **GIVEN** Enemy id 1 at (0, 0), Pc id 2 at (0, 1), and Pc id 3 at (1, 0), all Medium, Able, Standing, and reach 5
- **WHEN** flanking is resolved
- **THEN** the target is not flanked

#### Scenario: A corner counts as both edges

- **GIVEN** Enemy id 1 at (0, 0), Pc id 2 at (-2, -1) with reach 10, and Pc id 3 at (1, 0) with reach 5, all Medium, Able, and Standing
- **WHEN** flanking is resolved
- **THEN** the target is flanked

#### Scenario: Adjacent side interiors do not flank

- **GIVEN** Enemy id 1 at (0, 0), Pc id 2 at (-2, -2) with reach 10, and Pc id 3 at (0, 1) with reach 5, all Medium, Able, and Standing
- **WHEN** flanking is resolved
- **THEN** the target is not flanked

#### Scenario: Reach 10 opposite sides

- **GIVEN** Enemy id 1 at (0, 0), Pc id 2 at (0, 2) with reach 10, and Pc id 3 at (0, -1) with reach 5, all Medium, Able, and Standing
- **WHEN** flanking is resolved
- **THEN** the target is flanked

#### Scenario: Reach 10 opposite corners

- **GIVEN** Enemy id 1 at (0, 0), Pc id 2 at (2, 2), and Pc id 3 at (-2, -2), all Medium, Able, Standing, and reach 10
- **WHEN** flanking is resolved
- **THEN** the target is flanked

#### Scenario: Reach 10 off-axis opposite sides

- **GIVEN** Enemy id 1 at (0, 0), Pc id 2 at (-2, -1), and Pc id 3 at (2, 1), all Medium, Able, Standing, and reach 10
- **WHEN** flanking is resolved
- **THEN** the target is flanked

### Requirement: Squares have no map here

Positioning SHALL take raw squares and SHALL NOT take a map. A negative coordinate SHALL be legal. Centers SHALL use checked arithmetic. A value that does not fit SHALL be Overflow and SHALL NOT wrap.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2356 (Player Core pg. 421, Grid Movement)

#### Scenario: Negative squares are legal

- **GIVEN** no map, Enemy id 1 at (0, 0), Pc id 2 at (1, 0), and Pc id 3 at (-1, 0), all Medium, Able, Standing, and reach 5
- **WHEN** flanking is resolved
- **THEN** the target is flanked

#### Scenario: Extreme coordinates overflow

- **GIVEN** exactly two flankers, Enemy id 1 at (i32::MAX minus 1, 0), Pc id 2 at (i32::MAX, 0), and Pc id 3 at (i32::MAX minus 2, 0), all Medium, Able, Standing, and reach 5
- **WHEN** flanking is resolved
- **THEN** the result is Overflow

#### Scenario: Reach at the maximum u32 does not wrap

- **GIVEN** Enemy id 1 at (0, 0), Pc id 2 at (0, 1), and Pc id 3 at (0, -1), all Medium, Able, and Standing, both allies at reach u32::MAX
- **WHEN** flanking is resolved
- **THEN** the result is flanked
- **AND** the result is not a wrapped value

### Requirement: Threatening is reach and readiness

A creature SHALL threaten the target when the target is inside its supplied reach and its readiness is Able (DEV-25). Reach that is not a multiple of 5 SHALL be InvalidReach. Reach 0 SHALL be legal and SHALL NOT threaten. The reach query SHALL run from the flanker to the target with both creatures' real sizes.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2379 (Player Core pg. 426, Range and Reach)

#### Scenario: Out of reach does not flank

- **GIVEN** Enemy id 1 at (0, 0), Pc id 2 at (0, 1) with reach 5, and Pc id 3 at (0, -2) with reach 5, all Medium, Able, and Standing
- **WHEN** flanking is resolved
- **THEN** the target is not flanked

#### Scenario: A reach that is not a multiple of 5 is rejected

- **GIVEN** Enemy id 1 at (0, 0), Pc id 2 at (0, 1), and Pc id 3 at (0, -1), all Medium, Able, and Standing, both allies at reach 7
- **WHEN** flanking is resolved
- **THEN** the result is InvalidReach

#### Scenario: Reach 0 does not threaten

- **GIVEN** Enemy id 1 at (0, 0), Pc id 2 at (0, 1), and Pc id 3 at (0, -1), all Medium, Able, and Standing, both allies at reach 0
- **WHEN** flanking is resolved
- **THEN** the target is not flanked

#### Scenario: Reach 15 opposite corners flank

- **GIVEN** Enemy id 1 at (0, 0), Pc id 2 at (2, 2), and Pc id 3 at (-2, -2), all Medium, Able, Standing, and reach 15
- **WHEN** flanking is resolved
- **THEN** the target is flanked

#### Scenario: Reach 15 far corners do not flank

- **GIVEN** Enemy id 1 at (0, 0), Pc id 2 at (3, 3), and Pc id 3 at (-3, -3), all Medium, Able, Standing, and reach 15
- **WHEN** flanking is resolved
- **THEN** the target is not flanked

### Requirement: Flanking errors have one order

The checks SHALL run in this order: DuplicateId, then SelfId, then UnsupportedSize, then Overflow, then InvalidReach, then the boolean. An earlier failure SHALL win.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2375 (Player Core pg. 425, Flanking)

#### Scenario: A duplicate id is rejected

- **GIVEN** Enemy id 1 at (0, 0) and two Pc entries that both use id 2, at (0, 1) and (0, -1), all Medium and reach 5
- **WHEN** flanking and, with the defender standing, off-guard are resolved
- **THEN** both results are DuplicateId

#### Scenario: The target cannot flank itself

- **GIVEN** Enemy id 1 at (0, 0) and a second entry with id 1 at (0, 1), plus Pc id 2 at (0, -1), all Medium and reach 5
- **WHEN** flanking is resolved
- **THEN** the result is SelfId

#### Scenario: Duplicate id wins over self id

- **GIVEN** Pc id 1 at (0, 0) and two Pc entries that both use id 1, at (0, 1) and (0, -1), all Able, Standing, Medium, and reach 5
- **WHEN** flanking is resolved
- **THEN** the result is DuplicateId

#### Scenario: Self id wins over unsupported size

- **GIVEN** Pc id 1 at (0, 0), a Large Pc id 1 at (0, 1), and Medium Pc id 2 at (0, -1), all Able, Standing, and reach 5
- **WHEN** flanking is resolved
- **THEN** the result is SelfId

#### Scenario: Unsupported size wins over overflow

- **GIVEN** a Large target at (i32::MAX minus 1, 0) and Medium flankers at (i32::MAX, 0) and (i32::MAX minus 2, 0), reach 5
- **WHEN** flanking is resolved
- **THEN** the result is UnsupportedSize

### Requirement: Only Small and Medium can flank

Tiny, Large, Huge, and Gargantuan SHALL return UnsupportedSize. That check SHALL run before readiness. A Large target, a Tiny flanker, and a Large flanker who is unable to act SHALL each return UnsupportedSize. Tiny at reach 0 SHALL be UnsupportedSize and SHALL NOT be a boolean (DEV-19).

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2359 (Player Core pg. 421, Size, Space, and Reach)

#### Scenario: Large is unsupported

- **GIVEN** Large Enemy id 1 at (0, 0), Pc id 2 at (0, 1), and Pc id 3 at (0, -1), the allies Medium, Able, Standing, and reach 5
- **WHEN** flanking is resolved
- **THEN** the result is UnsupportedSize

#### Scenario: A tiny flanker is unsupported

- **GIVEN** Medium Enemy id 1 at (0, 0), Tiny Pc id 2 at (0, 1) with reach 0, and Medium Pc id 3 at (0, -1) with reach 5, all Able and Standing
- **WHEN** flanking is resolved
- **THEN** the result is UnsupportedSize

#### Scenario: A large unable flanker is unsupported

- **GIVEN** Medium Enemy id 1 at (0, 0), Large Pc id 2 at (0, 1) who is Unable, and Medium Pc id 3 at (0, -1) who is Able, reach 5
- **WHEN** flanking is resolved
- **THEN** the result is UnsupportedSize

### Requirement: Allies are the same Side

Two creatures SHALL be allies if and only if they have the same Side (DEV-26). Side SHALL be Pc or Enemy from add-encounter-turns. This change SHALL NOT define a faction id and SHALL NOT read consent or charm. Flankers SHALL share a Side, and the target SHALL have the other Side.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2375 (Player Core pg. 425, Flanking)

#### Scenario: Different sides do not flank

- **GIVEN** Enemy id 1 at (0, 0), Pc id 2 at (0, 1), and Enemy id 3 at (0, -1), all Medium, Able, Standing, and reach 5
- **WHEN** flanking is resolved
- **THEN** the target is not flanked

#### Scenario: Same side as the target does not flank

- **GIVEN** Enemy id 1 at (0, 0), Enemy id 2 at (0, 1), and Enemy id 3 at (0, -1), all Medium, Able, Standing, and reach 5
- **WHEN** flanking is resolved
- **THEN** the target is not flanked

### Requirement: Who can be a flanker

The attacker SHALL be an argument, and the other creatures SHALL be the partner list. Partner order SHALL NOT change the result. An empty partner list SHALL NOT flank. Two partners on opposite sides SHALL NOT flank for an attacker who does not complete a pair. Unable SHALL NOT flank. Prone SHALL NOT clear Able. The same square SHALL NOT flank.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2375 (Player Core pg. 425, Flanking)

#### Scenario: Unable to act does not flank

- **GIVEN** Enemy id 1 at (0, 0), Pc id 2 at (0, 1) who is Unable, and Pc id 3 at (0, -1) who is Able, all Medium, Standing, and reach 5
- **WHEN** flanking is resolved
- **THEN** the target is not flanked

#### Scenario: Prone does not clear able to act

- **GIVEN** Enemy id 1 at (0, 0), prone Pc id 2 at (0, 1), and prone Pc id 3 at (0, -1), all Medium, Able, and reach 5
- **WHEN** flanking is resolved
- **THEN** the target is flanked

#### Scenario: A prone target can be flanked

- **GIVEN** prone Enemy id 1 at (0, 0), Pc id 2 at (0, 1), and Pc id 3 at (0, -1), all Medium, Able, and reach 5
- **WHEN** flanking is resolved
- **THEN** the target is flanked

#### Scenario: Ally order does not matter

- **GIVEN** Enemy id 1 at (0, 0), attacker Pc id 2 at (0, 1), and partners Pc id 3 at (0, -1) and Pc id 4 at (1, 1), all Medium, Able, Standing, and reach 5
- **WHEN** flanking is resolved with the partners in either order
- **THEN** both results are flanked

#### Scenario: An attacker missing from partners does not flank

- **GIVEN** Enemy id 1 at (0, 0) and attacker Pc id 2 at (0, 1), all Medium, Able, Standing, and reach 5, and no partner in the list
- **WHEN** flanking is resolved
- **THEN** the target is not flanked

#### Scenario: A bystander is not a flanker

- **GIVEN** Enemy id 1 at (0, 0), attacker Pc id 2 at (1, 0), and partners Pc id 3 at (0, 1) and Pc id 4 at (0, -1), all Medium, Able, Standing, and reach 5
- **WHEN** melee off-guard is queried
- **THEN** the target is not off-guard

#### Scenario: Two flankers on one square do not flank

- **GIVEN** Enemy id 1 at (0, 0), Pc id 2 at (0, 1), and Pc id 3 at (0, 1), all Medium, Able, Standing, and reach 5
- **WHEN** flanking is resolved
- **THEN** the target is not flanked

#### Scenario: A flanker on the target square does not flank

- **GIVEN** Enemy id 1 at (0, 0), Pc id 2 at (0, 1), and Pc id 3 at (0, 0), all Medium, Able, Standing, and reach 5
- **WHEN** flanking is resolved
- **THEN** the target is not flanked

### Requirement: Off-guard from flanking is melee only

A melee attack from a flanking attacker SHALL be off-guard. A ranged attack SHALL NOT be off-guard from flanking alone. One flanking partner SHALL be enough. When the defender is prone, off-guard SHALL be true for melee and ranged before partner errors are considered.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2375 (Player Core pg. 425, Flanking)

#### Scenario: Melee from a flanker is off-guard

- **GIVEN** Enemy id 1 at (0, 0) and Pc ids 2 and 3 at (0, 1) and (0, -1), all Medium, Able, Standing, and reach 5
- **WHEN** off-guard is queried for melee
- **THEN** the target is off-guard

#### Scenario: Ranged from a flanker is not off-guard

- **GIVEN** Enemy id 1 at (0, 0) and Pc ids 2 and 3 at (0, 1) and (0, -1), all Medium, Able, Standing, and reach 5
- **WHEN** off-guard is queried for ranged
- **THEN** the target is not off-guard

#### Scenario: One flanking partner is enough

- **GIVEN** Enemy id 1 at (0, 0), attacker Pc id 2 at (0, 1), and partners Pc id 3 at (0, -1) and Pc id 4 at (1, 1), all Medium, Able, Standing, and reach 5
- **WHEN** off-guard is queried for melee
- **THEN** the target is off-guard

#### Scenario: Prone off-guard does not wait on partner geometry

- **GIVEN** prone Medium Enemy id 1 at (0, 0), attacker Pc id 3 at (0, -1), Medium, Able, Standing, and reach 5, and Large Pc id 2 at (0, 1) in the partner list
- **WHEN** off-guard is queried for melee and for ranged
- **THEN** both results are Ok and off-guard
- **AND** neither result is UnsupportedSize

### Requirement: Flanking ignores facing and the ground

Flanking SHALL NOT read facing. Terrain and other creatures on the line SHALL NOT change the result. Cover SHALL NOT be a flanking input.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2375 (Player Core pg. 425, Flanking)

#### Scenario: Terrain and creatures do not block flanking

- **GIVEN** Enemy id 1 at (0, 0), attacker Pc id 2 at (0, 2) with reach 10, and partner Pc id 3 at (0, -1) with reach 5, all three Medium, Able, and Standing. Difficult terrain at (0, 1) and an Enemy at (1, 1) are narrative only
- **WHEN** flanks is called for defender 1, attacker 2, and partners containing 3
- **THEN** the result is Ok and the target is flanked

### Requirement: Take Cover upgrades cover when not prone

When the creature is not prone, standard cover SHALL become greater. Greater cover SHALL stay greater. Lesser cover alone SHALL return RequirementNotMet, store nothing, keep the actions, and leave Armor Class at plus 1. Lesser cover plus a nearby feature SHALL become standard. A nearby feature with no cover SHALL become standard. No cover and no feature SHALL return RequirementNotMet.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2307 (Player Core pg. 418, Take Cover)

#### Scenario: Standard cover becomes greater

- **GIVEN** a standing creature with caller standard cover and no stored benefit
- **WHEN** Take Cover succeeds
- **THEN** the stored benefit is greater
- **AND** the Armor Class total is plus 4
- **AND** that plus 4 is one cover bonus

#### Scenario: Lesser cover alone is not enough

- **GIVEN** a standing creature with caller lesser cover, no nearby feature, no stored benefit, and a combatant state with 2 actions remaining
- **WHEN** Take Cover is attempted without that state as an argument
- **THEN** the result is RequirementNotMet
- **AND** nothing is stored
- **AND** the Armor Class total is plus 1
- **AND** the state still has 2 actions remaining

#### Scenario: Lesser cover and a feature become standard

- **GIVEN** a standing creature with caller lesser cover and a nearby feature
- **WHEN** Take Cover succeeds
- **THEN** the stored benefit is standard
- **AND** the Armor Class total is plus 2

#### Scenario: A nearby feature grants standard cover

- **GIVEN** a standing creature with no cover degree and a nearby feature
- **WHEN** Take Cover succeeds
- **THEN** the stored benefit is standard
- **AND** the Armor Class total is plus 2

#### Scenario: No cover and not prone

- **GIVEN** a standing creature with no cover degree and no nearby feature
- **WHEN** Take Cover is attempted
- **THEN** the result is RequirementNotMet
- **AND** nothing is stored

#### Scenario: Greater cover is not reduced

- **GIVEN** a standing creature with caller greater cover
- **WHEN** Take Cover succeeds
- **THEN** the stored benefit is greater
- **AND** the Armor Class total is plus 4

### Requirement: Take Cover while prone stacks

While prone, Take Cover SHALL stack with the cover the caller already has (DEV-24). For each attack the circumstance bonus SHALL be the higher of the two. The totals below include the prone minus 2. No prone case in this list SHALL return RequirementNotMet.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2307 (Player Core pg. 418, Take Cover)

#### Scenario: Prone with standard cover totals plus 2

- **GIVEN** a prone creature with caller standard cover and no nearby feature
- **WHEN** Take Cover succeeds and Armor Class is totaled for melee and ranged
- **THEN** both totals are plus 2

#### Scenario: Prone with greater cover totals plus 2

- **GIVEN** a prone creature with caller greater cover
- **WHEN** Take Cover succeeds and Armor Class is totaled for melee and ranged
- **THEN** both totals are plus 2

#### Scenario: Prone with a feature and no cover totals 0 and plus 2

- **GIVEN** a prone creature with no cover degree and a nearby feature
- **WHEN** Take Cover succeeds and Armor Class is totaled
- **THEN** the melee total is 0 and the ranged total is plus 2

#### Scenario: Prone with lesser cover totals minus 1 and plus 2

- **GIVEN** a prone creature with caller lesser cover and no nearby feature
- **WHEN** Take Cover succeeds and Armor Class is totaled
- **THEN** the melee total is minus 1 and the ranged total is plus 2

#### Scenario: Prone with lesser cover and a feature totals 0 and plus 2

- **GIVEN** a prone creature with caller lesser cover and a nearby feature
- **WHEN** Take Cover succeeds and Armor Class is totaled
- **THEN** the melee total is 0 and the ranged total is plus 2

#### Scenario: Prone with no cover totals minus 2 and plus 2

- **GIVEN** a prone creature with no cover degree and no nearby feature
- **WHEN** Take Cover succeeds and Armor Class is totaled
- **THEN** the melee total is minus 2 and the ranged total is plus 2

### Requirement: Repeating Take Cover keeps the better benefit

Take Cover SHALL replace the stored benefit with the better bonus for each attack. Stored greater, then Take Cover with caller standard, SHALL stay greater at plus 4. RequirementNotMet SHALL NOT replace the stored benefit. Dismissing when nothing is stored SHALL return Ok with no benefit.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2307 (Player Core pg. 418, Take Cover)

#### Scenario: Stored greater survives a later standard

- **GIVEN** a standing creature with stored greater cover and caller standard cover
- **WHEN** Take Cover succeeds
- **THEN** the stored benefit is still greater
- **AND** the Armor Class total is plus 4

#### Scenario: A failed requirement keeps the stored benefit

- **GIVEN** a standing creature with stored greater cover, caller lesser cover, and no nearby feature
- **WHEN** Take Cover is attempted
- **THEN** the result is RequirementNotMet
- **AND** the stored benefit is still greater

#### Scenario: Dismissing nothing stores nothing

- **GIVEN** no stored benefit
- **WHEN** the benefit is dismissed
- **THEN** the result is Ok and no benefit is stored

### Requirement: Take Cover has a cost and an end

Take Cover SHALL cost 1 action and SHALL NOT have the move trait. Mobility::Immobilized SHALL NOT block it. Dismiss SHALL be free and SHALL clear a stored benefit. Leaving the square, an attack, unconscious, dismiss, and forced movement SHALL clear the benefit and SHALL NOT clear prone. Drop Prone SHALL NOT clear it. Stand follows R-11.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2307 (Player Core pg. 418, Take Cover)

#### Scenario: Take Cover is one action without the move trait

- **GIVEN** a standing creature with Mobility::Immobilized and caller standard cover
- **WHEN** Take Cover succeeds
- **THEN** the cost is 1 action and the trait list is empty
- **AND** the stored benefit is greater

#### Scenario: Dismiss is free

- **GIVEN** a stored greater benefit
- **WHEN** the benefit is dismissed
- **THEN** the cost is a free action
- **AND** no benefit remains

#### Scenario: Leaving the square clears the benefit

- **GIVEN** a prone creature with a stored benefit and caller standard cover underneath
- **WHEN** the benefit ends because the creature left the square
- **THEN** the benefit is gone and the posture is still prone
- **AND** Armor Class with caller standard cover and OffGuard::Yes is 0

#### Scenario: An attack action clears the benefit

- **GIVEN** a prone creature with a stored benefit
- **WHEN** the benefit ends because the creature used an attack action
- **THEN** the benefit is gone and the posture is still prone

#### Scenario: Unconscious clears the benefit only

- **GIVEN** a prone creature with a stored benefit
- **WHEN** the benefit ends because the creature became unconscious
- **THEN** the benefit is gone and the posture is still prone
- **AND** Armor Class with no cover and OffGuard::Yes is minus 2

#### Scenario: Forced movement clears the benefit

- **GIVEN** a prone creature with a stored benefit
- **WHEN** the benefit ends because the creature was moved by force
- **THEN** the benefit is gone and the posture is still prone

### Requirement: Stand drops only the prone part of Take Cover

Stand SHALL drop only the prone-only part of Take Cover (R-11 extends DEV-24; docs/designer/rules-rulings.md on main). StandardPlusGreaterVsRanged SHALL become Standard at plus 2, and prone off-guard SHALL end. GreaterVsRanged alone SHALL end. Greater from standard cover SHALL stay Greater at plus 4.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2307 (Player Core pg. 418, Take Cover)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=88 (Player Core pg. 445, Prone)

#### Scenario: Stand turns standard plus greater against ranged into standard

- **GIVEN** a prone creature with Mobility::Free and stored StandardPlusGreaterVsRanged
- **WHEN** Stand succeeds
- **THEN** the posture is standing and the benefit is Standard
- **AND** Armor Class with OffGuard::No is plus 2 for melee and for ranged

#### Scenario: Stand ends a greater against ranged benefit

- **GIVEN** a prone creature with Mobility::Free, stored GreaterVsRanged, no nearby feature, and no caller cover
- **WHEN** Stand succeeds
- **THEN** the benefit is gone
- **AND** Armor Class with OffGuard::No and no cover is 0

#### Scenario: Stand keeps greater cover

- **GIVEN** a prone creature with Mobility::Free and stored Greater from standard cover
- **WHEN** Stand succeeds
- **THEN** the benefit is still Greater
- **AND** Armor Class with OffGuard::No is plus 4 for melee and for ranged

### Requirement: This change does not invent the deferred rules

A shield SHALL leave greater cover at plus 4 (change 7). Greater cover, OffGuard::No, ranged SHALL be plus 4 as one cover entry. A creature two sizes larger SHALL NOT be cover. Hidden SHALL NOT be an input (FIX-14). Overflow SHALL NOT wrap.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2316 (Player Core pg. 419, Raise a Shield)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2372 (Player Core pg. 424, Cover)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2373 (Player Core pg. 424, Cover)

#### Scenario: Greater cover does not add a shield

- **GIVEN** greater cover and no shield argument
- **WHEN** Armor Class is totaled
- **THEN** the total is plus 4
- **AND** the total is not plus 6

#### Scenario: Cover is an Armor Class bonus only

- **GIVEN** no stored benefit, caller greater cover, OffGuard::No, a ranged attack, and no extra modifiers
- **WHEN** armor_class is called with those arguments
- **THEN** the total is plus 4
- **AND** exactly one circumstance cover entry was applied

#### Scenario: A larger neighbour is not cover

- **GIVEN** a standing Medium creature at (0, 0), a Large creature at (1, 0), no cover degree, and no nearby feature
- **WHEN** Take Cover is attempted
- **THEN** the result is RequirementNotMet

#### Scenario: A standing defender with no flank is not off-guard

- **GIVEN** standing Enemy id 1 at (0, 0) and Pc id 2 at (0, 1), both Medium, Able, and reach 5, with no partner and OffGuard::No
- **WHEN** melee off-guard is queried
- **THEN** the target is not off-guard

#### Scenario: An Armor Class overflow does not wrap

- **GIVEN** off-guard and a caller untyped Armor Class penalty of the minimum i32
- **WHEN** Armor Class is totaled
- **THEN** the result is the modifier overflow
- **AND** the result is not a wrapped total
