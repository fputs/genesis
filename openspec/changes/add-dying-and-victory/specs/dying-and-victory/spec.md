# dying-and-victory

## Purpose

Resolves knockout, dying, recovery, temporary Hit Points, and which Side has won, without running initiative or Strike.

## ADDED Requirements

### Requirement: A blow to 0 Hit Points sets dying

A creature with uses_dying_rules that drops from above 0 Hit Points to 0 SHALL gain dying 1, or dying 2 when that blow is critical, plus its wounded value, and SHALL set current Hit Points to 0 in that same step. That blow SHALL NOT also apply damage-while-dying. The Creature SHALL stay unchanged. All damage is lethal (DEV-29): there is no nonlethal result, and the blow's critical flag is the only critical signal.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2324 (Player Core pg. 410, Getting Knocked Out)
**Source**: DEV-29. Nonlethal, persistent damage, death effects, save critical-failure dying 2, and regeneration are not modeled.

#### Scenario: A non-critical blow to 0 sets dying 1

- **GIVEN** uses_dying_rules, maximum Hit Points 9, current Hit Points 9, wounded 0, and a scripted d8 of 8 for non-critical damage 1d8+1
- **WHEN** that blow is applied
- **THEN** current Hit Points are 0, vital is Dying(1), wounded is 0, vital is not Dead, and the Creature still has 9 Hit Points

#### Scenario: A critical blow to 0 sets dying 2

- **GIVEN** uses_dying_rules, maximum Hit Points 16, current Hit Points 16, wounded 0, and a scripted d8 of 8 for critical damage 1d8
- **WHEN** that blow is applied
- **THEN** current Hit Points are 0 and vital is Dying(2)
- **AND** wounded is 0 and vital is not Dead

#### Scenario: The dropping blow does not also increase dying

- **GIVEN** uses_dying_rules, maximum Hit Points 16, current Hit Points 16, wounded 0, and a scripted d8 of 8 for critical damage 1d8
- **WHEN** that blow is applied
- **THEN** vital is Dying(2), not Dead from a second increase

#### Scenario: Damage above 0 does not start dying

- **GIVEN** uses_dying_rules, maximum Hit Points 9, current Hit Points 9, and a scripted d8 of 1 for non-critical damage 1d8+1
- **WHEN** that blow is applied
- **THEN** current Hit Points are 7, vital is Alive, and there is no dying value

#### Scenario: Damage past 0 sets dying 1

- **GIVEN** uses_dying_rules, maximum Hit Points 10, current Hit Points 3, wounded 0, and a scripted d8 rolling 7 with bonus 8 for non-critical damage 1d8+8
- **WHEN** that blow is applied
- **THEN** current Hit Points are 0, vital is Dying(1), and wounded is 0

### Requirement: No dying rules means death at 0

A creature without uses_dying_rules at 0 current Hit Points SHALL be Dead, with no dying value and no wounded change. That death SHALL clear temporary Hit Points and their rounds. It SHALL NOT be Unconscious.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2324 (Player Core pg. 410, Getting Knocked Out)

#### Scenario: An enemy at 0 Hit Points does not gain wounded

- **GIVEN** uses_dying_rules false, maximum Hit Points 9, current Hit Points 9, wounded 0, and a scripted d8 of 8 for damage 1d8+1
- **WHEN** that blow is applied
- **THEN** current Hit Points are 0, vital is Dead, wounded is 0, and there is no dying value

### Requirement: Unconscious is 0 Hit Points and not dying

Vital SHALL be Alive, Unconscious, Dying of 1, 2, or 3, or Dead. Unconscious SHALL mean 0 current Hit Points and not dying. It SHALL keep the unconscious penalty and SHALL NOT count as Alive. Positive healing SHALL make it Alive and SHALL leave wounded unchanged. The 10-minute natural wake-up is not modeled.

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=95 (Player Core pg. 446, Unconscious)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2328 (Player Core pg. 411, Losing the Dying Condition)

#### Scenario: Positive healing wakes unconscious without changing wounded

- **GIVEN** vital Unconscious, current Hit Points 0, wounded 1, and maximum Hit Points 9
- **WHEN** healing of 4 is applied
- **THEN** vital is Alive, current Hit Points are 4, and wounded is still 1

### Requirement: Nonzero Hit Points cannot be dying or unconscious

Dying or Unconscious with current Hit Points other than 0 SHALL be rejected. The operation SHALL return a typed error, draw no die, and write nothing. A successful change to Dying SHALL set current Hit Points to 0 in the same step.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2324 (Player Core pg. 410, Getting Knocked Out)

#### Scenario: Dying at nonzero Hit Points is rejected

- **GIVEN** vital Dying(1), current Hit Points 4, and a scripted d8 of 1
- **WHEN** that blow is applied
- **THEN** the result is a typed error, the die is unused, vital is still Dying(1), and current Hit Points stay 4

#### Scenario: Unconscious at nonzero Hit Points is rejected

- **GIVEN** vital Unconscious, current Hit Points 4, and a scripted d8 of 1
- **WHEN** that blow is applied
- **THEN** the result is a typed error, the die is unused, vital is still Unconscious, and current Hit Points stay 4

### Requirement: Dying applies the unconscious penalty

Dying and Unconscious SHALL take the unconscious penalty. Armor Class, Perception, and Reflex SHALL each include a status penalty of -4 in separate collections. Armor Class SHALL also include off-guard at -2 circumstance from ac_modifiers, totaling -6 with no other modifier. Perception and Reflex SHALL stay at -4. Alive and Dead SHALL NOT take the penalty. This change SHALL NOT apply prone or blinded (DEV-31) and SHALL NOT drop held items (DEV-12).

**Source**: https://2e.aonprd.com/Conditions.aspx?ID=95 (Player Core pg. 446, Unconscious)
**Source**: DEV-31 and the DEV-12 amendment are cited. The ledger edits land in a separate docs change.

#### Scenario: A dying creature takes -4 on three statistics

- **GIVEN** vital Dying(1) at 0 Hit Points and no other modifiers
- **WHEN** Armor Class, Perception, and Reflex are totaled separately
- **THEN** each statistic includes a status penalty of -4
- **AND** the Armor Class total is -6 and the Perception and Reflex totals are -4
- **AND** totaling one statistic does not remove the penalty on another

#### Scenario: A conscious creature takes no unconscious penalty

- **GIVEN** vital Alive
- **WHEN** Armor Class, Perception, and Reflex are totaled
- **THEN** none of those totals includes the unconscious penalty

#### Scenario: A dead creature takes no unconscious penalty

- **GIVEN** vital Dead at 0 Hit Points with no dying value
- **WHEN** Armor Class, Perception, and Reflex are totaled
- **THEN** none of those totals includes the unconscious penalty

#### Scenario: Waking removes the unconscious penalty

- **GIVEN** vital Unconscious, current Hit Points 0, wounded 1, and maximum Hit Points 9
- **WHEN** healing of 4 is applied and Armor Class, Perception, and Reflex are totaled
- **THEN** vital is Alive, current Hit Points are 4, wounded is still 1, and none of those totals includes the unconscious penalty

#### Scenario: Unconscious adds off-guard

- **GIVEN** vital Dying(1) and, separately, vital Unconscious, each at 0 Hit Points and with no other modifiers
- **WHEN** Armor Class, Perception, and Reflex are totaled
- **THEN** each Armor Class total is -6, from a status penalty of -4 and an off-guard circumstance penalty of -2
- **AND** each Perception total and each Reflex total is -4 and does not include that -2
- **AND** neither combatant is prone

### Requirement: Damage to unconscious starts dying

Damage greater than 0 to an Unconscious creature with uses_dying_rules SHALL apply Getting Knocked Out: dying 1, or 2 when the blow is critical, plus wounded, with current Hit Points staying 0. That gain SHALL set the initiative report (R-08).

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2324 (Player Core pg. 410, Getting Knocked Out)
**Source**: Ruling R-08 in docs/designer/rules-rulings.md

#### Scenario: Wounded 1 and 1 damage sets dying 2

- **GIVEN** vital Unconscious, current Hit Points 0, wounded 1, uses_dying_rules, maximum Hit Points 9, and a scripted d8 of 1 for non-critical damage 1d8
- **WHEN** that blow is applied and the initiative report is taken
- **THEN** vital is Dying(2), wounded is still 1, current Hit Points stay 0, and the report is set

### Requirement: Recovery checks use DC 10 plus dying

While Dying, start_of_turn SHALL roll one flat check at DC 10 plus dying and SHALL add no modifier. Critical success SHALL reduce dying by 2, success by 1, failure SHALL increase it by 1, and critical failure by 2. Alive, Unconscious, and Dead SHALL NOT roll. Equal inputs SHALL produce the same state, the same dice cursor, and the same return value. Doomed is not modeled, so the death threshold stays 4 (DEV-28).

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2326 (Player Core pg. 411, Recovery Checks)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2278 (Player Core pg. 400, Flat Checks)
**Source**: DEV-28. The doomed condition is not modeled (Player Core pg. 443).

#### Scenario: Recovery critical success reduces dying by 2

- **GIVEN** vital Dying(3) and a scripted natural 20
- **WHEN** start_of_turn runs
- **THEN** the flat check DC is 13, the degree is critical success, and vital is Dying(1)

#### Scenario: Recovery success reduces dying by 1

- **GIVEN** vital Dying(2) and a scripted natural 12
- **WHEN** start_of_turn runs
- **THEN** the flat check DC is 12, the degree is success, and vital is Dying(1)

#### Scenario: Recovery failure increases dying by 1

- **GIVEN** vital Dying(1) and a scripted natural 2
- **WHEN** start_of_turn runs
- **THEN** the flat check DC is 11, the degree is failure, and vital is Dying(2)

#### Scenario: Recovery critical failure increases dying by 2

- **GIVEN** vital Dying(1) and a scripted natural 1
- **WHEN** start_of_turn runs
- **THEN** the flat check DC is 11, the degree is critical failure, and vital is Dying(3)

#### Scenario: A conscious creature does not roll recovery

- **GIVEN** vital Alive and a scripted natural 20
- **WHEN** start_of_turn runs
- **THEN** the return is None, no recovery check is rolled, and the natural 20 is unused

#### Scenario: A dead creature does not roll recovery

- **GIVEN** vital Dead and a scripted natural 20
- **WHEN** start_of_turn runs
- **THEN** the return is None, no recovery check is rolled, and the natural 20 is unused

#### Scenario: The same recovery twice matches

- **GIVEN** vital Dying(2) and a scripted natural 12
- **WHEN** start_of_turn is resolved twice on equal starting states
- **THEN** the resulting states are equal, the dice cursors are equal, and the return values are equal

### Requirement: Recovery to 0 stays unconscious

When recovery reduces dying to 0 or below, the combatant SHALL lose dying, leave current Hit Points at 0, increase wounded by 1, and become Unconscious. It SHALL NOT set current Hit Points to 1 and SHALL NOT become Alive. Positive healing is the only wake. The 10-minute natural wake-up is not modeled. A result that stays at 1, 2, or 3 SHALL NOT change Hit Points or wounded. Recovery itself SHALL NOT change temporary Hit Points; expiry may already have cleared them.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2328 (Player Core pg. 411, Losing the Dying Condition)

#### Scenario: Recovery to 0 stays unconscious at 0 Hit Points

- **GIVEN** vital Dying(1), current Hit Points 0, wounded 0, 4 temporary Hit Points with 2 ticks left, and a scripted natural 11
- **WHEN** start_of_turn runs
- **THEN** the degree is success, vital is Unconscious, current Hit Points are 0, wounded is 1, and temporary Hit Points are 4 with 1 tick left

#### Scenario: A recovery that stays dying does not change Hit Points

- **GIVEN** vital Dying(2), current Hit Points 0, wounded 0, and a scripted natural 12
- **WHEN** start_of_turn runs
- **THEN** vital is Dying(1), current Hit Points stay 0, wounded stays 0, and vital is not Alive

#### Scenario: A natural 20 at dying 1 stays unconscious

- **GIVEN** vital Dying(1), current Hit Points 0, wounded 0, and a scripted natural 20
- **WHEN** start_of_turn runs
- **THEN** the DC is 11, the degree is critical success, vital is Unconscious, current Hit Points are 0, and wounded is 1

#### Scenario: A natural 19 at dying 3 reduces dying to 2

- **GIVEN** vital Dying(3), current Hit Points 0, wounded 0, and a scripted natural 19
- **WHEN** start_of_turn runs
- **THEN** the DC is 13, the degree is success, and vital is Dying(2)

#### Scenario: A natural 10 at dying 1 increases dying to 2

- **GIVEN** vital Dying(1), current Hit Points 0, wounded 0, and a scripted natural 10
- **WHEN** start_of_turn runs
- **THEN** the DC is 11, the degree is failure, and vital is Dying(2)

### Requirement: Wounded increases when dying is lost

Wounded SHALL be an i32. Losing dying SHALL increase wounded by 1. Gaining dying, damage while dying, death, and healing an Unconscious creature SHALL NOT increase wounded. Healing to maximum Hit Points SHALL NOT reduce wounded. A knockout whose dying sum does not fit in i32 SHALL be Dead and Ok, with wounded left unchanged. Healing or recovery that would increase wounded past i32::MAX SHALL return Overflow and SHALL leave the state unchanged; a recovery die already drawn stays drawn.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2328 (Player Core pg. 411, Losing the Dying Condition)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=99 (Player Core pg. 447, Wounded)

#### Scenario: Losing dying increases wounded by 1

- **GIVEN** vital Dying(1), wounded 0, and a scripted natural 11
- **WHEN** recovery removes dying
- **THEN** wounded is 1 and vital is Unconscious

#### Scenario: Gaining dying does not increase wounded

- **GIVEN** uses_dying_rules, wounded 0, maximum Hit Points 9, current Hit Points 9, and a scripted d8 of 8 for non-critical damage 1d8+1
- **WHEN** that blow is applied
- **THEN** vital is Dying(1) and wounded is still 0

#### Scenario: Damage while dying does not increase wounded

- **GIVEN** vital Dying(1), wounded 2, maximum Hit Points 9, and a scripted d8 of 1 for non-critical damage 1d8
- **WHEN** that blow is applied
- **THEN** vital is Dying(2) and wounded is still 2

#### Scenario: Death does not increase wounded

- **GIVEN** vital Dying(3), wounded 1, maximum Hit Points 9, and a scripted d8 of 1 for non-critical damage 1d8
- **WHEN** that blow is applied
- **THEN** vital is Dead and wounded is still 1

#### Scenario: Full Hit Points do not clear wounded

- **GIVEN** vital Alive, wounded 2, maximum Hit Points 9, and current Hit Points 9
- **WHEN** healing of 4 is applied
- **THEN** current Hit Points are 9 and wounded is still 2

#### Scenario: Maximum wounded dies on the knockout

- **GIVEN** uses_dying_rules, wounded 2147483647, maximum Hit Points 9, current Hit Points 9, and a scripted d8 of 8 for non-critical damage 1d8+1
- **WHEN** that blow is applied
- **THEN** the result is Ok, vital is Dead, no dying value is stored, and wounded is still 2147483647

#### Scenario: Healing at maximum wounded returns overflow

- **GIVEN** vital Dying(1), current Hit Points 0, wounded 2147483647, and maximum Hit Points 9
- **WHEN** healing of 1 is applied
- **THEN** the result is Overflow, vital is still Dying(1), current Hit Points stay 0, and wounded is still 2147483647

#### Scenario: Recovery at maximum wounded returns overflow

- **GIVEN** vital Dying(1), current Hit Points 0, wounded 2147483647, 4 temporary Hit Points with 2 ticks left, and a scripted natural 11
- **WHEN** start_of_turn runs
- **THEN** the result is Overflow, the die is consumed, vital is still Dying(1), current Hit Points stay 0, wounded is still 2147483647, and 4 temporary Hit Points with 2 ticks remain

### Requirement: Wounded adds to the new dying value

When an Alive or Unconscious creature gains dying, the value SHALL be 1 plus wounded, or 2 plus wounded when the blow is critical.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2324 (Player Core pg. 410, Getting Knocked Out)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=99 (Player Core pg. 447, Wounded)

#### Scenario: Wounded increases the next dying value

- **GIVEN** uses_dying_rules, wounded 1, maximum Hit Points 9, current Hit Points 9, and a scripted d8 of 8 for non-critical damage 1d8+1
- **WHEN** that blow is applied
- **THEN** vital is Dying(2) and wounded is still 1

#### Scenario: A critical blow adds wounded to 2

- **GIVEN** uses_dying_rules, wounded 1, maximum Hit Points 16, current Hit Points 16, and a scripted d8 of 8 for critical damage 1d8
- **WHEN** that blow is applied
- **THEN** vital is Dying(3) and wounded is still 1

### Requirement: Damage while dying increases dying

A blow whose damage is greater than 0 against a creature that is already Dying SHALL increase dying by 1, or by 2 when the blow is critical, even when temporary Hit Points absorb the whole blow (R-10). A resolved blow of 0 SHALL NOT change dying. A die that DEV-14 floors to 1 SHALL count as damage 1. The blow's critical flag SHALL NOT also apply a save critical failure.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2327 (Player Core pg. 411, Taking Damage while Dying)
**Source**: Ruling R-10 in docs/designer/rules-rulings.md

#### Scenario: Damage while dying increases dying by 1

- **GIVEN** vital Dying(1), current Hit Points 0, maximum Hit Points 9, and a scripted d8 of 1 for non-critical damage 1d8
- **WHEN** that blow is applied
- **THEN** vital is Dying(2) and current Hit Points stay 0

#### Scenario: A critical blow while dying increases dying by 2

- **GIVEN** vital Dying(1), current Hit Points 0, maximum Hit Points 9, and a scripted d8 of 1 for critical damage 1d8
- **WHEN** that blow is applied
- **THEN** vital is Dying(3)

#### Scenario: A resolved blow of 0 does not increase dying

- **GIVEN** vital Dying(1) and current Hit Points 0
- **WHEN** apply_resolved_blow is called with amount 0 and not critical
- **THEN** no die is rolled, the damage is 0, and vital is still Dying(1)

#### Scenario: A floored die of 1 increases dying

- **GIVEN** vital Dying(1), current Hit Points 0, and a scripted d4 rolling 1 with a bonus of -5
- **WHEN** that blow is applied
- **THEN** the damage is 1 under DEV-14 and vital is Dying(2)

#### Scenario: Temporary Hit Points do not block the dying increase

- **GIVEN** vital Dying(1), current Hit Points 0, 5 temporary Hit Points with 3 ticks left, maximum Hit Points 9, and a scripted d4 of 3 for non-critical damage 1d4
- **WHEN** that blow is applied
- **THEN** temporary Hit Points are 2 with 3 ticks left, current Hit Points stay 0, and vital is Dying(2)

#### Scenario: A blow absorbed down to 1 Hit Point does not start dying

- **GIVEN** uses_dying_rules, vital Alive, current Hit Points 3, maximum Hit Points 9, 5 temporary Hit Points with 3 ticks left, and a scripted d8 of 7 for non-critical damage 1d8
- **WHEN** that blow is applied
- **THEN** temporary Hit Points are 0, rounds are 0, current Hit Points are 1, and vital is Alive with no dying value

### Requirement: Dying 4 is death

A dying value of 4 or higher SHALL become death immediately, including a recovery that reaches 4. The threshold stays 4 because doomed is not modeled (DEV-28). The state SHALL NOT keep a dying value. Vital SHALL be Dead, not Alive. Wounded SHALL stay as it was. Every death SHALL clear temporary Hit Points and their rounds (R-10).

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2325 (Player Core pg. 411, Dying)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=69 (Player Core pg. 443, Dying)
**Source**: Ruling R-10 in docs/designer/rules-rulings.md. DEV-28.

#### Scenario: Dying 3 and one damage kills

- **GIVEN** vital Dying(3), wounded 0, maximum Hit Points 9, 4 temporary Hit Points with 2 ticks left, and a scripted d8 of 1 for non-critical damage 1d8
- **WHEN** that blow is applied
- **THEN** vital is Dead, no dying value is stored, wounded is 0, temporary Hit Points are 0, and rounds are 0

#### Scenario: A critical blow at dying 3 kills

- **GIVEN** vital Dying(3), maximum Hit Points 9, current Hit Points 0, and a scripted d8 of 1 for critical damage 1d8
- **WHEN** that blow is applied
- **THEN** vital is Dead

#### Scenario: Wounded 3 dies on the knockout

- **GIVEN** uses_dying_rules, wounded 3, maximum Hit Points 9, current Hit Points 9, 4 temporary Hit Points with 2 ticks left, and a scripted d8 of 8 for non-critical damage 1d8+1
- **WHEN** that blow is applied
- **THEN** vital is Dead, no dying value is stored, wounded is still 3, temporary Hit Points are 0, and rounds are 0

#### Scenario: A recovery critical failure at dying 2 kills

- **GIVEN** vital Dying(2), current Hit Points 0, 4 temporary Hit Points with 2 ticks left, and a scripted natural 1
- **WHEN** start_of_turn runs
- **THEN** the degree is critical failure, vital is Dead, temporary Hit Points are 0, and rounds are 0

#### Scenario: A natural 2 at dying 3 kills

- **GIVEN** vital Dying(3), current Hit Points 0, wounded 0, and a scripted natural 2
- **WHEN** start_of_turn runs
- **THEN** the DC is 13, the degree is critical failure, and vital is Dead

#### Scenario: A natural 12 at dying 3 kills

- **GIVEN** vital Dying(3), current Hit Points 0, wounded 0, and a scripted natural 12
- **WHEN** start_of_turn runs
- **THEN** the DC is 13, the degree is failure, and vital is Dead

#### Scenario: A dead creature is not conscious

- **GIVEN** a combatant who just died from dying 4
- **WHEN** consciousness is read
- **THEN** vital is Dead and the combatant is not Alive

### Requirement: Massive damage kills outright

Damage greater than or equal to twice maximum Hit Points SHALL kill immediately. The check SHALL use that damage, not Hit Points lost, and SHALL ignore temporary Hit Points. The combatant SHALL NOT gain dying. Wounded SHALL NOT change. Death SHALL set current Hit Points to 0 and SHALL clear temporary Hit Points and their rounds.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2332 (Player Core pg. 412, Massive Damage). FIX-15. R-06 (DEV-13).

#### Scenario: Damage equal to twice maximum Hit Points kills

- **GIVEN** uses_dying_rules, maximum Hit Points 10, current Hit Points 10, wounded 0, and a scripted d8 of 8 for non-critical damage 1d8+12
- **WHEN** that blow is applied
- **THEN** vital is Dead, no dying value is stored, wounded is 0, and current Hit Points are 0

#### Scenario: Damage one below twice maximum does not kill

- **GIVEN** uses_dying_rules, maximum Hit Points 10, current Hit Points 10, and a scripted d8 of 8 for non-critical damage 1d8+11
- **WHEN** that blow is applied
- **THEN** vital is Dying(1) and is not Dead

#### Scenario: Massive damage ignores temporary Hit Points

- **GIVEN** uses_dying_rules, maximum Hit Points 10, current Hit Points 10, 30 temporary Hit Points with 4 ticks left, and a scripted d8 of 8 for non-critical damage 1d8+12
- **WHEN** that blow is applied
- **THEN** vital is Dead, current Hit Points are 0, temporary Hit Points are 0, and rounds are 0

#### Scenario: A critical blow doubles before the massive check

- **GIVEN** uses_dying_rules, maximum Hit Points 10, current Hit Points 10, and a scripted d8 of 8 for critical damage 1d8+2
- **WHEN** that blow is applied
- **THEN** the doubled damage is 20 and vital is Dead

#### Scenario: The same blow without doubling does not kill

- **GIVEN** uses_dying_rules, maximum Hit Points 10, current Hit Points 10, and a scripted d8 of 8 for non-critical damage 1d8+2
- **WHEN** that blow is applied
- **THEN** the damage is 10, current Hit Points are 0, and vital is Dying(1)

#### Scenario: Massive damage while dying kills without another increase

- **GIVEN** uses_dying_rules, vital Dying(1), maximum Hit Points 10, current Hit Points 0, and a scripted d8 of 8 for non-critical damage 1d8+12
- **WHEN** that blow is applied
- **THEN** vital is Dead and the dying value is not 2

### Requirement: Blow resolution checks massive damage first

One blow SHALL judge massive damage before it spends temporary Hit Points. If the blow does not kill, it SHALL spend temporary Hit Points, then current Hit Points, then dying or death at 0. Draining temporary Hit Points to 0 SHALL also set their rounds to 0.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2332 (Player Core pg. 412, Massive Damage)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2321 (Player Core pg. 410, Temporary Hit Points)

#### Scenario: Massive damage is judged before temporary Hit Points

- **GIVEN** uses_dying_rules false, maximum Hit Points 10, current Hit Points 10, 30 temporary Hit Points with 4 ticks left, and a scripted d8 of 8 for non-critical damage 1d8+12
- **WHEN** that blow is applied
- **THEN** vital is Dead, current Hit Points are 0, temporary Hit Points are 0, and rounds are 0
- **AND** the result is not Alive at 10, which is where the creature would stand if 30 temporary Hit Points had covered 20 damage

### Requirement: Healing removes dying

A positive heal on a Dying combatant SHALL add that amount, cap it at maximum Hit Points, remove dying, increase wounded by 1, and leave vital Alive. It SHALL NOT set Hit Points to 1. Temporary Hit Points SHALL stay unchanged. Healing an Unconscious combatant SHALL leave wounded unchanged.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2328 (Player Core pg. 411, Losing the Dying Condition)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2320 (Player Core pg. 410, Healing)

#### Scenario: Healing removes dying and sets the healed Hit Points

- **GIVEN** vital Dying(1), current Hit Points 0, wounded 0, maximum Hit Points 9, and 4 temporary Hit Points
- **WHEN** healing of 4 is applied
- **THEN** current Hit Points are 4, temporary Hit Points are 4, wounded is 1, there is no dying value, and vital is Alive

#### Scenario: Healing is capped at maximum Hit Points

- **GIVEN** vital Dying(1), current Hit Points 0, wounded 0, and maximum Hit Points 9
- **WHEN** healing of 100 is applied
- **THEN** current Hit Points are 9, wounded is 1, and vital is Alive

#### Scenario: Healing to the Hit Point cap leaves the combatant alive

- **GIVEN** vital Dying(1), current Hit Points 0, wounded 0, and maximum Hit Points 2147483647
- **WHEN** healing of 2147483647 is applied
- **THEN** the result is Ok, current Hit Points are 2147483647, wounded is 1, and vital is Alive

### Requirement: Healing cannot revive the dead

Healing a dead combatant SHALL return a typed error and SHALL NOT change Hit Points, wounded, temporary Hit Points, or death.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2320 (Player Core pg. 410, Healing)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2329 (Player Core pg. 411, Death)

#### Scenario: Healing the dead returns an error

- **GIVEN** vital Dead at 0 current Hit Points, wounded 1, and 0 temporary Hit Points
- **WHEN** healing of 5 is applied
- **THEN** the result is a typed error, current Hit Points stay 0, wounded stays 1, and vital stays Dead

### Requirement: Non-positive healing is rejected

A heal of 0 or less SHALL return a typed error and SHALL NOT change the state. It SHALL NOT remove dying.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2320 (Player Core pg. 410, Healing)

#### Scenario: A zero heal does not remove dying

- **GIVEN** vital Dying(1) and current Hit Points 0
- **WHEN** healing of 0 is applied
- **THEN** the result is a typed error and vital is still Dying(1)

#### Scenario: A negative heal does not change Hit Points

- **GIVEN** vital Alive at 9 current Hit Points
- **WHEN** healing of -1 is applied
- **THEN** the result is a typed error and current Hit Points stay 9

### Requirement: Temporary Hit Points keep the higher pool

A new temporary amount SHALL replace the current pool only when it is strictly greater, and it SHALL keep the new duration. An equal amount SHALL keep the existing amount and its duration. A smaller amount SHALL keep the existing pool. The same grant twice SHALL produce the same state, the same dice cursor, and the same return value.

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
- **THEN** the resulting states are equal, the dice cursors are equal, and the return values are equal

### Requirement: Damage removes temporary Hit Points first

Damage SHALL reduce temporary Hit Points before current Hit Points. Only the remainder SHALL reduce current Hit Points, which SHALL NOT go below 0. When the amount reaches 0, the rounds SHALL become 0.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2321 (Player Core pg. 410, Temporary Hit Points)
**Source**: Ruling R-06 (DEV-13) in docs/designer/rules-rulings.md

#### Scenario: Seven damage removes 5 temporary Hit Points and 2 Hit Points

- **GIVEN** 5 temporary Hit Points with 3 ticks left, maximum Hit Points 9, and current Hit Points 9
- **WHEN** a scripted d8 of 7 for damage 1d8 is applied
- **THEN** temporary Hit Points are 0, rounds are 0, and current Hit Points are 7

#### Scenario: Damage less than temporary Hit Points leaves Hit Points

- **GIVEN** 5 temporary Hit Points with 3 ticks left and current Hit Points 9
- **WHEN** a scripted d4 of 3 for damage 1d4 is applied
- **THEN** temporary Hit Points are 2 with 3 ticks left and current Hit Points stay 9

#### Scenario: Damage equal to temporary Hit Points clears the pool

- **GIVEN** 5 temporary Hit Points with 3 ticks left and current Hit Points 9
- **WHEN** a scripted d8 of 5 for damage 1d8 is applied
- **THEN** temporary Hit Points are 0, rounds are 0, and current Hit Points stay 9

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

The hook SHALL reduce the recipient's stored duration by 1 before recovery (R-06, DEV-32). RAW counts that duration from the source's turn; ticking at the recipient's start is the engineering simplification. At 0 ticks, leftover temporary Hit Points SHALL vanish, their rounds SHALL be 0, and current Hit Points SHALL stay unchanged.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2321 (Player Core pg. 410, Temporary Hit Points)
**Source**: Ruling R-06 (DEV-13) and DEV-32 in docs/designer. DEV-32 lands in a separate docs change.

#### Scenario: Unused temporary Hit Points vanish on expiry

- **GIVEN** vital Alive, current Hit Points 9, 4 temporary Hit Points with 1 tick left, and a scripted natural 20
- **WHEN** start_of_turn runs
- **THEN** the return is None, temporary Hit Points are 0, rounds are 0, current Hit Points stay 9, and the natural 20 is unused

#### Scenario: A longer duration ticks once

- **GIVEN** vital Alive with 4 temporary Hit Points and 2 ticks left
- **WHEN** start_of_turn runs
- **THEN** the return is None, temporary Hit Points are 4, and 1 tick remains

#### Scenario: Expiry happens before recovery

- **GIVEN** vital Dying(1), current Hit Points 0, wounded 0, 4 temporary Hit Points with 1 tick left, and a scripted natural 11
- **WHEN** start_of_turn runs
- **THEN** temporary Hit Points are 0, rounds are 0, current Hit Points are 0, wounded is 1, and vital is Unconscious

### Requirement: Start of turn is not idempotent

start_of_turn(state, creature, dice) SHALL return Result<Option<FlatRoll>, HpError>. None SHALL mean Alive, Unconscious, or Dead rolled no recovery check. A second call SHALL tick again and MAY roll again. The function SHALL NOT be idempotent.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2428 (Player Core pg. 435, Start Your Turn)

#### Scenario: A second start of turn ticks again

- **GIVEN** vital Alive, 4 temporary Hit Points with 2 ticks left, and a scripted natural 20
- **WHEN** start_of_turn runs and then runs again on that result
- **THEN** the first return is None and 1 tick remains, with the natural 20 unused
- **AND** the second return is None, temporary Hit Points are 0, and rounds are 0
- **AND** the state after two calls is not the state after one call

### Requirement: Zero temporary Hit Points means none

An amount of 0 SHALL be no temporary Hit Points and SHALL NOT replace a positive pool. A fresh state SHALL have none, with rounds 0. After the pool reaches 0, a later greater grant SHALL be stored. A negative amount SHALL return a typed error and SHALL NOT change the state.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2321 (Player Core pg. 410, Temporary Hit Points)
**Source**: Ruling R-06 (DEV-13) in docs/designer/rules-rulings.md

#### Scenario: Fresh temporary Hit Points are none

- **GIVEN** a state just created from a Creature
- **WHEN** temporary Hit Points are read
- **THEN** the amount is 0 and the rounds are 0

#### Scenario: A zero grant does not replace a pool

- **GIVEN** 5 temporary Hit Points with 3 ticks left
- **WHEN** 0 temporary Hit Points with 9 ticks are granted
- **THEN** the pool is still 5 with 3 ticks left

#### Scenario: A grant after the pool reaches 0 is stored

- **GIVEN** temporary Hit Points reduced to 0 by damage, rounds 0
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

Gaining or losing temporary Hit Points SHALL NOT change dying, wounded, or vital by itself.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2321 (Player Core pg. 410, Temporary Hit Points)

#### Scenario: Granting temporary Hit Points does not remove dying

- **GIVEN** vital Dying(1) and wounded 0
- **WHEN** 5 temporary Hit Points with 2 ticks are granted
- **THEN** vital is still Dying(1) and wounded is still 0

### Requirement: A blow against the dead changes nothing

Once damage is resolved, a Dead combatant SHALL stay Dead. The die SHALL be consumed. A massive blow against the Dead SHALL also leave the state unchanged. An amount that overflows SHALL return a typed error and leave the state unchanged. A resolved blow of 2147483647 against maximum Hit Points 2147483647 SHALL NOT be massive.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2329 (Player Core pg. 411, Death)

#### Scenario: A blow against the dead leaves the state unchanged

- **GIVEN** vital Dead at 0 current Hit Points and a scripted d8 of 8 for damage 1d8+1
- **WHEN** that blow is applied
- **THEN** the die is consumed, current Hit Points stay 0, and vital stays Dead

#### Scenario: Massive damage against the dead leaves the state unchanged

- **GIVEN** vital Dead, maximum Hit Points 10, current Hit Points 0, 4 temporary Hit Points with 2 ticks left, and a scripted d8 of 8 for damage 1d8+12
- **WHEN** that blow is applied
- **THEN** the die is consumed, vital stays Dead, current Hit Points stay 0, and temporary Hit Points stay 4 with 2 ticks left

#### Scenario: An overflowing blow leaves a dying combatant unchanged

- **GIVEN** vital Dying(1), current Hit Points 0, and a scripted d8 of 1 with a bonus of 2147483647
- **WHEN** that blow is applied
- **THEN** the result is a typed overflow error and vital is still Dying(1)

#### Scenario: Maximum Hit Points and a maximum blow are not massive

- **GIVEN** uses_dying_rules, maximum Hit Points 2147483647, current Hit Points 2147483647, and wounded 0
- **WHEN** apply_resolved_blow is called with amount 2147483647 and not critical
- **THEN** the result is Ok, current Hit Points are 0, vital is Dying(1), and no die is rolled

### Requirement: One Side wins

Victory SHALL query combatant id, Side, and state. Side SHALL be Side::Pc or Side::Enemy from add-encounter-turns. A Side SHALL be defeated when none of its combatants is Alive (DEV-30). Fewer than two present Sides SHALL be Ongoing. Exactly one Side with an Alive combatant SHALL be Victory for that Side. Both Sides present and none Alive SHALL be MutualDefeat. Roster order SHALL NOT change the result. Equal rosters SHALL return the same outcome.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2426 (Player Core pg. 435, End the Encounter)
**Source**: DEV-30, cited here. The ledger row lands in a separate docs change.

#### Scenario: One side with no conscious combatant loses

- **GIVEN** Side::Pc with one Alive combatant, and Side::Enemy with one Dead combatant and one Dying combatant
- **WHEN** victory is queried
- **THEN** the result is Victory for Side::Pc

#### Scenario: A dying side loses to an alive side

- **GIVEN** Side::Pc with one Dying combatant and Side::Enemy with one Alive combatant
- **WHEN** victory is queried
- **THEN** the result is Victory for Side::Enemy

#### Scenario: Both sides conscious is not victory

- **GIVEN** Side::Pc and Side::Enemy each with one Alive combatant
- **WHEN** victory is queried
- **THEN** the result is Ongoing

#### Scenario: No conscious combatant is a mutual defeat

- **GIVEN** Side::Pc and Side::Enemy each with only Dead or Dying combatants
- **WHEN** victory is queried
- **THEN** the result is MutualDefeat

#### Scenario: A single side still fighting is not victory

- **GIVEN** only Side::Pc, with one Alive combatant
- **WHEN** victory is queried
- **THEN** the result is Ongoing

#### Scenario: A single wiped side is not a mutual defeat

- **GIVEN** only Side::Pc, with every combatant Dead
- **WHEN** victory is queried
- **THEN** the result is Ongoing

#### Scenario: An empty roster is not victory

- **GIVEN** no combatants
- **WHEN** victory is queried
- **THEN** the result is Ongoing

#### Scenario: Roster order does not change victory

- **GIVEN** a roster where Side::Pc wins, and the same combatants listed in reverse
- **WHEN** victory is queried for each roster
- **THEN** both results are Victory for Side::Pc

#### Scenario: The same roster twice matches

- **GIVEN** one roster
- **WHEN** victory is queried twice
- **THEN** the two outcomes are equal and the roster is unchanged

### Requirement: Victory is a query

The victory query SHALL be pure and SHALL NOT store a winner. It SHALL be read after a blow, a recovery, or a heal. Healing MAY turn Victory back into Ongoing. The encounter loop decides closure.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2426 (Player Core pg. 435, End the Encounter)
**Source**: DEV-30

#### Scenario: Blows in either order end in mutual defeat

- **GIVEN** two equal rosters, each with Side::Pc and Side::Enemy Alive at 5 current Hit Points, maximum 9, uses_dying_rules true
- **WHEN** each creature takes a scripted d8 of 5 for non-critical damage 1d8, Pc first on one roster and Enemy first on the other
- **THEN** after only the first blow the undamaged Side wins
- **AND** after the second blow each roster is MutualDefeat

#### Scenario: Recovery to unconscious is a mutual defeat

- **GIVEN** Side::Pc at Dying(1) and Side::Enemy at Dying(1), both at 0 Hit Points, and a scripted natural 11 for Side::Pc
- **WHEN** Side::Pc's start_of_turn runs and victory is queried
- **THEN** Side::Pc is Unconscious at 0 Hit Points, Side::Enemy is still Dying(1), and the result is MutualDefeat

#### Scenario: Healing a downed creature returns the fight to ongoing

- **GIVEN** Side::Pc at Dying(1) with 0 Hit Points and maximum 9, and Side::Enemy Alive, so the query is Victory for Side::Enemy
- **WHEN** Side::Pc receives healing of 4 and victory is queried again
- **THEN** Side::Pc is Alive and the result is Ongoing

### Requirement: Knockout sets an initiative report

Newly gaining dying SHALL set a report the caller can take once. Taking it SHALL clear it. Death SHALL clear a stale report. Healing SHALL NOT clear it. Massive damage, reaching 0 without uses_dying_rules, and damage while already dying SHALL NOT create a report. This change SHALL NOT reorder combatants.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2324 (Player Core pg. 410, Getting Knocked Out)

#### Scenario: Gaining dying reports an initiative move

- **GIVEN** uses_dying_rules, maximum Hit Points 9, current Hit Points 9, and a scripted d8 of 8 for non-critical damage 1d8+1
- **WHEN** that blow is applied and the report is taken, then taken again
- **THEN** the first take is true and the second take is false

#### Scenario: Massive damage does not report an initiative move

- **GIVEN** uses_dying_rules, maximum Hit Points 10, current Hit Points 10, no pending report, and a scripted d8 of 8 for non-critical damage 1d8+12
- **WHEN** that blow is applied and the report is taken
- **THEN** the take is false

#### Scenario: Death at 0 without dying rules does not report an initiative move

- **GIVEN** uses_dying_rules false, maximum Hit Points 9, current Hit Points 9, and a scripted d8 of 8 for damage 1d8+1
- **WHEN** that blow is applied and the report is taken
- **THEN** the die is consumed, vital is Dead, and the take is false

#### Scenario: Damage while dying does not create an initiative report

- **GIVEN** vital Dying(1), current Hit Points 0, no pending report, and a scripted d8 of 1 for non-critical damage 1d8
- **WHEN** that blow is applied and the report is taken
- **THEN** vital is Dying(2) and the take is false

#### Scenario: Two dropped combatants each report once

- **GIVEN** two combatants, each with uses_dying_rules, maximum Hit Points 9, current Hit Points 9, and wounded 0
- **WHEN** each takes a scripted d8 of 8 for non-critical damage 1d8+1, and each report is taken twice
- **THEN** each first take is true and each second take is false
- **AND** taking one combatant's report does not clear the other's

#### Scenario: Healing does not clear the initiative report

- **GIVEN** uses_dying_rules, maximum Hit Points 10, current Hit Points 10, and wounded 0
- **WHEN** a scripted d8 of 8 for non-critical damage 1d8+2 is applied, healing of 4 is applied, and the report is taken
- **THEN** vital is Alive at 4 Hit Points and the take is true

#### Scenario: Death clears a stale initiative report

- **GIVEN** uses_dying_rules, maximum Hit Points 10, current Hit Points 10, and wounded 0
- **WHEN** a scripted d8 of 8 for non-critical damage 1d8+2 is applied, healing of 4 is applied, then a scripted d8 of 8 for non-critical damage 1d8+12 is applied, and the report is taken
- **THEN** vital is Dead and the take is false

### Requirement: The start-of-turn hook does not run the turn

start_of_turn, a blow, and healing SHALL NOT change actions remaining or whether the reaction is ready.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2428 (Player Core pg. 435, Start Your Turn)

#### Scenario: Start of turn does not grant actions

- **GIVEN** vital Dying(1), current Hit Points 0, wounded 0, 2 actions remaining, a ready reaction, and a scripted natural 11
- **WHEN** start_of_turn runs
- **THEN** actions remaining are still 2, the reaction is still ready, current Hit Points are 0, wounded is 1, and vital is Unconscious

#### Scenario: A knockout does not spend actions

- **GIVEN** 2 actions remaining, a reaction that is not ready, and a blow that sets Dying(1)
- **WHEN** that blow is applied
- **THEN** actions remaining are still 2 and the reaction is still not ready

#### Scenario: Healing does not change actions

- **GIVEN** vital Unconscious, current Hit Points 0, wounded 1, maximum Hit Points 9, 2 actions remaining, and a ready reaction
- **WHEN** healing of 4 is applied
- **THEN** actions remaining are still 2 and the reaction is still ready
