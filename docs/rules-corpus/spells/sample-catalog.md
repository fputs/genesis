# Sample spell catalog (implementation priority)

A short list that exercises the framework—not Player Core completeness. Each entry: engine procedure, heighten, required substrate, Remaster notes. Paraphrase only; cite book chapter “Spells.”

**Status:** Fully specified for engine wiring below. Broader lists = content-pack stubs (`not yet specified`).

## Tier A — ship with Phase 7–8

### Shield (cantrip, abjuration-ish traits per Remaster)

| Field | Spec |
| --- | --- |
| Traditions | arcane, … (as printed) |
| Cast | 1 action |
| Rank | cantrip |
| Effect | +1 circumstance AC until start of next turn; can use Shield Block as if raised shield **or** follow exact Remaster Shield cantrip text (force shield / Hardness)—implement the Remaster cantrip, not Legacy confusion |
| Heighten | auto |
| Substrate | circumstance AC; optional Shield Block interaction |
| Proves | cantrip, duration to turn start |

### Electric Arc (cantrip)

| Field | Spec |
| --- | --- |
| Traditions | arcane, primal |
| Cast | 2 actions |
| Targets | 1 or 2 creatures within range |
| Defense | basic Reflex |
| Effect | electricity damage dice by rank |
| Heighten | +1 → more dice (per printed +1 pattern) |
| Substrate | basic save, multi-target |
| Proves | cantrip damage, two targets |

### Guidance (cantrip)

| Field | Spec |
| --- | --- |
| Traditions | divine, occult |
| Cast | 1 action |
| Effect | +1 status to one attack roll, Perception, or saving throw before duration ends; once |
| Substrate | timed status bonus token |
| Proves | buff cantrip |

### Detect Magic (cantrip) — optional stub

Exploration-oriented. **Encounter stub:** legal out of combat only, or no-op with ledger note until exploration mode needs it.

### Heal (rank 1+)

| Field | Spec |
| --- | --- |
| Traditions | divine, primal |
| Cast | 1 / 2 / 3 actions change targets/area per Remaster Heal |
| Effect | healing or vitality damage to undead; action-cost variants |
| Heighten | +1 more dice |
| Substrate | heal pipeline; undead trait; areas for 3-action |
| Proves | multi-action spell, heal, font synergy |

### Harm (rank 1+)

Mirror of Heal for void/negative-style Remaster naming; needed for Cleric font choice.

### Magic Missile (rank 1)

| Field | Spec |
| --- | --- |
| Traditions | arcane |
| Cast | 1–3 actions (missile count) |
| Defense | automatic hit (force); no AC |
| Heighten | more missiles / dice per print |
| Substrate | multi-missile event log |
| Proves | auto-hit; action scaling |

### Fear (rank 1)

| Field | Spec |
| --- | --- |
| Defense | Will |
| Effect | frightened value by degree |
| Substrate | frightened condition |
| Proves | valued condition from save |

### Grease (rank 1)

| Field | Spec |
| --- | --- |
| Effect | area creates difficult terrain / Acrobatics or Reflex to fall prone—implement printed Remaster procedure |
| Substrate | terrain overlay, prone |
| Proves | area + terrain mutation |

### Bless (rank 1)

| Field | Spec |
| --- | --- |
| Effect | emanation status bonus to attack rolls / saves vs fear as printed |
| Substrate | aura/emanation tracking on combatants |
| Proves | sustained or timed aura (if sustain required, add Sustain action) |

### Bane (rank 1)

Debuff counterpart pattern to Bless.

### Mending / Stabilize

Utility: Stabilize as moral support for dying track (medicine-ish cantrip/spell per print). Useful for fidelity with dying already implemented.

## Tier B — focus & class kits

### Courageous Anthem (bard composition cantrip)

| Field | Spec |
| --- | --- |
| Kind | composition cantrip |
| Effect | status bonus to attack and damage for allies in range while active |
| Substrate | composition / performance; sustain or duration per Remaster |
| Remaster note | Legacy name Inspire Courage |

### Lay on Hands / Touch of Corruption (champion devotion)

| Field | Spec |
| --- | --- |
| Kind | focus |
| Effect | heal or harm touch; optional AC bonus rider if printed for that cause |
| Substrate | focus pool; touch range |

### Phase Familiar / Telekinetic Hand hex stubs (witch)

One hex cantrip + one focus hex that applies a condition or flat damage. Familiar must exist for patron features that require it.

### Druid order spell (example: Thunderbolt / Heal-adjacent)

One order focus spell tied to [classes/druid.md](../classes/druid.md) first slice.

### Bloodline focus spell (sorcerer)

One focus spell from a single bloodline chosen for the fixture (e.g. elemental blast-style).

### Revelation focus spell (oracle)

One revelation; casting advances curse stage mild → moderate per mystery.

## Tier C — areas & complexity (Phase 13)

| Spell | Why later |
| --- | --- |
| Fireball | burst area, basic Reflex, heighten +1 dice |
| Haste / Slow | quickened/slowed |
| Invisibility | detection interaction |
| Summon animal/undead | summoned combatant lifecycle |
| Wall spells | terrain/blocking geometry |
| Dispel Magic | counteract |

Each needs a one-paragraph effect procedure before coding—copy this file’s table pattern.

## Registry stub (not specified)

All other Remaster spells from Player Core / PC2: content id reserved, `spec_status: unset`. Do not block framework PRs on filling them.

Approximate Remaster counts (community catalogs ~600+ including focus/rituals)—engine v1 should not treat “catalog complete” as a merge gate ([open-questions.md](../open-questions.md) item 11).

## Per-spell verification template

```text
test spell_<id>_rank_<n>_seeded:
  - prepare caster fixture
  - cast with ScriptedDice
  - assert HP/conditions/slots/FP
  - cite Player Core Spells section in test comment
```

## See also

- [framework.md](./framework.md)
- [tradition-notes.md](./tradition-notes.md)
