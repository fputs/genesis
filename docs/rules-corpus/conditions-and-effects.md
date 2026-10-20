# Conditions and effects — shared substrate

Shared condition track, bonuses/penalties, damage pipelines, and ephemeral effects required by classes and spells. Remaster Player Core / GM Core naming. Paraphrased procedures for implementers.

**Already in engine:** prone, dying, wounded, dead, unconscious (derived), off-guard (derived per Strike), raised shield, taking cover, detection (concealed/hidden/undetected), difficult terrain.

**Not yet:** valued conditions, persistent damage, temp HP, resistances, most spell effects.

## Condition instance

```text
ConditionState {
  id: ConditionId,
  value: Option<u8>,          // frightened 2, drained 1, …
  duration: Duration,         // rounds | end of turn | encounter | permanent | until_dismissed
  source: Option<EffectSource>,
  linked: Option<ConditionId>, // e.g. grabbed ↔ grabber
}
```

Rules of thumb:

- Valued conditions usually decrease by 1 at the end of the affected creature’s turn unless a longer duration is stated.
- Immunity to a condition prevents gaining it; already-present conditions are not always cleared—follow the specific immunity text when implementing.
- Some conditions are mutually exclusive or upgrade (hidden vs undetected is detection, not this map).

## Priority substrate for L1–2 classes / sample spells

| Condition | Value? | Why needed early |
| --- | --- | --- |
| Frightened | yes | Demoralize later; Bard/fear spells; Fighter Bravery at L3 |
| Sickened | yes | Disease/poison spells; alchemist mutagens later |
| Clumsy / Enfeebled / Stupefied | yes | Common spell debuffs; affect skills/attacks/DCs |
| Slowed / Quickened / Stunned | yes | Action economy changers |
| Grabbed / Restrained / Immobilized | flag/value | Athletics actions later; some spells |
| Fatigued | flag | Barbarian rage end |
| Blinded / Dazzled / Deafened | flag | Perception/Seek interactions |
| Drained / Doomed | yes | Necromancy; Champion/evil options |
| Confused / Controlled / Fleeing | flag | Higher spell tiers; stub until catalog needs them |
| Invisible | flag | interacts with detection |
| Off-guard | derived | prone, unconscious, flanked; also Feint/Surprise Attack keys |

### Rage {#rage}

Not always listed as a “condition” in casual talk; implement as a styled state:

- Start: 1 action; gain temporary HP; bonus damage; AC penalty; resist physical (instinct may alter); limited concentrate/actions per Remaster barbarian Rage.
- Ends: encounter end, unconscious, or voluntary; then fatigued until rest (Remaster).
- Store: `raging`, temp HP from rage, instinct id.

### Panache {#panache}

Swashbuckler resource-state: gained via style finishers/skills; spent on finishers. Not a valued condition. Precise Strike dice apply when panache rules say so.

### Curse (Oracle) {#curse}

Mystery-specific staged effect. Store stage; revelation spells advance it; daily preparations reset per mystery rules.

### Designated targets {#designated-targets}

Hunt Prey, some investigator/champion features: store `CombatantId` + feature id. Bonuses are synthetics gated by predicate `target is prey`.

## Bonus and penalty types

When combining modifiers on one roll or AC:

| Type | Stacking |
| --- | --- |
| Circumstance | best bonus, worst penalty |
| Status | best bonus, worst penalty |
| Item | best bonus, worst penalty |
| Untyped | all apply |
| Circumstance from MAP | special: MAP is a circumstance penalty on attack rolls; Aid bonus is circumstance and applies beside MAP (already implemented for Aid) |

Shield AC bonus and cover are both circumstance—engine already takes the larger.

## Damage pipeline

1. Determine base damage (dice + bonuses) and damage type tags (`slashing`, `fire`, `precision`, …).
2. Apply immunity (damage becomes 0 for that type if immune).
3. Apply weakness (increase) and resistance (decrease) per type.
4. Precision: if target is immune to precision, drop precision component.
5. Spend temporary HP, then real HP.
6. Persistent damage: separate instance with type + flat/dice; flat check to remove at end of turn (DC per Remaster).

### Precision damage {#precision-damage}

Used by Sneak Attack, Strategic Strike, hunter’s edge Precision, swashbuckler precise strike. Engine helper: `add_precision(damage, dice)` only if predicate passes (typically off-guard or feature-specific).

### Persistent damage

```text
Persistent { typ, formula, dc_to_remove }
```

Tick once at end of the creature’s turn before/after condition value decay in a documented order (pick one, test it, cite GM Core / Player Core recovery ordering).

## Temporary HP

- Separate pool; damage hits temp HP first.
- Multiple temp HP sources: generally take the higher remaining, don’t stack freely—follow Remaster temporary Hit Points rule when implementing.
- Rage temp HP is one source among others.

## Immunities, resistances, weaknesses

Store on prepared creature (from ancestry, spells, rage, items):

```text
Immunity: ConditionId | DamageType | Trait
Resistance: { typ, value }
Weakness: { typ, value }
```

Spells like “resist energy” grant timed resistance entries.

## Armor and item bonuses {#armor-and-item-bonuses}

- AC = 10 + Dex (capped by armor) + proficiency + item (armor) + other modifiers.
- Unarmored defense proficiency matters for monks/casters.
- Until itemization ships, fixtures may bake AC with a ledger note.

## Action-economy effects

| Effect | Engine meaning |
| --- | --- |
| Slowed N | reduce actions on turn start by N (min 0) |
| Quickened | +1 action restricted to the quicken grant |
| Stunned N | lose N actions; stunned interacts with slow—implement per Remaster table |
| Flourish used | round flag |
| Reaction spent | today’s `reaction_ready` |

## Spell effect ephemeral layer

Prefer small typed effects over string scripts:

```text
Effect =
  | ApplyCondition { … }
  | Damage { … }
  | Heal { … }
  | Move { … }
  | Summon stub
  | Bonus { selector, mod, duration }
```

Heightened spells change numbers via heighten rules in [spells/framework.md](./spells/framework.md).

## Consumables {#consumables}

Alchemist bombs/elixirs: item instances with `uses`, effect id, splash radius. Interacting/Activating may be manipulate; bombs usually are Strikes with attached effects.

## Fidelity substitutes (template)

When a condition cannot run fully (e.g. social-only frightened expiry):

- Rule + book section
- Why not in CRPG encounter
- Substitute
- Intent kept

File under [`../research/fidelity-ledger.md`](../research/fidelity-ledger.md) when code ships.

## Minimum implement set vs stub

| Fully specify before casters | Stub OK until a slice needs it |
| --- | --- |
| Frightened, clumsy, enfeebled, stupefied | Confused, controlled |
| Slowed, stunned, quickened | Petrified |
| Fatigued | Cursed (generic) |
| Temp HP, persistent, resist/weak | Complex auras |
| Precision helper | Spirit/vitality damage quirks beyond type tags |

## See also

- [architecture.md](./architecture.md)
- [implementation-roadmap.md](./implementation-roadmap.md) Phases 1–2
- Class pages under [classes/](./classes/)
