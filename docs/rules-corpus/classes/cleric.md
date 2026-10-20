# Cleric

**Source:** Player Core pg. 108 (Player Core). Remaster naming. Archives of Nethys class id `33`.

**Role in Genesis:** class content pack entry consumed by phased preparation in `crates/rules` (see [architecture.md](../architecture.md)). Fixtures remain hand-authored until the editor authors the same schemas.

## Identity

| Field | Value |
| --- | --- |
| Key attribute | Wisdom |
| HP per level | 8 plus your Constitution modifier (plus ancestry HP at level 1 total) |
| Book | Player Core |

## Initial proficiencies (level 1)

- At 1st level, you gain the listed proficiency ranks in the following statistics. You are untrained in anything not listed unless you gain a better proficiency rank in some other way.
- **Perception:** Trained in Perception
- **Saving Throws:** Trained in Fortitude
- Trained in Reflex
- Expert in Will
- **Skills:** Trained in Religion
- Trained in one skill determined by your choice of deity
- Trained in a number of additional skills equal to 2 plus your Intelligence modifier
- **Attacks:** Trained in simple weapons
- Trained in the favored weapon of your deity. If your deity's favored weapon is uncommon, you also gain access to that weapon.
- Trained in unarmed attacks
- **Defenses:** Untrained in all armor, though your doctrine might alter this
- Trained in unarmored defense
- **Class DC:** Trained in cleric class DC
- **Spells:** Trained in spell attack modifier
- Trained in spell DC

### Proficiency progression (engine)

Store rank enums (`Untrained/Trained/Expert/Master/Legendary`) per statistic. Class features that say “your proficiency rank for X increases to Y” are rule elements applied in preparation phases—not ad hoc branches in `encounter.rs`. Attack/defense/Perception/save/class-DC/spell ranks listed above are the L1 baseline; later rows in the advancement table upgrade them.

## Advancement table

| Level | Features gained |
| --- | --- |
| 1 | Ancestry and background, attribute boosts, initial proficiencies, deity, cleric spellcasting, divine font, doctrine |
| 2 | Cleric feat, skill feat |
| 3 | 2nd-rank spells, general feat, second doctrine, skill increase |
| 4 | Cleric feat, skill feat |
| 5 | 3rd-rank spells, attribute boosts, ancestry feat, perception expertise, skill increase |
| 6 | Cleric feat, skill feat |
| 7 | 4th-rank spells, general feat, skill increase, third doctrine |
| 8 | Cleric feat, skill feat |
| 9 | 5th-rank spells, ancestry feat, resolute faith, skill increase |
| 10 | Attribute boosts, cleric feat, skill feat |
| 11 | 6th-rank spells, fourth doctrine, general feat, reflex expertise, skill increase |
| 12 | Cleric feat, skill feat |
| 13 | 7th-rank spells, ancestry feat, divine defense, skill increase, weapon specialization |
| 14 | Cleric feat, skill feat |
| 15 | 8th-rank spells, attribute boosts, fifth doctrine, general feat, skill increase |
| 16 | Cleric feat, skill feat |
| 17 | 9th-rank spells, ancestry feat, skill increase |
| 18 | Cleric feat, skill feat |
| 19 | Final doctrine, general feat, miraculous spell, skill increase |
| 20 | Attribute boosts, cleric feat, skill feat |

## Class features (procedure reference)

Summaries below are implementation-shaped paraphrases for engine work. Cite the book section when verifying; do not paste Paizo text into code or fixtures.

### Deity (Level 1)

- **When gained:** Level 1
- **Action economy:** class feature / choice
- **Procedure (engine):** As a cleric, you are a mortal servitor of a deity you revere above all others. The most common deities in Pathfinder have edicts, areas of concern, and the benefits you get for being a cleric of that deity. Your deity grants you the trained proficiency rank in one skill and with the deity's favored weapon. If the favored weapon is uncommon, you also get access to that weapon. Your deity also adds spells to your…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Cleric Spellcasting (Level 1)

- **When gained:** Level 1
- **Action economy:** class feature / choice
- **Procedure (engine):** Your deity bestows on you the power to cast divine spells. You are a spellcaster, and you can cast spells of the divine tradition using the Cast a Spell activity. As a cleric, your chants generally invoke your deity and their powerful servants by name or title, while your gestures are followed by sacred symbols or other representations of your deity At 1st level, you can prepare two 1st-rank spells and five…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Divine Font (Level 1)

- **When gained:** Level 1
- **Action economy:** uses Heal/Harm cast
- **Procedure (engine):** Extra Heal or Harm slots from deity (font choice), prepared separately from normal slots.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Doctrine (Level 1)

- **When gained:** Level 1
- **Action economy:** class feature / choice
- **Procedure (engine):** Even among followers of the same deity, approaches vary. At 1st level, you select cloistered cleric or warpriest and gain the benefits of its first doctrine. At 3rd level and every four levels thereafter, you gain another benefit from your doctrine. Doctrines can be found here .
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Cleric Feats (Level 2)

- **Procedure:** At 2nd level and every 2 levels thereafter, you gain a cleric class feat .
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Skill Feats (Level 2)

- **Procedure:** At 2nd level and every 2 levels thereafter, you gain a skill feat . You must be trained or better in the corresponding skill to select a skill feat.
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### General Feats (Level 3)

- **Procedure:** At 3rd level and every 4 levels thereafter, you gain a general feat .
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Skill Increases (Level 3)

- **Procedure:** At 3rd level and every 2 levels thereafter, you gain a skill increase. You can use this increase to either become trained in one skill you're untrained in, or become an expert in one skill in which you're already trained. You can use any of these skill increases you gain at 7th level or higher to become a master in a skill in which you're already an expert, and any of these skill increases you gain at 15th level…
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Ancestry Feats (Level 5)

- **Procedure:** In addition to the ancestry feat you started with, you gain an ancestry feat at 5th level and every 4 levels thereafter. The list of ancestry feats available to you can be found in your ancestry's entry.
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Perception Expertise (Level 5)

- **When gained:** Level 5
- **Action economy:** class feature / choice
- **Procedure (engine):** You remain alert to threats around you. Your proficiency rank for Perception increases to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Resolute Faith (Level 9)

- **When gained:** Level 9
- **Action economy:** class feature / choice
- **Procedure (engine):** Belief provides a bulwark against insidious thoughts. Your proficiency rank for Will saves increases to master. When you roll a success at a Will save, you get a critical success instead.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Reflex Expertise (Level 11)

- **When gained:** Level 11
- **Action economy:** class feature / choice
- **Procedure (engine):** You've developed a knack for dodging danger. Your proficiency rank for Reflex saves increases to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Divine Defense (Level 13)

- **When gained:** Level 13
- **Action economy:** class feature / choice
- **Procedure (engine):** Your training and your deity protect you from harm. Your proficiency rank in unarmored defense increases to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Weapon Specialization (Level 13)

- **When gained:** Level 13
- **Action economy:** class feature / choice
- **Procedure (engine):** You've learned how to inflict greater injuries with your weapons. You deal 2 additional damage with weapons and unarmed attacks in which you are an expert. This damage increases to 3 if you're a master, and to 4 if you're legendary.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Miraculous Spell (Level 19)

- **When gained:** Level 19
- **Action economy:** class feature / choice
- **Procedure (engine):** You're exalted by your deity and gain truly incredible spells. You gain a single 10th-rank spell slot and can prepare a spell in that slot using cleric spellcasting. Unlike with other spell slots, you can't use 10-rank slots with abilities that give you more spell slots or that let you cast spells without expending spell slots. You don't gain more 10th-rank spells as you level up, but you can take the Maker of…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.


## Focus spells / inseparable kits

- **Divine Font:** extra Heal or Harm slots from deity; prepared separately from normal slots.

## Store vs derive

- **Store (character / preparation):** class id, level, key attribute choice, ancestry HP contribution, selected skills, feat picks (class/skill/general/ancestry), class choice subtree (racket/muse/order/doctrine/school/thesis/patron/bloodline/mystery/instinct/cause/style/methodology/research field), trained weapon/armor ranks overrides from features.
- **Store (encounter):** HP, conditions, focus points remaining, spell slots remaining / repertoire prepared-for-day, raised shield / stance / rage / panache / hunted prey id, reactions available.
- **Derive at prepare:** proficiency bonuses (level + rank), AC from armor + Dex + bonuses, Strike attack/damage from weapon + attr + proficiency + specialization, Perception/saves/class DC/spell DC, spell attack, MAP rules, off-guard sources.
- **Derive per check:** circumstance/status/item bonus nets by type, degree of success, precision applicability.
- **Spellcasting store:** tradition, casting style (prepared/spontaneous), spell list access, cantrips prepared or in repertoire, ranked slots or repertoire known, signature spells (spontaneous), focus spell ids, focus pool size.

## Dependencies (substrate not yet in engine or only partially present)

Baseline on `main` (~`2f56abb`) already runs Stride/Step/Strike, Reactive Strike, dying, shields, prone movement, cover/detection, manipulate, Delay/Ready, Aid (attack-bonus substitute). This class still needs:

- `architecture.md#class-choices`
- `architecture.md#feat-slots`
- `architecture.md#proficiency`
- `conditions-and-effects.md`
- `spells/divine-support.md`
- `spells/framework.md`

- Skill list + proficiency bonuses on checks (Aid currently substitutes Strike bonus; ledger entry).
- Feat slot model + at least data-driven class feats used in the first slice.
- Armor/weapon itemization beyond fixture-prebaked AC and one Strike (unless the slice keeps baked numbers with a Principle III note).
- Conditions listed under [conditions-and-effects.md](../conditions-and-effects.md) that this class applies or keys off.

## Suggested first slice (room-one style)

Level 1 prepared divine caster: deity package stub, Divine Font heal/harm slots, one doctrine. Cast Heal cantrip + Heal 1st-rank. Needs spellcasting framework.

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
