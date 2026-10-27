# Proposal

## Why

Every blow is still an untyped number. A creature cannot ignore, resist, or be weak to a damage type, and nothing deals damage again at the end of its turn. Phase 1 left that as DEV-09 and as the persistent-damage clause of DEV-29. This is Phase 2 item 2.

## What Changes

- A damage instance is a non-negative amount plus one damage type. Immunity applies first, then the highest weakness of that type, then the highest resistance of that type. Resistance stops at 0. The penalty floor of 1 and critical doubling stay where they are, before this adjustment.

- The baked strike and the Strike profile each carry one damage type. Strike, Reactive Strike, and a readied Strike adjust the rolled total, then Shield Block sees what remains. `basic_save` stays an untyped final amount.

- Persistent damage is a condition: a damage type and a flat amount. At the end of that creature's turn the amounts are adjusted, applied as one ordinary blow, then each condition rolls a DC 15 flat check to end.

- Assisted recovery is a non-goal. The only recovery in this change is that DC 15 check, plus bleed ending on a heal to full Hit Points.

- **BREAKING:** `MeleeStrike`, `StrikeProfile`, and `BuildParts` gain required fields. `end_turn` takes the injected dice. `ConditionSlot` becomes an enum. `Encounter` gains `gain_persistent_damage`. `TemplateError` and `EncounterError` gain a variant, and `ReadyEncounter` gains the matching variant.

This change depends on `add-conditions-with-durations`, which is being drafted in parallel and is not in this tree. Assumption, so the two specs can be reconciled: that change provides a generic condition on `CombatantState` and one end-of-turn tick. This change assumes nothing else about it. No durations, no condition values, and no named condition other than persistent damage. Persistent damage does not expire by counting rounds. If that change names the container or the tick differently, a persistent entry is still one condition value of (damage type, positive amount), processed in the end-of-turn tick, before any end-of-turn decrease of other conditions.

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

This pull request is the spec commit. It adds no code, no tests, and no changelog bullet, and it does not archive. Designer rules on the spec here. Code waits for a `spec-approved: <sha>` review from Evan. The ruling log is [docs/designer/rules-rulings.md](../../../docs/designer/rules-rulings.md). The code phase records DEV-59 and DEV-60 in `docs/designer/rules-deviations.md` and amends the persistent-damage clause of DEV-09 and DEV-29. The spec commit does not edit that file.

The archive commit is last, right before merge, and only after Architect and QA have both passed that head.
