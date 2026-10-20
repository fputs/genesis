# Implementation roadmap — finish the Remaster rules engine

How-to plan for stacked PRs after the baseline action stack on `main` (~`2f56abb`). Research corpus only; no engine code in this document’s delivery.

**Principles:** scaffold substrate before class special cases; one falsifiable done predicate per phase; Remaster naming; fixed 4 PC party; fidelity ledger for every substitute.

## Reading this plan

Each phase lists: depends-on, crates/files likely touched, done predicate, verification. Prefer small stacked PRs inside a phase when a phase is large.

Suggested class order after scaffold: **Fighter feats → Rogue → Ranger → Barbarian → Monk → Champion → spellcasting framework → Cleric → Wizard → Bard → Druid → Witch → Sorcerer → Oracle → Investigator → Swashbuckler → Alchemist**, with archetypes proving dedications once feat slots exist.

## Phase 0 — Inventory and ledger hygiene

**Depends on:** baseline merged.

**Touch:** [`../research/fidelity-ledger.md`](../research/fidelity-ledger.md), [`../crates/rules/README.md`](../crates/rules/README.md) (docs-only if needed).

**Done when:** ledger matches code for Aid substitute, initiative ties, downed space, single-reaction model; corpus links from repo docs are optional.

**Verify:** doc review only.

---

## Phase 1 — Shared substrate A (attributes, proficiency, skills, modifiers)

**Depends on:** Phase 0.

**Touch:** `crates/rules` (new modules), `crates/content` schemas, fixtures may gain attribute/skill fields; tests in `crates/rules/tests`.

**Build:**

- Attributes and proficiency ranks (`Untrained`…`Legendary`) with bonus `0 / level+2 / +4 / +6 / +8`.
- Skill list (Remaster Player Core skills) and trained-skill checks.
- Modifier accumulator by type (circumstance / status / item / untyped) with non-stacking rules.
- Replace baked Perception/AC/attack only where a test fixture opts into derived mode; keep room-one fighters working.

**Done when:** a unit test derives Perception = Wis + proficiency; two circumstance bonuses do not stack; Aid can optionally roll a skill when present (keep Strike-bonus substitute behind a ledger flag until skills are wired into Aid).

**Verify:** `cargo test -p rules`; headless room-one still exits Victory/Defeat as today.

---

## Phase 2 — Shared substrate B (conditions, damage types, temp HP, resistances)

**Depends on:** Phase 1.

**Touch:** `crates/rules` condition machine; encounter damage path; ledger.

**Build:**

- Condition track: value, duration (rounds / encounter / dismiss), source id.
- Minimum set: frightened, sickened, clumsy, enfeebled, stupefied, quickened, slowed, stunned, restrained, grabbed, immobilized, fatigued, blinded, dazzled, deafened, drained, doomed, dying/wounded (already), prone (already), concealed/hidden/undetected (already as detection).
- Persistent damage tick at end of turn; temporary HP pool; resistance/weakness/immunity hooks on damage type tags.
- Precision damage helper (Sneak Attack / Strategic Strike / hunter edge).

**Done when:** applying frightened 2 lowers it by 1 at end of creature’s turn (or Remaster duration rule chosen); persistent fire deals seeded damage; temp HP absorbs before HP.

**Verify:** unit tests with citations; headless fight regression.

---

## Phase 3 — Feat slots + rule-element interpreter (minimal)

**Depends on:** Phase 1 (Phase 2 recommended).

**Touch:** `crates/rules` prepare path; `crates/content` feat schema; fighter fixture gains a feat id.

**Build:**

- Feat picks on character defn; class/skill/general/ancestry slot kinds.
- Interpreter keys: `GrantProf`, `FlatModifier`, `GrantAction`, `GrantReaction`, `Note`.
- Migrate `ClassFeature::ReactiveStrike` and `shield_block` to granted capabilities via elements (keep Rust capability flags for hot path).

**Done when:** adding a data-only L1 fighter feat that grants a documented `FlatModifier` changes prepared attack bonus in tests without editing `encounter.rs` match arms beyond registering one new action handler if the feat is an action.

**Verify:** `cargo test -p rules`; optional headless with feat-enabled fighter.

---

## Phase 4 — Armor / weapons itemization (thin)

**Depends on:** Phase 1.

**Touch:** content item schemas; prepare AC/Strike from items; fixtures.

**Build:** weapon traits needed for agile MAP, finesse, reach; armor item bonus + Dex cap; shield as today.

**Done when:** one fixture fighter derives AC from scale mail + Dex (or leather) matching a hand-calculated Remaster number in a test.

**Verify:** unit tests; room-one either updated fixtures or legacy baked path flagged.

---

## Phase 5 — Class: Fighter L1–2 (feat framework proof)

**Depends on:** Phases 3–4 (4 may be stubbed with ledger).

**Touch:** content `class/fighter`; feat defs; game HUD if new actions; `TESTER.md`.

**Done when:** L1 fighter with Reactive Strike, Shield Block, one class feat action (e.g. Power Attack-style two-action Strike with penalty/extra damage—implement the chosen feat’s procedure, not a rename), skill feat slot recorded; room-one party can mix feat users.

**Verify:** unit + headless; live HUD names new feat if interactive.

---

## Phase 6 — Martial non-casters (Rogue, Ranger, Barbarian, Monk, Swashbuckler, Investigator)

**Depends on:** Phases 2–3; Swashbuckler/Investigator after precision + check substitution exist.

**Order inside phase (stacked PRs):**

1. **Rogue L1–2** — racket, Sneak Attack, Surprise Attack  
2. **Ranger L1–2** — Hunt Prey, Precision edge (companion later)  
3. **Barbarian L1–2** — Rage + instinct stub  
4. **Monk L1–2** — Flurry (flourish), Powerful Fist  
5. **Investigator L1–2** — Devise a Stratagem, Strategic Strike  
6. **Swashbuckler L1–2** — panache, precise strike, Confident Finisher  

**Done when (each PR):** class doc’s first-slice predicate holds; new conditions/states ledgered if substituted.

**Verify:** per-PR unit tests; one headless encounter fixture per class.

---

## Phase 7 — Spellcasting framework

**Depends on:** Phases 1–3; conditions for common spell effects (frightened, clumsy, …).

**Touch:** `crates/rules/src/spell.rs` (new), content spell schemas, action `Cast`, focus pool, daily prep API.

**Build:** tradition, rank, components as traits, casting time (actions), range/targets/areas, save vs spell DC / spell attack, heighten, cantrips, focus vs slots, prepared vs spontaneous, counteract stub optional.

**Done when:** Cast a Spell spends the correct actions; cantrip heightens to half level rounded up; focus spell spends 1 Focus Point; prepared slot is consumed; spontaneous repertoire cast consumes a slot of that rank.

**Verify:** unit tests for each casting style; no full catalog required.

Details: [spells/framework.md](./spells/framework.md).

---

## Phase 8 — Divine & arcane prepared (Cleric, Wizard, Druid, Witch)

**Depends on:** Phase 7.

**Stacked PRs:** Cleric (font + Heal) → Wizard (spellbook + Magic Missile/Shield) → Druid (order focus) → Witch (familiar stub + hex).

**Done when:** each class doc first-slice predicate holds with ≤5 spells implemented from [spells/](./spells/).

**Verify:** unit + headless caster encounter.

---

## Phase 9 — Spontaneous casters (Bard, Sorcerer, Oracle)

**Depends on:** Phase 7–8 patterns.

**Build:** repertoire, signature spells, composition cantrips (Bard), bloodline (Sorcerer), mystery curse (Oracle).

**Done when:** Bard Courageous Anthem applies status bonus; Sorcerer casts from repertoire; Oracle revelation advances curse stage.

**Verify:** unit + headless.

---

## Phase 10 — Champion + Alchemist

**Depends on:** Phase 7 for Champion devotion; Phase 2 + items for Alchemist bombs.

**Done when:** Champion Lay-on-Hands-style devotion spell; Alchemist bomb Strike with splash using infused reagent budget.

**Verify:** unit + headless.

---

## Phase 11 — Archetype dedication framework

**Depends on:** Phase 3; at least two classes.

**Build:** dedication feat gating (2 archetype feats before next dedication); multiclass restriction; Fighter/Rogue/Wizard/Cleric dedications granting training + shallow spell/feat access.

**Done when:** a Fighter cannot take Fighter Dedication; a Rogue with Wizard Dedication gains trained arcane spellcasting bounded as the dedication specifies; validation fails closed in content tests.

**Verify:** content validation tests + one multi-class headless smoke.

See [archetypes.md](./archetypes.md).

---

## Phase 12 — Companions, familiars, animal companions

**Depends on:** Phases 6–8 as needed (Ranger/Druid/Witch).

**Build:** non-party combatant linked to master; initiative options; familiar abilities subset.

**Done when:** witch familiar occupies a grid square and can be targeted; ranger companion optional PR after solo Hunt Prey ships.

**Verify:** unit + headless.

---

## Phase 13 — Spell catalog expansion & class levels 3+

**Depends on:** Phases 7–10.

**Build:** deepen catalogs by tradition/difficulty; implement landmark class features (weapon specialization, expert spellcaster, etc.) as rule elements.

**Done when:** each Player Core / Player Core 2 class has *specified* L1–2 playable and a tracked backlog for L3+ landmarks; not “all feats in print.”

**Verify:** incremental tests per spell school / landmark.

---

## Phase 14 — Editor parity

**Depends on:** content schemas stable through Phase 7+.

**Touch:** `crates/editor`, content loaders.

**Done when:** editor authors the same class/feat/spell fields the runtime prepares; no parallel ruleset (Constitution IV).

**Verify:** editor round-trip fixture + game load.

---

## Cross-cutting verification template

For every phase that changes play:

| Lane | Command / evidence |
| --- | --- |
| Unit | `cargo test -p rules` (and `-p content` when schemas change) |
| Workspace | `cargo test --workspace` before merge |
| Headless | `cargo run -p game -- --headless` (or project’s current headless entry) exits with claimed outcome |
| Live | When a bar button / targeting mode is added, HUD label + `TESTER.md` update |
| Fidelity | Ledger row added/updated for any substitute |

## Recommended first implementation phase

**Start at Phase 1 (attributes, proficiency, skills, modifier types).**  
Class features and spells cannot be faithful without it; Fighter/Rogue slices will otherwise keep lying with baked numbers and Aid’s attack-bonus substitute will never graduate.

## Out of roadmap (unless product opens them)

- Full encyclopedia of every archetype/spell in print
- Adventure-Path-only classes without a pack decision
- Rituals (see [open-questions.md](./open-questions.md))
- Ancestry/heritage completeness beyond what L1 fixtures need
- Campaign narrative content (Constitution V)
