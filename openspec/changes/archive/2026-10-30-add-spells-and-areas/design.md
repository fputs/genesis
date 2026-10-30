# Design

## Context

See proposal.md for why this change exists. `Encounter::basic_save` already rolls a save and applies a positive amount through `apply_resolved_blow`. `Encounter::strike` spends one action, applies the multiple attack penalty, and uses baked Armor Class. It is not the spell path. `distance` in `grid.rs` measures Small and Medium squares with the 5-then-10 diagonal rule and ignores terrain. The encounter does not store squares. `Statistic` has Attack, Armor Class, Perception, Damage, Reflex, Fortitude, and Will. `Proficiency` and `Level` already live in `save.rs`.

`add-resistances-weaknesses-persistent` is a dependency, drafted in parallel. `add-conditions-with-durations` is related, not a dependency. This design does not define their types. `add-skills` is not touched.

No file on `v1-reference` is named. This spec records ruling R-13 in `docs/designer/rules-rulings.md`. It does not edit `docs/designer/rules-deviations.md`.

## Goals / Non-Goals

**Goals:**

- One module, `crates/rules/src/spell.rs`, for the modifier, the spell DC, area membership, and the two reference spells.

- `Encounter::breathe_fire` calls `Encounter::basic_save`. `Encounter::telekinetic_projectile` does not call `strike`.

- Area membership calls the grid's existing diagonal alternation. It does not keep a second copy of that formula.

**Non-Goals:**

- Spell slots, focus points, heightened ranks, traditions, catalysts, deities, loci, costs, and multiple spellcasting sources.

- Conditions, durations, Sustain, and any condition application. Those belong to the related change `add-conditions-with-durations`.

- Damage types, resistance, weakness, immunity, and persistent damage. Those belong to `add-resistances-weaknesses-persistent`. This change does not add a damage-type enum. Breathe Fire's cost still names the fire trait.

- Reactive Strike resolution, disruption, Shield Block, cover on Reflex saves, line of effect, and walls. Standard or greater cover would apply against an area, and lesser cover would not (Rules 2384). Cover stays the Armor Class model already in positioning.

- Cones other than 15 feet. Large, Huge, Gargantuan, and Tiny spaces. Three-dimensional areas. A range increment on the projectile.

- Editing `Encounter::strike`, `basic_save`, or the save degree rules.

## Decisions

### Spellcasting is one optional entry

`Spellcasting` is `level: Level`, `attribute: i32`, and `proficiency: Proficiency`. `attribute` is a modifier, not a score. `derive` copies `Option<Spellcasting>` the way it copies saves: `Some` replaces, `None` leaves the previous layer. The base layer copies the build. Existing literals set `spellcasting: None`, including the facade doctest.

Alternative considered: bake one spell attack modifier the way Strike bakes its attack bonus. Rejected. The spell DC is 10 plus that same modifier, and saves already have the proficiency addend. A second baked number would drift from the DC.

### SpellAttack is a new statistic

`Statistic::SpellAttack` follows Will. Modifiers on it use the existing fold. `spell_attack_modifier` adds the attribute, `proficiency_bonus`, and that fold. `spell_dc` is `dc_from_modifier` of the result. The multiple attack penalty is added only on the attack roll, after that modifier, and it is not agile. A modifier on `Statistic::Attack` is not read.

Alternative considered: reuse `Statistic::Attack`. Rejected. That stack is the weapon Strike. Rules 2292 (Player Core pg. 403) apply bonuses that affect every attack roll and not weapon-only bonuses. This crate has no separate "all attacks" statistic (DEV-65).

### The two casts

Both spend 2 actions on the caster's open Alive turn through `spend_actions`, after the pre-die checks and only when those checks pass. A lost flat check also spends them. Neither cast calls `resolve_reactions`.

Breathe Fire's traits are concentrate, fire, and manipulate. Telekinetic Projectile's traits are attack, cantrip, concentrate, and manipulate. `Ranged` is not a trait of either spell. `ActionTrait` gains `Attack`, `Cantrip`, and `Fire`. The existing `Ranged` variant stays the engine stand-in other actions already use. The projectile takes the multiple attack penalty because its cost has the attack trait (Rules 2289, Player Core pg. 402): it reads `attack_trait_penalty` and increments the attack count on a rolled attack and on a lost flat check. It is not agile. Breathe Fire has no attack trait and does neither. Both spells are rank 1. Breathe Fire does not add 2d6 per rank, and the cantrip does not heighten to half the caster's level rounded up (DEV-71).

The spell attack degree is the existing d20 degree (Rules 2287). A natural 20 whose total is Armor Class minus 1 is a success, one step up from a failure, and the 2d6 is not doubled. A natural 1 whose total is Armor Class plus 10 is a success. A natural 1 whose total equals Armor Class is a failure. A natural 1 whose total is Armor Class minus 1 is a critical failure. A save uses that same shift: a natural 1 that meets the spell DC on the numbers is a failure.

Pre-die order for the projectile: unknown caster, turn, consciousness, spellcasting, actions, unknown target, dead target, size, then distance. `distance` above 30 is `OutOfSpellRange`. 30 and below is in range, including 0. Then the existing `resolve_targeting`. `CannotTarget` spends nothing. A lost check spends the actions and the attack count and returns `SpellAttackOutcome::Lost`. Otherwise one d20, then 2d6 only on a success or critical success. A miss leaves the damage face unrolled. Damage is the sum. A critical success doubles that sum once. A positive amount calls `apply_resolved_blow` with `Success` for a hit and `CriticalSuccess` for a critical hit. A 0 does not call it. The degree uses baked Armor Class. There is no range-increment penalty and no shield argument.

Pre-die order for Breathe Fire: the same caster checks, then duplicate ids, unknown ids, and size. Creatures in the 15-foot cone who lack saves are `NoSavingThrows` before any spend. A creature outside the cone with no saves is ignored. Dead creatures in the cone are skipped. Living creatures in the cone, in ascending `CombatantId`, each receive `basic_save` for Reflex, the caster's spell DC, and one shared modifier slice. Side is not read. The caster's square is not in the cone. When nobody living is inside, no die is drawn and `rolled_damage` is absent.

One 2d6 is rolled before those saves (Rules 2301). The two d6 faces are drawn first, then one d20 per living target.

### Areas

`x` increases east and `y` increases south. A corner is the northwest corner of the named square: the corner of square `(0, 0)` is the point those four squares share.

A burst square is inside when `grid_feet` from the origin corner to that square's farthest corner is at most the radius. Radius must be a positive multiple of 5. This is the existing alternation, started fresh, with no terrain and no turn counter. Checked counts: a 5-foot burst is 4 squares, a 10-foot burst is 12, and a 20-foot burst is 44.

A 15-foot diagonal cone is one quadrant of a 15-foot burst from the chosen corner of the origin square. The origin square is excluded. A 15-foot cardinal cone is the seven-square template in the spells spec, not that quadrant. Ruling R-13: both templates are the Areas diagram and are RAW, not a deviation. East from (0, 0) is (1, 0), (2, -1), (2, 0), (2, 1), (3, -1), (3, 0), (3, 1). Northeast is (1, -1), (2, -1), (3, -1), (1, -2), (2, -2), (1, -3). Lengths other than 15 return `UnsupportedCone` and no squares. That refusal is scope beside R-13, not a measurement deviation.

An emanation uses `distance` from the origin square. The origin square is included only when the caller says so. A line is one square wide. A cardinal line is `length / 5` squares starting at the neighbor. A diagonal line is the existing 5-10-5-10 distance (Rules 2357, Player Core pg. 421). A 30-foot northeast line is four squares, at 5, 15, 20, and 30 feet. That count is RAW, not a deviation. The cast does not test line of effect (DEV-72).

`grid_feet` is private in `grid.rs`. The implementation makes that function `pub(crate)` and calls it. It does not copy the formula. Movement behavior does not change. Area helpers stay `pub(crate)`. The encounter methods are the public casts.

Alternative considered: defer areas until a map editor exists. Rejected. The grid and `distance` already exist. The encounter not storing squares is the same shape Stride uses.

### Public surface

Root exports add `Aim`, `AreaCreature`, `BreatheFireOutcome`, `SpellAttackHit`, `SpellAttackOutcome`, and `Spellcasting`. `Encounter` adds `breathe_fire` and `telekinetic_projectile`. `SpellAttackOutcome` is `Lost(LostStrike)` or `Rolled(SpellAttackHit)`. `BreatheFireOutcome` carries the cost, `rolled_damage`, and the `SaveRoll` list. `AreaCreature` is `id`, `square`, and `size`.

`EncounterError` adds `NoSpellcasting`, `OutOfSpellRange`, `UnsupportedCone`, and `DuplicateAreaTarget`. An unsupported size is the existing `Grid(UnsupportedSize)`. An overflowing modifier is the existing `AttackOverflow`.

### Proposed deviations

The code phase appends these to `docs/designer/rules-deviations.md`. The spec commit does not edit that file.

Unused numbers (reserved, never written; do not reuse): DEV-33 to DEV-35, DEV-47 to DEV-49. DEV-59 to DEV-63 are reserved for add-skills (#91, DEV-59 to DEV-61) and add-resistances-weaknesses-persistent (#93, DEV-62 to DEV-63). This change uses DEV-64 to DEV-72. DEV-73 and above are reserved for add-conditions-with-durations.

| ID | Topic | Rule | Substitute | Intent |
|---|---|---|---|---|
| DEV-64 | No slot on Breathe Fire | https://2e.aonprd.com/Actions.aspx?ID=2734 (Player Core pg. 299) | The cast spends actions and does not remove a slot | No slot pool in this slice |
| DEV-65 | Attack stack | https://2e.aonprd.com/Rules.aspx?ID=2292 (Player Core pg. 403) | Only `SpellAttack` modifiers apply | No "all attacks" statistic |
| DEV-66 | Untyped damage | https://2e.aonprd.com/Spells.aspx?ID=1457 (Player Core pg. 319); https://2e.aonprd.com/Spells.aspx?ID=1718 (Player Core pg. 363) | Fire, and bludgeoning, piercing, or slashing, are a bare 2d6. The cost still names fire | Types belong to the resistance change |
| DEV-67 | Shield Block | https://2e.aonprd.com/Feats.aspx?ID=5212 (Player Core pg. 262) | The projectile takes no shield argument | Shield Block stays on Strike |
| DEV-68 | One source | https://2e.aonprd.com/Rules.aspx?ID=2292 (Player Core pg. 403) | One entry, attribute is a modifier, no tradition | No class list |
| DEV-69 | Cover on areas | https://2e.aonprd.com/Rules.aspx?ID=2384 (Player Core pg. 428) | No cover bonus on the Reflex save. Standard or greater would apply; lesser would not | Cover stays an Armor Class bonus |
| DEV-70 | Disruption | https://2e.aonprd.com/Actions.aspx?ID=2256 (Player Core pg. 138) | The cast does not call `resolve_reactions` and is not voided | Reactions stay the caller's existing call |
| DEV-71 | Rank 1 only | https://2e.aonprd.com/Spells.aspx?ID=1457 (Player Core pg. 319); https://2e.aonprd.com/Spells.aspx?ID=1718 (Player Core pg. 363) | Both spells deal 2d6. Breathe Fire does not add 2d6 per rank. The cantrip does not heighten to half level rounded up | No rank argument in this slice |
| DEV-72 | Line of effect | https://2e.aonprd.com/Rules.aspx?ID=2384 (Player Core pg. 428) | No line-of-effect test on the cone or the projectile | No walls in this slice |

## Risks / Trade-offs

- [A later ruling moves a square in the 15-foot cone] → R-13 locks the squares in the spells spec. A correction stays in this pull request and needs a new `spec-approved` review before tests.
- [The resistance change lands a damage-type enum first] → This change still passes a bare integer to `basic_save` and `apply_resolved_blow`. It does not import that enum. A later change can attach the label.
- [A public field on `BuildParts`] → Every literal gains `spellcasting: None`. The fold replaces on `Some`, matching saves.

## Migration Plan

The spec commit adds no Rust and no changelog bullet. Tests, then code, land on this branch after `spec-approved: <sha>`. The code phase adds one Unreleased bullet before the archive commit. Archive is the last commit, after Architect and QA have both passed that head. Rollback is reverting the branch. No runtime data is migrated.

## Open Questions

None. The related condition change and the resistance dependency keep their models. This change does not wait on their type names. R-13 closes the 15-foot cone.
