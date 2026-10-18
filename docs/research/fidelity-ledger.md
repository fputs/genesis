# Fidelity ledger

Principle III records. Knocked out, dying, Raise a Shield, and Shield Block run as written. The other entries are procedures this slice does not run as written at the table.

## Knocked out and dying

- **Rule and source.** Player Core, knocked out and dying. A hit that takes a creature from positive Hit Points to 0 makes them unconscious and prone. They gain dying 1, or dying 2 if that hit was a critical success, plus their current wounded value, and then wounded increases by 1. A successful Strike against an unconscious creature becomes a critical success before damage and before dying changes. Damage to a creature that already has dying increases dying by 1, or by 2 on a critical success, and does not increase wounded. Damage to an unconscious creature who is not dying gives them dying again by that same calculation. Dying 4 or more kills. If one hit deals damage at least twice maximum Hit Points and the creature had more than 0 Hit Points before that hit, the creature dies and dying is not also applied. At the start of a dying creature's turn, a d20 flat check against DC 10 plus dying lowers dying by 2 on a critical success, by 1 on a success, raises it by 1 on a failure, and by 2 on a critical failure. A natural 20 improves the degree by one step and a natural 1 worsens it by one. Dying 0 removes dying. The creature stays unconscious at 0 Hit Points, prone, and wounded, and does not act.
- **Runs.** The combatant stores prone, dying from 0 to 3, wounded from 0 upward, and dead. Conscious means Hit Points above 0 and not dead. Unconscious means 0 Hit Points and not dead. A dying creature is unconscious and prone. Off-guard for one Strike is prone, unconscious, or flanked by that attacker, and the circumstance penalty is −2 once. Flanking requires a line between square centers that crosses opposite sides or opposite corners, both attackers conscious and enemies of the target, and the target inside both melee reaches. Only those attackers get the penalty. Unconscious and dead creatures cannot Stride, Step, or Strike. Strikes against unconscious creatures are legal. Strikes against dead creatures are not. Dead creatures do not take turns, do not recover, and stay in their square. A dead party member still fills one of the four slots. The encounter does not rescale. Victory is every enemy dead. Defeat is every party member dead.

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
- **Substitute.** The closed set is Stride, Step, Strike, Raise a Shield, and End Turn. The 3-action turn still spends those actions as written.
- **Intent kept.** A turn is still three actions. The actions this fight offers spend that budget the way those actions do at the table.

## Reactions this iteration does not run

- **Rule and source.** Player Core, reactions, including free reactions and conditions that remove a reaction.
- **Why it does not operate here.** Those tracks are absent, so they cannot advance.
- **Substitute.** One reaction per creature, regained at the start of that creature's turn. A spent reaction stays spent until that turn starts.
- **Intent kept.** Each creature still has one reaction. Spending it is separate from Stride, Step, and Strike, and the reaction is ready again when that creature's turn starts.

## Disruption

- **Rule and source.** Player Core, Reactive Strike, disrupts a manipulate action on a critical hit.
- **Why it does not operate here.** This encounter has no manipulate action, so that cancel cannot run.
- **Substitute.** The critical hit deals its damage and the Stride continues unless the mover is at 0 HP. The reaction still punishes leaving a square.
- **Intent kept.** A critical Reactive Strike still deals double damage. Leaving a threatened square still draws the reaction.

## Raise a Shield and Shield Block

- **Rule and source.** Player Core, Raise a Shield and Shield Block. Raising a shield is one action. A raised shield gives a +2 circumstance bonus to Armor Class until the start of your next turn. Shield Block is a reaction with the trigger that you would take damage from a physical attack while your shield is raised. The shield prevents damage up to its Hardness. You and the shield each take the remaining damage. A shield at its Broken Threshold is broken. At 0 Hit Points it is destroyed. A broken or destroyed shield grants no bonus and cannot be raised or used to Shield Block.
- **Runs.** A wielded shield stores hardness, hit points, maximum hit points, broken threshold, and raised. Shield Block is a bool on the creature. `ClassFeature` stays Reactive Strike. Raise a Shield is `Action::RaiseShield`, legal only with a shield that is not broken and not destroyed. It sets raised until that creature's next turn starts, and the turn start clears it before actions. The +2 circumstance bonus applies while the shield is raised and neither broken nor destroyed. It is the only circumstance bonus to Armor Class, so it does not stack. Every strike in this engine deals physical damage. Shield Block is legal when the creature has the flag, the shield is raised, the shield is not broken and not destroyed, and the reaction is ready. Remaining damage is `max(0, damage - hardness)`. The creature and the shield each take that much. Hit points at or below the broken threshold break the shield. 0 destroys it. Shield Block spends the same reaction Reactive Strike spends, so the other is unavailable until that creature's next turn. The fighter fixture has the flag and a steel shield (Hardness 5, 20 Hit Points, Broken Threshold 10). The Hall Guard has neither.
