# Spellcasting framework

Implementation contract for Remaster spellcasting in Genesis `rules` + `content`. Spell **rank** replaces Legacy “spell level.” Traditions: arcane, divine, occult, primal.

**Sources:** Player Core ch. Spells / Playing the Game; Paizo Remaster spellcasting previews; ORC-friendly SRD summaries for procedure shape. Do not paste spell novels into the repo.

## Kinds of magic

| Kind | Resource | Heighten | Notes |
| --- | --- | --- | --- |
| Cantrip | none (at will) | Auto to half level rounded up | Prepared casters prepare N cantrips; spontaneous know N |
| Spell (ranked) | spell slot of that rank | Prepared in / cast from higher slot | Spontaneous must know heightened version or use signature rules |
| Focus spell | 1 Focus Point | Auto like cantrips | Pool size = number of FP-costing focus spells known, max 3 |
| Composition cantrip (bard) | none | Auto | Does not enlarge focus pool |
| Innate | per feature | per feature | Ancestry/items; separate tracker |
| Ritual | special | out of v1 default | See [open-questions.md](../open-questions.md) |

Refocus: 10-minute exploration activity restores 1 FP (encounter engine may expose as between-fight stub).

## Casting styles

### Prepared (Wizard, Cleric, Druid, Witch, Magus…)

- Daily preparations: choose spells from list/book/familiar into slots.
- Casting spends the prepared slot entry.
- Heightening = prepare in a higher-rank slot.

### Spontaneous (Bard, Sorcerer, Oracle, Summoner…)

- Repertoire of known spells; slots per day by rank.
- Cast any known spell of that rank by spending a slot.
- **Signature spells:** can be heightened freely into higher slots; other spells need known heightened entries per Remaster spontaneous rules.

### Focus-only / hybrid

Champion devotion, Monk qi (feat), Ranger focus feats, etc.: focus pool without ranked slots.

## Data shapes

```text
SpellDefn {
  id, name, traits: [tradition…, school-less Remaster traits, cantrip?, focus?, …],
  rank: u8,                     // minimum rank (0 for cantrip/focus display rules as needed)
  traditions: [arcane|divine|occult|primal],
  cost: { actions: 1|2|3|reaction|free, components: [material|somatic|verbal|focus…] },
  range, area, targets,         // structured, not prose
  defense: None | SpellAttack | Save { kind, basic: bool },
  effects: [ SpellEffect ],     // typed pipeline
  heighten: [ HeightenRule ],   // +1 cumulative | at rank N replace
}

SpellcastingProgression {
  style: prepared | spontaneous | focus_only,
  tradition,
  key_attribute,                // usually class key
  slots_by_level: { char_level -> { rank -> count } },
  cantrips_known_or_prepared: u8,
}

SpellcastingState {             // on character / daily
  prepared: [ PreparedSpell { spell_id, rank, slot_index } ],
  repertoire: [ KnownSpell { spell_id, ranks_known: [u8] } ],
  signature: [ spell_id ],
  focus_spells: [ spell_id ],
  focus_points: { current, max },
  slots_remaining: { rank -> count },
}
```

### Heighten rules

```text
HeightenRule =
  | PlusOne { each_rank_above_base: SpellEffectDelta }
  | AtRank { rank, replace_or_add: SpellEffectDelta }
```

Cantrips/focus: `effective_rank = ceil(character_level / 2)`.

## Encounter API sketch

```text
legal: CastSpell intents for each affordable casting
declare: CastSpell { caster, spell_id, rank, targets, area_origin? }
  -> spend actions
  -> provoke Reactive Strike if cast has manipulate components as applicable
  -> resolve attack or saves
  -> apply SpellEffects
  -> spend slot / FP
```

Components and Reactive Strike: somatic/material casts generally have manipulate; follow Remaster Cast a Spell + trait rules when wiring disruption (baseline already disrupts manipulate on critical Reactive Strike).

## Ranges, targets, areas

Structured enums, grid-aware:

- `Self`, `Touch`, `Feet(u32)`, `Sight` (stub carefully)
- Targets: creatures, objects, allies, enemies, counts
- Areas: burst/emanation/cone/line with foot sizes; template on grid

Line of effect / line of sight: reuse cover segment tests where possible; ledger substitutes if simplified.

## Defense resolution

- **Spell attack:** d20 + spell attack vs AC (MAP applies if the spell has the attack trait).
- **Save:** target rolls save vs caster spell DC; degree drives effect.
- **Basic save:** crit success no damage, success half, failure full, crit failure double—for damage effects.

Spell DC / attack = proficiency + key attribute + bonuses (prepare-derived).

## Required substrate

From [conditions-and-effects.md](../conditions-and-effects.md): conditions the sample catalog applies; heal; damage types; temporary HP for some focus spells.

## Counteracting

Stub: compare counteract rank + bonus vs target effect DC/rank. Full tables when Dispel Magic ships.

## Remaster rename notes (engine glossary)

| Prefer | Avoid (Legacy) |
| --- | --- |
| Spell rank | Spell level |
| Reactive Strike | Attack of Opportunity |
| Attribute | Ability score |
| Courageous Anthem (etc.) | Inspire Courage (Legacy name) |
| Schools as curricula (wizard) | Classic eight-school lock only |

Always store Remaster spell ids/names in fixtures.

## Verification predicates

1. Cantrip at level 1 is rank 1; at level 5 is rank 3.
2. Focus cast at 0 FP is illegal; at 1 FP succeeds and leaves 0.
3. Prepared caster cannot cast an unprepared ranked spell.
4. Spontaneous signature spell can be cast from a higher slot with heighten applied.
5. Basic save damage matches degree table with seeded dice.
6. Critical Reactive Strike against a manipulate casting disrupts per ledger/rules already used for Interact.

## See also

- [sample-catalog.md](./sample-catalog.md) — spells to implement first
- [tradition-notes.md](./tradition-notes.md) — per-tradition hooks
- [../implementation-roadmap.md](../implementation-roadmap.md) Phase 7+
