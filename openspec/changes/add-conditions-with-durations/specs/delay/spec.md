# delay

## MODIFIED Requirements

### Requirement: Return does not repeat the start hook

Return SHALL NOT change temporary Hit Points, their ticks, Hit Points, or wounded, and a shield the start already lowered SHALL stay down. A later resume SHALL also stay unticked. Delay SHALL run the condition end tick once. The turn return opened SHALL NOT run it again. Return SHALL NOT tick round durations. Persistent damage stays unmodeled (DEV-29). A missed arrival uses the ordinary start and ticks round durations once. Temporary Hit Points tick when DEV-32 is wired.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2294 (Player Core pg. 416, Delay)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2428 (Player Core pg. 435, Step 1: Start Your Turn)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2430 (Player Core pg. 436, Step 3: End Your Turn)
**Source**: DEV-29 in docs/designer/rules-deviations.md. Persistent damage is not modeled.
**Source**: DEV-32 in docs/designer/rules-deviations.md. The temporary Hit Point tick is the recipient's start.

#### Scenario: Return leaves temporary Hit Points as the start left them

- **GIVEN** CombatantId(1) is Alive at 9 Hit Points with 4 temporary Hit Points and 2 ticks, and CombatantId(2) is Alive, both Perception +0, scripted initiative naturals 15 then 10
- **WHEN** CombatantId(1)'s turn starts, CombatantId(1) Delays, CombatantId(2)'s turn ends, and CombatantId(1) returns
- **THEN** CombatantId(1) has 4 temporary Hit Points with 2 ticks, 9 Hit Points, and 3 actions
- **AND** the return rolls no die

#### Scenario: The later returned creature is not ticked

- **GIVEN** CombatantId(1), CombatantId(2), CombatantId(3), and CombatantId(4), Perception +0, scripted initiative naturals 20 then 15 then 10 then 5, CombatantId(1) and CombatantId(3) each Alive at 9 Hit Points with 4 temporary Hit Points and 2 ticks, both delayed, and CombatantId(4)'s turn has just ended
- **WHEN** CombatantId(3) and CombatantId(1) return, CombatantId(1)'s opened turn ends, and the next start is resolved
- **THEN** the round is 1 and CombatantId(3)'s turn is open with 3 actions and the reaction ready
- **AND** CombatantId(1) and CombatantId(3) each have 4 temporary Hit Points with 2 ticks and 9 Hit Points
- **AND** no die is rolled
- **WHEN** CombatantId(3)'s turn ends and the next start is resolved
- **THEN** the round is 2 and the open turn is CombatantId(2) with 3 actions

#### Scenario: A lowered shield stays down

- **GIVEN** CombatantId(1)'s shield was raised and the start lowered it before Delay, and CombatantId(1) then returns
- **WHEN** the shield is read
- **THEN** the shield is not raised and its Hit Points are unchanged

#### Scenario: Delay lowers frightened once

- **GIVEN** CombatantId(1) is frightened 2, and CombatantId(2) is Alive, both Perception +0, scripted initiative naturals 15 then 10
- **WHEN** CombatantId(1)'s turn starts and CombatantId(1) Delays
- **THEN** frightened is 1
- **AND** no die is rolled beyond initiative

#### Scenario: A missed return ticks the round duration on the next start

- **GIVEN** CombatantId(1) has sickened 1 for 2 rounds clocked to CombatantId(1), and CombatantId(2) is Alive, both Perception +0, scripted initiative naturals 15 then 10
- **WHEN** CombatantId(1)'s turn starts, CombatantId(1) Delays, CombatantId(2)'s turn starts and ends, and the next start is resolved
- **THEN** the round is 2 and CombatantId(1)'s turn is open
- **AND** sickened is absent
