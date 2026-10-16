# Rules simulation (PF2e-like) in code

## Goal

Constitution III: **tabletop fidelity first**. Encode PF2e procedures so video-game changes are rare, documented, and intentional. MVP slice 1 only needs Move + Strike for L1 fighters, but the architecture must grow without rewriting the world.

## Proven digital PF2e pattern: Foundry VTT system

Primary references:

- [foundryvtt/pf2e](https://github.com/foundryvtt/pf2e) (TypeScript system)
- [Rule Elements overview](https://www.mintlify.com/foundryvtt/pf2e/rule-elements/overview)
- [Quickstart guide for rule elements](https://github.com/foundryvtt/pf2e/wiki/Quickstart-guide-for-rule-elements)
- Actor preparation code paths in `src/module/actor/base.ts`

### Architecture (what to learn)

1. **Content is data** (JSON items/actors/feats/spells) with typed fields.
2. Many mechanical contributions are **Rule Elements**: small JSON objects with a `key` (`FlatModifier`, `Strike`, `AELike`, `RollOption`, …) deserialized to classes.
3. On actor prepare:
   - collect REs from owned items
   - sort by priority / item type ordering
   - run phased hooks (`applyAEs` → `beforeDerived` → `afterDerived` → `beforeRoll`)
   - accumulate **synthetics** (modifiers, notes, strikes) then derive final stats
4. Predicates / roll options gate when modifiers apply — evaluated at roll time when needed.

### What to copy for Genesis (Rust)

- **Phased preparation** with documented order (avoids recursive “modify X from final X” bugs — Foundry wiki calls this out explicitly).
- **Data-driven contributions** for feats/items/conditions; **Rust enums/traits** for the interpreter keys.
- **Synthetics accumulator** then derive — do not scatter ad hoc mutations across systems.
- Fail soft on a single bad rule in tools; **fail closed in tests/CI** for fixture validity.

### What not to copy blindly

- Foundry’s JS actor document model and UI coupling.
- Shipping the entire Foundry RE surface on day one — MVP only needs Strikes, Speed/Stride, attack roll math, MAP, HP, basic armor AC.
- Assuming Foundry JSON is a legal redistributable rules database for a commercial engine — treat as **architecture reference**; license/content policy is a separate user decision.

## IE / GemRB lesson (older pattern)

IESDP documents effects as **opcodes + parameters** applied in defined orders. That is the same family of idea as Rule Elements: compact data + interpreter.

Genesis should prefer **typed RE-like structs** over opaque numeric opcodes for maintainability and tests, while keeping the data/interpreter split.

## Recommended crate split (engine-agnostic)

```
genesis-rules        # pure Rust: stats, checks, actions, encounter flow
genesis-rules-data   # serde types + loaders (RON/JSON)
genesis-fixtures     # MVP encounter/party YAML/RON/JSON (test content only)
genesis-sim          # optional: world/time/mode orchestration without rendering
```

**Hard rule:** `genesis-rules` must not depend on Bevy/Fyrox/Godot. Presentation calls into rules; editor calls into the same API.

## Data vs logic boundary

| In data (fixtures / packs) | In Rust logic |
| --- | --- |
| Creature/PC stat blocks for MVP | Check resolution (d20 + mods vs DC) |
| Weapon Strike definitions | Action economy (3 actions, MAP) |
| Map/encounter spawn lists | Legal action generation |
| Traits as tags (when needed) | Degree of success |
| Later: feat RE lists | Condition machine, targeting |

MVP can hardcode fighter Move/Strike legality in Rust **if** the numbers still load from fixtures — but prefer declaring Strikes as data early so the editor path is obvious.

## Fidelity testing strategy

1. **Tabletop citation tests**: for each implemented procedure, a test names the rule source (Constitution VII allows tabletop citations in tests).
2. **Golden encounters**: seeded RNG scripts (e.g. known attack rolls) assert HP and MAP progression.
3. **Property tests** sparingly for modifier stacking invariants.
4. **Deviation ledger**: any intentional video-game change recorded per Constitution III (research/constitution, not code comments).
5. Headless `cargo test -p genesis-rules` in CI before UI work lands.

## External data sources (caution)

| Source | Role | Caution |
| --- | --- | --- |
| Archives of Nethys site + Elasticsearch API used by community tools | Reference lookup | Scraping/redistribution legality and ToS; not an official Paizo SDK |
| Community AoN scrapers / parsed JSON repos | Convenience dumps | Same legal caution; freshness varies |
| Foundry pf2e packs | Architecture + community data shapes | Foundry/system licensing; not automatically reusable in a shipped engine |
| Paizo ORC / Remaster texts | Authoritative rules intent | Follow Paizo licensing for what you redistribute in-repo |

**Unknown / open:** Genesis’s exact content licensing strategy (ORC-only subset vs private fixtures vs licensed data). For MVP, **hand-authored fixture stats** avoid blocking on redistribution questions.

## Determinism

- Inject `Rng` trait; production uses a seeded PRNG; tests use scripted dice streams.
- Separate “presentation roll display” from “resolution rolls” so UI never double-rolls.
- Community crate `loom_ruleset` (docs.rs) shows a deterministic JSON-AST evaluator pattern; **not** PF2e-complete — cite only as prior art for deterministic evaluation, not as a drop-in ruleset.

## MVP slice 1 minimum rules surface

Must work end-to-end:

- Initiative (simplified OK if documented)
- Turn/round advancement for 5 combatants (4 PCs + 1 enemy)
- Stride/Move spending actions and respecting Speed
- Strike: attack roll vs AC, damage on hit/crit, MAP
- HP / dying-or-defeat for the one enemy (PC death can be stubbed if needed, but 4 slots remain)

Defer: spells, feats beyond baseline fighter Strike, crafting, exploration activities, full condition library.
