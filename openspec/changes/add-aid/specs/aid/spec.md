# aid

## Purpose

Prepares to Aid one named ally with one action on your turn, then spends the reaction to roll DC 15 and return a circumstance modifier for that one attack roll.

## ADDED Requirements

### Requirement: The same aid twice matches

The same inputs SHALL return the same result. A call SHALL NOT read a file, the network, a clock, or a global generator. A fallible call SHALL return a typed error and SHALL NOT panic. The rules crate SHALL gain no dependency. Preparation SHALL be AidPreparation::NotPrepared or AidPreparation::Prepared naming one CombatantId. The circumstance SHALL be an AidCircumstance. Neither type is a bool.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2292 (Player Core pg. 416, Aid)

#### Scenario: The same aid twice matches

- **GIVEN** two encounters, each with Alive Pc CombatantId(1) and Alive Pc CombatantId(2), both Perception +0, aid modifier +7, AidRank::BelowMaster, and a script of initiative naturals 10 then 8 followed by a natural 10
- **WHEN** CombatantId(1) prepares for CombatantId(2) on their turn, the turn ends, CombatantId(2)'s turn starts, and CombatantId(1) aids CombatantId(2)
- **THEN** both totals are 17, both degrees are success, and both circumstances are Plus1

### Requirement: Preparation is one action on your turn

prepare_to_aid SHALL name one ally. It SHALL spend 1 action through spend_actions and then set AidPreparation::Prepared for that ally. It SHALL draw no die. Calling it again on that turn SHALL spend another action and SHALL leave the creature Prepared for the ally named by that call. It SHALL NOT set Prepared when the call returns an error.

**Rule**: You prepare to help, usually by using an action during your turn. The GM can require more actions for a long task and decides whether the help is possible. https://2e.aonprd.com/Actions.aspx?ID=2292 (Player Core pg. 416)
**Why not**: Phase 1 has no GM and no long-task clock.
**Substitute**: Preparation is exactly one action on your turn and names one ally. A second call on that turn spends another action and names the ally from that call. This cost is not DEV-51. DEV-51 is the trigger, narrowed to an attack roll.
**Intent**: The preparatory action is real, and its cost does not change from task to task.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2292 (Player Core pg. 416, Aid)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2429 (Player Core pg. 436, Step 2: Act)

#### Scenario: Preparing spends one action

- **GIVEN** Alive Pc CombatantId(1) with Perception +0, aid modifier +7, a scripted natural 10, their turn open with 3 actions, and AidPreparation::NotPrepared, and Alive Pc CombatantId(2)
- **WHEN** CombatantId(1) prepares for CombatantId(2)
- **THEN** the prepare succeeds, 2 actions remain, preparation names CombatantId(2), and the scripted 10 is still unused

#### Scenario: Preparing again only spends an action

- **GIVEN** Alive Pc CombatantId(1) whose turn is open with 3 actions, aid modifier +7, and preparation naming Alive Pc CombatantId(2)
- **WHEN** CombatantId(1) prepares for CombatantId(2)
- **THEN** 2 actions remain and preparation names CombatantId(2)

#### Scenario: Preparing again for a new ally replaces the name

- **GIVEN** Alive Pc CombatantId(1), Alive Pc CombatantId(2), and Alive Pc CombatantId(3), with CombatantId(1)'s turn open, 3 actions, aid modifier +7, preparation naming CombatantId(2), a ready reaction, and a scripted natural 10
- **WHEN** CombatantId(1) prepares for CombatantId(3) and then aids CombatantId(2)
- **THEN** 2 actions remain, preparation names CombatantId(3), the aid error is NotThePreparedAlly, the reaction stays ready, and the scripted 10 is still unused

#### Scenario: Preparing off turn is NotYourTurn

- **GIVEN** Alive CombatantId(1)'s turn is open, and Unconscious CombatantId(2) is NotPrepared with a ready reaction and a scripted natural 8
- **WHEN** CombatantId(2) prepares for CombatantId(1)
- **THEN** the error is NotYourTurn
- **AND** CombatantId(2) is still NotPrepared, the reaction is still ready, and the scripted 8 is still unused

#### Scenario: A dying preparer on their turn returns NotConscious

- **GIVEN** CombatantId(1)'s turn is open, CombatantId(1) is Dying(1) with 0 actions and NotPrepared, and a scripted natural 8
- **WHEN** CombatantId(1) prepares for CombatantId(2)
- **THEN** the error is NotConscious
- **AND** actions stay 0, preparation stays NotPrepared, and the scripted 8 is still unused

#### Scenario: No actions does not prepare

- **GIVEN** an Alive open turn with 0 actions, aid modifier +7, and NotPrepared
- **WHEN** that combatant prepares for an ally
- **THEN** the error is InsufficientActions and preparation stays NotPrepared

### Requirement: No aid modifier is checked before the action is spent

prepare_to_aid SHALL check NoAidModifier after the named-ally checks and before the action is spent. NoAidModifier SHALL leave the action count unchanged and SHALL leave preparation NotPrepared.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2292 (Player Core pg. 416, Aid)

#### Scenario: Preparing with no aid modifier does not spend the action

- **GIVEN** an Alive open turn with 3 actions, no aid modifier, and NotPrepared, and an Alive same-Side ally
- **WHEN** that combatant prepares for that ally
- **THEN** the error is NoAidModifier, 3 actions remain, and preparation stays NotPrepared

### Requirement: The named ally is checked before the action is spent

prepare_to_aid SHALL check the named ally before spending the action. That check follows TurnNotOpen, NotYourTurn, NotConscious, and InsufficientActions, and it precedes NoAidModifier. The order SHALL be UnknownCombatant, then SelfTarget, then NotAnAlly. Each refusal SHALL leave the actions unchanged and SHALL leave preparation NotPrepared.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2292 (Player Core pg. 416, Aid)

#### Scenario: Preparing for an enemy is NotAnAlly

- **GIVEN** Alive Pc CombatantId(1) with their turn open, 3 actions, aid modifier +7, and NotPrepared, and Alive Enemy CombatantId(2), and a scripted natural 10
- **WHEN** CombatantId(1) prepares for CombatantId(2)
- **THEN** the error is NotAnAlly, 3 actions remain, preparation is NotPrepared, and the scripted 10 is still unused

#### Scenario: Preparing for an unknown ally is UnknownCombatant

- **GIVEN** Alive Pc CombatantId(1) with their turn open, 3 actions, aid modifier +7, and NotPrepared, and a scripted natural 10
- **WHEN** CombatantId(1) prepares for CombatantId(9)
- **THEN** the error is UnknownCombatant, 3 actions remain, preparation is NotPrepared, and the scripted 10 is still unused

#### Scenario: Preparing for yourself is SelfTarget

- **GIVEN** Alive Pc CombatantId(1) with their turn open, 3 actions, aid modifier +7, and NotPrepared, and a scripted natural 10
- **WHEN** CombatantId(1) prepares for CombatantId(1)
- **THEN** the error is SelfTarget, 3 actions remain, preparation is NotPrepared, and the scripted 10 is still unused

### Requirement: Preparation lasts until the next start or a resolved aid

begin_encounter SHALL set every combatant to NotPrepared. A successful start_turn SHALL set the creature who starts to NotPrepared after the round increment has succeeded. Another creature's start SHALL leave this preparation in place. end_turn SHALL NOT clear it. RoundOverflow SHALL return before that start and SHALL leave preparation unchanged. An aid call that returns Ok SHALL set that aider to NotPrepared. An aid call that returns an error SHALL leave preparation unchanged.

**Rule**: You must prepare to help, usually by using an action during your turn. The book does not say when that preparation ends. https://2e.aonprd.com/Actions.aspx?ID=2292 (Player Core pg. 416)
**Why not**: An open-ended preparation would stay armed across later turns.
**Substitute**: prepare_to_aid names one ally. The preparation ends at the aider's next start_turn, or when aid returns Ok, whichever comes first. (DEV-50, proposed)
**Intent**: One preparation helps one named ally once.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2292 (Player Core pg. 416, Aid)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2428 (Player Core pg. 435, Step 1: Start Your Turn)

#### Scenario: A participant starts not prepared

- **GIVEN** one Alive participant whose state names an ally in Prepared, Perception +0, and a scripted natural 10
- **WHEN** the encounter begins
- **THEN** preparation is NotPrepared, the reaction is ready, actions are 0, and the scripted 10 was the initiative roll

#### Scenario: Another turn does not clear preparation

- **GIVEN** Alive Pc CombatantId(1) and Alive Pc CombatantId(2), both Perception +0, scripted initiative naturals 10 then 8, and CombatantId(1) prepared for CombatantId(2) before their turn ends
- **WHEN** CombatantId(2)'s turn starts
- **THEN** CombatantId(1) is still prepared for CombatantId(2)

#### Scenario: Your next turn clears preparation

- **GIVEN** Alive Pc CombatantId(1) and Alive Pc CombatantId(2), both Perception +0, scripted initiative naturals 10 then 8, CombatantId(1) prepared for CombatantId(2), and CombatantId(2)'s turn already ended
- **WHEN** CombatantId(1)'s next turn starts and CombatantId(1) then aids CombatantId(2) with a scripted natural 10
- **THEN** preparation is NotPrepared, CombatantId(1) has 3 actions and a ready reaction, the error is NotPrepared, and the scripted 10 is still unused

#### Scenario: An unused preparation expires at the next start

- **GIVEN** Alive Pc CombatantId(1) and Alive Pc CombatantId(2), both Perception +0, scripted initiative naturals 10 then 8, and CombatantId(1) prepared for CombatantId(2) on their turn
- **WHEN** CombatantId(1)'s turn ends, CombatantId(2)'s turn starts and ends with no aid call, and CombatantId(1)'s next turn starts
- **THEN** preparation is NotPrepared and CombatantId(1)'s reaction is ready

#### Scenario: A successful aid clears preparation

- **GIVEN** a Prepared Alive aider naming an Alive same-Side ally, aid modifier +7, AidRank::BelowMaster, a ready reaction, that ally's turn open, and a script of natural 10 then natural 8
- **WHEN** the aider aids that ally, and then aids that ally again
- **THEN** the first result is total 17, success, and Plus1, the reaction is not ready, and preparation is NotPrepared
- **AND** the second error is NotPrepared, not NoReaction, and the scripted 8 is still unused

#### Scenario: Aiding a different ally is NotThePreparedAlly

- **GIVEN** Alive Pc CombatantId(1) prepared for Alive Pc CombatantId(2), with a ready reaction, and Alive Pc CombatantId(3), and a scripted natural 10
- **WHEN** CombatantId(1) aids CombatantId(3) while a turn is open
- **THEN** the error is NotThePreparedAlly, preparation still names CombatantId(2), the reaction stays ready, and the scripted 10 is still unused

#### Scenario: Round overflow keeps preparation

- **GIVEN** the round is 4294967295, the order is CombatantId(1) then CombatantId(2), CombatantId(2)'s turn is open, and CombatantId(1) is prepared for CombatantId(2)
- **WHEN** CombatantId(2)'s turn ends
- **THEN** the error is RoundOverflow and CombatantId(1) is still prepared for CombatantId(2)

### Requirement: The reaction rolls DC 15 once

aid SHALL be usable while a turn is open, including another creature's turn. It SHALL roll one d20, add the aider's aid modifier, and compare the total with DC 15. It SHALL spend the reaction only through spend_reaction, and only after that roll is produced. It SHALL NOT spend an action, end the turn, or change Hit Points. The circumstance SHALL be aid_circumstance_bonus for that degree and the aider's AidRank. The ally's vital SHALL NOT be an error. The open turn SHALL NOT have to be the ally.

**Rule**: The trigger is an ally about to use an action that requires a skill check or attack roll. https://2e.aonprd.com/Actions.aspx?ID=2292 (Player Core pg. 416)
**Why not**: Phase 1 has no skill check on this action.
**Substitute**: The caller names one attack roll. A skill check is not a trigger. (DEV-51, proposed)
**Intent**: Aid still applies to the attack the caller is resolving.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2292 (Player Core pg. 416, Aid)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2432 (Player Core pg. 436, Reactions in Encounters)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2286 (Player Core pg. 401, Step 4: Degree of Success)
**Source**: DEV-02 and DEV-04. https://2e.aonprd.com/Actions.aspx?ID=2292 (Player Core pg. 416)

#### Scenario: A success returns plus 1

- **GIVEN** Alive Pc CombatantId(1) with aid modifier +7, AidRank::Master, 0 actions, attack count 0, a ready reaction, and prepared for Alive Pc CombatantId(2), and CombatantId(2)'s turn open with 3 actions, and a scripted natural 10
- **WHEN** CombatantId(1) aids CombatantId(2)
- **THEN** the total is 17, the degree is success, and the circumstance is Plus1
- **AND** CombatantId(1) still has 0 actions, attack count 0, the reaction is not ready, and preparation is NotPrepared
- **AND** CombatantId(2) still has 3 actions and the same Hit Points

#### Scenario: A master critical success returns plus 3

- **GIVEN** a Prepared Alive aider with aid modifier +7, AidRank::Master, and a ready reaction, naming an Alive same-Side ally whose turn is open
- **WHEN** the aider aids that ally with a scripted natural 18
- **THEN** the total is 25, the degree is critical success, and the circumstance is Plus3

#### Scenario: A legendary critical success returns plus 4

- **GIVEN** a Prepared Alive aider with aid modifier +7, AidRank::Legendary, and a ready reaction, naming an Alive same-Side ally whose turn is open
- **WHEN** the aider aids that ally with a scripted natural 18
- **THEN** the total is 25, the degree is critical success, and the circumstance is Plus4

#### Scenario: A failure returns zero

- **GIVEN** a Prepared Alive aider with aid modifier +7, AidRank::BelowMaster, and a ready reaction, naming an Alive same-Side ally whose turn is open
- **WHEN** the aider aids that ally with a scripted natural 4
- **THEN** the total is 11, the degree is failure, the circumstance is Zero, the reaction is not ready, and preparation is NotPrepared

#### Scenario: A critical failure returns minus 1

- **GIVEN** a Prepared Alive aider with aid modifier +4, AidRank::Master, and a ready reaction, naming an Alive same-Side ally whose turn is open
- **WHEN** the aider aids that ally with a scripted natural 1
- **THEN** the total is 5, the degree is critical failure, the circumstance is Minus1, the reaction is not ready, and preparation is NotPrepared

#### Scenario: A dead ally can be aided

- **GIVEN** Alive Pc CombatantId(1) with aid modifier +7, AidRank::BelowMaster, attack count 0, a ready reaction, and prepared for Dead Pc CombatantId(2), and Alive Pc CombatantId(3)'s turn open, and a scripted natural 10
- **WHEN** CombatantId(1) aids CombatantId(2)
- **THEN** the total is 17, the degree is success, and the circumstance is Plus1
- **AND** the reaction is not ready and preparation is NotPrepared

### Requirement: Aid and Reactive Strike share one reaction

Aid and Reactive Strike SHALL spend the same reaction. After spend_reaction has spent it, aid SHALL return NoReaction and SHALL draw no die, and preparation SHALL stay. The scenario calls spend_reaction and SHALL NOT depend on the reactive-strike API. The reverse SHALL be the same flag: Aid that returned Ok has spent the reaction, so a later Reactive Strike is NoReaction. That reverse scenario lives in the reactive-strike spec (PR #55).

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2432 (Player Core pg. 436, Reactions in Encounters)
**Source**: https://2e.aonprd.com/Actions.aspx?ID=2292 (Player Core pg. 416, Aid)

#### Scenario: Aid with the reaction already spent is NoReaction

- **GIVEN** a Prepared Alive aider naming an Alive same-Side ally, whose reaction spend_reaction has already spent, a turn open, and a scripted natural 10
- **WHEN** that aider aids that ally
- **THEN** the error is NoReaction, preparation still names that ally, and the scripted 10 is still unused

### Requirement: Aid has no attack trait

Aid SHALL NOT have the attack trait and SHALL NOT change the aider's attack count.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2292 (Player Core pg. 416, Aid)

#### Scenario: Aid does not change the attack count

- **GIVEN** a Prepared Alive aider with attack count 0, aid modifier +7, AidRank::BelowMaster, and a ready reaction, naming an Alive same-Side ally whose turn is open
- **WHEN** the aider aids that ally with a scripted natural 10
- **THEN** the result is Ok and the aider's attack count is still 0

### Requirement: The circumstance applies to one check

The returned circumstance SHALL NOT be written onto the ally, the Creature, or a later check. The caller SHALL pass that circumstance's value only for the check being resolved. A later call that passes 0 SHALL NOT include it. Encounter::strike SHALL NOT store an Aid bonus. Plus1, Plus2, Plus3, and Plus4 SHALL be circumstance bonuses of +1, +2, +3, and +4. Minus1 SHALL be a circumstance penalty of -1. Zero SHALL add no modifier. attack_modifiers SHALL omit a 0 Aid value.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2292 (Player Core pg. 416, Aid)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2281 (Player Core pg. 400, Bonuses)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2282 (Player Core pg. 400, Penalties)

#### Scenario: The next check does not keep the bonus

- **GIVEN** an Aid success whose circumstance is Plus1, an ally attack bonus of +0, and a natural 10
- **WHEN** attack_check_total is called with attack_modifiers of penalty 0, not prone, and value +1, and called again with value 0
- **THEN** the first total is 11 and the second total is 10
- **AND** the ally's attack bonus is still +0

#### Scenario: Zero adds no modifier

- **GIVEN** a natural 10, attack bonus +0, and AidCircumstance::Zero
- **WHEN** attack_modifiers is called with penalty 0, not prone, and Aid value 0, and that list is passed to attack_check_total
- **THEN** the modifier list is empty and the total is 10

### Requirement: Same-type circumstance modifiers keep the highest and the worst

Same-type circumstance modifiers SHALL keep the highest bonus and the worst penalty. A circumstance bonus and a circumstance penalty SHALL both apply. Two Aid bonuses SHALL keep the higher. An Aid bonus and an Aid penalty SHALL both apply.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2281 (Player Core pg. 400, Bonuses)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2282 (Player Core pg. 400, Penalties)

#### Scenario: A critical failure is a circumstance penalty

- **GIVEN** a natural 10, attack bonus +0, and AidCircumstance::Minus1
- **WHEN** attack_check_total is called with that Aid value and no other circumstance modifier
- **THEN** the Aid modifier is a circumstance penalty of -1 and the total is 9

#### Scenario: The worse circumstance penalty wins

- **GIVEN** a natural 10, attack bonus +0, AidCircumstance::Minus1, and a caller circumstance penalty of -2
- **WHEN** attack_check_total is called with both
- **THEN** the total is 8

#### Scenario: Plus 1 plus a caller circumstance plus 2 totals 12

- **GIVEN** a natural 10, attack bonus +0, AidCircumstance::Plus1, and a caller circumstance bonus of +2
- **WHEN** attack_check_total is called with both
- **THEN** the total is 12

#### Scenario: Plus 4 plus a caller circumstance plus 2 totals 14

- **GIVEN** a natural 10, attack bonus +0, AidCircumstance::Plus4, and a caller circumstance bonus of +2
- **WHEN** attack_check_total is called with both
- **THEN** the total is 14

#### Scenario: Two aid bonuses keep the higher

- **GIVEN** a natural 10, attack bonus +0, and circumstance bonuses of +2 and +1 from two aiders
- **WHEN** attack_check_total is called with both
- **THEN** the total is 12

#### Scenario: A circumstance bonus and penalty both apply

- **GIVEN** a natural 10, attack bonus +0, an Aid circumstance bonus of +2, and an Aid circumstance penalty of -1
- **WHEN** attack_check_total is called with both
- **THEN** the total is 11

### Requirement: Same Side may aid

An ally SHALL be willing when they share the aider's Side, including Side::Enemy with Side::Enemy. The same id SHALL be SelfTarget. A different Side SHALL be NotAnAlly. Distance SHALL NOT be checked. A repeat SHALL NOT raise the DC. Willing SHALL use DEV-17 and DEV-26. This requirement SHALL NOT add a willing deviation.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2292 (Player Core pg. 416, Aid)
**Source**: DEV-17. Willing is the same faction. https://2e.aonprd.com/Rules.aspx?ID=2360 (Player Core pg. 422)
**Source**: DEV-26. Allies share a Side. https://2e.aonprd.com/Rules.aspx?ID=2375 (Player Core pg. 425)

#### Scenario: An enemy may aid an enemy

- **GIVEN** Alive Enemy CombatantId(1) and Alive Enemy CombatantId(2), CombatantId(1) prepared for CombatantId(2) with aid modifier +7, AidRank::BelowMaster, and a ready reaction
- **WHEN** CombatantId(1) aids CombatantId(2) with a scripted natural 10 while a turn is open
- **THEN** the total is 17, the degree is success, and the circumstance is Plus1

#### Scenario: Aiding yourself is SelfTarget

- **GIVEN** Alive CombatantId(1) prepared for CombatantId(1), with a ready reaction, aid modifier +7, and a scripted natural 10
- **WHEN** CombatantId(1) aids CombatantId(1) while a turn is open
- **THEN** the error is SelfTarget, the reaction stays ready, preparation still names CombatantId(1), and the scripted 10 is still unused

#### Scenario: A different side is NotAnAlly

- **GIVEN** Alive Pc CombatantId(1), NotPrepared, with no ready reaction, and Alive Enemy CombatantId(2), and a scripted natural 10
- **WHEN** CombatantId(1) aids CombatantId(2) while a turn is open
- **THEN** the error is NotAnAlly and the scripted 10 is still unused

### Requirement: An aider who is not Alive returns NotConscious

An aider whose vital is not Alive SHALL return NotConscious and SHALL draw no die, when a turn is open. The reaction SHALL stay as it was. Unconscious, Dying, and Dead are not Alive. TurnNotOpen SHALL win over NotConscious.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2429 (Player Core pg. 436, Step 2: Act)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=95 (Player Core pg. 446, Unconscious)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2325 (Player Core pg. 411, you are unconscious while dying)

#### Scenario: A dying aider returns NotConscious

- **GIVEN** Dying(1) CombatantId(1), prepared for an Alive same-Side CombatantId(2), with a ready reaction and aid modifier +7, a turn open, and a scripted natural 18
- **WHEN** CombatantId(1) aids CombatantId(2)
- **THEN** the error is NotConscious, the reaction stays ready, and the scripted 18 is still unused

#### Scenario: A dead aider returns NotConscious

- **GIVEN** Dead CombatantId(1), prepared for an Alive same-Side CombatantId(2), with a ready reaction, a turn open, and a scripted natural 10
- **WHEN** CombatantId(1) aids CombatantId(2)
- **THEN** the error is NotConscious, the reaction stays ready, and the scripted 10 is still unused

### Requirement: Aid errors follow one order

aid SHALL draw no die on an error. It SHALL check UnknownCombatant for the aider, then UnknownCombatant for the ally, then TurnNotOpen, NotConscious, SelfTarget, NotAnAlly, NotThePreparedAlly, NotPrepared, NoAidModifier, AidOverflow, and NoReaction. AidOverflow SHALL be AidOverflow of ArithmeticOverflow when modifier plus 20 does not fit in i32. The CheckModifier gate for every face in 0..=255 SHALL NOT be used. That gate is for a die that can show 255. This roll is a d20, so the bound is 20.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2292 (Player Core pg. 416, Aid)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2432 (Player Core pg. 436, Reactions in Encounters)

#### Scenario: An unknown aider rolls nothing

- **GIVEN** an encounter of Alive CombatantId(1), with no turn open, and a scripted natural 8
- **WHEN** CombatantId(9) aids CombatantId(1)
- **THEN** the error is UnknownCombatant, not TurnNotOpen, and the scripted 8 is still unused

#### Scenario: An unknown ally rolls nothing

- **GIVEN** Alive Prepared CombatantId(1) with a ready reaction, no turn open, and a scripted natural 8
- **WHEN** CombatantId(1) aids CombatantId(9)
- **THEN** the error is UnknownCombatant and the scripted 8 is still unused

#### Scenario: Aid with no open turn is TurnNotOpen

- **GIVEN** Alive CombatantId(1) prepared for Alive same-Side CombatantId(2), with a ready reaction and aid modifier +7, no turn open, and a scripted natural 10
- **WHEN** CombatantId(1) aids CombatantId(2)
- **THEN** the error is TurnNotOpen, the reaction stays ready, preparation still names CombatantId(2), and the scripted 10 is still unused

#### Scenario: TurnNotOpen wins over NotConscious

- **GIVEN** Unconscious CombatantId(1), prepared for CombatantId(2), with no turn open, and a scripted natural 8
- **WHEN** CombatantId(1) aids CombatantId(2)
- **THEN** the error is TurnNotOpen, not NotConscious, and the scripted 8 is still unused

#### Scenario: NotConscious wins over later errors

- **GIVEN** Unconscious CombatantId(1), NotPrepared, with no aid modifier and no ready reaction, a turn open, and a scripted natural 8
- **WHEN** CombatantId(1) aids CombatantId(1)
- **THEN** the error is NotConscious
- **AND** the reaction stays unready and the scripted 8 is still unused

#### Scenario: An unprepared aider does not roll

- **GIVEN** Alive CombatantId(1), NotPrepared, with no aid modifier and no ready reaction, and an Alive same-Side CombatantId(2), a turn open, and a scripted natural 10
- **WHEN** CombatantId(1) aids CombatantId(2)
- **THEN** the error is NotPrepared and the scripted 10 is still unused

#### Scenario: No aid modifier does not spend the reaction

- **GIVEN** Alive CombatantId(1) prepared for an Alive same-Side CombatantId(2), with no aid modifier and no ready reaction, a turn open, and a scripted natural 10
- **WHEN** CombatantId(1) aids CombatantId(2)
- **THEN** the error is NoAidModifier, the reaction stays unready, and the scripted 10 is still unused

#### Scenario: An aid modifier at the d20 limit is rolled

- **GIVEN** Alive CombatantId(1) prepared for an Alive same-Side ally, with aid modifier 2147483627, AidRank::BelowMaster, and a ready reaction, a turn open, and a scripted natural 20
- **WHEN** CombatantId(1) aids that ally
- **THEN** the result is Ok, the total is 2147483647, the degree is critical success, and the circumstance is Plus2
- **AND** the scripted 20 was drawn

#### Scenario: The next aid modifier above the d20 limit is AidOverflow

- **GIVEN** Alive CombatantId(1) prepared for an Alive same-Side ally, with aid modifier 2147483628, AidRank::BelowMaster, and a ready reaction, a turn open, and a scripted natural 20
- **WHEN** CombatantId(1) aids that ally
- **THEN** the error is AidOverflow of ArithmeticOverflow, the reaction stays ready, and the scripted 20 is still unused
