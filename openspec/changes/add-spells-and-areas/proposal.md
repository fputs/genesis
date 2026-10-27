# Proposal

## Why

The encounter can Strike and roll a basic save, and the grid already measures 5-foot squares, but nothing casts a spell. Phase 2 needs the smallest casting procedure: an action cost, a spell attack, a spell DC, and areas on that grid, before slots, focus points, or a spell list.

## What Changes

- Add `spells`. Cast a Spell spends the spell's actions on the caster's open turn. The cost carries that spell's traits. This change does not resolve Reactive Strike and does not void a disrupted cast.

- Spell attack rolls and spell DCs use one spellcasting entry: an attribute modifier, a level, and a proficiency rank. The rank is an addend, as saves already do. The spell DC is 10 plus the spell attack modifier. The multiple attack penalty applies to a spell attack roll and not to the spell DC. The degree uses the target's baked Armor Class, as Strike does.

- A save spell calls `Encounter::basic_save`. It does not reimplement degrees, the natural 1 and natural 20 shift, or the basic-save multipliers.

- Areas are burst, cone, line, and emanation on the existing grid. Membership uses that grid's diagonal measurement and ignores difficult terrain. The encounter does not store squares. The caller passes them, as Stride and Reactive Strike already require. Only Small and Medium spaces are accepted.

- Two reference spells: Breathe Fire (15-foot cone, 2d6, basic Reflex) and Telekinetic Projectile (range 30 feet, spell attack, 2d6). One damage roll covers every creature in an area.

**BREAKING:** `BuildParts`, `Contribution`, and `Creature` gain `spellcasting: Option<Spellcasting>`. `Statistic` gains `SpellAttack`. `Encounter` gains `breathe_fire` and `telekinetic_projectile`. `EncounterError` gains `NoSpellcasting`, `OutOfSpellRange`, `UnsupportedCone`, and `DuplicateAreaTarget`.

## Capabilities

### New Capabilities

- `spells`: Cast a Spell, the spell attack modifier and spell DC, area membership, and the two reference spells.

### Modified Capabilities

- `creature-template`: `derive` copies optional spellcasting and does not recompute Armor Class, the strike, Perception, or saves from it.

- `modifiers`: `SpellAttack` is its own statistic. A modifier on `Attack` does not apply to it. Proficiency stays out of the bonus types.

- `api-facade`: The public name set and the public `Encounter` method list gain the spell types and the two cast methods.

## Impact

New module `crates/rules/src/spell.rs`, re-exported from the crate root for the types a caller must name. Area membership stays `pub(crate)` and calls the existing grid distance. `breathe_fire` calls `Encounter::basic_save`. `telekinetic_projectile` does not call `strike`. `crates/rules` gains no dependency. No file on `v1-reference` is named, so none is ported.

This change depends on `add-conditions-with-durations` and `add-resistances-weaknesses-persistent`, both drafted in parallel. It does not define a condition, a duration, a damage type, resistance, weakness, immunity, or persistent damage. `add-skills` is a separate change and is not modified.

This pull request is the spec commit. It adds no code, no tests, and no changelog bullet, and it does not archive. Designer rules on the spec here. Code waits for a `spec-approved: <sha>` comment from Evan.

The archive commit is last, right before merge, and only after Architect and QA have both passed that head. Proposed deviations are in design.md. The ruling log is [docs/designer/rules-rulings.md](../../../docs/designer/rules-rulings.md). The code phase records those deviations in `docs/designer/rules-deviations.md`.
