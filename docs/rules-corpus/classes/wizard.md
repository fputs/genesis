# Wizard

**Source:** Player Core pg. 192 (Player Core). Remaster naming. Archives of Nethys class id `39`.

**Role in Genesis:** class content pack entry consumed by phased preparation in `crates/rules` (see [architecture.md](../architecture.md)). Fixtures remain hand-authored until the editor authors the same schemas.

## Identity

| Field | Value |
| --- | --- |
| Key attribute | Intelligence |
| HP per level | 6 plus your Constitution modifier (plus ancestry HP at level 1 total) |
| Book | Player Core |

## Initial proficiencies (level 1)

- At 1st level, you gain the listed proficiency ranks in the following statistics. You are untrained in anything not listed unless you gain a better proficiency rank in some other way.
- **Perception:** Trained in Perception
- **Saving Throws:** Trained in Fortitude
- Trained in Reflex
- Expert in Will
- **Skills:** Trained in Arcana
- Trained in a number of additional skills equal to 2 plus your Intelligence modifier
- **Attacks:** Trained in simple weapons
- Trained in unarmed attacks
- **Defenses:** Untrained in all armor
- Trained in unarmored defense
- **Class DC:** Trained in wizard class DC
- **Spells:** Trained in spell attack modifier
- Trained in spell DC

### Proficiency progression (engine)

Store rank enums (`Untrained/Trained/Expert/Master/Legendary`) per statistic. Class features that say “your proficiency rank for X increases to Y” are rule elements applied in preparation phases—not ad hoc branches in `encounter.rs`. Attack/defense/Perception/save/class-DC/spell ranks listed above are the L1 baseline; later rows in the advancement table upgrade them.

## Advancement table

| Level | Features gained |
| --- | --- |
| 1 | Ancestry and background, attribute boosts, initial proficiencies, wizard spellcasting, arcane thesis, arcane school, arcane bond |
| 2 | Skill feat, wizard feat |
| 3 | 2nd-rank spells, general feat, skill increase |
| 4 | Skill feat, wizard feat |
| 5 | 3rd-rank spells, attribute boosts, ancestry feat, reflex expertise, skill increase |
| 6 | Skill feat, wizard feat |
| 7 | 4th-rank spells, expert spellcaster, general feat, skill increase |
| 8 | Skill feat, wizard feat |
| 9 | 5th-rank spells, ancestry feat, magical fortitude, skill increase |
| 10 | Attribute boosts, skill feat, wizard feat |
| 11 | 6th-rank spells, general feat, perception expertise, skill increase, wizard weapon expertise |
| 12 | Skill feat, wizard feat |
| 13 | 7th-rank spells, ancestry feat, defensive robes, skill increase, weapon specialization |
| 14 | Skill feat, wizard feat |
| 15 | 8th-rank spells, attribute boosts, general feat, master spellcaster, skill increase |
| 16 | Skill feat, wizard feat |
| 17 | 9th-rank spells, ancestry feat, prodigious will, skill increase |
| 18 | Skill feat, wizard feat |
| 19 | Archwizard’s spellcraft, general feat, legendary spellcaster, skill increase |
| 20 | Attribute boosts, skill feat, wizard feat |

## Class features (procedure reference)

Summaries below are implementation-shaped paraphrases for engine work. Cite the book section when verifying; do not paste Paizo text into code or fixtures.

### Wizard Spellcasting (Level 1)

- **When gained:** Level 1
- **Action economy:** Cast a Spell (per spell)
- **Procedure (engine):** Prepared arcane caster. Daily: prepare cantrips and ranked spells into slots from spellbook. Cast spends the prepared entry. Spell attack/DC from Int + proficiency.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Arcane Thesis (Level 1)

- **When gained:** Level 1
- **Action economy:** class feature / choice
- **Procedure (engine):** During your studies to become a full-fledged wizard, you produced a thesis of unique magical research. You gain a special benefit depending on the topic of your thesis research. The arcane thesis topics presented in this book are below; your specific thesis probably has a much longer and more technical title like “ On the Methods of Spell Interpolation and the Genesis of a New Understanding of the Building Blocks…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Arcane School (Level 1)

- **When gained:** Level 1
- **Action economy:** class feature / choice
- **Procedure (engine):** Most wizards acquire their knowledge of spells from a formal educational institution, such as the Arcanamirium or the Magaambya. At 1st level, you choose your arcane school , which grants you magical abilities. You gain additional spells and spell slots from the curriculum taught at the school you attended. Some wizards follow the school of unified magical theory, which attempts to forge a new school by studying…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Arcane Bond (Level 1)

- **When gained:** Level 1
- **Action economy:** class feature / choice
- **Procedure (engine):** You place some of your magical power in a bonded item. Each day when you prepare your spells, you can designate a single item you own as your bonded item. This is typically an item associated with spellcasting, such as a wand, ring, or staff, but you are free to designate a weapon or other item. You gain the Drain Bonded Item free action. Drain Bonded Item [free-action] Arcane Wizard Source Player Core pg. 197…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Skill Feats (Level 2)

- **Procedure:** At 2nd level and every 2 levels thereafter, you gain a skill feat . You must be trained or better in the corresponding skill to select a skill feat.
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Wizard Feats (Level 2)

- **Procedure:** At 2nd level and every even-numbered level thereafter, you gain a wizard class feat .
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### General Feats (Level 3)

- **Procedure:** At 3rd level and every 4 levels thereafter, you gain a general feat .
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Skill Increases (Level 3)

- **Procedure:** At 3rd level and every 2 levels thereafter, you gain a skill increase. You can use this increase either to increase your proficiency rank to trained in one skill you're untrained in, or to increase your proficiency rank in one skill in which you're already trained to expert. At 7th level, you can use skill increases to increase your proficiency rank to master in a skill in which you're already an expert, and at…
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Ancestry Feats (Level 5)

- **Procedure:** In addition to the ancestry feat you started with, you gain an ancestry feat at 5th level and every 4 levels thereafter. The list of ancestry feats available to you can be found in your ancestry's entry.
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Reflex Expertise (Level 5)

- **When gained:** Level 5
- **Action economy:** class feature / choice
- **Procedure (engine):** You've developed a knack for dodging danger. Your proficiency rank for Reflex saves increases to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Expert Spellcaster (Level 7)

- **When gained:** Level 7
- **Action economy:** class feature / choice
- **Procedure (engine):** Extended practice of the arcane has improved your capabilities. Your proficiency ranks for spell attack modifier and spell DC increase to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Magical Fortitude (Level 9)

- **When gained:** Level 9
- **Action economy:** class feature / choice
- **Procedure (engine):** Magical power has improved your body's resiliency. Your proficiency rank for Fortitude saves increases to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Perception Expertise (Level 11)

- **When gained:** Level 11
- **Action economy:** class feature / choice
- **Procedure (engine):** You remain alert to threats around you. Your proficiency rank for Perception increases to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Weapon Expertise (Level 11)

- **When gained:** Level 11
- **Action economy:** class feature / choice
- **Procedure (engine):** Through sheer experience, you've improved your technique with your weapons. Your proficiency ranks for simple weapons and unarmed attacks increase to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Defensive Robes (Level 13)

- **When gained:** Level 13
- **Action economy:** class feature / choice
- **Procedure (engine):** The flow of magic and your training combine to help you avoid attacks. Your proficiency rank in unarmored defense increases to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Weapon Specialization (Level 13)

- **When gained:** Level 13
- **Action economy:** class feature / choice
- **Procedure (engine):** You've learned how to inflict greater injuries with the weapons you know best. You deal an additional 2 damage with weapons and unarmed attacks in which you are an expert. This damage increases to 3 if you're a master, and 4 if you're legendary.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Master Spellcaster (Level 15)

- **When gained:** Level 15
- **Action economy:** class feature / choice
- **Procedure (engine):** You command superlative spellcasting ability. Your proficiency ranks for spell attack modifier and spell DC increase to master.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Prodigious Will (Level 17)

- **When gained:** Level 17
- **Action economy:** class feature / choice
- **Procedure (engine):** Your mind is simply too complex and sophisticated to be tampered with. Your proficiency rank for Will saves increases to master. When you roll a success at a Will save, you get a critical success instead.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Archwizard's Spellcraft (Level 19)

- **When gained:** Level 19
- **Action economy:** class feature / choice
- **Procedure (engine):** You command the most potent arcane magic and can cast a spell of truly incredible power. You gain a single 10th-rank spell slot and can prepare a spell in that slot using wizard spellcasting. Unlike with other spell slots, you can't use 10th-rank slots with abilities that give you more spell slots or that let you cast spells without expending spell slots. You don't gain more 10th-rank spells as you level up, but…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Legendary Spellcaster (Level 19)

- **When gained:** Level 19
- **Action economy:** class feature / choice
- **Procedure (engine):** You are a consummate spellcaster, blending both arcane theory and practical spellcraft. Your proficiency ranks for spell attack modifier and spell DC increase to legendary. Site Owner: Rose-Winds LLC (Blake Davis) (adsbygoogle = window.adsbygoogle || []).push({}); Character Creation + Ancestries Archetypes Backgrounds Classes Skills Companions Familiars Equipment + All Equipment Adventuring Gear Alchemical Items…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.


## Focus spells / inseparable kits

- **School spell:** curriculum grants additional prepared school spell slots / cantrips; Drain Bonded Item recovers a slot.

## Store vs derive

- **Store (character / preparation):** class id, level, key attribute choice, ancestry HP contribution, selected skills, feat picks (class/skill/general/ancestry), class choice subtree (racket/muse/order/doctrine/school/thesis/patron/bloodline/mystery/instinct/cause/style/methodology/research field), trained weapon/armor ranks overrides from features.
- **Store (encounter):** HP, conditions, focus points remaining, spell slots remaining / repertoire prepared-for-day, raised shield / stance / rage / panache / hunted prey id, reactions available.
- **Derive at prepare:** proficiency bonuses (level + rank), AC from armor + Dex + bonuses, Strike attack/damage from weapon + attr + proficiency + specialization, Perception/saves/class DC/spell DC, spell attack, MAP rules, off-guard sources.
- **Derive per check:** circumstance/status/item bonus nets by type, degree of success, precision applicability.
- **Spellcasting store:** tradition, casting style (prepared/spontaneous), spell list access, cantrips prepared or in repertoire, ranked slots or repertoire known, signature spells (spontaneous), focus spell ids, focus pool size.
- **Companion/familiar:** separate creature id linked to master when feature grants one; not a party slot.

## Dependencies (substrate not yet in engine or only partially present)

Baseline on `main` (~`2f56abb`) already runs Stride/Step/Strike, Reactive Strike, dying, shields, prone movement, cover/detection, manipulate, Delay/Ready, Aid (attack-bonus substitute). This class still needs:

- `architecture.md#class-choices`
- `architecture.md#feat-slots`
- `architecture.md#proficiency`
- `conditions-and-effects.md`
- `spells/framework.md`

- Skill list + proficiency bonuses on checks (Aid currently substitutes Strike bonus; ledger entry).
- Feat slot model + at least data-driven class feats used in the first slice.
- Armor/weapon itemization beyond fixture-prebaked AC and one Strike (unless the slice keeps baked numbers with a Principle III note).
- Conditions listed under [conditions-and-effects.md](../conditions-and-effects.md) that this class applies or keys off.

## Suggested first slice (room-one style)

Level 1 prepared arcane: spellbook, school cantrips/slots, arcane bond Drain Bonded Item. Cast Shield cantrip + Magic Missile. Needs prepared spellcasting.

**Done predicate:** headless encounter with one PC of this class at level 1 resolves the L1 defining action(s) with seeded dice; `cargo test -p rules` covers the new procedures; fidelity ledger updated for any temporary substitute.

## Higher levels

Levels 3–20 features appear in the advancement table. Full feat catalogs and every subclass option are **not** expanded here. Treat unlisted feat bodies as content-pack work: each feat is an id + rule elements + optional action/reaction definition. When implementing past L2, expand this file’s feature section one landmark at a time (expertise upgrades, specialization, spell rank unlocks).

## Remaster / Legacy notes

- Use Remaster names (`attribute` not ability score; `Reactive Strike` not Attack of Opportunity; spell **rank** not level).
- If a Legacy-only option is ever ingested, mark it `legacy` in content metadata and keep it out of default Remaster packs ([open-questions.md](../open-questions.md)).

## See also

- [architecture.md](../architecture.md)
- [implementation-roadmap.md](../implementation-roadmap.md)
- [spells/framework.md](../spells/framework.md)
- [archetypes.md](../archetypes.md)
