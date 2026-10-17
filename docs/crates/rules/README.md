# rules

Pathfinder Remaster procedures for one encounter. No Bevy types, no rendering, no file I/O.

Callers inject dice through `Dice`. `SeededDice` is the gameplay stream. `ScriptedDice` is the test stream.

## Surface

- `prepare_combatant` copies a `CreatureTemplate` through `apply_effects` into derived stats (HP, Armor Class, Perception, Speed, Strike) and the class feature list. The only feature is `ClassFeature::ReactiveStrike`.
- `start_encounter` requires four party templates, places everyone on a `Grid` of 5-foot squares, and rolls Perception initiative.
- `legal_actions` lists `Action::Stride { destination }`, `Action::Step { destination }`, and `Action::Strike { target }` for the active combatant.
- `declare_action` spends one action. A Stride moves up to Speed. Each square it leaves is checked, in order, for opponents who have Reactive Strike, a ready reaction, and that square in reach. Those reactors go in `CombatantId` order. Each rolls at multiple attack penalty 0, spends that reaction, and does not change the strike count. The event is emitted before the mover enters the next square. At 0 HP the mover stays on the square they were leaving and later squares are skipped. A Step moves 5 feet when Speed is at least 10. It does not change the strike count and does not provoke. A Strike rolls against Armor Class and applies multiple attack penalty −5 / −10 on later Strikes that turn. A Strike does not provoke. Step and Strike do not spend a reaction.
- `reactors_for_left_square` lists the reactors for one square a creature is leaving.
- `reaction_ready` reports the flag on a combatant. `start_encounter` sets it for every conscious creature, and the start of that creature's turn sets it again. `spend_reaction` clears it and returns `Illegal::NoReaction` when it is already clear.
- `advance_if_turn_done` / `end_turn` walk the initiative order. A creature at 0 HP is skipped and a party member stays in one of the four slots.
- The encounter ends in `Victory` when every enemy is at 0 HP, or `Defeat` when every party slot is at 0 HP.

`Encounter` owns the initiative order, the current index, combatants keyed by `CombatantId`, and the grid.

## Fidelity notes

Dying checks, initiative ties that need a GM, Step on difficult terrain, the basic actions this fight does not run, reaction rules this iteration does not run, Reactive Strike disruption, and Shield Block are recorded in `docs/research/fidelity-ledger.md`.
