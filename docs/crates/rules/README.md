# rules

Pathfinder Remaster procedures for one encounter. No Bevy types, no rendering, no file I/O.

Callers inject dice through `Dice`. `SeededDice` is the gameplay stream. `ScriptedDice` is the test stream.

## Surface

- `prepare_combatant` copies a `CreatureTemplate` through an empty synthetics pass into derived stats (HP, Armor Class, Perception, Speed, Strike).
- `start_encounter` requires four party templates, places everyone on a `Grid` of 5-foot squares, and rolls Perception initiative.
- `legal_actions` lists `Action::Stride { destination }` and `Action::Strike { target }` for the active combatant.
- `declare_action` spends one action. A Stride moves up to Speed. A Strike rolls against Armor Class and applies multiple attack penalty −5 / −10 on later Strikes that turn.
- `advance_if_turn_done` / `end_turn` walk the initiative order. A creature at 0 HP is skipped and a party member stays in one of the four slots.
- The encounter ends in `Victory` when every enemy is at 0 HP, or `Defeat` when every party slot is at 0 HP.

`Encounter` owns the initiative order, the current index, combatants keyed by `CombatantId`, and the grid.

## Fidelity notes

Dying checks and initiative ties that need a GM are recorded in `docs/research/fidelity-ledger.md`.
