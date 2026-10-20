# Oracle

**Source:** Player Core 2 pg. 128 (Player Core 2). Remaster naming. Archives of Nethys class id `61`.

**Role in Genesis:** class content pack entry consumed by phased preparation in `crates/rules` (see [architecture.md](../architecture.md)). Fixtures remain hand-authored until the editor authors the same schemas.

## Identity

| Field | Value |
| --- | --- |
| Key attribute | Charisma |
| HP per level | 8 plus your Constitution modifier (plus ancestry HP at level 1 total) |
| Book | Player Core 2 |

## Initial proficiencies (level 1)

- At 1st level, you gain the listed proficiency ranks in the following statistics. You are untrained in anything not listed unless you gain a better proficiency rank in some other way.
- **Perception:** Trained in Perception
- **Saving Throws:** Trained in Fortitude
- Trained in Reflex
- Expert in Will
- **Skills:** Trained in Religion
- Trained in one or more skills determined by your mystery
- Trained in a number of additional skills equal to 3 plus your Intelligence modifier
- **Attacks:** Trained in simple weapons
- Trained in unarmed attacks
- **Defenses:** Trained in light armor
- Trained in unarmored defense
- **Class DC:** Trained in oracle class DC
- **Spells:** Trained in spell attack modifier
- Trained in spell DC

### Proficiency progression (engine)

Store rank enums (`Untrained/Trained/Expert/Master/Legendary`) per statistic. Class features that say “your proficiency rank for X increases to Y” are rule elements applied in preparation phases—not ad hoc branches in `encounter.rs`. Attack/defense/Perception/save/class-DC/spell ranks listed above are the L1 baseline; later rows in the advancement table upgrade them.

## Advancement table

| Level | Features gained |
| --- | --- |
| 1 | Ancestry and background, attribute boosts, initial proficiencies, oracle spellcasting, spell repertoire, mystery |
| 2 | Oracle feat, skill feat |
| 3 | 2nd-rank spells, general feat, signature spells, skill increase |
| 4 | Oracle feat, skill feat |
| 5 | 3rd-rank spells, attribute boosts, ancestry feat, skill increase |
| 6 | Oracle feat, skill feat |
| 7 | 4th-rank spells, expert spellcaster, general feat, mysterious resolve, skill increase |
| 8 | Oracle feat, skill feat |
| 9 | 5th-rank spells, ancestry feat, magical fortitude, skill increase |
| 10 | Attribute boosts, oracle feat, skill feat |
| 11 | 6th-rank spells, divine access, general feat, major curse, oracular senses, skill increase, weapon expertise |
| 12 | Oracle feat, skill feat |
| 13 | 7th-rank spells, ancestry feat, light armor expertise, premonition’s reflexes, skill increase, weapon specialization |
| 14 | Oracle feat, skill feat |
| 15 | 8th-rank spells, attribute boosts, general feat, master spellcaster, skill increase |
| 16 | Oracle feat, skill feat |
| 17 | 9th-rank spells, ancestry feat, extreme curse, greater mysterious resolve, skill increase |
| 18 | Oracle feat, skill feat |
| 19 | General feat, legendary spellcaster, oracular clarity, skill increase |
| 20 | Attribute boosts, oracle feat, skill feat |

## Class features (procedure reference)

Summaries below are implementation-shaped paraphrases for engine work. Cite the book section when verifying; do not paste Paizo text into code or fixtures.

### Oracle Spellcasting (Level 1)

- **When gained:** Level 1
- **Action economy:** class feature / choice
- **Procedure (engine):** You have an unfiltered connection to the great powers of the universe and the planes beyond, and you can let this power spill forth in the form of divine magic. You are a spellcaster, and you can cast spells of the divine tradition using the Cast a Spell activity. As an oracle, when you cast spells, your incantations may spill from your lips rapidly as you speak in tongues or intone in a voice not quite your own,…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Spell Repertoire (Level 1)

- **When gained:** Level 1
- **Action economy:** class feature / choice
- **Procedure (engine):** The collection of spells you can cast is called your spell repertoire. At 1st level, you learn three 1st-rank divine spells of your choice and five divine cantrips of your choice. You choose these from the common spells on the divine list or from other divine spells to which you have access. You can cast any spell in your spell repertoire by using a spell slot of an appropriate spell rank. You add to this spell…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Mystery (Level 1)

- **When gained:** Level 1
- **Action economy:** class feature / choice
- **Procedure (engine):** An oracle wields divine power, but not from a single divine being. This power could come from a potent concept or ideal, the attention of multiple divine entities whose areas of concern all touch on that subject, or a direct and dangerous conduit to raw divine power. This is the oracle's mystery, a source of divine magic not beholden to any deity. Choose the mystery that empowers your magic. Your mystery grants…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Oracle Feats (Level 2)

- **Procedure:** At 2nd level and every 2 levels thereafter, you gain an oracle class feat .
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Skill Feats (Level 2)

- **Procedure:** At 2nd level and every 2 levels thereafter, you gain a skill feat. You must be trained or better in the corresponding skill to select a skill feat.
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### General Feats (Level 3)

- **Procedure:** At 3rd level and every 4 levels thereafter, you gain a general feat.
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Signature Spells (Level 3)

- **When gained:** Level 3
- **Action economy:** class feature / choice
- **Procedure (engine):** Experience enables you to cast some spells more flexibly. For each spell rank you have access to, choose one spell of that rank to be a signature spell. You don’t need to learn heightened versions of signature spells separately; instead, you can heighten these spells freely. If you’ve learned a signature spell at a higher rank than its minimum, you can also cast all its lower-rank versions without learning those…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Skill Increases (Level 3)

- **Procedure:** At 3rd level and every 2 levels thereafter, you gain a skill increase. You can use this increase to either become trained in one skill you're untrained in, or become an expert in one skill in which you're already trained. At 7th level, you can use skill increases to become a master in a skill in which you're already an expert, and at 15th level, you can use them to become legendary in a skill in which you're…
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Ancestry Feats (Level 5)

- **Procedure:** In addition to the ancestry feat you started with, you gain an ancestry feat at 5th level and every 4 levels thereafter.
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Expert Spellcaster (Level 7)

- **When gained:** Level 7
- **Action economy:** class feature / choice
- **Procedure (engine):** The intricacy of your divine power has grown clearer over time. Your proficiency ranks for spell attack modifier and spell DC increase to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Mysterious Resolve (Level 7)

- **When gained:** Level 7
- **Action economy:** class feature / choice
- **Procedure (engine):** The power of your mystery blazing in your soul makes it harder for other powers to grip your mind. Your proficiency rank for Will saves increases to master. When you roll a success on a Will save, you get a critical success instead.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Magical Fortitude (Level 9)

- **When gained:** Level 9
- **Action economy:** class feature / choice
- **Procedure (engine):** Magical power has improved your body’s resiliency. Your proficiency rank for Fortitude saves increases to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Divine Access (Level 11)

- **When gained:** Level 11
- **Action economy:** class feature / choice
- **Procedure (engine):** Your mystery offers you strange access to spells typically reserved for more conventional worshippers. Choose one deity who grants one of your mystery’s granted domains. Add up to three cleric spells of your choice granted by that deity to your spell list, and to your spell repertoire as soon as you can cast spells of the appropriate rank.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Major Curse (Level 11)

- **When gained:** Level 11
- **Action economy:** class feature / choice
- **Procedure (engine):** You’ve learned to better tolerate the conflicting powers wreaking havoc on your body. The maximum cursebound value you can have increases from cursebound 2 to cursebound 3.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Oracular Senses (Level 11)

- **When gained:** Level 11
- **Action economy:** class feature / choice
- **Procedure (engine):** You have always been able to sense a bit more than others. Your proficiency rank for Perception increases to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Weapon Expertise (Level 11)

- **When gained:** Level 11
- **Action economy:** class feature / choice
- **Procedure (engine):** You’ve dedicated yourself to learning the intricacies of your weapons. Your proficiency ranks for simple weapons and unarmed attacks increase to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Light Armor Expertise (Level 13)

- **When gained:** Level 13
- **Action economy:** class feature / choice
- **Procedure (engine):** You’ve learned how to dodge while wearing light or no armor. Your proficiency rank for light armor and unarmored defense increases to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Premonition’s Reflexes (Level 13)

- **When gained:** Level 13
- **Action economy:** class feature / choice
- **Procedure (engine):** A chill runs through your spine as danger strikes, giving you a hair’s more time to dodge or cover yourself. Your proficiency rank for Reflex saves increases to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Weapon Specialization (Level 13)

- **When gained:** Level 13
- **Action economy:** class feature / choice
- **Procedure (engine):** You’ve learned how to inflict greater injuries with the weapons you know best. You deal 2 additional damage with weapons and unarmed attacks in which you are an expert. This damage increases to 3 if you’re a master and 4 if you’re legendary.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Master Spellcaster (Level 15)

- **When gained:** Level 15
- **Action economy:** class feature / choice
- **Procedure (engine):** You truly understand the deep and complex divine power within your mystery. Your proficiency ranks for spell attack modifiers and spell DCs increase to master.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Extreme Curse (Level 17)

- **When gained:** Level 17
- **Action economy:** class feature / choice
- **Procedure (engine):** You have mastered a precarious balance between the conflicting divine powers of your mystery, allowing you to tolerate a perilous degree of divine curse. The maximum cursebound value you can have increases from cursebound 3 to cursebound 4.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Greater Mysterious Resolve (Level 17)

- **When gained:** Level 17
- **Action economy:** class feature / choice
- **Procedure (engine):** Your time spent in contemplation of the mysteries of creation has given you a powerful mind and soul. Your proficiency rank for Will saves increases to legendary. When you roll a success on a Will save, you get a critical success instead. When you roll a critical failure on a Will save, you get a failure instead. When you fail a Will save against a damaging effect, you take half damage.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Legendary Spellcaster (Level 19)

- **When gained:** Level 19
- **Action economy:** class feature / choice
- **Procedure (engine):** You can harness divine power at a level few others can match. Your proficiency ranks for spell attack modifiers and spell DCs increase to legendary.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Oracular Clarity (Level 19)

- **When gained:** Level 19
- **Action economy:** class feature / choice
- **Procedure (engine):** You now fully grasp the nature of the divine power behind your mystery, allowing you to work magic akin to miracles. Add two common 10th-rank divine spells to your repertoire. You gain a single 10th-rank spell slot you can use to cast one of those two spells using oracle spellcasting. You don’t gain more 10th-rank spells as you level up, unlike other spell slots, and you can’t use 10th-rank slots with abilities…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.


## Focus spells / inseparable kits

- **Revelation spells:** focus spells from mystery; casting advances curse.

## Store vs derive

- **Store (character / preparation):** class id, level, key attribute choice, ancestry HP contribution, selected skills, feat picks (class/skill/general/ancestry), class choice subtree (racket/muse/order/doctrine/school/thesis/patron/bloodline/mystery/instinct/cause/style/methodology/research field), trained weapon/armor ranks overrides from features.
- **Store (encounter):** HP, conditions, focus points remaining, spell slots remaining / repertoire prepared-for-day, raised shield / stance / rage / panache / hunted prey id, reactions available.
- **Derive at prepare:** proficiency bonuses (level + rank), AC from armor + Dex + bonuses, Strike attack/damage from weapon + attr + proficiency + specialization, Perception/saves/class DC/spell DC, spell attack, MAP rules, off-guard sources.
- **Derive per check:** circumstance/status/item bonus nets by type, degree of success, precision applicability.
- **Spellcasting store:** tradition, casting style (prepared/spontaneous), spell list access, cantrips prepared or in repertoire, ranked slots or repertoire known, signature spells (spontaneous), focus spell ids, focus pool size.
- **Curse store:** mystery id, curse stage (mild/moderate/major/…); advances on revelation spell casts per mystery rules.

## Dependencies (substrate not yet in engine or only partially present)

Baseline on `main` (~`2f56abb`) already runs Stride/Step/Strike, Reactive Strike, dying, shields, prone movement, cover/detection, manipulate, Delay/Ready, Aid (attack-bonus substitute). This class still needs:

- `architecture.md#feat-slots`
- `architecture.md#proficiency`
- `conditions-and-effects.md`
- `conditions-and-effects.md#armor-and-item-bonuses`
- `conditions-and-effects.md#curse`
- `spells/framework.md`

- Skill list + proficiency bonuses on checks (Aid currently substitutes Strike bonus; ledger entry).
- Feat slot model + at least data-driven class feats used in the first slice.
- Armor/weapon itemization beyond fixture-prebaked AC and one Strike (unless the slice keeps baked numbers with a Principle III note).
- Conditions listed under [conditions-and-effects.md](../conditions-and-effects.md) that this class applies or keys off.

## Suggested first slice (room-one style)

Level 1 spontaneous divine: mystery, curse mild track, revelation focus spell. Needs curse condition track.

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
