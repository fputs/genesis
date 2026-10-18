# rules

Pathfinder Remaster procedures for one encounter. No Bevy types, no rendering, no file I/O.

Callers inject dice through `Dice`. `SeededDice` is the gameplay stream. `ScriptedDice` is the test stream.

## Surface

- `prepare_combatant` copies a `CreatureTemplate` through `apply_effects` into derived stats (HP, Armor Class, Perception, Speed, Strike) and the class feature list. The only feature is `ClassFeature::ReactiveStrike`.
- `start_encounter` requires four party templates, places everyone on a `Grid` of 5-foot squares, and rolls Perception initiative.
- `legal_actions` lists `Action::Stride { destination }`, `Action::Step { destination }`, and `Action::Strike { target }` for the active conscious combatant. Unconscious and dead creatures have none. A Strike may target an unconscious opponent. A Strike may not target a dead one.
- `declare_action` spends one action. A Stride moves up to Speed. Each square it leaves is checked, in order, for opponents who have Reactive Strike, a ready reaction, and that square in reach. Those reactors go in `CombatantId` order. Each rolls at multiple attack penalty 0, spends that reaction, and does not change the strike count. The event is emitted before the mover enters the next square. At 0 HP the mover stays on the square they were leaving and later squares are skipped. A Step moves 5 feet when Speed is at least 10. It does not change the strike count and does not provoke. A Strike rolls against Armor Class and applies multiple attack penalty −5 / −10 on later Strikes that turn. A Strike does not provoke. Step and Strike do not spend a reaction.
- A Strike's Armor Class is 2 lower when the target is off-guard to that attacker. Off-guard means prone, unconscious, or flanked by that attacker. Those circumstance penalties do not stack. Two conscious enemies of the target flank when the target is inside both melee reaches and a line between their square centers crosses opposite sides or opposite corners of the target.
- A successful Strike against an unconscious creature becomes a critical success before damage. A hit that reduces positive HP to 0 drops the creature prone and sets dying to 1 plus wounded, or 2 plus wounded on a critical success, then increases wounded by 1. Damage against a creature that already has dying increases dying by 1, or by 2 on a critical success, and does not increase wounded. Damage against an unconscious creature who is not dying gives them dying again by that same 1 or 2 plus wounded. Dying of 4 or more kills. One hit that deals at least twice maximum HP, dealt while the creature had more than 0 HP, kills and does not also apply dying.
- `reactors_for_left_square` lists the reactors for one square a creature is leaving.
- `reaction_ready` reports the flag on a combatant. `start_encounter` sets it for every conscious creature, and the start of that creature's turn sets it again. `spend_reaction` clears it and returns `Illegal::NoReaction` when it is already clear.
- `advance_if_turn_done` / `end_turn` take the dice and walk the initiative order. A dying creature rolls a d20 flat check at DC 10 plus dying, then the turn passes if they still cannot act. Critical success lowers dying by 2, success by 1, failure raises it by 1, and critical failure raises it by 2. A natural 20 or 1 adjusts that degree. Dying 0 removes dying and leaves the creature unconscious, prone, and wounded. Dead creatures do not take turns. A party member stays in one of the four slots.
- The encounter ends in `Victory` when every enemy is dead, or `Defeat` when every party member is dead. A creature at 0 HP who is not dead does not end it.

`Encounter` owns the initiative order, the current index, combatants keyed by `CombatantId`, and the grid.

## Fidelity notes

Knocked out and dying run as written in Player Core. Initiative ties that need a GM, Step on difficult terrain, the basic actions this fight does not run, reaction rules this iteration does not run, Reactive Strike disruption, and Shield Block are recorded in `docs/research/fidelity-ledger.md`.
