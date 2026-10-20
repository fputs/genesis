# Archetypes — dedications and framework

Remaster archetype rules (Player Core, Archetypes chapter) as an implementation model. Full encyclopedia of every printed archetype is **not** required; prove the framework with multiclass dedications and one companion-style primitive.

**Sources:** Player Core (archetypes / dedication details); Player Core 2 (additional multiclass dedications); Archives of Nethys archetype rules pages.

## How archetypes work (Remaster)

1. Spend a **class feat slot** on an archetype’s **dedication** feat (some archetype feats use skill slots—honor the `skill` trait).
2. After dedication, you may take other feats from that archetype in place of class feats (subject to the same level/trait restrictions as the replaced feat).
3. **Dedication lock:** you cannot take another dedication until you have taken **two** additional feats from the current archetype.
4. You cannot retrain the dedication while you still have other feats from that archetype.
5. **Multiclass** dedications have the `multiclass` trait. You **cannot** take a multiclass dedication for a class you already have.
6. Feats that grant another feat still count as **one** feat toward completing the dedication.

Legacy vs Remaster: dedication lock wording is the same idea; always use Remaster feat names and levels from Player Core / PC2.

## Data model

```text
ArchetypeDefn {
  id, name, traits: [multiclass?, …],
  dedication_feat_id,
  feats: [ FeatRef ],           // including additional feats lists with adjusted levels
  source_book,
}

ArchetypeProgress {
  archetype_id,
  feats_taken: [ feat_id ],     // includes dedication
}

// Validation
- dedication present before non-dedication archetype feats
- if multiclass: character.class_id != archetype.class_id
- if starting new dedication: count(feats_taken in current) >= 3  // dedication + 2
```

Archetype feats are ordinary `FeatureDefn`s with rule elements (proficiency, spellcasting training, skill increases, granted actions).

## Registry pattern (everything else)

Maintain a compact registry table in content packs:

| archetype_id | book | dedication_feat | notes |
| --- | --- | --- | --- |
| fighter | Player Core | Fighter Dedication | prove martial |
| rogue | Player Core | Rogue Dedication | prove skills/sneak |
| wizard | Player Core | Wizard Dedication | prove shallow casting |
| cleric | Player Core | Cleric Dedication | prove divine shallow |
| familiar-master | Player Core 2 / related | Familiar Master Dedication | prove familiar primitive |
| beastmaster | Player Core 2 | Beastmaster Dedication | prove animal companion |

Unspecified archetypes: **id + source + “not yet specified”** is enough. Do not block the framework on Acrobat…Wrestler completeness.

## Specs to prove the framework

### Fighter Dedication (multiclass)

**Source:** Player Core, Fighter archetype.

**Typical grants (verify against current Remaster text when implementing):**

- Prerequisites: Strength or Dexterity threshold as printed.
- Trained in martial weapons / light/medium/heavy armor as the dedication states (do not invent).
- Shield Block often appears on the dedication chain—grant only if the feat text does.

**Engine:** proficiency rule elements only at dedication; Reactive Strike is **not** assumed unless a later archetype feat grants it.

**First proof slice:** non-fighter PC takes Fighter Dedication → martial weapon training reflected in prepare → can Strike with a martial weapon legally.

### Rogue Dedication

**Source:** Player Core, Rogue archetype.

**Typical grants:** training in Stealth or a racket skill; sneak attack dice at a reduced progression via later feats (Basic/Advanced intrigue-style feats—use Remaster feat names).

**Engine:** Sneak Attack must be a parameterized precision grant (`dice: 1d6`) so dedication and class share one pipeline.

**First proof slice:** Fighter + Rogue Dedication + feat that grants sneak attack → precision when off-guard.

### Wizard Dedication

**Source:** Player Core, Wizard archetype.

**Typical grants:** trained in Arcana; cast a small number of arcane cantrips / later basic spellcasting feat for slots.

**Engine:** reuses [spells/framework.md](./spells/framework.md) with `casting_source: archetype` and capped slots from feat ladder (not full wizard progression).

**First proof slice:** martial PC casts Shield cantrip from Wizard Dedication chain.

### Cleric Dedication

**Source:** Player Core, Cleric archetype.

**Typical grants:** Religion training; deity choice subset; divine cantrips / basic dogma feat for font-like or slot casting per feat text.

**Engine:** deity id required on dedication; spell tradition divine.

**First proof slice:** divine cantrip Heal or Guidance available in encounter.

### Familiar Master Dedication {#companions-and-familiars}

**Why:** introduces familiar primitive without taking Witch.

**Engine needs:** familiar creature template, master link, limited actions, HP/AC, delivery of touch spells later.

**First proof slice:** familiar on grid, can be Striked, dies without ending the party.

### Beastmaster Dedication

**Why:** animal companion without Ranger feat tree.

**Engine needs:** companion creature, Hunt Prey optional synergy later, support actions.

**First proof slice:** companion Stride + Strike on command using the Remaster companion action economy subset you implement—ledger any simplification.

## Interaction with class feat slots

- At each even class level (typical), the character may pick class feat **or** archetype feat.
- Some classes grant extra class feats; archetype feats may fill those if restrictions allow.
- Free Archetype variant: [open-questions.md](./open-questions.md) item 9.

## Validation (content crate)

Fail closed in CI:

- Missing dedication
- Multiclass same-as-class
- Archetype feat level > character level
- Incomplete dedication when a second dedication appears
- Spellcasting grants without spell framework fields

## Verification predicates

| Proof | Test idea |
| --- | --- |
| Lock | Character with only dedication cannot take another dedication |
| Unlock | Dedication + 2 feats → second dedication allowed |
| Multiclass ban | Fighter + Fighter Dedication rejected |
| Shared precision | Rogue dedication sneak attack uses same helper as Rogue class |
| Shallow cast | Wizard dedication cantrip heightens with level |

## See also

- [architecture.md](./architecture.md)
- [implementation-roadmap.md](./implementation-roadmap.md) Phase 11
- [classes/](./classes/) for full class progressions dedications approximate
