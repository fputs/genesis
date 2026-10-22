# Proposal

## Why

A strike on the grid cannot yet tell whether the defender is off-guard from being prone or from flanking, or whether Take Cover changed the cover bonus those penalties stack with. Player Core states those procedures in one family: posture, opposite sides or corners, and the cover step. The modifier totals and the reach query already exist, and this change is the procedure that feeds them. Encounter turns, reactions, and hidden creatures are later changes.

## What Changes

- Add the `positioning` capability. It owns Drop Prone, Stand, the prone condition's off-guard and attack penalty, flanking on the 5-foot grid, off-guard as one circumstance penalty to Armor Class, and Take Cover. It calls the existing modifier total and the existing reach query. It does not reimplement stacking, diagonals, or Speed.
- Drop Prone and Stand are each 1 action with the move trait, named as data. They do not spend actions, read a turn, or resolve Reactive Strike. Immobilized blocks them through the existing move-trait check. While prone, Stand is the only legal move this change allows.
- Flanking uses square centers. A line through opposite side interiors, or through a pair of opposite corners, of the target's square qualifies. Both flankers must be able to act, share a faction with each other and not with the target, and have the target inside melee reach. The target is off-guard to those flankers' melee attacks only. A center line that only clips a corner and another side is left not-yet-ruled.
- Off-guard from prone, from flanking, or from both is one circumstance −2 to Armor Class, produced by the existing Armor Class modifiers and folded by the existing total. The prone attack penalty is the existing circumstance −2 on the attack roll. A cover bonus and that penalty both apply.
- Take Cover upgrades standard cover to greater, grants standard cover from lesser cover or a caller-supplied nearby feature, and, while prone with no other cover, grants greater cover against ranged attacks only. Ending it is caller data: left the square, made an attack, became unconscious, or dismissed it. Two overlaps stay a typed not-yet-ruled error until Designer answers the questions in `design.md`.
- Dependencies are `modifiers` and `grid-movement` only. No creature-template delta. No new crate and no new dependency. This change names no `v1-reference` file, so none is ported.

Out of this change: Crawl, Leap, Reactive Strike, shields, Seek, hidden, undetected, and concealed; sharing a square with a prone creature; walls and cover geometry; action spending and turn order.

## Capabilities

### New Capabilities

- `positioning`: Prone posture, flanking geometry, off-guard fed through the existing modifier total, and Take Cover.

### Modified Capabilities

None. `modifiers` already totals circumstance bonuses and penalties, cover, the prone attack penalty, and one off-guard penalty. `grid-movement` already measures reach and blocks an immobilized move. This change calls those functions and does not change their requirements.

## Impact

New module `crates/rules/src/positioning.rs`, re-exported from the crate root. Callers pass squares, reach, faction, posture, and cover in. Prone and the Take Cover benefit are returned values, not fields of `Creature` or `CombatantState`. `crates/rules` gains no dependency. The implementation pull request adds the Unreleased changelog bullet; this pull request is the spec only.
