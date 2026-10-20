# Sorcerer

**Source:** Player Core 2 pg. 144 (Player Core 2). Remaster naming. Archives of Nethys class id `62`.

**Role in Genesis:** class content pack entry consumed by phased preparation in `crates/rules` (see [architecture.md](../architecture.md)). Fixtures remain hand-authored until the editor authors the same schemas.

## Identity

| Field | Value |
| --- | --- |
| Key attribute | Charisma |
| HP per level | 6 plus your Constitution modifier (plus ancestry HP at level 1 total) |
| Book | Player Core 2 |

## Initial proficiencies (level 1)

- At 1st level, you gain the listed proficiency ranks in the following statistics. You are untrained in anything not listed unless you gain a better proficiency rank in some other way.
- **Perception:** Trained in Perception
- **Saving Throws:** Trained in Fortitude
- Trained in Reflex
- Expert in Will
- **Skills:** Trained in two skills determined by your bloodline
- Trained in a number of additional skills equal to 2 plus your Intelligence modifier
- **Attacks:** Trained in simple weapons
- Trained in unarmed attacks
- **Defenses:** Untrained in all armor
- Trained in unarmored defense
- **Class DC:** Trained in sorcerer class DC
- **Spells:** Trained in spell attack modifier
- Trained in spell DC

### Proficiency progression (engine)

Store rank enums (`Untrained/Trained/Expert/Master/Legendary`) per statistic. Class features that say “your proficiency rank for X increases to Y” are rule elements applied in preparation phases—not ad hoc branches in `encounter.rs`. Attack/defense/Perception/save/class-DC/spell ranks listed above are the L1 baseline; later rows in the advancement table upgrade them.

## Advancement table

| Level | Features gained |
| --- | --- |
| 1 | Ancestry and background, attribute boosts, initial proficiencies, bloodline, sorcerer spellcasting, spell repertoire, sorcerous potency |
| 2 | Skill feat, sorcerer feat |
| 3 | 2nd-rank spells, general feat, signature spells, skill increase |
| 4 | Skill feat, sorcerer feat |
| 5 | 3rd-rank spells, attribute boosts, ancestry feat, magical fortitude, skill increase |
| 6 | Skill feat, sorcerer feat |
| 7 | 4th-rank spells, expert spellcaster, general feat, skill increase |
| 8 | Skill feat, sorcerer feat |
| 9 | 5th-rank spells, ancestry feat, reflex expert, skill increase |
| 10 | Attribute boosts, skill feat, sorcerer feat |
| 11 | 6th-rank spells, general feat, perception expert, skill increase, weapon expertise |
| 12 | Skill feat, sorcerer feat |
| 13 | 7th-rank spells, ancestry feat, defensive robes, skill increase, weapon specialization |
| 14 | Skill feat, sorcerer feat |
| 15 | 8th-rank spells, attribute boosts, general feat, master spellcaster, skill increase |
| 16 | Skill feat, sorcerer feat |
| 17 | 9th-rank spells, ancestry feat, majestic will, skill increase |
| 18 | Skill feat, sorcerer feat |
| 19 | Bloodline paragon, general feat, legendary spellcaster, skill increase |
| 20 | Attribute boosts, skill feat, sorcerer feat |

## Class features (procedure reference)

Summaries below are implementation-shaped paraphrases for engine work. Cite the book section when verifying; do not paste Paizo text into code or fixtures.

### Bloodline (Level 1)

- **When gained:** Level 1
- **Action economy:** class feature / choice
- **Procedure (engine):** Choose a bloodline that gives you your spellcasting talent. This choice determines the type of spells you cast and the spell list you choose them from, the additional spells you learn, and your additional trained skills. You also gain Focus Points and special focus spells based on your bloodline. Bloodlines can be found here .
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Sorcerer Spellcasting (Level 1)

- **When gained:** Level 1
- **Action economy:** class feature / choice
- **Procedure (engine):** Your bloodline provides you with incredible magical power. You are a spellcaster and can cast spells using the Cast a Spell activity. The tradition from which your spells come is determined by your bloodline. As a sorcerer, you draw your power from within, and the gestures and incantations you use to shape your power might evoke the nature of your bloodline. For instance, a sorcerer with a demonic bloodline might…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Spell Repertoire (Level 1)

- **When gained:** Level 1
- **Action economy:** class feature / choice
- **Procedure (engine):** The collection of spells you can cast is called your spell repertoire. At 1st level, you learn two 1st-rank spells of your choice and four cantrips of your choice, as well as an additional spell and cantrip from your bloodline You choose these from the common spells from the tradition corresponding to your bloodline, or from other spells from that tradition to which you have access. You can cast any spell in your…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Sorcerous Potency (Level 1)

- **When gained:** Level 1
- **Action economy:** class feature / choice
- **Procedure (engine):** Because of the magical power inherent in your blood, your spells that hurt or cure are stronger than those of other spellcasters. When you Cast a Spell from your spell slots that either deals damage or restores Hit Points, you gain a status bonus to that spell's damage or healing equal to the spell's rank. This applies only to the initial damage or healing the spell deals when cast. An individual creature takes…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Skill Feats (Level 2)

- **Procedure:** At 2nd level and every 2 levels thereafter, you gain a skill feat. You must be trained or better in the corresponding skill to select a skill feat.
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Sorcerer Feats (Level 2)

- **Procedure:** At 2nd level and every even-numbered level thereafter, you gain a sorcerer class feat .
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Signature Spells (Level 3)

- **When gained:** Level 3
- **Action economy:** class feature / choice
- **Procedure (engine):** Your innate power allows you to cast some of your spells more flexibly. For each spell rank you have access to, choose one spell of that rank to be a signature spell. You don’t need to learn heightened versions of signature spells separately; instead, you can heighten these spells freely. If you’ve learned a signature spell at a higher rank than its minimum, you can also cast all its lower-rank versions without…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Skill Increases (Level 3)

- **Procedure:** At 3rd level and every 2 levels thereafter, you gain a skill increase. You can use an increase to either become trained in one skill you're untrained in, or to increase your proficiency rank in one skill in which you're already trained to expert. At 7th level, you can use skill increases to become a master in a skill in which you're already an expert, and at 15th level, you can use them to become legendary in a…
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Ancestry Feats (Level 5)

- **Procedure:** In addition to the ancestry feat you started with, you gain an ancestry feat at 5th level and every 4 levels thereafter.
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Magical Fortitude (Level 5)

- **When gained:** Level 5
- **Action economy:** class feature / choice
- **Procedure (engine):** Magical power has improved your body’s resiliency. Your proficiency rank for Fortitude saves increases to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Expert Spellcaster (Level 7)

- **When gained:** Level 7
- **Action economy:** class feature / choice
- **Procedure (engine):** Your inherent magic responds easily and powerfully to your command. Your proficiency ranks for spell attack modifier and spell DC increase to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Reflex Expertise (Level 9)

- **When gained:** Level 9
- **Action economy:** class feature / choice
- **Procedure (engine):** Your reflexes are lightning fast. Your proficiency rank for Reflex saves increases to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Perception Expertise (Level 11)

- **When gained:** Level 11
- **Action economy:** class feature / choice
- **Procedure (engine):** You remain alert to threats around you. Your proficiency rank for Perception increases to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Weapon Expertise (Level 11)

- **When gained:** Level 11
- **Action economy:** class feature / choice
- **Procedure (engine):** You’ve improved your combat skill. Your proficiency ranks for simple weapons and unarmed attacks increase to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Defensive Robes (Level 13)

- **When gained:** Level 13
- **Action economy:** class feature / choice
- **Procedure (engine):** The flow of magic and your defensive training combine to help you avoid attacks. Your proficiency rank in unarmored defense increases to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Weapon Specialization (Level 13)

- **When gained:** Level 13
- **Action economy:** class feature / choice
- **Procedure (engine):** You’ve learned to inflict grave wounds with your most practiced weapons. You deal 2 additional damage with weapons and unarmed attacks in which you are an expert. This damage increases to 3 if you’re a master, and 4 if you’re legendary.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Master Spellcaster (Level 15)

- **When gained:** Level 15
- **Action economy:** class feature / choice
- **Procedure (engine):** You have achieved mastery over the magic in your blood. Your proficiency ranks for spell attack modifiers and spell DCs increase to master.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Majestic Will (Level 17)

- **When gained:** Level 17
- **Action economy:** class feature / choice
- **Procedure (engine):** Your mind and will are tempered by your mastery of magical forces. Your proficiency rank for Will saves increases to master. When you roll a success on a Will save, you get a critical success instead.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Bloodline Paragon (Level 19)

- **When gained:** Level 19
- **Action economy:** class feature / choice
- **Procedure (engine):** You have perfected the magic in your bloodline. Add two common 10th-rank spells of your tradition to your repertoire. You gain a single 10th-rank spell slot you can use to cast these spells, using sorcerer spellcasting. Unlike other spell slots, you don’t gain more 10th-rank spells as you level up, and they can’t be used for abilities that let you cast spells without expending spell slots or abilities that give…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Legendary Spellcaster (Level 19)

- **When gained:** Level 19
- **Action economy:** class feature / choice
- **Procedure (engine):** You demonstrate prodigious talent for spellcasting. Your proficiency ranks for spell attack modifiers and spell DCs increase to legendary. Site Owner: Rose-Winds LLC (Blake Davis) (adsbygoogle = window.adsbygoogle || []).push({}); Character Creation + Ancestries Archetypes Backgrounds Classes Skills Companions Familiars Equipment + All Equipment Adventuring Gear Alchemical Items Armor Held Items Runes Shields…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.


## Focus spells / inseparable kits

- **Bloodline spells:** granted spells into repertoire + bloodline focus spell.

## Store vs derive

- **Store (character / preparation):** class id, level, key attribute choice, ancestry HP contribution, selected skills, feat picks (class/skill/general/ancestry), class choice subtree (racket/muse/order/doctrine/school/thesis/patron/bloodline/mystery/instinct/cause/style/methodology/research field), trained weapon/armor ranks overrides from features.
- **Store (encounter):** HP, conditions, focus points remaining, spell slots remaining / repertoire prepared-for-day, raised shield / stance / rage / panache / hunted prey id, reactions available.
- **Derive at prepare:** proficiency bonuses (level + rank), AC from armor + Dex + bonuses, Strike attack/damage from weapon + attr + proficiency + specialization, Perception/saves/class DC/spell DC, spell attack, MAP rules, off-guard sources.
- **Derive per check:** circumstance/status/item bonus nets by type, degree of success, precision applicability.
- **Spellcasting store:** tradition, casting style (prepared/spontaneous), spell list access, cantrips prepared or in repertoire, ranked slots or repertoire known, signature spells (spontaneous), focus spell ids, focus pool size.

## Dependencies (substrate not yet in engine or only partially present)

Baseline on `main` (~`2f56abb`) already runs Stride/Step/Strike, Reactive Strike, dying, shields, prone movement, cover/detection, manipulate, Delay/Ready, Aid (attack-bonus substitute). This class still needs:

- `architecture.md#feat-slots`
- `architecture.md#proficiency`
- `conditions-and-effects.md`
- `spells/framework.md`

- Skill list + proficiency bonuses on checks (Aid currently substitutes Strike bonus; ledger entry).
- Feat slot model + at least data-driven class feats used in the first slice.
- Armor/weapon itemization beyond fixture-prebaked AC and one Strike (unless the slice keeps baked numbers with a Principle III note).
- Conditions listed under [conditions-and-effects.md](../conditions-and-effects.md) that this class applies or keys off.

## Suggested first slice (room-one style)

Level 1 spontaneous (bloodline tradition): repertoire, bloodline spell as focus, Sorcerous Potency. Needs spontaneous + focus.

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
