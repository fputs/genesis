# MVP slice 1 — first vertical

## Loop

1. Party enters a **small room** in **exploration mode** (real-time movement).
2. An encounter starts against a **single enemy**.
3. Engine switches to **turn-based** combat (PF2e initiative + action economy).
4. Positions **snap to a 5-foot grid** on encounter start.
5. Play proceeds with the constrained action set below until the encounter can resolve.

## Party

- Fixed **4** player characters (Principle II).
- All are **level 1 fighters**.
- Available player actions in this slice: **Move** and **Strike** only.

## Content

- **Hand-authored** test fixture data only (not campaign content; no external PF2e data packs for MVP).
- One small room, one enemy, four level-1 fighters.
- Rules/fixture terminology: **Pathfinder Remaster**.

## Stack (locked)

- **Bevy** game host; pure Rust `rules` + `content` crates (no Bevy in rules).
- Architecture: [mvp-architecture.md](./mvp-architecture.md).

## Authoring

- **Engine only** until this loop is solid.
- Load **test encounter data** from fixtures (no editor path in this slice).

## Out of scope for this slice

- Content editor / in-product authoring
- Full PF2e action/feat/spell surface
- Character building beyond the fixture party
- Campaign narrative / lore
- Multiplayer, RTwP, difficulty scaling by party size
