# Proposal

## Why

A strike on the grid cannot yet tell whether the defender is off-guard from being prone or from flanking, or whether Take Cover changed the cover bonus those penalties stack with. Player Core states those procedures in one family: posture, opposite sides or corners, and the cover step. The modifier totals and the reach query already exist. This change is the procedure that feeds them.

## What Changes

- Add the `positioning` capability. It owns Drop Prone, Stand, the prone condition's off-guard and attack penalty, flanking on the 5-foot grid, off-guard as one circumstance penalty to Armor Class, and Take Cover. It calls the existing modifier total and the existing reach query. It does not reimplement stacking, diagonals, or Speed.
- The team identifier is `Side` (`Pc` or `Enemy`) from `add-encounter-turns`. This change does not declare a second team type and does not use `FactionId`. Allies for flanking are DEV-26: two creatures are allies if and only if they share a Side (`Side::{Pc, Enemy}`). There is no consent or charm value.
- A corner counts as both edges that meet there (DEV-23). The center line flanks when the edges it touches include an opposite pair, or when it passes through two opposite corners. There is no not-yet-ruled result.
- Take Cover while prone stacks with other cover (DEV-24). For each attack the circumstance bonus is the higher of the two. Lesser cover alone, when the creature is not prone, is `RequirementNotMet`.
- A creature threatens when the target is inside the supplied reach and `Readiness::Able` (DEV-25's `able_to_act`). The creature stays in its square. Stand is not blocked by an occupant (DEV-27). Another creature ending a move in that square is grid-movement's `OccupiedEnd`.
- Drop Prone and Stand each name 1 action with the move trait. Change 8 treats any `PostureChange` with `ActionTrait::Move` as the reactive trigger (FIX-06). They do not spend actions and do not resolve Reactive Strike. There is no trigger flag. Leap while prone returns `ProneRestrictsMovement` and no distance. A standing Leap returns `Unsupported`. Stride and Step stay in grid-movement. Crawl, and the distance of a standing Leap, are `add-crawl-and-leap`.
- Hidden off-guard is change 10 only (FIX-14). This change does not take a hidden input.
- DEV-23 through DEV-32 are cited from `docs/designer/rules-deviations.md` on main. This change does not edit that file or any other file under `docs/designer/`. R-11 is on main in `docs/designer/rules-rulings.md`.

Out of this change: resolving Reactive Strike, shields, Seek, hidden, undetected, and concealed; walls and cover geometry; Crawl and a standing Leap's distance; action spending and turn order.

## Capabilities

### New Capabilities

- `positioning`: Prone posture, flanking geometry, off-guard fed through the existing modifier total, and Take Cover.

### Modified Capabilities

- `grid-movement`: the reason Crawl and Leap are not implemented here changes from "belong to positioning" to "belong to `add-crawl-and-leap`". Reach, Stride, Step, and `OccupiedEnd` are unchanged.

`modifiers` already totals circumstance bonuses and penalties, cover, the prone attack penalty, and one off-guard penalty. `add-encounter-turns` already names `Side`. This change calls those functions and does not change their requirements.

## Impact

New module `crates/rules/src/positioning.rs`, re-exported from the crate root. It imports `Side` and does not define it. Implementation starts once `add-encounter-turns` (#47) is merged to main. Callers pass squares, reach, `Side`, posture, and cover in. Prone and the Take Cover benefit are returned values, not fields of `Creature` or `CombatantState`. `crates/rules` gains no dependency. The spec-only pull request does not add a changelog bullet and does not edit `docs/designer/`. The implementation pull request adds the Unreleased changelog bullet.
