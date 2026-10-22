# positioning

## Purpose

Decides prone posture, flanking on a 5-foot grid, off-guard, and Take Cover, and folds those bonuses and penalties through the existing modifier total.

## ADDED Requirements

### Requirement: Positioning is pure

The same positioning inputs SHALL return the same result. A call SHALL NOT roll dice, read a clock, or read a file. A fallible call SHALL return a typed error and SHALL NOT panic. The rules crate SHALL gain no dependency.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2356 (Player Core pg. 421, Grid Movement)

#### Scenario: The same flanking query twice matches

- **GIVEN** a Medium target at (0, 0) and two Medium flankers at (0, 1) and (0, -1), both reach 5, both able to act, sharing a faction that is not the target's, and a scripted d20 holding 8
- **WHEN** flanking is resolved twice
- **THEN** the two results are equal and both say the target is flanked
- **AND** the scripted 8 is still unused

### Requirement: Posture is caller state

Prone and a Take Cover benefit SHALL be values the caller keeps. They SHALL NOT be fields of the creature or of combatant state. A positioning call SHALL NOT change Hit Points, actions remaining, or whether the reaction is ready.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=88 (Player Core pg. 445, Prone)

#### Scenario: Drop Prone leaves the combatant state

- **GIVEN** a standing creature that is not immobilized, a combatant state with 2 actions remaining and the reaction not ready, and square (3, 4)
- **WHEN** Drop Prone succeeds without that state as an argument
- **THEN** the returned posture is prone on square (3, 4)
- **AND** the state still has 2 actions remaining and the reaction is still not ready

### Requirement: Drop Prone names its cost

Drop Prone SHALL name a cost of 1 action with the move trait. It SHALL NOT spend that action and SHALL NOT resolve a reaction.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2295 (Player Core pg. 416, Drop Prone)

#### Scenario: Drop Prone reports one move action

- **GIVEN** a standing creature that is not immobilized
- **WHEN** Drop Prone succeeds
- **THEN** the result names 1 action with the move trait
- **AND** no strike is resolved

### Requirement: Stand names its cost

Stand SHALL name a cost of 1 action with the move trait. It SHALL NOT spend that action and SHALL NOT resolve a reaction. On success the creature SHALL be standing.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2303 (Player Core pg. 418, Stand)

#### Scenario: Stand reports one move action

- **GIVEN** a prone creature that is not immobilized, and a reaction that is ready
- **WHEN** Stand succeeds
- **THEN** the posture is standing
- **AND** the result names 1 action with the move trait
- **AND** no strike is resolved and the reaction is still ready

### Requirement: Immobilized blocks Drop Prone and Stand

An immobilized creature SHALL NOT Drop Prone or Stand. The posture and square SHALL stay as given. Take Cover SHALL still be legal.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=81 (Player Core pg. 444, Immobilized)
**Source**: https://2e.aonprd.com/Actions.aspx?ID=2295 (Player Core pg. 416, Drop Prone)
**Source**: https://2e.aonprd.com/Actions.aspx?ID=2303 (Player Core pg. 418, Stand)

#### Scenario: An immobilized creature cannot Drop Prone or Stand

- **GIVEN** an immobilized standing creature on (1, 1), and separately an immobilized prone creature on (2, 2)
- **WHEN** the standing creature Drops Prone and the prone creature Stands
- **THEN** both results are an immobilized error
- **AND** the standing creature is still standing on (1, 1)
- **AND** the prone creature is still prone on (2, 2)

### Requirement: Stand requires prone

A creature that is not prone SHALL NOT Stand. The result SHALL be a not-prone error and the posture SHALL stay standing. When the creature is also immobilized, the result SHALL be the immobilized error.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2303 (Player Core pg. 418, Stand)

#### Scenario: Standing cannot Stand

- **GIVEN** a standing creature that is not immobilized
- **WHEN** it Stands
- **THEN** the result is a not-prone error and the posture stays standing

#### Scenario: Immobilized wins over not prone

- **GIVEN** an immobilized standing creature
- **WHEN** it Stands
- **THEN** the result is an immobilized error
- **AND** the result is not a not-prone error

### Requirement: Prone limits other moves

While a creature is prone, Stand SHALL be the only successful move. Drop Prone, Stride, and Step SHALL each return a prone-movement error and SHALL leave the posture prone. This capability SHALL NOT return a Crawl distance or a Leap distance.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=88 (Player Core pg. 445, Prone)
**Source**: https://2e.aonprd.com/Actions.aspx?ID=2293 (Player Core pg. 416, Crawl)
**Source**: https://2e.aonprd.com/Actions.aspx?ID=2298 (Player Core pg. 417, Leap)

#### Scenario: Only Stand is legal while prone

- **GIVEN** a prone creature that is not immobilized, on square (4, 5)
- **WHEN** it Stands, and separately when it Drops Prone, Strides, or Steps
- **THEN** Stand succeeds and the posture is standing
- **AND** Drop Prone, Stride, and Step each return a prone-movement error
- **AND** those three leave the posture prone on (4, 5)

#### Scenario: Crawl and Leap are not resolved

- **GIVEN** a prone creature with Speed 25
- **WHEN** its legal moves are resolved
- **THEN** no result is a Crawl distance or a Leap distance

### Requirement: Drop Prone keeps the square

Drop Prone on a standing creature that is not immobilized SHALL make it prone on the same square. A Take Cover benefit already stored SHALL remain stored.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2295 (Player Core pg. 416, Drop Prone)

#### Scenario: Drop Prone keeps the square and the cover benefit

- **GIVEN** a standing creature on (6, 1) with greater cover already stored by Take Cover
- **WHEN** Drop Prone succeeds
- **THEN** the posture is prone on (6, 1)
- **AND** the stored benefit is still greater cover

### Requirement: Prone applies off-guard and the attack penalty

A prone creature SHALL be off-guard to melee and ranged attacks. Off-guard SHALL be one circumstance penalty of -2 to Armor Class. Its attack roll SHALL take one circumstance penalty of -2. Both come from the existing modifiers. Prone alone SHALL grant no cover. Positioning SHALL NOT add the unconscious status penalty.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=88 (Player Core pg. 445, Prone)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=58 (Player Core pg. 445, Off-Guard)

#### Scenario: Prone off-guard and the attack penalty

- **GIVEN** a prone creature, no cover, and no flanking
- **WHEN** Armor Class modifiers and attack modifiers are totaled separately
- **THEN** Armor Class includes one circumstance penalty of -2 and no cover bonus
- **AND** the attack roll includes one circumstance penalty of -2
- **AND** neither total includes a status penalty of -4

#### Scenario: Prone off-guard does not wait on an unruled pair

- **GIVEN** a prone target at (0, 0), a creature at (-2, -1) with reach 10, and a creature at (1, 0) with reach 5, both able to act, sharing a faction that is not the target's
- **WHEN** off-guard is asked for either creature's melee attack
- **THEN** the answer is off-guard
- **AND** the result is not a not-yet-ruled error

#### Scenario: Prone off-guard applies to melee and ranged

- **GIVEN** a prone creature that is not flanked
- **WHEN** off-guard is asked for a melee attack and for a ranged attack
- **THEN** both answers are off-guard

### Requirement: The prone penalty stacks as circumstance

The prone attack penalty SHALL remain circumstance when an untyped penalty from another source is totaled with it. The existing total SHALL add that untyped penalty.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2282 (Player Core pg. 400, Penalties)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=88 (Player Core pg. 445, Prone)

#### Scenario: Prone -2 and an untyped -5 total -7

- **GIVEN** the prone attack penalty and an untyped penalty of -5 from another source
- **WHEN** the attack roll is totaled
- **THEN** the total is -7
- **AND** the circumstance penalty of -2 is applied
- **AND** the untyped penalty of -5 is applied

### Requirement: Cover and off-guard both apply

On Armor Class, a cover circumstance bonus and the off-guard circumstance penalty SHALL both apply through the existing total. Off-guard from prone and from flanking together SHALL be one circumstance penalty of -2.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2281 (Player Core pg. 400, Bonuses)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2282 (Player Core pg. 400, Penalties)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=58 (Player Core pg. 445, Off-Guard)

#### Scenario: Standard cover and off-guard total 0

- **GIVEN** standard cover and off-guard, and no other Armor Class modifier from positioning
- **WHEN** Armor Class modifiers are totaled
- **THEN** the circumstance bonus of +2 is applied
- **AND** the circumstance penalty of -2 is applied
- **AND** the Armor Class total is 0

#### Scenario: Greater cover and off-guard total +2

- **GIVEN** greater cover and off-guard
- **WHEN** Armor Class modifiers are totaled
- **THEN** the Armor Class total is +2

#### Scenario: Prone and flanking are off-guard once

- **GIVEN** a prone target flanked for a melee attack
- **WHEN** Armor Class modifiers for that attack are totaled
- **THEN** one circumstance penalty of -2 is applied
- **AND** the penalty is not -4

### Requirement: Attacker and defender stay apart

The prone penalty on an attack roll and off-guard on a defender's Armor Class SHALL be totaled separately. Neither total SHALL drop the other's penalty.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=88 (Player Core pg. 445, Prone)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=58 (Player Core pg. 445, Off-Guard)

#### Scenario: A prone attacker and an off-guard defender

- **GIVEN** a prone attacker and a defender who is off-guard to that attacker's melee attack
- **WHEN** the attack roll and the Armor Class are totaled separately
- **THEN** the attack roll includes -2 circumstance
- **AND** the Armor Class includes -2 circumstance
- **AND** neither total drops the other's penalty

### Requirement: Flanking geometry uses the center line

For Small and Medium squares, flanking geometry SHALL hold when the line between the flankers' square centers meets the target square in opposite side interiors or in one pair of opposite corners. A miss, a single point, or two adjacent side interiors SHALL NOT hold. Any other hit SHALL return a not-yet-ruled error and SHALL NOT report flanking.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2375 (Player Core pg. 425, Flanking)

#### Scenario: Opposite sides at reach 5

- **GIVEN** a Medium target at (0, 0) and Medium flankers at (0, 1) and (0, -1), both reach 5, both able to act, sharing a faction that is not the target's
- **WHEN** flanking is resolved
- **THEN** the target is flanked
- **AND** flankers at (1, 0) and (-1, 0) with the same reach, faction, and able-to-act flags also flank it

#### Scenario: Opposite corners at reach 5

- **GIVEN** a Medium target at (0, 0) and Medium flankers at (1, 1) and (-1, -1), both reach 5, both able to act, sharing a faction that is not the target's
- **WHEN** flanking is resolved
- **THEN** the target is flanked
- **AND** flankers at (-1, 1) and (1, -1) with the same reach, faction, and able-to-act flags also flank it

#### Scenario: The same side does not flank

- **GIVEN** a target at (0, 0) and flankers at (-1, 1) and (0, 1), both reach 5, both able to act, sharing a faction that is not the target's
- **WHEN** flanking is resolved
- **THEN** the target is not flanked

#### Scenario: Adjacent sides do not flank

- **GIVEN** a target at (0, 0) and flankers at (0, 1) and (1, 0), both reach 5, both able to act, sharing a faction that is not the target's
- **WHEN** flanking is resolved
- **THEN** the target is not flanked

#### Scenario: Reach 10 opposite sides

- **GIVEN** a target at (0, 0), a flanker at (0, 2) with reach 10, and a flanker at (0, -1) with reach 5, both able to act, sharing a faction that is not the target's
- **WHEN** flanking is resolved
- **THEN** the target is flanked

#### Scenario: Reach 10 opposite corners

- **GIVEN** a target at (0, 0) and flankers at (2, 2) and (-2, -2), both reach 10, both able to act, sharing a faction that is not the target's
- **WHEN** flanking is resolved
- **THEN** the target is flanked

#### Scenario: Reach 10 off-axis opposite sides

- **GIVEN** a target at (0, 0) and flankers at (-2, -1) and (2, 1), both reach 10, both able to act, sharing a faction that is not the target's
- **WHEN** flanking is resolved
- **THEN** the target is flanked

#### Scenario: Adjacent side interiors do not flank

- **GIVEN** a target at (0, 0) and flankers at (-2, -2) and (0, 1), both reach 10, both able to act, sharing a faction that is not the target's
- **WHEN** flanking is resolved
- **THEN** the target is not flanked

#### Scenario: A corner and a side are not yet ruled

- **GIVEN** a target at (0, 0), a flanker at (-2, -1) with reach 10, and a flanker at (1, 0) with reach 5, both able to act, sharing a faction that is not the target's
- **WHEN** flanking is resolved
- **THEN** the result is a not-yet-ruled error
- **AND** the result does not say the target is flanked or not flanked

### Requirement: Flanking needs reach and an action

Each flanker SHALL have the target inside melee reach by the existing reach query, and SHALL be marked able to act. Otherwise the pair SHALL NOT flank. The same square SHALL NOT count as reach. A prone flanker in reach who can act SHALL still flank. A prone target SHALL still be flankable.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2375 (Player Core pg. 425, Flanking)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2379 (Player Core pg. 426, Range and Reach)

#### Scenario: Out of reach does not flank

- **GIVEN** a target at (0, 0) and flankers at (0, 1) and (0, -2), both reach 5, both able to act, sharing a faction that is not the target's
- **WHEN** flanking is resolved
- **THEN** the target is not flanked

#### Scenario: Unable to act does not flank

- **GIVEN** flankers at (0, 1) and (0, -1), both reach 5, sharing a faction that is not the target's, and one marked unable to act
- **WHEN** flanking is resolved
- **THEN** the target is not flanked

#### Scenario: A prone flanker still flanks

- **GIVEN** flankers at (1, 0) and (-1, 0), both reach 5 and able to act, sharing a faction that is not the target's, and one of them prone
- **WHEN** flanking is resolved
- **THEN** the target is flanked

#### Scenario: A prone target can be flanked

- **GIVEN** a prone target at (0, 0) and flankers at (0, 1) and (0, -1), both reach 5 and able to act, sharing a faction that is not the target's
- **WHEN** flanking is resolved
- **THEN** the target is flanked

#### Scenario: The same square is not reach

- **GIVEN** one flanker on the target's square (0, 0) and another at (0, 1), both reach 5 and able to act, sharing a faction that is not the target's
- **WHEN** flanking is resolved
- **THEN** the target is not flanked

### Requirement: Able to act is caller data

The caller SHALL pass whether each flanker can act. This capability SHALL NOT infer that flag, and prone SHALL NOT clear it.

**Rule**: Player Core pg. 425 requires both flankers able to act. https://2e.aonprd.com/Rules.aspx?ID=2375
**Why**: Unconscious belongs to a later change.
**Substitute**: The caller passes the flag. Prone leaves it as passed.
**Intent**: A later change can pass false.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2375 (Player Core pg. 425, Flanking)

#### Scenario: Prone does not clear able to act

- **GIVEN** a prone flanker marked able to act, opposite an ally who is also able to act and in reach
- **WHEN** flanking is resolved
- **THEN** the able-to-act flag is still true
- **AND** the target is flanked

### Requirement: Flankers are allies against an enemy

The flankers SHALL share one faction, and that faction SHALL differ from the target's. Otherwise they SHALL NOT flank.

**Rule**: Player Core pg. 425 requires an ally against an enemy. https://2e.aonprd.com/Rules.aspx?ID=2375
**Why**: No ally flag exists besides faction.
**Substitute**: A shared faction counts as allies. The target's faction differs.
**Intent**: Enemies do not flank each other, and friends do not flank a friend.

#### Scenario: Different factions do not flank

- **GIVEN** flankers at (0, 1) and (0, -1), both reach 5 and able to act, with factions that differ from each other
- **WHEN** flanking is resolved
- **THEN** the target is not flanked

#### Scenario: Allies of the target do not flank it

- **GIVEN** flankers at (1, 0) and (-1, 0), both reach 5 and able to act, sharing the target's faction
- **WHEN** flanking is resolved
- **THEN** the target is not flanked

### Requirement: Flanking off-guard is melee from the pair

A flanked target SHALL be off-guard to each flanker's melee attacks and SHALL NOT be off-guard from flanking to a ranged attack or to a creature outside a flanking pair. Prone off-guard SHALL still apply when flanking does not. A not-yet-ruled pair SHALL return that error instead of a boolean.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2375 (Player Core pg. 425, Flanking)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=58 (Player Core pg. 445, Off-Guard)

#### Scenario: Melee from a flanker is off-guard

- **GIVEN** a target at (0, 0) that is not prone, and flankers at (0, 1) and (0, -1), both reach 5 and able to act, sharing a faction that is not the target's
- **WHEN** off-guard is asked for each flanker's melee attack
- **THEN** both answers are off-guard

#### Scenario: A flanker's ranged attack is not off-guard

- **GIVEN** that same pair and a target that is not prone
- **WHEN** off-guard is asked for a flanker's ranged attack
- **THEN** the answer is not off-guard

#### Scenario: A bystander is not a flanker

- **GIVEN** those two flankers and a third creature at (1, 1), reach 5, able to act, and of their faction
- **WHEN** off-guard is asked for the third creature's melee attack
- **THEN** the answer is not off-guard

#### Scenario: A clear pair beats an unruled pair

- **GIVEN** a target at (0, 0) that is not prone, an attacker at (1, 0) with reach 5, an ally at (-1, 0) with reach 5, and an ally at (-2, -1) with reach 10, all able to act and sharing a faction that is not the target's
- **WHEN** off-guard is asked for the attacker's melee attack
- **THEN** the answer is off-guard
- **AND** the result is not a not-yet-ruled error

#### Scenario: An unruled pair does not guess off-guard

- **GIVEN** a target at (0, 0) that is not prone, a flanker at (-2, -1) with reach 10, and a flanker at (1, 0) with reach 5, both able to act, sharing a faction that is not the target's
- **WHEN** off-guard is asked for either flanker's melee attack
- **THEN** the result is a not-yet-ruled error
- **AND** the result does not say the target is off-guard or not

#### Scenario: Prone still applies when the attack is ranged

- **GIVEN** a prone target and a flanker pair at (0, 1) and (0, -1)
- **WHEN** off-guard is asked for a flanker's ranged attack
- **THEN** the answer is off-guard

### Requirement: Only Small and Medium flank

A creature that is not Small or Medium SHALL return the existing unsupported-size error. The result SHALL NOT say whether anyone flanks. Coordinate overflow from the reach query SHALL return an overflow error and SHALL NOT return a wrapped boolean.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2359 (Player Core pg. 421, Size, Space, and Reach)

#### Scenario: Large is unsupported

- **GIVEN** a Large flanker opposite a Medium target, both otherwise able to flank
- **WHEN** flanking is resolved
- **THEN** the result is an unsupported-size error
- **AND** the result does not say the target is flanked

#### Scenario: Extreme coordinates overflow

- **GIVEN** a Medium target and a Medium flanker on the extreme positive and extreme negative squares these rules store, with a third Medium flanker at a neighbor of the target
- **WHEN** flanking is resolved
- **THEN** the result is an overflow error
- **AND** the result is not a flanking boolean

### Requirement: Creatures are combatant ids

A target and each flanker SHALL be a CombatantId. Two entries with one id, or a flanker id equal to the target id, SHALL return a typed error. List position SHALL NOT identify them.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2375 (Player Core pg. 425, Flanking)

#### Scenario: A duplicate id is rejected

- **GIVEN** two flanker entries with one CombatantId, on (0, 1) and (0, -1)
- **WHEN** flanking is resolved
- **THEN** the result is a duplicate-id error
- **AND** the result does not say the target is flanked

#### Scenario: The target cannot flank itself

- **GIVEN** a flanker whose CombatantId is the target's, and another flanker at (0, 1)
- **WHEN** flanking is resolved
- **THEN** the result is a self-id error
- **AND** the target is not reported as flanked

### Requirement: Take Cover upgrades standard cover

When the creature is not prone and the caller supplies standard cover, Take Cover SHALL store greater cover. Armor Class SHALL show one circumstance bonus of +4 from the existing cover modifier, and SHALL NOT also keep +2.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2307 (Player Core pg. 418, Take Cover)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2372 (Player Core pg. 424, Cover)

#### Scenario: Standard cover becomes greater

- **GIVEN** a standing creature with standard cover and no Take Cover benefit yet
- **WHEN** Take Cover succeeds and Armor Class is totaled
- **THEN** the stored benefit is greater cover
- **AND** one circumstance bonus of +4 is applied
- **AND** the Armor Class total is +4

### Requirement: Take Cover can grant standard cover

When the creature is not prone, and the caller supplies lesser cover or marks a nearby feature with no cover degree, Take Cover SHALL store standard cover. The Armor Class bonus SHALL be +2 circumstance. The nearby feature SHALL be caller data. This capability SHALL NOT test walls.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2307 (Player Core pg. 418, Take Cover)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2372 (Player Core pg. 424, Cover)

#### Scenario: Lesser cover becomes standard

- **GIVEN** a standing creature with lesser cover and no nearby-feature flag
- **WHEN** Take Cover succeeds and Armor Class is totaled
- **THEN** the stored benefit is standard cover
- **AND** the circumstance bonus is +2

#### Scenario: A nearby feature grants standard cover

- **GIVEN** a standing creature with no cover degree and a nearby feature marked
- **WHEN** Take Cover succeeds and Armor Class is totaled
- **THEN** the stored benefit is standard cover
- **AND** the circumstance bonus is +2
- **AND** no wall geometry was required

### Requirement: Take Cover needs a requirement

When the creature is not prone, has no cover degree, and has no nearby feature, Take Cover SHALL return a requirement error and SHALL store no benefit. Actions remaining SHALL stay unchanged.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2307 (Player Core pg. 418, Take Cover)

#### Scenario: No cover and not prone

- **GIVEN** a standing creature with no cover, no nearby feature, and 3 actions remaining
- **WHEN** Take Cover is attempted
- **THEN** the result is a requirement error
- **AND** no cover benefit is stored
- **AND** 3 actions remain

### Requirement: Greater cover stays greater

When the creature is not prone and the caller supplies greater cover, Take Cover SHALL succeed and the stored benefit SHALL stay greater cover. The Armor Class bonus SHALL stay +4 circumstance.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2307 (Player Core pg. 418, Take Cover)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2372 (Player Core pg. 424, Cover)

#### Scenario: Greater cover is not reduced

- **GIVEN** a standing creature with greater cover
- **WHEN** Take Cover succeeds and Armor Class is totaled
- **THEN** the stored benefit is greater cover
- **AND** the circumstance bonus is +4

### Requirement: Prone Take Cover is ranged only

When the creature is prone, has no cover degree, and has no nearby feature, Take Cover SHALL grant greater cover against a ranged attack and no cover against a melee attack. Off-guard SHALL still apply to both. The ranged total SHALL be +2. The melee total SHALL be -2.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2307 (Player Core pg. 418, Take Cover)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=88 (Player Core pg. 445, Prone)

#### Scenario: Prone Take Cover against ranged

- **GIVEN** a prone creature with no cover and no nearby feature
- **WHEN** Take Cover succeeds and Armor Class is totaled for a ranged attack
- **THEN** the circumstance bonus of +4 is applied
- **AND** the circumstance penalty of -2 is applied
- **AND** the Armor Class total is +2

#### Scenario: Prone Take Cover against melee

- **GIVEN** that same stored benefit
- **WHEN** Armor Class is totaled for a melee attack
- **THEN** no cover bonus is applied
- **AND** the circumstance penalty of -2 is applied
- **AND** the Armor Class total is -2

### Requirement: Prone plus other cover is not yet ruled

When the creature is prone and the caller supplies a cover degree or a nearby feature, Take Cover SHALL return a not-yet-ruled error. The posture SHALL stay prone and no new benefit SHALL be stored.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2307 (Player Core pg. 418, Take Cover)

#### Scenario: Prone and standard cover are not ruled

- **GIVEN** a prone creature with standard cover and no stored Take Cover benefit
- **WHEN** Take Cover is attempted
- **THEN** the result is a not-yet-ruled error
- **AND** no benefit is stored
- **AND** the creature stays prone

#### Scenario: Prone and a nearby feature are not ruled

- **GIVEN** a prone creature with no cover degree and a nearby feature marked
- **WHEN** Take Cover is attempted
- **THEN** the result is a not-yet-ruled error
- **AND** the creature stays prone

### Requirement: Take Cover names its cost

Take Cover SHALL name a cost of 1 action and SHALL NOT carry the move trait. Dismissing the benefit SHALL name a free action. Neither SHALL spend actions or resolve a reaction. Immobilized SHALL NOT block Take Cover.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2307 (Player Core pg. 418, Take Cover)

#### Scenario: Take Cover is one action without the move trait

- **GIVEN** an immobilized prone creature with no cover, no nearby feature, and 2 actions remaining
- **WHEN** Take Cover succeeds
- **THEN** the result names 1 action and no move trait
- **AND** greater cover against ranged attacks is stored
- **AND** 2 actions remain

#### Scenario: Dismiss is free

- **GIVEN** a stored Take Cover benefit and 2 actions remaining
- **WHEN** the benefit is dismissed
- **THEN** the result names a free action
- **AND** the benefit is cleared
- **AND** 2 actions remain

### Requirement: Take Cover ends when the caller says so

The stored benefit SHALL end when the caller reports that the creature left its square, used an attack action, became unconscious, or dismissed the benefit. Ending SHALL NOT clear prone. Cover the caller still supplies SHALL remain. Drop Prone and Stand SHALL NOT end the benefit.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2307 (Player Core pg. 418, Take Cover)

#### Scenario: Leaving the square clears the benefit

- **GIVEN** greater cover stored by Take Cover over caller-supplied standard cover
- **WHEN** the caller reports the creature left its square and Armor Class is totaled
- **THEN** the Take Cover benefit is gone
- **AND** the circumstance bonus is +2

#### Scenario: An attack action clears the benefit

- **GIVEN** a stored Take Cover benefit
- **WHEN** the caller reports an attack action
- **THEN** the benefit is gone

#### Scenario: Unconscious clears the benefit only

- **GIVEN** a prone creature with greater cover against ranged attacks from Take Cover
- **WHEN** the caller reports unconscious
- **THEN** the benefit is gone
- **AND** the creature is still prone

#### Scenario: Stand keeps the cover benefit

- **GIVEN** a prone creature that is not immobilized, with greater cover against ranged attacks from Take Cover
- **WHEN** Stand succeeds
- **THEN** the posture is standing
- **AND** a ranged attack still has the +4 circumstance bonus
- **AND** a melee attack has no cover bonus from that benefit
- **AND** the creature is not off-guard from prone

### Requirement: Shields and hidden stay out

Positioning SHALL NOT apply a shield bonus. It SHALL NOT treat hidden, undetected, or concealed as off-guard. A creature that is not prone and not flanked SHALL NOT be off-guard.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=58 (Player Core pg. 445, Off-Guard)
**Source**: https://2e.aonprd.com/Actions.aspx?ID=2316 (Player Core pg. 419, Raise a Shield)

#### Scenario: No shield and no hidden off-guard

- **GIVEN** a standing creature who is not flanked, and a raised shield that would grant +2 Armor Class
- **WHEN** positioning totals Armor Class
- **THEN** the total is 0
- **AND** the shield bonus is not applied
- **AND** the creature is not off-guard

### Requirement: Squares are not shared

Drop Prone, Stand, and Take Cover SHALL keep the creature on its square. This capability SHALL NOT permit another creature to end a move in a prone creature's square.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2361 (Player Core pg. 422, Sharing a Space)

#### Scenario: Prone does not open the square

- **GIVEN** a prone creature on (8, 8)
- **WHEN** Stand and Take Cover are resolved, and Drop Prone is attempted
- **THEN** every result that succeeds stays on (8, 8)
- **AND** no result permits another creature to end a move on (8, 8)
