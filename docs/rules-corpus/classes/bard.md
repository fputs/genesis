# Bard

**Source:** Player Core pg. 94 (Player Core). Remaster naming. Archives of Nethys class id `32`.

**Role in Genesis:** class content pack entry consumed by phased preparation in `crates/rules` (see [architecture.md](../architecture.md)). Fixtures remain hand-authored until the editor authors the same schemas.

## Identity

| Field | Value |
| --- | --- |
| Key attribute | Charisma |
| HP per level | 8 plus your Constitution modifier (plus ancestry HP at level 1 total) |
| Book | Player Core |

## Initial proficiencies (level 1)

- At 1st level, you gain the listed proficiency ranks in the following statistics. You are untrained in anything not listed unless you gain a better proficiency rank in some other way.
- **Perception:** Expert in Perception
- **Saving Throws:** Trained in Fortitude
- Trained in Reflex
- Expert in Will
- **Skills:** Trained in Occultism
- Trained in Performance
- Trained in a number of additional skills equal to 4 plus your Intelligence modifier
- **Attacks:** Trained in simple weapons
- Trained in martial weapons
- Trained in unarmed attacks
- **Defenses:** Trained in light armor
- Trained in unarmored defense
- **Class DC:** Trained in bard class DC
- **Spells:** Trained in spell attack modifier
- Trained in spell DC

### Proficiency progression (engine)

Store rank enums (`Untrained/Trained/Expert/Master/Legendary`) per statistic. Class features that say “your proficiency rank for X increases to Y” are rule elements applied in preparation phases—not ad hoc branches in `encounter.rs`. Attack/defense/Perception/save/class-DC/spell ranks listed above are the L1 baseline; later rows in the advancement table upgrade them.

## Advancement table

| Level | Features gained |
| --- | --- |
| 1 | Ancestry and background, attribute boosts, bard spellcasting, initial proficiencies, spell repertoire, composition spells, muse |
| 2 | Bard feat, skill feat |
| 3 | 2nd-rank spells, general feat, reflex expertise, signature spells, skill increase |
| 4 | Bard feat, skill feat |
| 5 | 3rd-rank spells, attribute boosts, ancestry feat, skill increase |
| 6 | Bard feat, skill feat |
| 7 | 4th-rank spells, expert spellcaster, general feat, skill increase |
| 8 | Bard feat, skill feat |
| 9 | 5th-rank spells, ancestry feat, fortitude expertise, performer’s heart, skill increase |
| 10 | Attribute boosts, bard feat, skill feat |
| 11 | 6th-rank spells, bard weapon expertise, general feat, skill increase, perception mastery |
| 12 | Bard feat, skill feat |
| 13 | 7th-rank spells, ancestry feat, light armor expertise, skill increase, weapon specialization |
| 14 | Bard feat, skill feat |
| 15 | 8th-rank spells, attribute boosts, general feat, master spellcaster, skill increase |
| 16 | Bard feat, skill feat |
| 17 | 9th-rank spells, ancestry feat, greater performer’s heart, skill increase |
| 18 | Bard feat, skill feat |
| 19 | General feat, legendary spellcaster, magnum opus, skill increase |
| 20 | Attribute boosts, bard feat, skill feat |

## Class features (procedure reference)

Summaries below are implementation-shaped paraphrases for engine work. Cite the book section when verifying; do not paste Paizo text into code or fixtures.

### Bard Spellcasting (Level 1)

- **When gained:** Level 1
- **Action economy:** class feature / choice
- **Procedure (engine):** You draw upon magic from esoteric knowledge. You are a spellcaster and can cast spells of the occult tradition using the Cast a Spell activity (see Casting Spells ). As a bard, when you cast spells, your incantations might be musical riffs or clever limericks, your gestures might incorporate dance and dramatic pantomiming, and you might accompany your spellcasting by playing a musical instrument. Each day, you…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Spell Repertoire (Level 1)

- **When gained:** Level 1
- **Action economy:** class feature / choice
- **Procedure (engine):** The collection of spells you can cast is called your spell repertoire. At 1st level, you learn two 1st-rank occult spells of your choice and five occult cantrips of your choice. You choose these from the common spells from the occult list or from other occult spells to which you have access. You can cast any spell in your spell repertoire by using a spell slot of an appropriate spell rank. You add to this spell…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Composition Spells (Level 1)

- **When gained:** Level 1
- **Action economy:** per composition spell
- **Procedure (engine):** Composition focus spells/cantrips. FP pool starts at 1; composition cantrips do not spend FP or enlarge the pool. Refocus restores 1 FP.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Muses (Level 1)

- **When gained:** Level 1
- **Action economy:** class feature / choice
- **Procedure (engine):** As a bard, you select one muse at 1st level. This muse leads you to great things and might be someone you know, a supernatural creature, location, deity, philosophy, or captivating mystery. Depending on the type of inspiration you receive, your 1st-level muse grants you a specific 1st-level bard feat and adds a spell to your repertoire. In addition to this, each muse opens the door to many later bard feats. If…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Bard Feats (Level 2)

- **Procedure:** At 2nd level and every even-numbered level, you gain a bard class feat .
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Skill Feats (Level 2)

- **Procedure:** At 2nd level and every 2 levels thereafter, you gain a skill feat . You must be trained or better in the corresponding skill to select a skill feat.
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### General Feats (Level 3)

- **Procedure:** At 3rd level and every 4 levels thereafter, you gain a general feat .
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Reflex Expertise (Level 3)

- **When gained:** Level 3
- **Action economy:** class feature / choice
- **Procedure (engine):** You've developed a knack for dodging danger. Your proficiency rank for Reflex saves increases to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Signature Spells (Level 3)

- **When gained:** Level 3
- **Action economy:** class feature / choice
- **Procedure (engine):** Experience allows you to cast some spells more flexibly. For each spell rank you have access to, choose one spell of that rank to be a signature spell. You don't need to learn heightened versions of signature spells separately; instead, you can heighten these spells freely. If you've learned a signature spell at a higher rank than its minimum, you can also cast all its lower-rank versions without learning those…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Skill Increases (Level 3)

- **Procedure:** At 3rd level and every 2 levels thereafter, you gain a skill increase. You can use this increase to either become trained in one skill you're untrained in or become an expert in one skill in which you're already trained. At 7th level, you can use skill increases to become a master in a skill in which you're already an expert, and at 15th level, you can use them to become legendary in a skill in which you're…
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Ancestry Feats (Level 5)

- **Procedure:** In addition to the ancestry feat you started with, you gain an ancestry feat at 5th level and every 4 levels thereafter. The list of ancestry feats available to you can be found in your ancestry's entry.
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Expert Spellcaster (Level 7)

- **When gained:** Level 7
- **Action economy:** class feature / choice
- **Procedure (engine):** Your magical techniques grow stronger. Your proficiency ranks for spell attack modifiers and spell DCs increase to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Fortitude Expertise (Level 9)

- **When gained:** Level 9
- **Action economy:** class feature / choice
- **Procedure (engine):** Your physique has grown hardy in your adventures. Your proficiency rank for Fortitude saves increases to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Performer's Heart (Level 9)

- **When gained:** Level 9
- **Action economy:** class feature / choice
- **Procedure (engine):** The greatest performances require an understanding of your own mind and flawless concentration. Your proficiency rank for Will saves increases to master. When you roll a success on a Will save, you get a critical success instead.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Bard Weapon Expertise (Level 11)

- **When gained:** Level 11
- **Action economy:** class feature / choice
- **Procedure (engine):** You have become thoroughly adept with bardic weapons. Your proficiency rank for martial weapons, simple weapons, and unarmed attacks increases to expert. When you critically succeed at an attack roll using one of these weapons or an unarmed attack while one of your compositions is active, you apply the critical specialization effect for that weapon or unarmed attack.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Perception Mastery (Level 11)

- **When gained:** Level 11
- **Action economy:** class feature / choice
- **Procedure (engine):** You've developed keen awareness and attention to detail. Your proficiency rank for Perception increases to master.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Light Armor Expertise (Level 13)

- **When gained:** Level 13
- **Action economy:** class feature / choice
- **Procedure (engine):** You've learned how to dodge while wearing light or no armor. Your proficiency ranks for light armor and unarmored defense increase to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Weapon Specialization (Level 13)

- **When gained:** Level 13
- **Action economy:** class feature / choice
- **Procedure (engine):** You've learned how to inflict greater injuries with the weapons you know best. You deal 2 additional damage with weapons and unarmed attacks in which you're an expert. This damage increases to 3 if you're a master and 4 if you're legendary.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Master Spellcaster (Level 15)

- **When gained:** Level 15
- **Action economy:** class feature / choice
- **Procedure (engine):** You pluck the threads of magic like the strings of a harp. Your proficiency ranks for spell attack modifiers and spell DCs increase to master.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Greater Performer's Heart (Level 17)

- **When gained:** Level 17
- **Action economy:** class feature / choice
- **Procedure (engine):** Your performances warp the mind and cause emotions to flower, and understanding how to do this also forges an incomparable understanding of your own mind. Your proficiency rank for Will saves increases to legendary. When you roll a critical failure on a Will save, you get a failure instead. When you fail a Will save against an effect that deals damage, you take half damage.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Magnum Opus (Level 19)

- **When gained:** Level 19
- **Action economy:** class feature / choice
- **Procedure (engine):** You have tuned your spellcasting to the highest caliber. Add two common 10th-rank occult spells to your repertoire. You gain a single 10th-rank spell slot you can use to cast these spells using bard spellcasting. Unlike other spell slots, you can't use 10th-rank slots with abilities that give you more spell slots or that let you cast spells without expending spell slots. You don't gain more 10th-rank spells as…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Legendary Spellcaster (Level 19)

- **When gained:** Level 19
- **Action economy:** class feature / choice
- **Procedure (engine):** Your command of magic is the stuff of legends. Your proficiency ranks for spell attack modifiers and spell DCs increase to legendary. Site Owner: Rose-Winds LLC (Blake Davis) (adsbygoogle = window.adsbygoogle || []).push({}); Character Creation + Ancestries Archetypes Backgrounds Classes Skills Companions Familiars Equipment + All Equipment Adventuring Gear Alchemical Items Armor Held Items Runes Shields Weapons…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.


## Focus spells / inseparable kits

- **Composition spells:** focus spells + composition cantrips (Courageous Anthem family). Focus pool starts at 1; Refocus restores 1. Composition cantrips do not spend Focus Points.

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
- `conditions-and-effects.md#armor-and-item-bonuses`
- `spells/framework.md`
- `spells/occult-support.md`

- Skill list + proficiency bonuses on checks (Aid currently substitutes Strike bonus; ledger entry).
- Feat slot model + at least data-driven class feats used in the first slice.
- Armor/weapon itemization beyond fixture-prebaked AC and one Strike (unless the slice keeps baked numbers with a Principle III note).
- Conditions listed under [conditions-and-effects.md](../conditions-and-effects.md) that this class applies or keys off.

## Suggested first slice (room-one style)

Level 1 spontaneous occult: repertoire, Courageous Anthem composition cantrip, one muse feat. Needs spontaneous casting + composition/sustain.

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
