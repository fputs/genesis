# Proposal

## Why

Every blow is still an untyped number. A creature cannot ignore, resist, or be weak to a damage type, and nothing deals damage again at the end of its turn. Phase 1 left that as DEV-09 and as the persistent-damage clause of DEV-29. This is Phase 2 item 2.

## What Changes

- A damage instance is a non-negative amount plus one damage type. Immunity applies first, then the highest weakness of that type, then the highest resistance of that type. Resistance stops at 0. The penalty floor of 1 and critical doubling stay where they are, before this adjustment.

- The baked strike and the Strike profile each carry one damage type. Strike, Reactive Strike, and a readied Strike adjust the rolled total, then Shield Block sees what remains. `basic_save` stays an untyped final amount.

- Persistent damage is a condition: a damage type and a flat amount. At the end of that creature's turn the amounts are adjusted, applied as one ordinary blow, then each condition rolls a DC 15 flat check to end. A critical hit's caller passes twice the listed persistent amount. Persistent Fire 2 is stored as 4, not 6.

- Assisted recovery is a non-goal. The only recovery in this change is that DC 15 check, plus bleed ending on a heal to full Hit Points.

- **BREAKING:** `MeleeStrike`, `StrikeProfile`, and `BuildParts` gain required fields. `end_turn` takes the injected dice. `ConditionSlot` becomes an enum. `Encounter` gains `gain_persistent_damage`. `TemplateError` and `EncounterError` gain a variant, and `ReadyEncounter` gains the matching variant.

This change lands first and owns `ConditionSlot` and the persistent container. `add-conditions-with-durations` rebases onto it. Origin has an empty `ConditionSlot` and a `conditions` vector. This change needs a container of (damage type, positive amount) entries and one end-of-turn hook. That hook runs before any end-of-turn decrease ([Player Core pg. 436, Step 3: End Your Turn](https://2e.aonprd.com/Rules.aspx?ID=2430)). It does not assume a conditions interface that is not on origin. Persistent damage does not end by counting rounds. Durations and every other named condition stay in the conditions change.

## Capabilities

### New Capabilities

- `damage-defenses`: Damage types, immunity then weakness then resistance, and persistent damage at end of turn.

### Modified Capabilities

- `creature-template`: The strike records a damage type. Defenses are copied, not derived. The penalty floor stays; resistance may then reduce the result to 0.

- `encounter-turns`: With no persistent damage, the end step still rolls nothing and does not change Hit Points. With persistent damage, it follows `damage-defenses`. `end_turn` takes dice.

- `shields`: Shield Block triggers on bludgeoning, piercing, or slashing only, after the adjustment. Persistent damage still does not block.

- `api-facade`: The new types are exported. `end_turn` takes dice. `gain_persistent_damage` is public.

- `dying-and-victory`: The DEV-29 note no longer says persistent damage is unmodeled. A blow is still lethal, and the critical flag is still the only critical signal.

## Impact

`crates/rules` only. No new crate and no new dependency. No file on `v1-reference` is named, so none is ported. `basic_save` is not edited. Skills, spells, areas, and the condition duration model are not edited.

This pull request is the spec commit. It adds no code, no tests, and no changelog bullet, and it does not archive. Designer rules on the spec here. Code waits for a `spec-approved: <sha>` review from Evan. The ruling log is [docs/designer/rules-rulings.md](../../../docs/designer/rules-rulings.md). The code phase records DEV-62 and DEV-63 in `docs/designer/rules-deviations.md` and amends the persistent-damage clause of DEV-09 and DEV-29. That edit notes the reserved ranges: DEV-59 through DEV-61 are `add-skills` (pull request 91), and DEV-64 through DEV-72 are `add-spells-and-areas` (pull request 92). DEV-73 and later stay with `add-conditions-with-durations`. The spec commit does not edit that file.

The archive commit is last, right before merge, and only after Architect and QA have both passed that head.
