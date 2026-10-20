# Tradition notes

Hooks that differ by tradition or casting class. Keep spell definitions tradition-tagged; do not fork resolution code per tradition.

## Arcane

- **Primary classes:** Wizard (prepared), Sorcerer bloodlines that are arcane, Magus (later pack).
- **Skills:** Arcana for many features; Wizard Dedication trains Arcana.
- **Sample slice spells:** Shield, Magic Missile, Electric Arc, Grease, Fear.
- **Wizard-only:** spellbook, arcane school curriculum slots, arcane bond (Drain Bonded Item recovers a spent slot of a chosen rank once/day—store bonded item flag).

## Divine

- **Primary classes:** Cleric, Oracle, Champion devotion (focus), some Sorcerer bloodlines.
- **Deity package:** favored weapon access, divine skill, font Heal vs Harm, domain/spells per deity data—not hardcoded per deity name in Rust.
- **Sample slice spells:** Heal, Harm, Guidance, Bless, Bane, Stabilize.
- **Cleric doctrines:** warpriest vs cloistered alter armor/proficiency—class choice rule elements.

## Occult

- **Primary classes:** Bard, Witch (often), Psychic (later pack), some bloodlines.
- **Bard:** compositions + Performance; repertoire occult.
- **Sample slice spells:** Guidance, Fear, Courageous Anthem, one occult damage cantrip (e.g. Telekinetic Projectile if in Remaster list—verify name).
- **Witch:** tradition from **patron**; hex kit inseparable ([classes/witch.md](../classes/witch.md)).

## Primal

- **Primary classes:** Druid, Ranger focus later, some sorcerers.
- **Druid:** anathema + Wildsong; order skill + order focus spell; Shield Block at L1.
- **Sample slice spells:** Electric Arc, Heal, Stabilize, Grease (if primal on Remaster list—verify), one order focus.

## Focus across traditions

Focus spells keep the tradition of the class/feat that granted them. Pool is per creature, not per tradition.

## Multiclass / archetype casting

Shallow spellcasting from dedications uses the same `SpellcastingState` with tiny slot tables from the archetype feat ladder. Tradition locked by dedication.

## See also

- [framework.md](./framework.md)
- [sample-catalog.md](./sample-catalog.md)
- [../archetypes.md](../archetypes.md)
