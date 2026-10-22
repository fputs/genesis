# dying-and-victory

## Purpose

Resolves knockout, dying, recovery, temporary Hit Points, and which side has won, without running initiative or Strike.

## ADDED Requirements

### Requirement: A blow to 0 Hit Points sets dying

A creature with uses_dying_rules that drops from above 0 Hit Points to 0 SHALL gain dying 1, or dying 2 when that blow is critical, plus its wounded value. It SHALL become unconscious. That blow SHALL NOT also apply damage-while-dying. The Creature SHALL stay unchanged.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2324 (Player Core pg. 410, Getting Knocked Out)

#### Scenario: A non-critical blow to 0 sets dying 1

- **GIVEN** uses_dying_rules, maximum Hit Points 9, current Hit Points 9, wounded 0, and a scripted d8 of 8 for non-critical damage 1d8+1
- **WHEN** that blow is applied
- **THEN** current Hit Points are 0, the dying value is 1, wounded is 0, the combatant is unconscious and not dead, and the Creature still has 9 Hit Points

#### Scenario: A critical blow to 0 sets dying 2

- **GIVEN** uses_dying_rules, maximum Hit Points 16, current Hit Points 16, wounded 0, and a scripted d8 of 8 for critical damage 1d8
- **WHEN** that blow is applied
- **THEN** current Hit Points are 0 and the dying value is 2
- **AND** wounded is 0 and the combatant is unconscious and not dead

#### Scenario: The dropping blow does not also increase dying

- **GIVEN** uses_dying_rules, maximum Hit Points 16, current Hit Points 16, wounded 0, and a scripted d8 of 8 for critical damage 1d8
- **WHEN** that blow is applied
- **THEN** the dying value is 2, not 4

#### Scenario: Damage above 0 does not start dying

- **GIVEN** uses_dying_rules, maximum Hit Points 9, current Hit Points 9, and a scripted d8 of 1 for non-critical damage 1d8+1
- **WHEN** that blow is applied
- **THEN** current Hit Points are 7, the combatant is conscious, and there is no dying value

### Requirement: No dying rules means death at 0

A creature without uses_dying_rules at 0 current Hit Points SHALL be dead, with no dying value and no wounded change. It SHALL NOT be unconscious.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2324 (Player Core pg. 410, Getting Knocked Out)

#### Scenario: An enemy at 0 Hit Points does not gain wounded

- **GIVEN** uses_dying_rules false, maximum Hit Points 9, current Hit Points 9, wounded 0, and a scripted d8 of 8 for damage 1d8+1
- **WHEN** that blow is applied
- **THEN** current Hit Points are 0, the combatant is dead, wounded is 0, and there is no dying value

### Requirement: Dying applies the unconscious penalty

A dying combatant SHALL be unconscious. Armor Class, Perception, and Reflex SHALL each include a status penalty of -4 while unconscious, and those collections SHALL NOT compete. A conscious or dead combatant SHALL NOT take that penalty. This change SHALL NOT add off-guard or prone.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=95 (Player Core pg. 446, Unconscious)

#### Scenario: A dying creature takes -4 on three statistics

- **GIVEN** a dying combatant and no other modifiers
- **WHEN** Armor Class, Perception, and Reflex are totaled separately
- **THEN** each total includes a status penalty of -4
- **AND** totaling one statistic does not remove the penalty on another

#### Scenario: A conscious creature takes no unconscious penalty

- **GIVEN** a conscious combatant
- **WHEN** Armor Class, Perception, and Reflex are totaled
- **THEN** none of those totals includes the unconscious penalty

#### Scenario: A dead creature takes no unconscious penalty

- **GIVEN** a combatant who is dead at 0 Hit Points with no dying value
- **WHEN** Armor Class, Perception, and Reflex are totaled
- **THEN** none of those totals includes the unconscious penalty

#### Scenario: Waking removes the unconscious penalty

- **GIVEN** dying 1, current Hit Points 0, and a scripted natural 11
- **WHEN** recovery removes dying and Armor Class, Perception, and Reflex are totaled
- **THEN** none of those totals includes the unconscious penalty

#### Scenario: Unconscious does not add off-guard

- **GIVEN** a dying combatant and no off-guard or prone input
- **WHEN** Armor Class is totaled
- **THEN** the total is a status penalty of -4
- **AND** it does not include an off-guard circumstance penalty of -2

### Requirement: Recovery checks use DC 10 plus dying

While dying, start of turn SHALL roll one flat check at DC 10 plus dying and SHALL add no modifier. Critical success SHALL reduce dying by 2, success by 1, failure SHALL increase it by 1, and critical failure by 2. A conscious or dead combatant SHALL NOT roll. Equal inputs SHALL match.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2326 (Player Core pg. 411, Recovery Checks)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2278 (Player Core pg. 400, Flat Checks)

#### Scenario: Recovery critical success reduces dying by 2

- **GIVEN** dying 3 and a scripted natural 20
- **WHEN** the start-of-turn hook runs
- **THEN** the flat check DC is 13, the degree is critical success, and the dying value is 1

#### Scenario: Recovery success reduces dying by 1

- **GIVEN** dying 2 and a scripted natural 12
- **WHEN** the start-of-turn hook runs
- **THEN** the flat check DC is 12, the degree is success, and the dying value is 1

#### Scenario: Recovery failure increases dying by 1

- **GIVEN** dying 1 and a scripted natural 2
- **WHEN** the start-of-turn hook runs
- **THEN** the flat check DC is 11, the degree is failure, and the dying value is 2

#### Scenario: Recovery critical failure increases dying by 2

- **GIVEN** dying 1 and a scripted natural 1
- **WHEN** the start-of-turn hook runs
- **THEN** the flat check DC is 11, the degree is critical failure, and the dying value is 3

#### Scenario: A conscious creature does not roll recovery

- **GIVEN** a conscious combatant and a scripted natural 20
- **WHEN** the start-of-turn hook runs
- **THEN** no recovery check is rolled and the natural 20 is unused

#### Scenario: A dead creature does not roll recovery

- **GIVEN** a dead combatant and a scripted natural 20
- **WHEN** the start-of-turn hook runs
- **THEN** no recovery check is rolled and the natural 20 is unused

#### Scenario: The same recovery twice matches

- **GIVEN** dying 2 and a scripted natural 12
- **WHEN** the start-of-turn hook is resolved twice on equal starting states
- **THEN** the two results are equal

### Requirement: Dying 0 restores 1 Hit Point

When recovery reduces dying to 0 or below, the combatant SHALL lose dying, set current Hit Points to 1, increase wounded by 1, and be conscious. Temporary Hit Points SHALL stay unchanged. A reduction that stays above 0 SHALL NOT change Hit Points or wounded.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2328 (Player Core pg. 411, Losing the Dying Condition)

#### Scenario: Recovery to 0 wakes at 1 Hit Point

- **GIVEN** dying 1, current Hit Points 0, wounded 0, 4 temporary Hit Points, and a scripted natural 11
- **WHEN** the start-of-turn hook runs
- **THEN** the degree is success, there is no dying value, current Hit Points are 1, wounded is 1, temporary Hit Points are 4, and the combatant is conscious

#### Scenario: A recovery that stays dying does not change Hit Points

- **GIVEN** dying 2, current Hit Points 0, wounded 0, and a scripted natural 12
- **WHEN** the start-of-turn hook runs
- **THEN** the dying value is 1, current Hit Points stay 0, wounded stays 0, and the combatant stays unconscious

### Requirement: Wounded increases when dying is lost

Losing dying SHALL increase wounded by 1. Gaining dying, damage while dying, and death SHALL NOT increase wounded. Healing to maximum Hit Points SHALL NOT reduce wounded.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2328 (Player Core pg. 411, Losing the Dying Condition)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=99 (Player Core pg. 447, Wounded)

#### Scenario: Losing dying increases wounded by 1

- **GIVEN** dying 1, wounded 0, and a scripted natural 11
- **WHEN** recovery removes dying
- **THEN** wounded is 1

#### Scenario: Gaining dying does not increase wounded

- **GIVEN** uses_dying_rules, wounded 0, maximum Hit Points 9, current Hit Points 9, and a scripted d8 of 8 for non-critical damage 1d8+1
- **WHEN** that blow is applied
- **THEN** the dying value is 1 and wounded is still 0

#### Scenario: Damage while dying does not increase wounded

- **GIVEN** dying 1, wounded 2, and a non-critical blow of 1 damage
- **WHEN** that blow is applied
- **THEN** the dying value is 2 and wounded is still 2

#### Scenario: Death does not increase wounded

- **GIVEN** dying 3, wounded 1, and a non-critical blow of 1 damage
- **WHEN** that blow is applied
- **THEN** the combatant is dead and wounded is still 1

#### Scenario: Full Hit Points do not clear wounded

- **GIVEN** a conscious combatant, wounded 2, maximum Hit Points 9, and current Hit Points 9
- **WHEN** healing of 4 is applied
- **THEN** current Hit Points are 9 and wounded is still 2

### Requirement: Wounded adds to the new dying value

When a conscious creature gains dying, the value SHALL be 1 plus wounded, or 2 plus wounded when the blow is critical.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2324 (Player Core pg. 410, Getting Knocked Out)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=99 (Player Core pg. 447, Wounded)

#### Scenario: Wounded increases the next dying value

- **GIVEN** uses_dying_rules, wounded 1, maximum Hit Points 9, current Hit Points 9, and a scripted d8 of 8 for non-critical damage 1d8+1
- **WHEN** that blow is applied
- **THEN** the dying value is 2 and wounded is still 1

#### Scenario: A critical blow adds wounded to 2

- **GIVEN** uses_dying_rules, wounded 1, maximum Hit Points 16, current Hit Points 16, and a scripted d8 of 8 for critical damage 1d8
- **WHEN** that blow is applied
- **THEN** the dying value is 3 and wounded is still 1

### Requirement: Damage while dying increases dying

A blow whose damage is greater than 0 against a creature that is already dying SHALL increase dying by 1, or by 2 when the blow is critical. Damage 0 SHALL NOT change dying. Temporary Hit Points SHALL NOT prevent that increase.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2327 (Player Core pg. 411, Taking Damage while Dying)

#### Scenario: Damage while dying increases dying by 1

- **GIVEN** dying 1, current Hit Points 0, maximum Hit Points 9, and a scripted d8 of 1 for non-critical damage 1d8
- **WHEN** that blow is applied
- **THEN** the dying value is 2 and current Hit Points stay 0

#### Scenario: A critical blow while dying increases dying by 2

- **GIVEN** dying 1, current Hit Points 0, maximum Hit Points 9, and a scripted d8 of 1 for critical damage 1d8
- **WHEN** that blow is applied
- **THEN** the dying value is 3

#### Scenario: Zero damage while dying does not increase dying

- **GIVEN** dying 1, current Hit Points 0, and a scripted d4 of 1 with a bonus of -5
- **WHEN** that blow is applied
- **THEN** the damage is 0 and the dying value is still 1

#### Scenario: Temporary Hit Points do not block the dying increase

- **GIVEN** dying 1, current Hit Points 0, 5 temporary Hit Points, and a non-critical blow of 3 damage
- **WHEN** that blow is applied
- **THEN** temporary Hit Points are 2, current Hit Points stay 0, and the dying value is 2

### Requirement: Dying 4 is death

A dying value of 4 or higher SHALL become death immediately. The state SHALL NOT keep a dying value. The combatant SHALL NOT be conscious. Wounded SHALL stay as it was.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2325 (Player Core pg. 411, Dying)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=69 (Player Core pg. 443, Dying)

#### Scenario: Dying 3 and one damage kills

- **GIVEN** dying 3, wounded 0, maximum Hit Points 9, and a non-critical blow of 1 damage
- **WHEN** that blow is applied
- **THEN** the combatant is dead, no dying value is stored, and wounded is 0

#### Scenario: A critical blow at dying 3 kills

- **GIVEN** dying 3, maximum Hit Points 9, current Hit Points 0, and a scripted d8 of 1 for critical damage 1d8
- **WHEN** that blow is applied
- **THEN** the combatant is dead

#### Scenario: Wounded 3 dies on the knockout

- **GIVEN** uses_dying_rules, wounded 3, maximum Hit Points 9, current Hit Points 9, and a scripted d8 of 8 for non-critical damage 1d8+1
- **WHEN** that blow is applied
- **THEN** the combatant is dead, no dying value is stored, and wounded is still 3

#### Scenario: A recovery critical failure at dying 2 kills

- **GIVEN** dying 2 and a scripted natural 1
- **WHEN** the start-of-turn hook runs
- **THEN** the degree is critical failure and the combatant is dead

#### Scenario: A dead creature is not conscious

- **GIVEN** a combatant who just died from dying 4
- **WHEN** consciousness is read
- **THEN** the combatant is not conscious

### Requirement: Massive damage kills outright

Damage greater than or equal to twice maximum Hit Points SHALL kill immediately. The check SHALL use that damage, not Hit Points lost, and SHALL ignore temporary Hit Points. The combatant SHALL NOT gain dying. Wounded SHALL NOT change. Death SHALL set current Hit Points to 0 and clear temporary Hit Points.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2332 (Player Core pg. 412, Massive Damage). FIX-15. R-06 (DEV-13).

#### Scenario: Damage equal to twice maximum Hit Points kills

- **GIVEN** uses_dying_rules, maximum Hit Points 10, current Hit Points 10, wounded 0, and a scripted d8 of 8 for non-critical damage 1d8+12
- **WHEN** that blow of 20 damage is applied
- **THEN** the combatant is dead, no dying value is stored, wounded is 0, and current Hit Points are 0

#### Scenario: Damage one below twice maximum does not kill

- **GIVEN** uses_dying_rules, maximum Hit Points 10, current Hit Points 10, and a scripted d8 of 8 for non-critical damage 1d8+11
- **WHEN** that blow of 19 damage is applied
- **THEN** the combatant is dying 1 and is not dead

#### Scenario: Massive damage ignores temporary Hit Points

- **GIVEN** uses_dying_rules, maximum Hit Points 10, current Hit Points 10, 30 temporary Hit Points, and a blow of 20 damage
- **WHEN** that blow is applied
- **THEN** the combatant is dead, current Hit Points are 0, and temporary Hit Points are 0

#### Scenario: A critical blow doubles before the massive check

- **GIVEN** uses_dying_rules, maximum Hit Points 10, current Hit Points 10, and a scripted d8 of 8 for critical damage 1d8+2
- **WHEN** that blow is applied
- **THEN** the doubled damage is 20 and the combatant is dead

#### Scenario: The same blow without doubling does not kill

- **GIVEN** uses_dying_rules, maximum Hit Points 10, current Hit Points 10, and a scripted d8 of 8 for non-critical damage 1d8+2
- **WHEN** that blow is applied
- **THEN** the damage is 10, current Hit Points are 0, and the dying value is 1

#### Scenario: Massive damage while dying kills without another increase

- **GIVEN** dying 1, maximum Hit Points 10, current Hit Points 0, and a blow of 20 damage
- **WHEN** that blow is applied
- **THEN** the combatant is dead and the dying value is not 2

#### Scenario: Massive damage kills when temporary Hit Points would have covered it

- **GIVEN** uses_dying_rules false, maximum Hit Points 10, current Hit Points 10, 30 temporary Hit Points, and a blow of 20 damage
- **WHEN** that blow is applied
- **THEN** the combatant is dead

### Requirement: Blow resolution checks massive damage first

One blow SHALL judge massive damage before it spends temporary Hit Points. If the blow does not kill, it SHALL spend temporary Hit Points, then current Hit Points, then dying or death at 0.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2332 (Player Core pg. 412, Massive Damage)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2321 (Player Core pg. 410, Temporary Hit Points)

#### Scenario: Massive damage is judged before temporary Hit Points

- **GIVEN** maximum Hit Points 10, current Hit Points 10, 30 temporary Hit Points, and a blow of 20 damage
- **WHEN** that blow is applied
- **THEN** the combatant is dead even though 30 temporary Hit Points would have covered 20 damage

### Requirement: Healing removes dying

A positive heal on a dying combatant SHALL add that amount, cap it at maximum Hit Points, remove dying, increase wounded by 1, and leave the combatant conscious. Temporary Hit Points SHALL stay unchanged. An add that does not fit SHALL return a typed error and leave dying unchanged.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2328 (Player Core pg. 411, Losing the Dying Condition)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2320 (Player Core pg. 410, Healing)

#### Scenario: Healing removes dying and sets the healed Hit Points

- **GIVEN** dying 1, current Hit Points 0, wounded 0, maximum Hit Points 9, and 4 temporary Hit Points
- **WHEN** healing of 4 is applied
- **THEN** current Hit Points are 4, temporary Hit Points are 4, wounded is 1, there is no dying value, and the combatant is conscious

#### Scenario: Healing is capped at maximum Hit Points

- **GIVEN** dying 1, current Hit Points 0, wounded 0, and maximum Hit Points 9
- **WHEN** healing of 100 is applied
- **THEN** current Hit Points are 9, wounded is 1, and the combatant is conscious

#### Scenario: Healing overflow leaves dying unchanged

- **GIVEN** a dying combatant whose current Hit Points are 2147483647
- **WHEN** healing of 1 is applied
- **THEN** the result is a typed overflow error, the dying value is unchanged, and wounded is unchanged

### Requirement: Healing cannot revive the dead

Healing a dead combatant SHALL return a typed error and SHALL NOT change Hit Points, wounded, temporary Hit Points, or death.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2320 (Player Core pg. 410, Healing)

#### Scenario: Healing the dead returns an error

- **GIVEN** a dead combatant at 0 current Hit Points, wounded 1, and 0 temporary Hit Points
- **WHEN** healing of 5 is applied
- **THEN** the result is a typed error, current Hit Points stay 0, wounded stays 1, and the combatant stays dead

### Requirement: Non-positive healing is rejected

A heal of 0 or less SHALL return a typed error and SHALL NOT change the state. It SHALL NOT remove dying.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2320 (Player Core pg. 410, Healing)

#### Scenario: A zero heal does not remove dying

- **GIVEN** dying 1 and current Hit Points 0
- **WHEN** healing of 0 is applied
- **THEN** the result is a typed error and the dying value is still 1

#### Scenario: A negative heal does not change Hit Points

- **GIVEN** a conscious combatant at 9 current Hit Points
- **WHEN** healing of -1 is applied
- **THEN** the result is a typed error and current Hit Points stay 9

### Requirement: Temporary Hit Points keep the higher pool

A new temporary amount SHALL replace the current pool only when it is strictly greater, and it SHALL keep the new duration. An equal amount SHALL keep the existing amount and its duration. A smaller amount SHALL keep the existing pool. The same grant twice SHALL match.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2321 (Player Core pg. 410, Temporary Hit Points)
**Source**: Ruling R-06 (DEV-13) in docs/designer/rules-rulings.md

#### Scenario: A smaller pool keeps the higher amount and its duration

- **GIVEN** 5 temporary Hit Points with 3 start-of-turn ticks left
- **WHEN** 3 temporary Hit Points with 10 ticks are granted
- **THEN** the pool is 5 temporary Hit Points with 3 ticks left

#### Scenario: A tied pool keeps the existing one

- **GIVEN** 5 temporary Hit Points with 2 start-of-turn ticks left
- **WHEN** 5 temporary Hit Points with 8 ticks are granted
- **THEN** the pool is 5 temporary Hit Points with 2 ticks left

#### Scenario: A larger pool replaces the amount and its duration

- **GIVEN** 5 temporary Hit Points with 2 start-of-turn ticks left
- **WHEN** 8 temporary Hit Points with 1 tick are granted
- **THEN** the pool is 8 temporary Hit Points with 1 tick left

#### Scenario: The same temporary grant twice matches

- **GIVEN** 5 temporary Hit Points with 3 ticks left
- **WHEN** a grant of 8 for 1 tick is resolved twice on equal starting states
- **THEN** the two results are equal

### Requirement: Damage removes temporary Hit Points first

Damage SHALL reduce temporary Hit Points before current Hit Points. Only the remainder SHALL reduce current Hit Points, which SHALL NOT go below 0.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2321 (Player Core pg. 410, Temporary Hit Points)
**Source**: Ruling R-06 (DEV-13) in docs/designer/rules-rulings.md

#### Scenario: Seven damage removes 5 temporary Hit Points and 2 Hit Points

- **GIVEN** 5 temporary Hit Points, maximum Hit Points 9, and current Hit Points 9
- **WHEN** a blow of 7 damage is applied
- **THEN** temporary Hit Points are 0 and current Hit Points are 7

#### Scenario: Damage less than temporary Hit Points leaves Hit Points

- **GIVEN** 5 temporary Hit Points and current Hit Points 9
- **WHEN** a blow of 3 damage is applied
- **THEN** temporary Hit Points are 2 and current Hit Points stay 9

#### Scenario: Damage equal to temporary Hit Points clears the pool

- **GIVEN** 5 temporary Hit Points and current Hit Points 9
- **WHEN** a blow of 5 damage is applied
- **THEN** temporary Hit Points are 0 and current Hit Points stay 9

### Requirement: Healing does not restore temporary Hit Points

Healing SHALL add to current Hit Points only and SHALL leave temporary Hit Points unchanged. Temporary Hit Points SHALL NOT raise the maximum.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2321 (Player Core pg. 410, Temporary Hit Points)
**Source**: Ruling R-06 (DEV-13) in docs/designer/rules-rulings.md

#### Scenario: Healing does not restore temporary Hit Points

- **GIVEN** 2 temporary Hit Points, current Hit Points 4, and maximum Hit Points 9
- **WHEN** healing of 3 is applied
- **THEN** current Hit Points are 7 and temporary Hit Points stay 2

#### Scenario: Temporary Hit Points do not raise the maximum

- **GIVEN** 5 temporary Hit Points, current Hit Points 9, and maximum Hit Points 9
- **WHEN** healing of 4 is applied
- **THEN** current Hit Points stay 9 and temporary Hit Points stay 5

### Requirement: Expired temporary Hit Points vanish

The start-of-turn hook SHALL reduce the stored duration by 1 before recovery. At 0 ticks, leftover temporary Hit Points SHALL vanish and current Hit Points SHALL stay unchanged.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2321 (Player Core pg. 410, Temporary Hit Points)
**Source**: Ruling R-06 (DEV-13) in docs/designer/rules-rulings.md

#### Scenario: Unused temporary Hit Points vanish on expiry

- **GIVEN** a conscious combatant, current Hit Points 9, 4 temporary Hit Points with 1 tick left, and a scripted natural 20
- **WHEN** the start-of-turn hook runs
- **THEN** temporary Hit Points are 0, current Hit Points stay 9, and the natural 20 is unused

#### Scenario: A longer duration ticks once

- **GIVEN** a conscious combatant with 4 temporary Hit Points and 2 ticks left
- **WHEN** the start-of-turn hook runs
- **THEN** temporary Hit Points are 4 and 1 tick remains

#### Scenario: Expiry happens before recovery

- **GIVEN** dying 1, current Hit Points 0, wounded 0, 4 temporary Hit Points with 1 tick left, and a scripted natural 11
- **WHEN** the start-of-turn hook runs
- **THEN** temporary Hit Points are 0, current Hit Points are 1, wounded is 1, and the combatant is conscious

### Requirement: Zero temporary Hit Points means none

An amount of 0 SHALL be no temporary Hit Points and SHALL NOT replace a positive pool. A fresh state SHALL have none. After the pool reaches 0, a later greater grant SHALL be stored. A negative amount SHALL return a typed error and SHALL NOT change the state.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2321 (Player Core pg. 410, Temporary Hit Points)
**Source**: Ruling R-06 (DEV-13) in docs/designer/rules-rulings.md

#### Scenario: Fresh temporary Hit Points are none

- **GIVEN** a state just created from a Creature
- **WHEN** temporary Hit Points are read
- **THEN** the amount is 0

#### Scenario: A zero grant does not replace a pool

- **GIVEN** 5 temporary Hit Points with 3 ticks left
- **WHEN** 0 temporary Hit Points with 9 ticks are granted
- **THEN** the pool is still 5 with 3 ticks left

#### Scenario: A grant after the pool reaches 0 is stored

- **GIVEN** temporary Hit Points reduced to 0 by damage
- **WHEN** 4 temporary Hit Points with 2 ticks are granted
- **THEN** the pool is 4 with 2 ticks left

#### Scenario: A negative temporary amount is rejected

- **GIVEN** 5 temporary Hit Points
- **WHEN** a grant of -1 is applied
- **THEN** the result is a typed error and the pool is still 5

### Requirement: A zero duration is not a pool

A grant of temporary Hit Points with 0 start-of-turn ticks SHALL return a typed error and SHALL NOT change the current pool.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2321 (Player Core pg. 410, Temporary Hit Points)

#### Scenario: A zero duration does not replace a pool

- **GIVEN** 5 temporary Hit Points with 3 ticks left
- **WHEN** 8 temporary Hit Points with 0 ticks are granted
- **THEN** the result is a typed error and the pool is still 5 with 3 ticks left

### Requirement: Temporary Hit Points do not remove dying

Gaining or losing temporary Hit Points SHALL NOT change dying, wounded, or consciousness by itself.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2321 (Player Core pg. 410, Temporary Hit Points)

#### Scenario: Granting temporary Hit Points does not remove dying

- **GIVEN** dying 1 and wounded 0
- **WHEN** 5 temporary Hit Points with 2 ticks are granted
- **THEN** the dying value is still 1, wounded is still 0, and the combatant stays unconscious

### Requirement: A blow against the dead changes nothing

Once damage is resolved, a dead combatant SHALL stay dead. An amount that overflows SHALL return a typed error and leave the state unchanged.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2332 (Player Core pg. 412, Massive Damage)

#### Scenario: A blow against the dead leaves the state unchanged

- **GIVEN** a dead combatant at 0 current Hit Points and a scripted d8 of 8 for damage 1d8+1
- **WHEN** that blow is applied
- **THEN** the die is consumed, current Hit Points stay 0, and the combatant stays dead

#### Scenario: An overflowing blow leaves a dying combatant unchanged

- **GIVEN** dying 1, current Hit Points 0, and a scripted d8 of 1 with a bonus of 2147483647
- **WHEN** that blow is applied
- **THEN** the result is a typed overflow error and the dying value is still 1

### Requirement: One conscious side wins

A side is the label the caller gives each combatant. A side with a conscious combatant wins when every other present side has none. No conscious combatant on any side is a mutual defeat, not a victory. A roster of one side, or an empty roster, SHALL NOT be a victory. Roster order SHALL NOT change the result. The same roster SHALL return the same result.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2426 (Player Core pg. 435, End the Encounter)

#### Scenario: One side with no conscious combatant loses

- **GIVEN** side A with one conscious combatant and side B with one dead combatant and one dying combatant
- **WHEN** victory is resolved
- **THEN** side A wins

#### Scenario: Both sides conscious is not victory

- **GIVEN** side A and side B each with one conscious combatant
- **WHEN** victory is resolved
- **THEN** there is no winner and it is not a mutual defeat

#### Scenario: No conscious combatant is a mutual defeat

- **GIVEN** side A and side B each with only dead or dying combatants
- **WHEN** victory is resolved
- **THEN** the result is a mutual defeat and neither side wins

#### Scenario: A single side still fighting is not victory

- **GIVEN** only side A, with one conscious combatant
- **WHEN** victory is resolved
- **THEN** there is no winner

#### Scenario: An empty roster is not victory

- **GIVEN** no combatants
- **WHEN** victory is resolved
- **THEN** there is no winner and it is not a mutual defeat

#### Scenario: Roster order does not change victory

- **GIVEN** a roster where side A wins, and the same combatants listed in reverse
- **WHEN** victory is resolved for each roster
- **THEN** both results name side A

#### Scenario: The same roster twice matches

- **GIVEN** one roster
- **WHEN** victory is resolved twice
- **THEN** the two results are equal

### Requirement: Knockout reports an initiative move

Newly gaining dying SHALL report an initiative move to just before the current turn. The caller SHALL be able to take that report once, which clears it. Death, massive damage, a blow while already dying, and reaching 0 without uses_dying_rules SHALL NOT create the report. This change SHALL NOT reorder combatants.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2324 (Player Core pg. 410, Getting Knocked Out)

#### Scenario: Gaining dying reports an initiative move

- **GIVEN** uses_dying_rules, maximum Hit Points 9, current Hit Points 9, and a scripted d8 of 8 for non-critical damage 1d8+1
- **WHEN** that blow is applied and the report is taken
- **THEN** it says initiative moves, and taking it again says it does not

#### Scenario: Massive damage does not report an initiative move

- **GIVEN** a blow of massive damage that kills
- **WHEN** the report is taken
- **THEN** it says initiative does not move

#### Scenario: Death at 0 without dying rules does not report an initiative move

- **GIVEN** uses_dying_rules false and a blow that reduces the creature to 0 Hit Points
- **WHEN** the report is taken
- **THEN** it says initiative does not move

#### Scenario: Damage while dying does not create an initiative report

- **GIVEN** a creature that is already dying and no pending report
- **WHEN** a blow of 1 damage is applied and the report is taken
- **THEN** it says initiative does not move

### Requirement: The start-of-turn hook does not run the turn

The start-of-turn hook, a blow, and healing SHALL NOT change actions remaining or whether the reaction is ready.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2428 (Player Core pg. 435, Start Your Turn)

#### Scenario: Start of turn does not grant actions

- **GIVEN** dying 1, 2 actions remaining, a ready reaction, and a scripted natural 11
- **WHEN** the start-of-turn hook runs
- **THEN** actions remaining are still 2, the reaction is still ready, and the combatant is conscious at 1 Hit Point

#### Scenario: A knockout does not spend actions

- **GIVEN** 2 actions remaining, a reaction that is not ready, and a blow that sets dying 1
- **WHEN** that blow is applied
- **THEN** actions remaining are still 2 and the reaction is still not ready
