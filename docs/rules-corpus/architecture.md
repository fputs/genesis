# Rules architecture — classes, archetypes, spells

How class/archetype/spell content plugs into Genesis `crates/rules` without bolting class-specific branches onto `encounter.rs`.

**Baseline today (`main` ~`2f56abb`):** `CreatureTemplate` → `apply_effects` / `Synthetics` → `PreparedCombatant` → `Combatant` in an `Encounter`. Actions are a closed `Action` enum. The only `ClassFeature` variant is `ReactiveStrike`. Shield Block is a separate `bool`. Numbers are mostly prebaked in fixtures.

**Target:** Foundry-style phased preparation + synthetics (already sketched in repo research), with content-driven features, while keeping resolution pure Rust and headless-testable.

## Current types (attach here)

| Type | Role today | Growth |
| --- | --- | --- |
| `CreatureTemplate` | Fixture input: HP, AC, Perception, Speed, one Strike, features, shield, `shield_block`, `stealth_dc` | Become the *compiled* snapshot input, or split into `CreatureDefn` (authored) + prepare |
| `Synthetics` | Accumulator: AC/attack/Perception/Speed deltas + feature list | Grow typed buckets: modifiers-by-type, strikes, immunities, actions granted, roll options |
| `ClassFeature` | Enum with `ReactiveStrike` only | Prefer data ids + capability flags over a mega-enum; keep hot-path capabilities as Rust enums/bitflags |
| `Action` | Closed set of encounter actions | Extend carefully; spellcasting becomes `CastSpell { spell_id, rank, targets, … }` or a parallel intent type |
| `Combatant` | Runtime encounter state | Conditions map, resources (focus, slots), designations (prey), stances, panache/rage, spell effects |
| `prepare_combatant` / `apply_effects` | Thin copy today | Multi-phase prepare (see below) |

Do **not** encode “if fighter …” inside Strike resolution. Encode “has reactive strike capability”, “precision dice when predicate”, “modifier with type/predicate”.

## Layering

```
content pack (RON/JSON)
  ancestry, background, class, feats, items, spells, archetypes
        │
        ▼
character build (editor or fixture authoring)
  choices + level → owned feature list
        │
        ▼
daily preparation
  slots, infusions, flexibility feats, bonded item, etc.
        │
        ▼
rules::prepare (phased)
  collect rule elements → synthetics → derive stats → grant actions
        │
        ▼
Encounter runtime
  legal_actions / declare_action / reactions / conditions
```

`crates/content` owns serde schemas and validation. `crates/rules` owns procedures. `crates/game` / `crates/editor` never fork rules.

## Phased preparation

Recommended phase order (names are local; keep them stable in docs/tests):

1. **Collect** — gather rule elements from class features, feats, items, active conditions, archetype feats.
2. **Early** — base HP from ancestry+class+Con; proficiency ranks; attribute modifiers; class choices.
3. **Before derived** — flat modifiers, granted strikes, armor contributions, sense/stealth DC.
4. **Derive** — final AC, attack bonuses, damage formulas, DCs, Speed, Perception, saves.
5. **After derived** — effects that need final numbers (some specializations).
6. **Runtime hooks** — predicates evaluated at roll time (`before_roll` / `after_roll`), not at prepare.

`Synthetics` should accumulate:

- Modifiers: `{ selector, value, type: circumstance|status|item|untyped, predicate }`
- Strikes: weapon/unarmed definitions
- Granted actions / reactions / free actions (ids)
- Resources: focus pool max, spell slot tables, reagents
- Tags / roll options: `off-guard`, `target:hunted-prey`, `self:raging`, …

## Proposed data shapes

### Character (authored)

```text
CharacterDefn {
  id, name, level,
  ancestry_id, heritage_id, background_id,
  attributes: { str, dex, con, int, wis, cha },
  class_id,
  class_choices: ClassChoices,          // racket | muse | school | …
  feats: [ FeatPick { feat_id, slot: class|skill|general|ancestry|archetype } ],
  skills: { Skill -> Rank },
  inventory: [ ItemInstance ],
  spells: SpellcastingState,            // optional
  archetype_dedications: [ ArchetypeProgress ],
}
```

### Class definition (content)

```text
ClassDefn {
  id, key_attribute: Attribute | Choice<[Attribute]>,
  hp_per_level: u8,
  initial_proficiencies: ProficiencyBlock,
  features_by_level: { level -> [FeatureRef] },
  spellcasting: Option<SpellcastingProgression>,
}
```

### Feature / feat as rule elements

```text
FeatureDefn {
  id, name, traits[],
  grant_at: Level | Choice,
  elements: [ RuleElement ],
  // optional explicit action definition
  action: Option<ActionDefn>,
}

RuleElement =
  | Grant prof { statistic, rank }
  | FlatModifier { … }
  | GrantAction { action_id }
  | GrantReaction { reaction_id }
  | GrantSpell { spell_id, kind: slot|focus|cantrip|innate }
  | AdjustHitPoints { … }
  | Note / Choice stub
```

### Runtime combatant extensions

Keep `Combatant` fields for hot state; avoid stringly maps on every path:

```text
conditions: BTreeMap<ConditionId, ConditionState>  // value, duration, source
resources: { focus, spell_slots, infusions, … }
designations: { hunted_prey: Option<CombatantId>, … }
style_state: { raging, panache, stance_id, taking_cover, … }  // expand today’s bools
spell_effects: ephemeral timed effects
```

Off-guard stays **derived** per Strike (prone, unconscious, flanked, and later: surprise attack, feint, etc.).

## Actions, reactions, free actions

- **Baseline actions** remain first-class Rust variants for the encounter loop.
- **Class feats** that are actions register through a capability table: `ActionId` → handler in rules, legality predicates, MAP interaction, traits (`flourish`, `press`, `manipulate`, `attack`, `concentrate`, `stance`, …).
- **Flourish:** at most one flourish action per round per creature (Monk Flurry, many fighter techniques).
- **Press:** legal only under MAP; Ready cannot use press (Player Core fighter key terms).
- **Reactions:** generalize today’s single `reaction_ready` into “reaction budget” when features grant extra reactive uses (still regain on turn start unless a feature says otherwise). Record fidelity substitutes if simplified.

Casting: prefer `Action::Cast { spell_instance_id, … }` once the spell framework exists, rather than one enum variant per spell.

## Conditions and modifiers

Shared substrate lives in [conditions-and-effects.md](./conditions-and-effects.md). Preparation and checks must:

- Track bonus/penalty **type** and keep the best bonus and worst penalty per type (except untyped).
- Apply immunities before damage; resistances/weaknesses after.
- Treat temporary HP as a separate pool spent before real HP.

## Spellcasting plug-in

See [spells/framework.md](./spells/framework.md). Preparation produces a `SpellcastingBundle` on the prepared creature: tradition, attack/DC, slot table or repertoire, focus pool, known/prepared lists. Encounter spend/refund (Drain Bonded Item, etc.) mutates runtime resources, then may require a light re-derive.

## Archetypes

See [archetypes.md](./archetypes.md). Dedication and archetype feats are feats with `archetype` trait occupying class-feat slots (sometimes skill slots). They contribute the same rule elements as class feats. Multiclass dedications cannot apply if `class_id` matches.

## Items and armor

MVP fixtures bake AC and one Strike. Class work needs:

- Armor category + item bonus + Dex cap → AC
- Weapon group, traits (`finesse`, `agile`, `deadly`, `reach`, …), damage die, hands
- Shield as today, plus later affixed runes as modifiers

Until itemization lands, a class slice may still ship with **baked** L1 numbers **only if** the fidelity ledger records that prepare is skipped for that fixture.

## What not to do

- Class-named branches in `declare_action` / `finish_strike`
- Copying Foundry JSON packs wholesale (licensing; see [open-questions.md](./open-questions.md))
- Growing `ClassFeature` into dozens of unit variants without data
- Putting spell text or feat novels into Rust source

## Mapping to files (expected)

| Concern | Likely home |
| --- | --- |
| Prepare phases, synthetics | `crates/rules/src/combatant.rs` (split modules as it grows) |
| Checks, MAP, degree | `crates/rules/src/check.rs` |
| Encounter loop, actions | `crates/rules/src/encounter.rs` (handlers dispatch by capability) |
| Conditions | new `crates/rules/src/condition.rs` |
| Spells | new `crates/rules/src/spell.rs` + content schemas |
| Schemas / fixtures | `crates/content`, `fixtures/` |

## See also

- Repo: [`../crates/rules/README.md`](../crates/rules/README.md), [`../research/rules-simulation.md`](../research/rules-simulation.md), [`../research/fidelity-ledger.md`](../research/fidelity-ledger.md)
- Corpus: [implementation-roadmap.md](./implementation-roadmap.md), [classes/](./classes/)
