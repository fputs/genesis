# Investigator

**Source:** Player Core 2 pg. 100 (Player Core 2). Remaster naming. Archives of Nethys class id `59`.

**Role in Genesis:** class content pack entry consumed by phased preparation in `crates/rules` (see [architecture.md](../architecture.md)). Fixtures remain hand-authored until the editor authors the same schemas.

## Identity

| Field | Value |
| --- | --- |
| Key attribute | Intelligence |
| HP per level | 8 plus your Constitution modifier (plus ancestry HP at level 1 total) |
| Book | Player Core 2 |

## Initial proficiencies (level 1)

- At 1st level, you gain the listed proficiency ranks in the following statistics. You are untrained in anything not listed unless you gain a better proficiency rank in some other way.
- **Perception:** Expert in Perception
- **Saving Throws:** Trained in Fortitude
- Expert in Reflex
- Expert in Will
- **Skills:** Trained in Society
- Trained in one or more skills determined by your methodology
- Trained in a number of additional skills equal to 4 plus your Intelligence modifier
- **Attacks:** Trained in simple and martial weapons
- Trained in unarmed attack
- **Defenses:** Trained in light armor
- Trained in unarmored defense
- **Class DC:** Trained in investigator class DC

### Proficiency progression (engine)

Store rank enums (`Untrained/Trained/Expert/Master/Legendary`) per statistic. Class features that say “your proficiency rank for X increases to Y” are rule elements applied in preparation phases—not ad hoc branches in `encounter.rs`. Attack/defense/Perception/save/class-DC/spell ranks listed above are the L1 baseline; later rows in the advancement table upgrade them.

## Advancement table

| Level | Features gained |
| --- | --- |
| 1 | Ancestry and background, attribute boosts, initial proficiencies, on the case, Devise A Stratagem, methodology, investigator feat, strategic strike 1d6 |
| 2 | Investigator feat, skill feat, skill increase |
| 3 | General feat, keen recollection, skill increase, skillful lesson |
| 4 | Investigator feat, skill feat, skill increase |
| 5 | Ancestry feat, attribute boosts, skill increase, skillful lesson, strategic strike 2d6, weapon expertise |
| 6 | Investigator feat, skill feat, skill increase |
| 7 | General feat, skill increase, skillful lesson, vigilant senses, weapon specialization |
| 8 | Investigator feat, skill feat, skill increase |
| 9 | Ancestry feat, fortitude expertise, investigator expertise, skill increase, skillful lesson, strategic strike 3d6 |
| 10 | Attribute boosts, investigator feat, skill feat, skill increase |
| 11 | Deductive improvisation, dogged will, general feat, skill increase, skillful lesson |
| 12 | Investigator feat, skill feat, skill increase |
| 13 | Ancestry feat, incredible senses, light armor expertise, skill increase, skillful lesson, strategic strike 4d6, weapon mastery |
| 14 | Investigator feat, skill feat, skill increase |
| 15 | Attribute boosts, general feat, greater weapon specialization, savvy reflexes, skill increase, skillful lesson |
| 16 | Investigator feat, skill feat, skill increase |
| 17 | Ancestry feat, greater dogged will, skill increase, skillful lesson, strategic strike 5d6 |
| 18 | Investigator feat, skill feat, skill increase |
| 19 | General feat, light armor mastery, master detective, skill increase, skillful lesson |
| 20 | Attribute boosts, investigator feat, skill feat, skill increase |

## Class features (procedure reference)

Summaries below are implementation-shaped paraphrases for engine work. Cite the book section when verifying; do not paste Paizo text into code or fixtures.

### On the Case (Level 1)

- **When gained:** Level 1
- **Action economy:** class feature / choice
- **Procedure (engine):** As an investigator, you think of your adventures as cases waiting to be solved. You gain one activity and one reaction you can use to investigate cases: Pursue a Lead and Clue In. Pursue a Lead Concentrate Exploration Investigator Source Player Core 2 pg. 102 You name one detail you've identified that you think is part of a larger mystery, then spend 1 minute examining it. The detail is typically either obvious…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Devise a Stratagem (Level 1)

- **When gained:** Level 1
- **Action economy:** see procedure (often 1 action or free)
- **Procedure (engine):** You can play out a battle in your head, using brains rather than brawn to execute an attack. Devise a Stratagem [free-action] or [one-action] Concentrate Investigator Source Player Core 2 pg. 103 Frequency once per round You assess a foe's weaknesses in combat and use them to formulate a plan of attack. Choose a creature you can see. You can Devise a Stratagem as a free action if you're aware that creature could…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Methodology (Level 1)

- **When gained:** Level 1
- **Action economy:** class feature / choice
- **Procedure (engine):** Your studies have made you savvy in a broad spectrum of areas, but one particular investigative process drew your intense—even obsessive—interest. Choose a methodology. Methodologies can be found here .
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Investigator Feats (cadence)

- **Procedure:** At 1st level and every even-numbered level, you gain an investigator class feat .
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Strategic Strike (Level 1)

- **When gained:** Level 1
- **Action economy:** see procedure (often 1 action or free)
- **Procedure (engine):** When you strike carefully and with forethought, you deal a telling blow. When making a Strike that adds your Intelligence modifier to your attack roll due to Devising a Stratagem, you deal an additional 1d6 precision damage. As your investigator level increases, so too does the deadliness of your strategic strike. Increase the number of dice by one at 5th, 9th, 13th, and 17th levels.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Skill Feats (Level 2)

- **Procedure:** At 2nd level and every 2 levels thereafter, you gain a skill feat. You must be trained or better in the corresponding skill to select a skill feat.
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Skill Increases (Level 2)

- **Procedure:** You gain more skill increases than members of other classes. At 2nd level and every level thereafter, you gain a skill increase. You can use this increase to either become trained in one skill you're untrained in or to become an expert in one skill in which you're already trained. At 7th level, you can use skill increases to become a master in a skill in which you're already an expert, and at 15th level, you can…
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### General Feats (Level 3)

- **Procedure:** At 3rd level and every 4 levels thereafter, you gain a general feat.
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Keen Recollection (Level 3)

- **When gained:** Level 3
- **Action economy:** class feature / choice
- **Procedure (engine):** You can recall pertinent facts on topics that aren’t your specialty. Your proficiency bonus to untrained skill checks to Recall Knowledge is equal to your level instead of +0.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Skillful Lessons (Level 3)

- **When gained:** Level 3
- **Action economy:** class feature / choice
- **Procedure (engine):** At 3rd level and every odd-numbered level thereafter, you gain a skill feat. This feat must be for an Intelligence-, Wisdom-, or Charisma-based skill, or for the skill you gained from your methodology.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Ancestry Feats (Level 5)

- **Procedure:** In addition to the ancestry feat you started with, you gain an ancestry feat at 5th level and every 4 levels thereafter.
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Weapon Expertise (Level 5)

- **When gained:** Level 5
- **Action economy:** class feature / choice
- **Procedure (engine):** You’ve dedicated yourself to learning the intricacies of your weapons. Your proficiency ranks for simple weapons, martial weapons, and unarmed attacks increase to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Vigilant Senses (Level 7)

- **When gained:** Level 7
- **Action economy:** class feature / choice
- **Procedure (engine):** Through your adventures, you’ve developed keen awareness and attention to detail. Your proficiency rank in Perception increases to master.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Weapon Specialization (Level 7)

- **When gained:** Level 7
- **Action economy:** class feature / choice
- **Procedure (engine):** You can inflict greater injuries with the weapons you know best. You deal 2 additional damage with weapons and unarmed attacks in which you are an expert. This damage increases to 3 if you’re a master, and to 4 if you’re legendary.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Fortitude Expertise (Level 9)

- **When gained:** Level 9
- **Action economy:** class feature / choice
- **Procedure (engine):** Your physique has grown hardy in your adventures. Your proficiency rank for Fortitude saves increases to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Investigator Expertise (Level 9)

- **When gained:** Level 9
- **Action economy:** class feature / choice
- **Procedure (engine):** You’ve refined your investigative techniques to an exceptional degree. Your circumstance bonus from Pursue a Lead increases to +2. Your proficiency rank for your investigator class DC increases to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Deductive Improvisation (Level 11)

- **When gained:** Level 11
- **Action economy:** class feature / choice
- **Procedure (engine):** You use your skills at deduction to perform whatever task is necessary to solve the case. You can attempt any check that requires you to be trained in a skill, even if you’re untrained in it; you can attempt any check that requires you to have expert proficiency in a skill so long as you’re trained in it; and you can attempt any check that requires you to have master proficiency in a skill as long as you have…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Dogged Will (Level 11)

- **When gained:** Level 11
- **Action economy:** class feature / choice
- **Procedure (engine):** In your pursuit of truth and knowledge, you won’t be dissuaded. Your proficiency rank for Will saves increases to master. When you roll a success on a Will save, you get a critical success instead.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Incredible Senses (Level 13)

- **When gained:** Level 13
- **Action economy:** class feature / choice
- **Procedure (engine):** You notice things that are almost impossible for an ordinary person to detect. Your proficiency rank for Perception increases to legendary.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Light Armor Expertise (Level 13)

- **When gained:** Level 13
- **Action economy:** class feature / choice
- **Procedure (engine):** You’ve learned how to dodge while wearing light or no armor. Your proficiency ranks for light armor and unarmored defense increase to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Weapon Mastery (Level 13)

- **When gained:** Level 13
- **Action economy:** class feature / choice
- **Procedure (engine):** You fully understand your weapons. Your proficiency ranks for simple weapons, martial weapons, and unarmed attacks increase to master.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Greater Weapon Specialization (Level 15)

- **When gained:** Level 15
- **Action economy:** class feature / choice
- **Procedure (engine):** Your damage from weapon specialization increases to 4 with weapons and unarmed attacks in which you’re an expert, 6 if you’re a master, and 8 if you’re legendary.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Savvy Reflexes (Level 15)

- **When gained:** Level 15
- **Action economy:** class feature / choice
- **Procedure (engine):** With an eye for incoming danger, you masterfully keep yourself out of harm’s way. Your proficiency rank for Reflex saves increases to master. When you roll a success on a Reflex save, you get a critical success instead.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Greater Dogged Will (Level 17)

- **When gained:** Level 17
- **Action economy:** class feature / choice
- **Procedure (engine):** Your willpower becomes truly unbreakable. Your proficiency rank for Will saves increases to legendary. When you roll a success on a Will save, you get a critical success. When you roll a critical failure on a Will save, you get a failure instead. When you fail a Will save against a damaging effect, you take half damage.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Light Armor Mastery (Level 19)

- **When gained:** Level 19
- **Action economy:** class feature / choice
- **Procedure (engine):** Your skill with light armor improves, increasing your ability to dodge blows. Your proficiency ranks for light armor and unarmored defense increase to master.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Master Detective (Level 19)

- **When gained:** Level 19
- **Action economy:** class feature / choice
- **Procedure (engine):** As a master detective, you can solve any case and find all the clues available. Your reputation precedes you, and even the most skilled criminals and deceivers fear you. Your proficiency rank for your investigator class DC increases to master. In addition, if you enter a new location that includes another clue toward answering the question at the heart of one of your active investigations, the GM informs you of…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.


## Focus spells / inseparable kits

- No focus/spell kit required for the class's martial L1 slice beyond listed features.

## Store vs derive

- **Store (character / preparation):** class id, level, key attribute choice, ancestry HP contribution, selected skills, feat picks (class/skill/general/ancestry), class choice subtree (racket/muse/order/doctrine/school/thesis/patron/bloodline/mystery/instinct/cause/style/methodology/research field), trained weapon/armor ranks overrides from features.
- **Store (encounter):** HP, conditions, focus points remaining, spell slots remaining / repertoire prepared-for-day, raised shield / stance / rage / panache / hunted prey id, reactions available.
- **Derive at prepare:** proficiency bonuses (level + rank), AC from armor + Dex + bonuses, Strike attack/damage from weapon + attr + proficiency + specialization, Perception/saves/class DC/spell DC, spell attack, MAP rules, off-guard sources.
- **Derive per check:** circumstance/status/item bonus nets by type, degree of success, precision applicability.
- **Stratagem store:** active stratagem target + rolled d20 substitution token for next Strike against that target.

## Dependencies (substrate not yet in engine or only partially present)

Baseline on `main` (~`2f56abb`) already runs Stride/Step/Strike, Reactive Strike, dying, shields, prone movement, cover/detection, manipulate, Delay/Ready, Aid (attack-bonus substitute). This class still needs:

- `architecture.md#checks`
- `architecture.md#class-choices`
- `architecture.md#feat-slots`
- `architecture.md#proficiency`
- `conditions-and-effects.md#armor-and-item-bonuses`

- Skill list + proficiency bonuses on checks (Aid currently substitutes Strike bonus; ledger entry).
- Feat slot model + at least data-driven class feats used in the first slice.
- Armor/weapon itemization beyond fixture-prebaked AC and one Strike (unless the slice keeps baked numbers with a Principle III note).
- Conditions listed under [conditions-and-effects.md](../conditions-and-effects.md) that this class applies or keys off.

## Suggested first slice (room-one style)

Level 1: Devise a Stratagem (free/action variants stubbed to encounter action), Strategic Strike 1d6. Needs check substitution on Strike.

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
