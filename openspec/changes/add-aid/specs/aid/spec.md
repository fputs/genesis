# aid

## Purpose

Prepares to Aid with one action on your turn, then spends the reaction to roll DC 15 and return a circumstance modifier for that one check.

## ADDED Requirements

### Requirement: The same aid twice matches

The same inputs SHALL return the same result. A call SHALL NOT read a file, the network, a clock, or a global generator. A fallible call SHALL return a typed error and SHALL NOT panic. The rules crate SHALL gain no dependency. Preparation SHALL be AidPreparation::Prepared or AidPreparation::NotPrepared. The circumstance SHALL be an AidCircumstance. Neither type is a bool.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2292 (Player Core pg. 416, Aid)

#### Scenario: The same aid twice matches

- **GIVEN** two encounters, each with Alive Pc CombatantId(1) and Alive Pc CombatantId(2), both Perception +0, aid modifier +7, AidRank::Trained, and a script of initiative naturals 10 then 8 followed by a natural 10
- **WHEN** CombatantId(1) prepares on their turn, the turn ends, CombatantId(2)'s turn starts, and CombatantId(1) aids CombatantId(2)
- **THEN** both totals are 17, both degrees are success, and both circumstances are Plus1

### Requirement: Preparation is one action on your turn

prepare_to_aid SHALL spend 1 action through spend_actions and then set AidPreparation::Prepared. It SHALL draw no die. Calling it again on that turn SHALL spend another action and SHALL leave the creature Prepared. It SHALL NOT set Prepared when spend_actions returns an error. spend_actions decides TurnNotOpen, NotYourTurn, NotConscious, and InsufficientActions, in that order.

**Rule**: You prepare to help, usually by using an action during your turn. The GM can require more actions for a long task and decides whether the help is possible. https://2e.aonprd.com/Actions.aspx?ID=2292 (Player Core pg. 416)
**Why not**: Phase 1 has no GM and no long-task clock.
**Substitute**: Preparation is exactly one action on your turn. A second call on that turn spends another action and stays Prepared. (DEV-51, proposed)
**Intent**: The preparatory action is real, and its cost does not change from task to task.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2292 (Player Core pg. 416, Aid)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2336 (Player Core pg. 414, Activities)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2429 (Player Core pg. 436, Step 2: Act)

#### Scenario: Preparing spends one action

- **GIVEN** Alive CombatantId(1) with Perception +0, a scripted natural 10, their turn open with 3 actions, and AidPreparation::NotPrepared
- **WHEN** CombatantId(1) prepares
- **THEN** the prepare succeeds, 2 actions remain, preparation is Prepared, and the scripted 10 is still unused

#### Scenario: Preparing again only spends an action

- **GIVEN** Alive CombatantId(1) whose turn is open with 3 actions and preparation Prepared
- **WHEN** CombatantId(1) prepares
- **THEN** 2 actions remain and preparation is Prepared

#### Scenario: Preparing off turn is NotYourTurn

- **GIVEN** Alive CombatantId(1)'s turn is open, and Unconscious CombatantId(2) is NotPrepared with a ready reaction and a scripted natural 8
- **WHEN** CombatantId(2) prepares
- **THEN** the error is NotYourTurn
- **AND** CombatantId(2) is still NotPrepared, the reaction is still ready, and the scripted 8 is still unused

#### Scenario: A dying preparer on their turn returns NotConscious

- **GIVEN** CombatantId(1)'s turn is open, CombatantId(1) is Dying(1) with 0 actions and NotPrepared, and a scripted natural 8
- **WHEN** CombatantId(1) prepares
- **THEN** the error is NotConscious
- **AND** actions stay 0, preparation stays NotPrepared, and the scripted 8 is still unused

#### Scenario: No actions does not prepare

- **GIVEN** an Alive open turn with 0 actions and NotPrepared
- **WHEN** that combatant prepares
- **THEN** the error is InsufficientActions and preparation stays NotPrepared

### Requirement: Preparation lasts until that creature starts again

begin_encounter SHALL set every combatant to NotPrepared. A successful start_turn SHALL set the creature who starts to NotPrepared after the round increment has succeeded. Another creature's start SHALL leave this preparation in place. end_turn SHALL NOT clear it. RoundOverflow SHALL return before that start and SHALL leave preparation unchanged. Using the reaction SHALL leave the creature Prepared.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2292 (Player Core pg. 416, Aid)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2428 (Player Core pg. 435, Step 1: Start Your Turn)

#### Scenario: A participant starts not prepared

- **GIVEN** one Alive participant whose state is Prepared, Perception +0, and a scripted natural 10
- **WHEN** the encounter begins
- **THEN** preparation is NotPrepared, the reaction is ready, actions are 0, and the scripted 10 was the initiative roll

#### Scenario: Another turn does not clear preparation

- **GIVEN** Alive Pc CombatantId(1) and Alive Pc CombatantId(2), both Perception +0, scripted initiative naturals 10 then 8, and CombatantId(1) Prepared before their turn ends
- **WHEN** CombatantId(2)'s turn starts
- **THEN** CombatantId(1) is still Prepared

#### Scenario: Your next turn clears preparation

- **GIVEN** Alive Pc CombatantId(1) and Alive Pc CombatantId(2), both Perception +0, scripted initiative naturals 10 then 8, CombatantId(1) Prepared, and CombatantId(2)'s turn already ended
- **WHEN** CombatantId(1)'s next turn starts and CombatantId(1) then aids CombatantId(2) with a scripted natural 10
- **THEN** preparation is NotPrepared, CombatantId(1) has 3 actions and a ready reaction, the error is NotPrepared, and the scripted 10 is still unused

#### Scenario: Round overflow keeps preparation

- **GIVEN** the round is 4294967295, the order is CombatantId(1) then CombatantId(2), CombatantId(2)'s turn is open, and CombatantId(1) is Prepared
- **WHEN** CombatantId(2)'s turn ends
- **THEN** the error is RoundOverflow and CombatantId(1) is still Prepared

### Requirement: The reaction rolls DC 15 once

aid SHALL be usable while another creature's turn is open. It SHALL roll one d20, add the aider's aid modifier, and compare the total with DC 15. It SHALL spend the reaction only through spend_reaction, and only after that roll is produced. It SHALL NOT spend an action, end the turn, or change Hit Points. The circumstance SHALL be aid_circumstance_bonus for that degree and the aider's AidRank. The ally's vital SHALL NOT be an error. The open turn SHALL NOT have to be the ally.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2292 (Player Core pg. 416, Aid)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2432 (Player Core pg. 436, Reactions in Encounters)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2286 (Player Core pg. 401, Step 4: Degree of Success)
**Source**: DEV-02 and DEV-04. https://2e.aonprd.com/Actions.aspx?ID=2292 (Player Core pg. 416)

#### Scenario: A success returns plus 1

- **GIVEN** Alive Pc CombatantId(1) with aid modifier +7, AidRank::Master, 0 actions, a ready reaction, and Prepared, and Alive Pc CombatantId(2)'s turn open with 3 actions, and a scripted natural 10
- **WHEN** CombatantId(1) aids CombatantId(2)
- **THEN** the total is 17, the degree is success, and the circumstance is Plus1
- **AND** CombatantId(1) still has 0 actions, the reaction is not ready, and preparation is still Prepared
- **AND** CombatantId(2) still has 3 actions and the same Hit Points

#### Scenario: An untrained critical success returns plus 1

- **GIVEN** a Prepared Alive aider with aid modifier +7, AidRank::Untrained, and a ready reaction, and an Alive same-Side ally whose turn is open
- **WHEN** the aider aids that ally with a scripted natural 18
- **THEN** the total is 25, the degree is critical success, and the circumstance is Plus1

#### Scenario: A master critical success returns plus 4

- **GIVEN** a Prepared Alive aider with aid modifier +7, AidRank::Master, and a ready reaction, and an Alive same-Side ally whose turn is open
- **WHEN** the aider aids that ally with a scripted natural 18
- **THEN** the total is 25, the degree is critical success, and the circumstance is Plus4

#### Scenario: A failure returns zero

- **GIVEN** a Prepared Alive aider with aid modifier +7, AidRank::Untrained, and a ready reaction, and an Alive same-Side ally whose turn is open
- **WHEN** the aider aids that ally with a scripted natural 4
- **THEN** the total is 11, the degree is failure, the circumstance is Zero, and the reaction is not ready

#### Scenario: A critical failure returns minus 1

- **GIVEN** a Prepared Alive aider with aid modifier +4, AidRank::Master, and a ready reaction, and an Alive same-Side ally whose turn is open
- **WHEN** the aider aids that ally with a scripted natural 1
- **THEN** the total is 5, the degree is critical failure, the circumstance is Minus1, and the reaction is not ready

#### Scenario: A second aid is NoReaction

- **GIVEN** a Prepared Alive aider who has already aided this round, so the reaction is not ready and preparation is still Prepared, and a scripted natural 10
- **WHEN** that aider aids the same ally again
- **THEN** the error is NoReaction, preparation stays Prepared, and the scripted 10 is still unused

### Requirement: The circumstance applies to one check

The returned circumstance SHALL NOT be written onto the ally, the Creature, or a later check. attack_modifiers SHALL receive that circumstance's value only for the check the caller is resolving. A later attack_modifiers call that passes 0 SHALL NOT include it. Encounter::strike SHALL NOT store an Aid bonus. Plus1's value is +1, Plus2 is +2, Plus3 is +3, Plus4 is +4, Zero is 0, and Minus1 is -1. Zero SHALL add no modifier.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2292 (Player Core pg. 416, Aid)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2281 (Player Core pg. 400, Bonuses)

#### Scenario: The next check does not keep the bonus

- **GIVEN** an Aid success whose circumstance is Plus1, an ally attack bonus of +0, and a natural 10
- **WHEN** attack_check_total is called with attack_modifiers of penalty 0, not prone, and value +1, and called again with value 0
- **THEN** the first total is 11 and the second total is 10
- **AND** the ally's attack bonus is still +0

### Requirement: Same Side may aid

An ally SHALL be willing when they share the aider's Side, including Side::Enemy with Side::Enemy. The same id SHALL be SelfTarget. A different Side SHALL be NotAnAlly. Distance SHALL NOT be checked. A repeat SHALL NOT raise the DC.

**Rule**: The ally is willing, and the GM decides the check, the DC, proximity, and whether a repeat is harder. https://2e.aonprd.com/Actions.aspx?ID=2292 (Player Core pg. 416)
**Why not**: Phase 1 has no GM, no skill list, and no distance check on this action.
**Substitute**: Willing means the same Side. You cannot aid yourself. The caller names the ally and the one attack check. (DEV-51, proposed)
**Intent**: The reaction fires for a same-Side ally without a GM at the table.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2292 (Player Core pg. 416, Aid)

#### Scenario: An enemy may aid an enemy

- **GIVEN** Alive Enemy CombatantId(1) and Alive Enemy CombatantId(2), CombatantId(1) Prepared with aid modifier +7, AidRank::Trained, and a ready reaction
- **WHEN** CombatantId(1) aids CombatantId(2) with a scripted natural 10
- **THEN** the total is 17, the degree is success, and the circumstance is Plus1

#### Scenario: Aiding yourself is SelfTarget

- **GIVEN** Alive Prepared CombatantId(1) with a ready reaction, aid modifier +7, and a scripted natural 10
- **WHEN** CombatantId(1) aids CombatantId(1)
- **THEN** the error is SelfTarget, the reaction stays ready, preparation stays Prepared, and the scripted 10 is still unused

#### Scenario: A different side is NotAnAlly

- **GIVEN** Alive Pc CombatantId(1), NotPrepared, with no ready reaction, and Alive Enemy CombatantId(2), and a scripted natural 10
- **WHEN** CombatantId(1) aids CombatantId(2)
- **THEN** the error is NotAnAlly and the scripted 10 is still unused

### Requirement: An aider who is not Alive returns NotConscious

An aider whose vital is not Alive SHALL return NotConscious and SHALL draw no die. The reaction SHALL stay as it was. Unconscious, Dying, and Dead are not Alive.

**Source**: https://2e.aonprd.com/Rules.aspx?ID=2429 (Player Core pg. 436, Step 2: Act)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=95 (Player Core pg. 446, Unconscious)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2325 (Player Core pg. 411, you are unconscious while dying)

#### Scenario: A dying aider returns NotConscious

- **GIVEN** Dying(1) CombatantId(1), Prepared, with a ready reaction, aid modifier +7, and an Alive same-Side CombatantId(2), and a scripted natural 18
- **WHEN** CombatantId(1) aids CombatantId(2)
- **THEN** the error is NotConscious, the reaction stays ready, and the scripted 18 is still unused

#### Scenario: A dead aider returns NotConscious

- **GIVEN** Dead CombatantId(1), Prepared, with a ready reaction, and an Alive same-Side CombatantId(2), and a scripted natural 10
- **WHEN** CombatantId(1) aids CombatantId(2)
- **THEN** the error is NotConscious, the reaction stays ready, and the scripted 10 is still unused

### Requirement: Aid errors follow one order

aid SHALL draw no die on an error. It SHALL check UnknownCombatant for the aider, then for the ally, then NotConscious, SelfTarget, NotAnAlly, NotPrepared, NoAidModifier, AidOverflow, and NoReaction. AidOverflow SHALL be a modifier that cannot be added to every face in 0..=255.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2292 (Player Core pg. 416, Aid)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2432 (Player Core pg. 436, Reactions in Encounters)

#### Scenario: An unknown aider rolls nothing

- **GIVEN** an encounter of Alive CombatantId(1) and a scripted natural 8
- **WHEN** CombatantId(9) aids CombatantId(1)
- **THEN** the error is UnknownCombatant and the scripted 8 is still unused

#### Scenario: An unknown ally rolls nothing

- **GIVEN** Alive Prepared CombatantId(1) with a ready reaction and a scripted natural 8
- **WHEN** CombatantId(1) aids CombatantId(9)
- **THEN** the error is UnknownCombatant and the scripted 8 is still unused

#### Scenario: NotConscious wins over later errors

- **GIVEN** Unconscious CombatantId(1), NotPrepared, with no aid modifier and no ready reaction, and a scripted natural 8
- **WHEN** CombatantId(1) aids CombatantId(1)
- **THEN** the error is NotConscious
- **AND** the reaction stays unready and the scripted 8 is still unused

#### Scenario: An unprepared aider does not roll

- **GIVEN** Alive CombatantId(1), NotPrepared, with no aid modifier and no ready reaction, and an Alive same-Side CombatantId(2), and a scripted natural 10
- **WHEN** CombatantId(1) aids CombatantId(2)
- **THEN** the error is NotPrepared and the scripted 10 is still unused

#### Scenario: No aid modifier does not spend the reaction

- **GIVEN** Alive Prepared CombatantId(1) with no aid modifier and no ready reaction, and an Alive same-Side CombatantId(2), and a scripted natural 10
- **WHEN** CombatantId(1) aids CombatantId(2)
- **THEN** the error is NoAidModifier, the reaction stays unready, and the scripted 10 is still unused

#### Scenario: An aid modifier that does not fit rolls nothing

- **GIVEN** Alive Prepared CombatantId(1) with aid modifier 2147483647, AidRank::Trained, and no ready reaction, and an Alive same-Side CombatantId(2), and a scripted natural 10
- **WHEN** CombatantId(1) aids CombatantId(2)
- **THEN** the error is AidOverflow, the reaction stays unready, and the scripted 10 is still unused
