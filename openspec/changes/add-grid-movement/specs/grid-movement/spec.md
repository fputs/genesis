# grid-movement

## Purpose

Measures Stride, Step, reach, and range on a 5-foot grid for Small and Medium creatures, including diagonal cost, difficult terrain, and occupied squares.

## ADDED Requirements

### Requirement: Grid measurement is pure

Grid measurement SHALL be a pure function of the squares, terrain, occupants, Speed, size, and diagonal counter the caller supplies. It SHALL NOT read a file, the network, a clock, or a global, and SHALL NOT roll dice. The same inputs SHALL return the same result. Fallible measurement SHALL return a typed error, not a panic. Coordinate and cost arithmetic SHALL be checked. The rules crate SHALL gain no dependency.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2356 (Player Core pg. 421, Grid Movement)

#### Scenario: The same Stride twice matches

- **GIVEN** a Medium creature with Speed 25, a fresh diagonal counter, an open 10-by-10 map, and a scripted d20 holding 8
- **WHEN** the same one-square orthogonal Stride is resolved twice
- **THEN** the two results are equal and the scripted 8 is still unused

### Requirement: Size data records all six sizes

Size data SHALL record Tiny, Small, Medium, Large, Huge, and Gargantuan. Tiny space SHALL be under 5 feet with reach 0 and 0. Small and Medium SHALL each be a 5-foot space with reach 5 and 5. Large SHALL be 10 feet with tall reach 10 and long reach 5. Huge SHALL be 15 feet with reach 15 and 10. Gargantuan space SHALL be at least 20 feet with reach 20 and 15.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2359 (Player Core pg. 421, Size, Space, and Reach)

#### Scenario: The size table keeps every size

- **GIVEN** the six sizes
- **WHEN** their space and reach are read
- **THEN** Tiny is under 5 feet with reach 0 and 0, Small and Medium are 5 feet with reach 5 and 5, Large is 10 feet with tall reach 10 and long reach 5, Huge is 15 feet with reach 15 and 10, and Gargantuan is at least 20 feet with reach 20 and 15

### Requirement: Only Small and Medium move

Movement, distance, and reach SHALL accept only Small and Medium, each one square. Tiny, Large, Huge, and Gargantuan SHALL return an unsupported-size error and SHALL NOT change position or the diagonal counter.

**Rule**: Player Core pg. 421 gives six sizes, and a Large or larger creature occupies more than one square. https://2e.aonprd.com/Rules.aspx?ID=2359
**Why**: Phase 1 fights use one square per creature.
**Substitute**: The size table stores all six sizes, and movement accepts only Small and Medium.
**Intent**: Larger footprints can be added later without a new table.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2359 (Player Core pg. 421, Size, Space, and Reach)

#### Scenario: Only Small and Medium can move

- **GIVEN** Speed 25, an open adjacent square, and a fresh counter
- **WHEN** a Large creature and a Tiny creature each Stride into that square, and a Small creature and a Medium creature do the same
- **THEN** Large and Tiny each return an unsupported-size error and stay put, and Small and Medium each succeed at a cost of 5 feet

### Requirement: The diagonal counter is turn state

Each creature SHALL have one diagonal counter in turn state, not on Creature and not on CombatantState. It SHALL start so the next diagonal costs 5 feet. Stride and Step on that turn SHALL share it. An orthogonal square SHALL NOT change it. Ending the turn SHALL reset it so the next diagonal costs 5 feet again. Movement off that turn SHALL use a fresh counter and SHALL leave the stored counter unchanged.

**Rule**: Player Core pg. 421 tracks the diagonal count across movement during your turn and resets it at end of turn. https://2e.aonprd.com/Rules.aspx?ID=2357
**Why**: The book does not say how a reaction or a readied Stride counts diagonals on someone else's turn.
**Substitute**: That action uses a fresh counter and does not change the mover's turn counter.
**Intent**: Off-turn movement still alternates, and the mover's own turn count stays put.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2357 (Player Core pg. 421, Diagonal Movement)

#### Scenario: Diagonal parity across actions

- **GIVEN** a Medium creature with Speed 25 and a fresh diagonal counter
- **WHEN** it Strides one diagonal square and then Strides one diagonal square again
- **THEN** the first Stride costs 5 feet, the second costs 10 feet, and both used the same turn counter

#### Scenario: Odd diagonal at turn end

- **GIVEN** a creature that ended its turn after one diagonal square
- **WHEN** the counter is reset for the next turn and the creature Strides one diagonal square
- **THEN** that square costs 5 feet

#### Scenario: Orthogonal does not flip parity

- **GIVEN** a fresh counter and Speed 25
- **WHEN** the creature Strides one diagonal square, one orthogonal square, and one diagonal square
- **THEN** the costs are 5, then 5, then 10, for a total of 20 feet

#### Scenario: The counter is not on the creature

- **GIVEN** a derived Creature, a CombatantState built from it, and a fresh counter
- **WHEN** a diagonal Stride updates the counter
- **THEN** the Creature still equals its original value, the CombatantState still equals a fresh state from that Creature, and the updated counter is a separate value

#### Scenario: Off-turn movement uses a fresh counter

- **GIVEN** a stored turn counter that already counts one diagonal, and Speed 25
- **WHEN** the creature Strides one diagonal square off-turn, and separately Strides two diagonal squares off-turn
- **THEN** the one-square Stride costs 5 feet, the two-square Stride costs 15 feet, and the stored counter still counts one diagonal

#### Scenario: Shared diagonal counter includes Step

- **GIVEN** Speed 25 and one fresh turn counter
- **WHEN** the creature Strides one diagonal square, Steps one orthogonal square, and Strides one diagonal square, all on that counter
- **THEN** the costs are 5 feet, then 5 feet for the Step, then 10 feet, and the orthogonal Step did not advance the counter

### Requirement: Entering a square has one cost

The cost to enter a square SHALL be a base plus that square's terrain only. An orthogonal base SHALL be 5 feet. A diagonal base SHALL alternate 5 feet, then 10 feet, using the turn's diagonal counter. Difficult terrain SHALL add 5 feet and greater difficult terrain SHALL add 10 feet. That extra SHALL NOT be multiplied on a diagonal. The square being left SHALL add no cost.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2356 (Player Core pg. 421, one square is 5 feet)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2357 (Player Core pg. 421, diagonals alternate)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2366 (Player Core pg. 423, Difficult Terrain)

#### Scenario: Difficult on a diagonal

- **GIVEN** a fresh counter and a second diagonal square that is difficult terrain
- **WHEN** the creature Strides the open first diagonal and then that difficult square
- **THEN** the second square costs 15 feet, not 20

#### Scenario: Greater difficult on a diagonal

- **GIVEN** a fresh counter
- **WHEN** the first diagonal square is greater difficult terrain
- **THEN** that square costs 15 feet
- **AND** when the first diagonal is open and the second is greater difficult terrain, the second square costs 20 feet

#### Scenario: Leaving difficult terrain

- **GIVEN** a difficult start square and an open orthogonal destination
- **WHEN** the creature moves one orthogonal square
- **THEN** the cost is 5 feet

### Requirement: Stride follows Speed along the path

Stride SHALL succeed only when every entered square is adjacent to the previous one and the total cost is at most Speed. A cost equal to Speed SHALL succeed. Speed left over SHALL NOT carry into the next action. On success Stride SHALL return that total cost, the end square, and the updated counter.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2305 (Player Core pg. 418, Stride)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2347 (Player Core pg. 420, Speed)

#### Scenario: Exact Speed

- **GIVEN** a Medium creature with Speed 25 on open ground and a fresh counter
- **WHEN** it Strides five orthogonal squares
- **THEN** the move succeeds and the cost is 25 feet
- **AND** when it instead Strides six orthogonal squares, the result is an exceeds-Speed error, the position stays the start square, and the counter stays fresh

#### Scenario: Terrain cost exceeds the remainder

- **GIVEN** Speed 25, four open orthogonal squares, and a fifth square of difficult terrain
- **WHEN** the creature Strides that path
- **THEN** the path would cost 30 feet, the result is an exceeds-Speed error, and the creature is still on the start square

#### Scenario: Leftover movement is not banked

- **GIVEN** Speed 25 and a fresh counter
- **WHEN** the creature Strides one orthogonal square and then Strides five orthogonal squares
- **THEN** the first cost is 5 feet, the second cost is 25 feet, and the second move is not limited to the 20 feet the first move did not use

### Requirement: Movement Speeds are multiples of five

Stride and Step SHALL accept Speed 0 and any positive multiple of 5 feet. A Speed that is not a multiple of 5 SHALL return a typed error. Speed 0 SHALL allow a Stride that enters no square. A path whose cost is above Speed SHALL return an exceeds-Speed error, including a non-empty path at Speed 0.

**Rule**: Player Core movement uses Speeds in multiples of 5 feet. The book does not tell an engine to reject another stored number. https://2e.aonprd.com/Rules.aspx?ID=2347
**Why**: A creature build may store a non-negative Speed that is not a multiple of 5, because creature validation only rejects a negative Speed.
**Substitute**: Movement returns a typed error and does not change creature validation.
**Intent**: Stride cannot spend a Speed the grid cannot divide into squares.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2347 (Player Core pg. 420, Speed)
**Source**: https://2e.aonprd.com/Actions.aspx?ID=2304 (Player Core pg. 418, Step requires Speed 10)

#### Scenario: Speed 0

- **GIVEN** Speed 0, an open adjacent square, and a fresh counter
- **WHEN** the creature Strides an empty path
- **THEN** the move succeeds, the cost is 0 feet, and it remains on the start square
- **AND** a Stride into the adjacent square is an exceeds-Speed error and the position stays
- **AND** a Step into that square is rejected because Speed is under 10 feet, and the position stays

#### Scenario: Speed not a multiple of five

- **GIVEN** Speed 7 and an empty path
- **WHEN** the creature Strides
- **THEN** the result is a speed-not-multiple-of-five error and the position stays the start square

### Requirement: Step is five feet

Step SHALL follow a path of exactly one adjacent square and SHALL require Speed of at least 10 feet. It SHALL NOT enter difficult or greater difficult terrain. Its distance SHALL be 5 feet even when the counter would charge 10, and that distance SHALL NOT be charged against Speed. A diagonal Step SHALL advance the shared counter. An orthogonal Step SHALL NOT.

**Rule**: Player Core pg. 421 counts all movement in the turn, and Step on pg. 418 moves 5 feet and does not enter difficult terrain. https://2e.aonprd.com/Rules.aspx?ID=2357 https://2e.aonprd.com/Actions.aspx?ID=2304
**Why**: A diagonal Step can land on the count that would otherwise cost 10 feet.
**Substitute**: The Step moves 5 feet, does not spend Speed, and still advances the counter.
**Intent**: Step stays a 5-foot action, and the turn's alternation stays honest.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2304 (Player Core pg. 418, Step)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2366 (Player Core pg. 423, no Step into difficult terrain)

#### Scenario: Step into difficult terrain

- **GIVEN** Speed 25 and an adjacent difficult square
- **WHEN** the creature Steps into that square
- **THEN** the Step is rejected and the position stays
- **AND** a Step into an adjacent greater-difficult square is rejected the same way

#### Scenario: Step with Speed 5

- **GIVEN** Speed 5 and an open adjacent square
- **WHEN** the creature Steps into that square
- **THEN** the Step is rejected because Speed is under 10 feet, and the position stays

#### Scenario: Step advances parity

- **GIVEN** a fresh counter and Speed 25
- **WHEN** the creature Steps one diagonal square and then Strides one diagonal square
- **THEN** the Step distance is 5 feet and the Stride costs 10 feet

#### Scenario: Step on a 10-ft diagonal costs 5

- **GIVEN** a turn counter that already counts one diagonal, and Speed 25
- **WHEN** the creature Steps one diagonal square and then Strides one diagonal square
- **THEN** the Step costs 5 feet rather than 10, the counter then counts two diagonals, and the following Stride costs 5 feet

#### Scenario: A Step is one square

- **GIVEN** Speed 25 and two open orthogonal squares in a row
- **WHEN** the creature Steps along both squares, and separately Steps an empty path
- **THEN** both Steps return a typed error and the position stays the start square

### Requirement: Same faction can be crossed

Same faction SHALL count as willing. A path SHALL be allowed to enter a same-faction square and SHALL NOT end on a square occupied by another creature. Entering another faction's square SHALL return an enemy-square error. Ending on a same-faction creature SHALL return an occupied-end error. Passing through an unwilling creature SHALL NOT be resolved here.

**Rule**: Player Core pg. 422 lets you move through a willing creature. An unwilling creature needs Tumble Through. A move may end in a willing creature's square only when another move immediately leaves. https://2e.aonprd.com/Rules.aspx?ID=2360
**Why**: Phase 1 has no willingness flag and no action sequencing. Tumble Through needs a skill check that is deferred.
**Substitute**: Same faction may be passed through and may not be the end square. Another faction blocks the square.
**Intent**: Allies do not stop a Stride, enemies do, and a move does not finish while sharing a square.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2360 (Player Core pg. 422, Moving Through a Creature's Space)

#### Scenario: Through an ally

- **GIVEN** a same-faction creature on the first square of a two-square open path, and an empty end square
- **WHEN** the creature Strides through that square to the empty end
- **THEN** the Stride succeeds, it entered the same-faction square, and it did not end there

#### Scenario: Through an enemy

- **GIVEN** an other-faction creature on a square of an otherwise open path
- **WHEN** the creature Strides through that square, and separately ends a Stride on that square
- **THEN** both results are an enemy-square error and the position stays the start square

#### Scenario: Ending in an ally square

- **GIVEN** a same-faction creature on the destination square
- **WHEN** the creature ends its Stride on that square
- **THEN** the result is an occupied-end error and the position stays the start square

### Requirement: The caller supplies the map

The caller SHALL supply the map width and height in squares. A square SHALL be inside only when both coordinates are at least 0 and strictly below those sizes. A move whose start or path square is outside SHALL return an off-grid error.

**Rule**: Player Core has no rule for the edge of a map.
**Why**: A path can name a square the map does not contain.
**Substitute**: The whole action is rejected and the creature stays put.
**Intent**: There is no partial move and no stop at the edge.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2356 (Player Core pg. 421, squares on the grid)

#### Scenario: Off-grid

- **GIVEN** a map 10 squares wide and a path that includes a square outside it
- **WHEN** the creature Strides that path
- **THEN** the whole Stride is an off-grid error and the position is unchanged

### Requirement: Overflow is a typed error

Coordinate subtraction and cost addition SHALL be checked. When that arithmetic overflows, the call SHALL return an overflow error and SHALL NOT return a wrapped distance or a new square.

#### Scenario: Coordinate overflow is an error

- **GIVEN** two Medium creatures on squares whose coordinates are the extreme positive and extreme negative whole numbers these rules store
- **WHEN** the distance between them is measured
- **THEN** the result is an overflow error and not a numeric distance

### Requirement: A failed move does nothing

A failed Stride or Step SHALL return one typed error and no new square. The caller's position and stored diagonal counter SHALL stay as they were. No prefix of the path SHALL be applied.

#### Scenario: A gap in the path is rejected

- **GIVEN** a start square and a path square two orthogonal squares away, with no square between them
- **WHEN** the creature Strides that path
- **THEN** the result is a typed error, the position stays the start square, and the counter is unchanged

### Requirement: Distance ignores terrain and creatures

Distance between two Small or Medium creatures SHALL be the grid cost between their squares, alternating from 5 feet, ignoring terrain, other creatures, and the turn counter. It SHALL be symmetric. The same square SHALL be 0 feet. Adjacent SHALL mean the squares share an edge or a corner.

**Rule**: Player Core pg. 428 measures areas the same way as movement and ignores difficult terrain. No single passage states creature-to-creature distance. https://2e.aonprd.com/Rules.aspx?ID=2384
**Why**: Reach and range need a distance, and the book does not put two spaces into one formula.
**Substitute**: Use the grid cost between the two squares, alternating from 5 feet, ignoring terrain and creatures.
**Intent**: Reach and range share one measurement, and it does not depend on the turn's diagonal count.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2384 (Player Core pg. 428, Areas)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2357 (Player Core pg. 421, the diagonal count starts at 5 feet)

#### Scenario: Distance 0 and adjacency

- **GIVEN** two Medium creatures one diagonal square apart, a difficult square between other routes, an other-faction creature nearby, and a turn counter that already counts one diagonal
- **WHEN** distance and adjacency are measured, including from the second creature back to the first and from a creature to itself
- **THEN** the diagonal pair is distance 5 feet and adjacent, the reversed pair is also 5 feet, the self measurement is 0 feet and not adjacent, terrain and the other creature do not change the 5 feet, and the turn counter still counts one diagonal

### Requirement: Reach uses grid distance

A creature SHALL NOT count as inside its own reach. Reach of 5 feet SHALL include only an adjacent creature. Reach of 10 feet SHALL include a creature at distance at most 10 feet, and a creature exactly two diagonal squares away. Reach above 10 feet SHALL include a creature only when the distance is at most the reach.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2379 (Player Core pg. 426, Range and Reach)

#### Scenario: Reach 5 is only adjacency

- **GIVEN** a Medium creature with reach 5
- **WHEN** reach is asked for a diagonal neighbor, a creature two orthogonal squares away, and the creature itself
- **THEN** only the diagonal neighbor is in reach

#### Scenario: Reach 10 diagonal exception

- **GIVEN** reach 10
- **WHEN** reach is asked for a creature two diagonal squares away, a creature at a 2-by-1 offset, and a creature at a 3-by-1 offset
- **THEN** the two-diagonal creature is in reach even though the grid cost is 15 feet, the 2-by-1 creature is in reach at 10 feet, and the 3-by-1 creature is not in reach at 15 feet
- **AND** with reach 20, a creature three diagonal squares away is in reach at 20 feet

### Requirement: Range increments are plain numbers

A range query SHALL return an increment count and an untyped penalty number and SHALL NOT apply either to an attack roll. The count SHALL be the distance divided by the increment, rounded up, and at least 1. An exact multiple SHALL stay in that lower count. The penalty SHALL be -2 times the count minus 1. A count above 6 SHALL be an illegal-range error.

**Rule**: Player Core pg. 403 charges -2 per range increment after the first, through six increments, and pg. 400 says that penalty is untyped and adds with other untyped penalties. https://2e.aonprd.com/Rules.aspx?ID=2290 https://2e.aonprd.com/Rules.aspx?ID=2282
**Why**: Adding the penalty to an attack belongs to encounter-core, and the modifiers change is being written separately.
**Substitute**: The query returns the increment count and the untyped penalty number.
**Intent**: A later attack roll can add that number to the multiple attack penalty without this change stacking modifiers.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2290 (Player Core pg. 403, Range Penalty)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2282 (Player Core pg. 400, untyped penalties)

#### Scenario: Range boundary

- **GIVEN** a range increment of 60 feet
- **WHEN** the distance is 0, 60, 65, 360, and 365 feet
- **THEN** 0 feet is count 1 and penalty 0, 60 feet is count 1 and penalty 0, 65 feet is count 2 and penalty -2, 360 feet is count 6 and penalty -10, and 365 feet is an illegal-range error rather than a penalty of -12

#### Scenario: A boundary uses the lower increment

- **GIVEN** a range increment of 60 feet
- **WHEN** the distance is exactly 120 feet, and separately 121 feet
- **THEN** 120 feet is count 2 and penalty -2, and 121 feet is count 3 and penalty -4

#### Scenario: Range penalty is plain data

- **GIVEN** a distance of 130 feet and a range increment of 60 feet
- **WHEN** the range query is resolved
- **THEN** the count is 3 and the untyped penalty is -4, and the result includes no attack total and no multiple-attack-penalty term

### Requirement: Invalid measures are rejected

A negative Speed, a range increment of 0 or less, and a negative distance SHALL each return a typed error. The value SHALL NOT be clamped to a usable number.

**Rule**: Player Core does not define a negative Speed or a non-positive range increment.
**Why**: A caller can still pass those numbers, and clamping would invent a distance.
**Substitute**: Negative Speed, a non-positive increment, and a negative distance are typed errors.
**Intent**: A bad measure fails instead of moving or shooting a made-up distance. Speed 0 stays legal for an empty Stride.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2347 (Player Core pg. 420, Speed)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2290 (Player Core pg. 403, Range Penalty)

#### Scenario: Negative or zero inputs

- **GIVEN** Speed -5, a range increment of 0 at 60 feet, and a distance of -1 with increment 30
- **WHEN** each value is used for Stride or a range query
- **THEN** Speed -5 is a negative-Speed error and not a stay in place, increment 0 is an invalid-increment error and not penalty 0, and distance -1 is a negative-distance error

### Requirement: Immobilized blocks the move trait

An immobilized flag SHALL make Stride and Step illegal. That check SHALL also be usable with no path, so a later move-trait action can call the same check. This capability SHALL NOT implement Crawl, Stand, or Drop Prone.

**Rule**: Player Core pg. 444 says an immobilized creature cannot use a move action. Player Core pg. 420 says penalties cannot reduce Speed below 5 feet. https://2e.aonprd.com/Conditions.aspx?ID=81 https://2e.aonprd.com/Rules.aspx?ID=2346
**Why**: Phase 1 has no Speed penalties, and immobilized is not yet a condition.
**Substitute**: The 5-foot floor is not applied. Immobilized is a caller flag. Speed 0 allows an empty Stride and rejects Step.
**Intent**: The move-trait check is ready for later actions, and Speed is used as given.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=81 (Player Core pg. 444, Immobilized)

#### Scenario: Immobilized

- **GIVEN** an immobilized creature with Speed 25 and an open adjacent square
- **WHEN** it Strides into that square and, separately, Steps into that square
- **THEN** both are rejected, the position stays, and the diagonal counter stays

#### Scenario: The immobilized check needs no path

- **GIVEN** the immobilized flag set, and separately the flag clear
- **WHEN** the move-trait check is asked with no path
- **THEN** the set flag is an immobilized error and the clear flag is allowed

### Requirement: The caller lists every square

Stride and Step SHALL move only through squares the caller listed, in that order. They SHALL NOT invent squares toward a destination. An empty Stride path SHALL cost 0 feet, emit no events, and end on the start square.

#### Scenario: An empty path does not search

- **GIVEN** Speed 25, an empty path, and a destination three orthogonal squares away
- **WHEN** the creature Strides
- **THEN** it stays on the start square, the cost is 0 feet, no square is entered, and the destination is not added to the path

### Requirement: Movement reports leave and enter

A successful move SHALL return leave and enter events in path order. Each entered square SHALL be preceded by leaving the square the creature occupied before it. A Stride result SHALL be marked triggering. A Step result SHALL be marked non-triggering. No reaction SHALL be resolved.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2305 (Player Core pg. 418, Stride)
**Source**: https://2e.aonprd.com/Actions.aspx?ID=2304 (Player Core pg. 418, Step triggers no reactions)

#### Scenario: Stride emits left and entered events

- **GIVEN** a start at the origin, an open orthogonal square, and an open diagonal square after it, with Speed 25 and a fresh counter
- **WHEN** the creature Strides that two-square path
- **THEN** the cost is 10 feet, the result is marked triggering, and the events are leave origin, enter the orthogonal square, leave that square, enter the diagonal square, in that order

#### Scenario: Step events are non-triggering

- **GIVEN** Speed 25 and one open diagonal square
- **WHEN** the creature Steps into that square
- **THEN** the distance is 5 feet, the result is marked non-triggering, and the events are leave the start square, then enter the destination

### Requirement: Movement does not spend actions

Stride and Step SHALL NOT read or write actions remaining and SHALL NOT return an action cost.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2305 (Player Core pg. 418, Stride costs 1 action; spending it is encounter-core)

#### Scenario: A Stride does not spend actions

- **GIVEN** a CombatantState with 2 actions remaining and its reaction not ready, and a legal one-square Stride
- **WHEN** that Stride is resolved without passing the state
- **THEN** the state still has 2 actions remaining, the reaction is still not ready, and the Stride result has a foot cost and no action cost
