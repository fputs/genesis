# Fidelity ledger

Principle III records for procedures that this slice does not run as written at the table.

## Knocked out and dying

- **Rule and source.** Player Core, knocked out and dying: a creature reduced to 0 HP falls unconscious and gains the dying condition. Recovery checks, the wounded condition, and death at dying 4 follow. A critical hit that drops a creature, or massive damage, can change how far dying advances.
- **Why it does not operate here.** Those steps need the condition track, recovery checks on the creature's turns, and death thresholds. This slice only resolves Stride and Strike, so there is nowhere for that track to advance.
- **Substitute.** Hit points stop at 0. The creature is down, skipped in the initiative order, and still occupies its party slot. An enemy at 0 HP counts as defeated. Damage does not go negative, and there is no separate massive-damage death.
- **Intent kept.** Dropping the enemy to 0 HP ends the fight on the players' side. Dropping every party slot to 0 HP ends it the other way. A downed character is not deleted and does not change encounter strength.

## Initiative ties

- **Rule and source.** Player Core, initiative: when totals tie, the GM decides the order, often by comparing initiative modifiers.
- **Why it does not operate here.** A single-player fight has no GM to break the tie, and a seeded encounter needs one stable order.
- **Substitute.** Higher initiative total first. On a tie, higher Perception modifier first. If that also ties, the lower `CombatantId` goes first.
- **Intent kept.** The higher modifier still wins the tie the way a GM is told to prefer. The remaining key only keeps the result repeatable.

## Downed space

- **Rule and source.** A dying creature is prone in its space; other creatures treat that space under the normal movement rules.
- **Why it does not operate here.** Prone, difficult terrain, and forced movement are outside Stride-to-a-square and Strike.
- **Substitute.** A creature at 0 HP does not block movement or the square another creature stops in.
- **Intent kept.** A body does not erase the slot or the grid. It also does not stall the remaining turns in a room with no forced-movement action.

## Difficult terrain

- **Rule and source.** Player Core, Step, which bars difficult terrain.
- **Why it does not operate here.** The hall has no difficult terrain, so the check cannot run.
- **Substitute.** The check is a walkable square at 5 feet and Speed of at least 10 feet. Step stays a 5-foot action.
- **Intent kept.** Step is still one action that moves 5 feet onto a square the creature can walk, and it is not a Stride.

## Basic actions this encounter does not run

- **Rule and source.** Player Core: Delay, Ready, Drop Prone, Stand, Seek, Interact, Release, Take Cover, and Point Out.
- **Why it does not operate here.** The level-1 fight can start, act, and end without them, so there is no procedure here for those actions to advance.
- **Substitute.** The closed set is Stride, Step, Strike, and End Turn. The 3-action turn still spends Stride, Step, and Strike as written.
- **Intent kept.** A turn is still three actions. The actions this fight offers spend that budget the way those three actions do at the table.
