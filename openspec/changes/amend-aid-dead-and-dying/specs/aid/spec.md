# aid

## MODIFIED Requirements

### Requirement: The named ally is checked before the action is spent

prepare_to_aid SHALL check the named ally before spending the action. That check follows TurnNotOpen, NotYourTurn, NotConscious, and InsufficientActions, and it precedes NoAidModifier. The order SHALL be UnknownCombatant, then SelfTarget, then NotAnAlly, then AllyDead when that ally is Dead. An Unconscious or Dying ally SHALL be accepted, because they may wake before the aider's next turn starts. Each refusal SHALL leave the actions unchanged and SHALL leave preparation NotPrepared.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2292 (Player Core pg. 416, Aid)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=95 (Player Core pg. 447, Unconscious)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2324 (Player Core pg. 410, Getting Knocked Out)

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

#### Scenario: Preparing for an unconscious or dying ally is legal

- **GIVEN** Alive Pc CombatantId(1) with their turn open, 3 actions, aid modifier +7, and NotPrepared, Unconscious Pc CombatantId(2), Dying(1) Pc CombatantId(3), and a scripted natural 10
- **WHEN** CombatantId(1) prepares for CombatantId(2) and then prepares for CombatantId(3)
- **THEN** the prepare for CombatantId(2) succeeds, 2 actions remain, and preparation names CombatantId(2)
- **AND** the prepare for CombatantId(3) succeeds, 1 action remains, preparation names CombatantId(3), and the scripted 10 is still unused

#### Scenario: Preparing for a dead ally is rejected

- **GIVEN** Alive Pc CombatantId(1) with their turn open, 3 actions, aid modifier +7, and NotPrepared, Dead Pc CombatantId(2), and a scripted natural 10
- **WHEN** CombatantId(1) prepares for CombatantId(2)
- **THEN** the error is AllyDead, 3 actions remain, preparation is NotPrepared, and the scripted 10 is still unused

### Requirement: The reaction rolls DC 15 once

On any open turn, including another turn, aid SHALL roll one d20 plus the aid modifier at DC 15. The circumstance SHALL be aid_circumstance_bonus for the degree and AidRank. It SHALL spend the reaction only through spend_reaction, after the roll, and only when that ally attempts the check. If the trigger never fires, it SHALL spend nothing and draw no die. It SHALL NOT spend an action, end the turn, or change Hit Points. A Dead ally SHALL be AllyDead. Unconscious and Dying SHALL NOT be an error.

**Rule**: The trigger is an ally about to use an action that requires a skill check or attack roll. https://2e.aonprd.com/Actions.aspx?ID=2292 (Player Core pg. 416)
**Why not**: Phase 1 has no skill check on this action.
**Substitute**: The caller names one attack roll. A skill check is not a trigger. (DEV-51, proposed)
**Intent**: Aid still applies to the attack the caller is resolving.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2292 (Player Core pg. 416, Aid)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2432 (Player Core pg. 436, Reactions in Encounters)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2286 (Player Core pg. 401, Step 4: Degree of Success)
**Source**: DEV-02 and DEV-04. https://2e.aonprd.com/Actions.aspx?ID=2292 (Player Core pg. 416)
**Source**: https://2e.aonprd.com/Conditions.aspx?ID=95 (Player Core pg. 447, Unconscious)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2324 (Player Core pg. 410, Getting Knocked Out)

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
- **THEN** the error is AllyDead
- **AND** the reaction stays ready, preparation still names CombatantId(2), the attack count stays 0, and the scripted 10 is still unused

#### Scenario: Aid against a dead ally is rejected

- **GIVEN** Alive Pc CombatantId(1) with aid modifier +7, AidRank::BelowMaster, attack count 0, a ready reaction, and prepared for Dead Pc CombatantId(2), and Alive Pc CombatantId(3)'s turn open, and a scripted natural 10
- **WHEN** CombatantId(1) aids CombatantId(2)
- **THEN** the error is AllyDead
- **AND** the reaction stays ready, preparation still names CombatantId(2), the attack count stays 0, and the scripted 10 is still unused

#### Scenario: A trigger that never fires spends nothing

- **GIVEN** Alive Pc CombatantId(1) and Alive Pc CombatantId(2), both Perception +0, with scripted initiative naturals 10 then 8, CombatantId(1) prepared for CombatantId(2) with a ready reaction, and a scripted natural 10 still unused
- **WHEN** CombatantId(1)'s turn ends and CombatantId(2)'s turn starts and ends with no aid call
- **THEN** CombatantId(1)'s reaction is ready, preparation still names CombatantId(2), and the scripted 10 is still unused

### Requirement: Aid errors follow one order

aid SHALL draw no die on an error. It SHALL check UnknownCombatant for the aider, then UnknownCombatant for the ally, then TurnNotOpen, NotConscious, SelfTarget, NotAnAlly, NotThePreparedAlly, NotPrepared, AllyDead, NoAidModifier, AidOverflow, and NoReaction. SelfTarget SHALL win over NotPrepared. AidOverflow wraps ArithmeticOverflow when modifier plus 20 does not fit in i32. The 0..=255 CheckModifier gate SHALL NOT be used, because this roll is a d20 and the bound is 20.

**Source**: https://2e.aonprd.com/Actions.aspx?ID=2292 (Player Core pg. 416, Aid)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2432 (Player Core pg. 436, Reactions in Encounters)
**Source**: https://2e.aonprd.com/Rules.aspx?ID=2324 (Player Core pg. 410, Getting Knocked Out)

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
