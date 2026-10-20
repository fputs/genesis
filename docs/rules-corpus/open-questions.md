# Open product questions

Decisions only the operator / product owner can settle. Each item has a **recommended default** so implementers are not blocked if silence continues—but defaults are not silent constitution amendments. Record the final call here when made.

## 1. Which books are in engine v1 content packs?

**Question:** Player Core only, Player Core + Player Core 2, or also Guns & Gears / Dark Archive / War of Immortals / Battlecry / Impossible Magic classes?

**Recommended default:** **Player Core + Player Core 2** classes and their multiclass dedications. Other Remaster classes are registry stubs (id + source) until a pack is explicitly opened. Legacy-only options stay out of default packs.

**Blocks:** Phase 6–10 prioritization; editor class list.

## 2. Content licensing and redistribution

**Question:** Hand-authored fixtures only, ORC-friendly paraphrased data, licensed Paizo data, or community scrapes (AoN/Foundry)?

**Recommended default:** **Hand-authored fixtures + engine procedures** for shipping; treat AoN/Foundry as research references only (already noted in [`../research/sources.md`](../research/sources.md)). Do not vendor Foundry JSON.

**Blocks:** How large spell/feat catalogs can be in-repo.

## 3. Rituals in v1?

**Question:** Support exploration-time rituals (often 1 hour+, secondary casters)?

**Recommended default:** **Out of v1 encounter engine.** Schema may reserve `kind: ritual` as unsupported. Revisit after spell slots/cantrips/focus are stable.

**Blocks:** Spell schema completeness; GM Core ritual list.

## 4. Anathema / tenets enforcement

**Question:** Does the engine mechanically punish Champion/Druid/Cleric anathema, or is that a narrative/editor warning?

**Recommended default:** **Warn in editor / log in debug; no automatic fall-from-class in MVP.** If enforced later, make it data-driven per deity/order.

**Blocks:** Champion/Druid “hard” fidelity.

## 5. Animal companions and familiars as party members?

**Question:** Do companions consume party slots, act on separate initiative, or share the master’s turn?

**Recommended default:** **Not one of the 4 PC slots.** Familiar/companion is a linked combatant; initiative per Remaster companion rules (often acts on owner’s turn for animal companions—implement the Remaster companion support action model). Cap complexity: one companion active per master in v1.

**Blocks:** Phase 12; Ranger/Druid/Witch slices that need companions.

## 6. Skill actions scope for “rules complete”

**Question:** Must Grapple/Trip/Shove/Demoralize/Feint/Treat Wounds ship before class work?

**Recommended default:** **No.** Land substrate + class defining features first. Skill actions queue behind Rogue/Swashbuckler needs (Feint/Tumble) when those slices require them. Escape when grabbed exists.

**Blocks:** Whether Phase 1 includes Athletics action suite.

## 7. Character creation vs encounter-only fixtures

**Question:** Does v1 include a full chargen UI, or only editor/fixture authoring?

**Recommended default:** **Editor + fixtures first;** in-game chargen later. Rules prepare must still accept a complete `CharacterDefn` so chargen can plug in later.

**Blocks:** Editor milestones vs game HUD work.

## 8. Counteracting and downtime

**Question:** Full counteract rules, crafting, Earn Income?

**Recommended default:** **Counteract stub** (rank + bonus vs DC) when a spell needs it; **no downtime economy** in the encounter engine. Crafting-lite for Alchemist daily infusions only.

**Blocks:** Alchemist depth; dispel magic.

## 9. Dual-class / Free Archetype variant

**Question:** Support Free Archetype (extra dedication feats) campaign variants?

**Recommended default:** **Data flag on campaign/encounter pack** `free_archetype: bool` later; not in first archetype phase. First phase = standard dedication gating only.

**Blocks:** Archetype validation rules.

## 10. Aid and skills

**Question:** When skills exist, does Aid switch to the “usually a skill check” tabletop guidance?

**Recommended default:** **Yes for the matching skill when the player/AI picks one; keep attack-roll Aid for Strike triggers.** Remove the Strike-bonus-only substitute from the ledger when skill Aid ships.

**Blocks:** Phase 1 Aid follow-up.

## 11. Spell catalog breadth for “done”

**Question:** How many spells before casters are considered shippable?

**Recommended default:** Framework + **~15–25** spells that exercise attack rolls, basic saves, heal, conditions, areas, and heighten (+1 and specific ranks). Full Player Core list is a long-running content pack, not a single PR.

**Blocks:** Phase 13 exit criteria.

## 12. Relinking Origin vs GitHub

**Question:** Project notes mention Origin-bound forge.

**Recommended default:** Out of scope for rules corpus; coordinator tracks repo hosting separately.

## Decision log

| Date | Item | Decision |
| --- | --- | --- |
| _(empty)_ | | |

When a decision is made, add a row and update the relevant corpus page’s status note.
