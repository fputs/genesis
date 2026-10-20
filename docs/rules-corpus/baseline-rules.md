# Baseline rules (historical)

> **Completed.** The seven baseline slices landed on `main` (tip researched for the rules corpus: `2f56abb`, including Aid). This note is the original program brief—not the live plan. For current engine surface see [`../crates/rules/README.md`](../crates/rules/README.md) and [`../research/fidelity-ledger.md`](../research/fidelity-ledger.md). For classes, archetypes, and spells see [README.md](./README.md) and [implementation-roadmap.md](./implementation-roadmap.md).

The seven slices were stacked from a tip that still stopped at 0 HP with only Stride, Step, Strike, and Reactive Strike. This program replaced that stop with the Player Core dying track, then added the basic actions the fidelity ledger still listed as absent. Shield Block was included because the same ledger named it for the fighter.

A player in the room-one fight can drop a guard from positive HP through dying to dead, and can use each new action from the bar. The next engineer inherits one `Action` enum, derived off-guard, and a ledger that only records procedures this tree still does not run.

No class, archetype, or spell is added unless a baseline action cannot be resolved without it. Shield Block is the exception already named. It is one ability flag on the fighter, not a class framework.

## Done when

- Room one still loads four level-1 fighters and the Hall Guard.
- A creature reduced to 0 HP is unconscious, prone, and dying. The fight ends when every enemy is dead, or every party member is dead. A downed character stays in a party slot.
- The action bar can run Drop Prone, Stand, Crawl, Leap, Raise a Shield, Shield Block, Seek, Point Out, Interact, Release, Take Cover, Delay, Ready, and Aid, on top of Stride, Step, and Strike.
- `cargo test --workspace` passes on each pull request's head.
- Each pull request's headless game run exits. Where the slice adds a bar button, the HUD names that action.
- [`../research/fidelity-ledger.md`](../research/fidelity-ledger.md) no longer calls a finished procedure a substitute.

## Out

- Other classes, archetypes, and spells.
- Grapple, Trip, Shove, Disarm, Demoralize, Feint, Escape, Treat Wounds, and the rest of the skill actions. Nothing in this fight imposes grabbed, so Escape has no legal use.
- Climb, Fly, Swim, Burrow, Mount, Avert Gaze, Arrest a Fall, and Grab an Edge.
- A skill list. Aid rolls the creature's attack bonus against DC 15 and the ledger says so.
- New floor tools in the editor. Difficult terrain is a grid flag the rules read. Tests set it. The editor does not paint it.

## Order

Each row is one pull request from `main` or from the previous row. The next row starts only after the previous head is tested.

1. Dying, wounded, unconscious, death, massive damage, and flanking off-guard. No new bar buttons.
2. Raise a Shield and Shield Block.
3. Drop Prone, Stand, Crawl, Leap, and difficult terrain on Step and Stride.
4. Cover, Take Cover, Seek, and Point Out.
5. Interact, Release, and Reactive Strike disruption on a critical hit against a manipulate action.
6. Delay and Ready.
7. Aid.

## Check each row

- `cargo test --workspace` on that head.
- `cargo run -p game -- --headless` exits with the encounter outcome that row claims.
- The fidelity ledger and [`../crates/rules/README.md`](../crates/rules/README.md) match the code.
- The window driver `control-genesis` is not in the git tree. Do not treat a missing driver as a passed window lane. Record the gap and use the headless log until that script is restored.

## Data shape

Stored on the combatant. Prone. Dying from 0 to 3. Wounded from 0 up. Dead. A raised shield, once that row exists.

Derived, not stored. Conscious means hit points above 0 and not dead. Unconscious means 0 hit points and not dead. Off-guard for one strike means the target is prone, unconscious, or flanked by that attacker. Flanking is computed from positions and reach.

`ClassFeature` stays Reactive Strike until the shield row. Shield Block is its own flag, not a new class.
