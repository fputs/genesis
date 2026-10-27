# Design

## Context

See proposal.md for why this change exists. `Encounter::basic_save` already rolls a save and applies a positive amount through `apply_resolved_blow`. `Encounter::strike` spends one action, applies the multiple attack penalty, and uses baked Armor Class. It is not the spell path. `distance` in `grid.rs` measures Small and Medium squares with the 5-then-10 diagonal rule and ignores terrain. The encounter does not store squares. `Statistic` has Attack, Armor Class, Perception, Damage, Reflex, Fortitude, and Will. `Proficiency` and `Level` already live in `save.rs`.

`add-conditions-with-durations` and `add-resistances-weaknesses-persistent` are drafted in parallel. This design does not define their types. `add-skills` is not touched.

No file on `v1-reference` is named. The spec commit does not edit `docs/designer/`.

## Goals / Non-Goals

**Goals:**

- One module, `crates/rules/src/spell.rs`, for the modifier, the spell DC, area membership, and the two reference spells.

- `Encounter::breathe_fire` calls `Encounter::basic_save`. `Encounter::telekinetic_projectile` does not call `strike`.

- Area membership calls the grid's existing diagonal alternation. It does not keep a second copy of that formula.

**Non-Goals:**

- Spell slots, focus points, heightened ranks, traditions, catalysts, deities, loci, costs, and multiple spellcasting sources.

- Conditions, durations, Sustain, and any condition application. Those belong to `add-conditions-with-durations`.

- Damage types, resistance, weakness, immunity, and persistent damage. Those belong to `add-resistances-weaknesses-persistent`. This change does not add a damage-type enum.

- Reactive Strike resolution, disruption, Shield Block, cover on Reflex saves, line of effect, and walls.

- Cones other than 15 feet. Large, Huge, Gargantuan, and Tiny spaces. Three-dimensional areas. A range increment on the projectile.

- Editing `Encounter::strike`, `basic_save`, or the save degree rules.

## Decisions

### Spellcasting is one optional entry

`Spellcasting` is `level: Level`, `attribute: i32`, and `proficiency: Proficiency`. `attribute` is a modifier, not a score. `derive` copies `Option<Spellcasting>` the way it copies saves: `Some` replaces, `None` leaves the previous layer. The base layer copies the build. Existing literals set `spellcasting: None`, including the facade doctest.

Alternative considered: bake one spell attack modifier the way Strike bakes its attack bonus. Rejected. The spell DC is 10 plus that same modifier, and saves already have the proficiency addend. A second baked number would drift from the DC.

### SpellAttack is a new statistic

`Statistic::SpellAttack` follows Will. Modifiers on it use the existing fold. `spell_attack_modifier` adds the attribute, `proficiency_bonus`, and that fold. `spell_dc` is `dc_from_modifier` of the result. The multiple attack penalty is added only on the attack roll, after that modifier, and it is not agile. A modifier on `Statistic::Attack` is not read.

Alternative considered: reuse `Statistic::Attack`. Rejected. That stack is the weapon Strike. Rules 2292 apply bonuses that affect every attack roll and not weapon-only bonuses. This crate has no separate "all attacks" statistic (DEV-60).

### The two casts

Both spend 2 actions on the caster's open Alive turn through `spend_actions`, after the pre-die checks and only when those checks pass. A lost flat check also spends them. Neither cast calls `resolve_reactions`.

Breathe Fire's traits are concentrate and manipulate. Telekinetic Projectile adds ranged. The projectile is an attack-trait check: it reads `attack_trait_penalty` and increments the attack count on a rolled attack and on a lost flat check. Breathe Fire does neither.

Pre-die order for the projectile: unknown caster, turn, consciousness, spellcasting, actions, unknown target, dead target, size, then distance. `distance` above 30 is `OutOfSpellRange`. 30 and below is in range, including 0. Then the existing `resolve_targeting`. `CannotTarget` spends nothing. A lost check spends the actions and the attack count and returns `SpellAttackOutcome::Lost`. Otherwise one d20, then 2d6 only on a success or critical success. A miss leaves the damage face unrolled. Damage is the sum. A critical success doubles that sum once. A positive amount calls `apply_resolved_blow` with `Success` for a hit and `CriticalSuccess` for a critical hit. A 0 does not call it. The degree uses baked Armor Class. There is no range-increment penalty and no shield argument.

Pre-die order for Breathe Fire: the same caster checks, then duplicate ids, unknown ids, and size. Creatures in the 15-foot cone who lack saves are `NoSavingThrows` before any spend. A creature outside the cone with no saves is ignored. Dead creatures in the cone are skipped. Living creatures in the cone, in ascending `CombatantId`, each receive `basic_save` for Reflex, the caster's spell DC, and one shared modifier slice. Side is not read. The caster's square is not in the cone. When nobody living is inside, no die is drawn and `rolled_damage` is absent.

One 2d6 is rolled before those saves (Rules 2301). The two d6 faces are drawn first, then one d20 per living target.

### Areas

`x` increases east and `y` increases south. A corner is the northwest corner of the named square: the corner of square `(0, 0)` is the point those four squares share.

A burst square is inside when `grid_feet` from the origin corner to that square's farthest corner is at most the radius. Radius must be a positive multiple of 5. This is the existing alternation, started fresh, with no terrain and no turn counter. Checked counts: a 5-foot burst is 4 squares, a 10-foot burst is 12, and a 20-foot burst is 44.

A 15-foot diagonal cone is one quadrant of a 15-foot burst from the chosen corner of the origin square. The origin square is excluded. A 15-foot cardinal cone is the seven-square template in the spells spec, not that quadrant. A quarter-circle from the edge does not produce the diagram's orthogonal 15-foot cone, which drops the two diagonally adjacent squares and then runs three squares wide. Lengths other than 15 return `UnsupportedCone` and no squares.

An emanation uses `distance` from the origin square. The origin square is included only when the caller says so. A line is one square wide. A cardinal line is `length / 5` squares starting at the neighbor. A diagonal line is each diagonal step whose `distance` from the origin is at most the length, so a 30-foot northeast line is four squares (DEV-63).

`grid_feet` is private in `grid.rs`. The implementation makes that function `pub(crate)` and calls it. It does not copy the formula. Movement behavior does not change. Area helpers stay `pub(crate)`. The encounter methods are the public casts.

Alternative considered: defer areas until a map editor exists. Rejected. The grid and `distance` already exist. The encounter not storing squares is the same shape Stride uses.

### Public surface

Root exports add `Aim`, `AreaCreature`, `BreatheFireOutcome`, `SpellAttackHit`, `SpellAttackOutcome`, and `Spellcasting`. `Encounter` adds `breathe_fire` and `telekinetic_projectile`. `SpellAttackOutcome` is `Lost(LostStrike)` or `Rolled(SpellAttackHit)`. `BreatheFireOutcome` carries the cost, `rolled_damage`, and the `SaveRoll` list. `AreaCreature` is `id`, `square`, and `size`.

`EncounterError` adds `NoSpellcasting`, `OutOfSpellRange`, `UnsupportedCone`, and `DuplicateAreaTarget`. An unsupported size is the existing `Grid(UnsupportedSize)`. An overflowing modifier is the existing `AttackOverflow`.

### Proposed deviations

The code phase appends these to `docs/designer/rules-deviations.md`. It does not reuse DEV-47 or DEV-49. The spec commit does not edit that file.

| ID | Topic | Rule | Substitute | Intent |
|---|---|---|---|---|
| DEV-59 | No slot on Breathe Fire | https://2e.aonprd.com/Actions.aspx?ID=2734 (Player Core pg. 299) | The cast spends actions and does not remove a slot | No slot pool in this slice |
| DEV-60 | Attack stack | https://2e.aonprd.com/Rules.aspx?ID=2292 (Player Core, Spell Attack Rolls) | Only `SpellAttack` modifiers apply | No "all attacks" statistic |
| DEV-61 | Physical type | https://2e.aonprd.com/Spells.aspx?ID=1718 (Player Core pg. 363) | 2d6 with no type label | Types belong to the resistance change |
| DEV-62 | Shield Block | https://2e.aonprd.com/Feats.aspx?ID=5212 (Player Core pg. 262) | The projectile takes no shield argument | Shield Block stays on Strike |
| DEV-63 | Diagonal line | https://2e.aonprd.com/Rules.aspx?ID=2384 (Player Core pg. 428) | Four squares at 30 feet, by `distance` | One measurement function |
| DEV-64 | Other cones | https://2e.aonprd.com/Rules.aspx?ID=2386 (Player Core pg. 428) | Lengths other than 15 are `UnsupportedCone` | The diagram's longer cones are not Breathe Fire |
| DEV-65 | One source | https://2e.aonprd.com/Rules.aspx?ID=2292 (Player Core, Spell Attack Rolls) | One entry, attribute is a modifier, no tradition | No class list |
| DEV-66 | Cover on areas | https://2e.aonprd.com/Rules.aspx?ID=2384 (Player Core pg. 428) | No cover bonus on the Reflex save | Cover stays an Armor Class bonus |
| DEV-67 | Disruption | https://2e.aonprd.com/Actions.aspx?ID=2256 (Player Core pg. 138) | The cast does not call `resolve_reactions` and is not voided | Reactions stay the caller's existing call |

## Risks / Trade-offs

- [Designer rejects the orthogonal cone or the diagonal line] → The squares are scenarios. A correction stays in this pull request and needs a new `spec-approved` review before tests.
- [The resistance change lands a damage-type enum first] → This change still passes a bare integer to `basic_save` and `apply_resolved_blow`. It does not import that enum. A later change can attach the label.
- [A public field on `BuildParts`] → Every literal gains `spellcasting: None`. The fold replaces on `Some`, matching saves.

## Migration Plan

The spec commit adds no Rust and no changelog bullet. Tests, then code, land on this branch after `spec-approved: <sha>`. The code phase adds one Unreleased bullet before the archive commit. Archive is the last commit, after Architect and QA have both passed that head. Rollback is reverting the branch. No runtime data is migrated.

## Open Questions

None. The parallel condition and resistance changes keep their models. This change does not wait on their type names.
