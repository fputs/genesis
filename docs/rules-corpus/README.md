# Genesis rules corpus — Pathfinder 2e Remaster

Implementation-shaped research for finishing the Genesis rules engine: classes, archetypes, and spells. **Not** engine code. **Not** a paste of Player Core.

**Repo tip researched:** `fputs/genesis` `main` @ `2f56abb` (baseline actions merged).  
**Product constraints:** Remaster naming; tabletop fidelity first; fixed 4 PC party; `rules` / `content` shared by game and editor.

## How to read this

| Reader | Path |
| --- | --- |
| New implementer | This README → [architecture.md](./architecture.md) → [implementation-roadmap.md](./implementation-roadmap.md) Phase 1 |
| Class work | [classes/README.md](./classes/README.md) → one class file → linked conditions/spells |
| Caster work | [spells/framework.md](./spells/framework.md) → [spells/sample-catalog.md](./spells/sample-catalog.md) |
| Multiclass | [archetypes.md](./archetypes.md) |
| Product scope calls | [open-questions.md](./open-questions.md) |

Diátaxis: roadmap = how-to; class/spell pages = reference; architecture = explanation.

## Already implemented vs planned

### On engine `main` (baseline)

Stride, Step, Strike, Reactive Strike, dying/wounded/dead, Raise a Shield / Shield Block, Drop Prone / Stand / Crawl / Leap, difficult terrain, Seek / Point Out / Take Cover / cover / detection, Interact / Release / manipulate disruption, Delay / Ready, Aid (Strike bonus vs DC 15 substitute). See [`../crates/rules/README.md`](../crates/rules/README.md) and [`../research/fidelity-ledger.md`](../research/fidelity-ledger.md). The completed baseline program brief is [baseline-rules.md](./baseline-rules.md) (historical).

### Planned in this corpus

| Area | Corpus location |
| --- | --- |
| Prepare / synthetics / feats | [architecture.md](./architecture.md) |
| Phased PR plan | [implementation-roadmap.md](./implementation-roadmap.md) |
| 16 Remaster core classes | [classes/](./classes/) |
| Dedications + companion primitives | [archetypes.md](./archetypes.md) |
| Spellcasting + sample spells | [spells/](./spells/) |
| Conditions, precision, temp HP | [conditions-and-effects.md](./conditions-and-effects.md) |

## Reading order for implementers

1. [`../constitution.md`](../constitution.md) + [`../crates/rules/README.md`](../crates/rules/README.md) + [`../research/fidelity-ledger.md`](../research/fidelity-ledger.md)  
2. [architecture.md](./architecture.md)  
3. [conditions-and-effects.md](./conditions-and-effects.md)  
4. [implementation-roadmap.md](./implementation-roadmap.md) **Phase 1**  
5. [classes/fighter.md](./classes/fighter.md) then [classes/rogue.md](./classes/rogue.md)  
6. [spells/framework.md](./spells/framework.md) before any caster class  
7. [archetypes.md](./archetypes.md) after feat slots exist  

## File map

```
rules-corpus/
  README.md
  architecture.md
  implementation-roadmap.md
  conditions-and-effects.md
  archetypes.md
  open-questions.md
  baseline-rules.md # historical: completed baseline action program
  classes/          # 16 class specs + index
  spells/           # framework, sample catalog, tradition notes
```

Do not duplicate [`../research/`](../research/); cross-link the fidelity ledger and rules-simulation notes from here. The live substrate plan is [implementation-roadmap.md](./implementation-roadmap.md)—older fighter-encounter execution checklists are not checked into this tree.

## Copyright / fidelity

- Paraphrase procedures; cite book + chapter/section.  
- Remaster terms (`rank`, `Reactive Strike`, `attribute`).  
- Temporary substitutes use the same four-part record as the fidelity ledger: rule, why not, substitute, intent kept.  

## Status note (coverage)

| Deliverable | Status |
| --- | --- |
| Architecture + data shapes tied to `CreatureTemplate` / `Synthetics` / `Action` | **Fully specified** |
| Implementation roadmap Phases 0–14 | **Fully specified** |
| Conditions / modifiers / damage substrate | **Fully specified** for L1–2 needs; exotic conditions stubbed |
| Player Core classes (8) | **Fully specified** at L1–2 playable slice + full advancement table + feature procedure summaries; individual feat bodies stubbed |
| Player Core 2 classes (8) | Same depth as Player Core |
| Other Remaster classes | **Registry only** in classes index |
| Archetype framework + Fighter/Rogue/Wizard/Cleric dedications + Familiar Master / Beastmaster proofs | **Fully specified**; other archetypes registry pattern |
| Spellcasting framework | **Fully specified** |
| Sample spells | **~15–20** Tier A/B specified; Tier C (Fireball, summons, walls) stubbed; full print catalog **not** specified |
| Open product questions | **12** with recommended defaults |

**Recommended first implementation phase:** [implementation-roadmap.md](./implementation-roadmap.md) **Phase 1** — attributes, proficiency ranks, skills, modifier types.

**Blocking product questions:** books in v1 packs; content licensing; companions vs party slots; rituals; anathema enforcement — see [open-questions.md](./open-questions.md).
