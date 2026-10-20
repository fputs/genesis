# Monk

**Source:** Player Core 2 pg. 114 (Player Core 2). Remaster naming. Archives of Nethys class id `60`.

**Role in Genesis:** class content pack entry consumed by phased preparation in `crates/rules` (see [architecture.md](../architecture.md)). Fixtures remain hand-authored until the editor authors the same schemas.

## Identity

| Field | Value |
| --- | --- |
| Key attribute | Strength or Dexterity |
| HP per level | 10 plus your Constitution modifier (plus ancestry HP at level 1 total) |
| Book | Player Core 2 |

## Initial proficiencies (level 1)

- At 1st level, you gain the listed proficiency ranks in the following statistics. You are untrained in anything not listed unless you gain a better proficiency rank in some other way.
- **Perception:** Trained in Perception
- **Saving Throws:** Expert in Fortitude
- Expert in Reflex
- Expert in Will
- **Skills:** Trained in a number of skills equal to 4 plus your Intelligence modifier
- **Attacks:** Trained in simple weapons
- Trained in unarmed attacks
- **Defenses:** Untrained in all armor
- Expert in unarmored defense
- **Class DC:** Trained in monk class DC

### Proficiency progression (engine)

Store rank enums (`Untrained/Trained/Expert/Master/Legendary`) per statistic. Class features that say “your proficiency rank for X increases to Y” are rule elements applied in preparation phases—not ad hoc branches in `encounter.rs`. Attack/defense/Perception/save/class-DC/spell ranks listed above are the L1 baseline; later rows in the advancement table upgrade them.

## Advancement table

| Level | Features gained |
| --- | --- |
| 1 | Ancestry and background, attribute boosts, initial proficiencies, flurry of blows, monk feat, powerful fist |
| 2 | Monk feat, skill feat |
| 3 | General feat, incredible movement +10 feet, mystic strikes, skill increase |
| 4 | Monk feat, skill feat |
| 5 | Attribute boosts, ancestry feat, expert strikes, perception expertise, skill increase |
| 6 | Monk feat, skill feat |
| 7 | General feat, incredible movement +15 feet, path to perfection, skill increase, weapon specialization |
| 8 | Monk feat, skill feat |
| 9 | Ancestry feat, metal strikes, monk expertise, skill increase |
| 10 | Attribute boosts, monk feat, skill feat |
| 11 | General feat, incredible movement +20 feet, second path to perfection, skill increase |
| 12 | Monk feat, skill feat |
| 13 | Ancestry feat, graceful mastery, master strikes, skill increase |
| 14 | Monk feat, skill feat |
| 15 | Attribute boosts, general feat, greater weapon specialization, incredible movement +25 feet, skill increase, third path to perfection |
| 16 | Monk feat, skill feat |
| 17 | Adamantine strikes, ancestry feat, graceful legend, skill increase |
| 18 | Monk feat, skill feat |
| 19 | General feat, incredible movement +30 feet, perfected form, skill increase |
| 20 | Attribute boosts, monk feat, skill feat |

## Class features (procedure reference)

Summaries below are implementation-shaped paraphrases for engine work. Cite the book section when verifying; do not paste Paizo text into code or fixtures.

### Flurry of Blows (Level 1)

- **When gained:** Level 1
- **Action economy:** 1 action (flourish)
- **Procedure (engine):** Make two unarmed Strikes; MAP applies across them. Only one flourish action per round.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Monk Feats (cadence)

- **Procedure:** At 1st level and every even-numbered level thereafter, you gain a monk class feat .
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Powerful Fist (Level 1)

- **When gained:** Level 1
- **Action economy:** class feature / choice
- **Procedure (engine):** Your fists are deadly weapons. The damage die for your fist increases to 1d6 instead of 1d4. You don’t take the normal –2 circumstance penalty when making a lethal attack with your fist or any other unarmed attacks.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Skill Feats (Level 2)

- **Procedure:** At 2nd level and every 2 levels thereafter, you gain a skill feat. You must be trained or better in the corresponding skill to select a skill feat.
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### General Feats (Level 3)

- **Procedure:** At 3rd level and every 4 levels thereafter, you gain a general feat.
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Incredible Movement (Level 3)

- **When gained:** Level 3
- **Action economy:** class feature / choice
- **Procedure (engine):** You move like the wind. You gain a +10-foot status bonus to your Speed whenever you’re not wearing armor. The bonus increases by 5 feet for every 4 levels you have beyond 3rd.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Mystic Strikes (Level 3)

- **When gained:** Level 3
- **Action economy:** see procedure (often 1 action or free)
- **Procedure (engine):** Focusing your will into your physical attacks imbues them with mystical energy. Your unarmed attacks become magical, allowing them to get past resistances to non-magical attacks. However, you still need an item such as handwraps of mighty blows to gain an item bonus to attack rolls or increase your attacks' weapon damage dice.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Skill Increases (Level 3)

- **Procedure:** At 3rd level and every 2 levels thereafter, you gain a skill increase. You can use an increase to either become trained in one skill you're untrained in, or to increase your proficiency rank in one skill in which you're already trained to expert. At 7th level, you can use skill increases to become a master in a skill in which you're already an expert, and at 15th level, you can use them to become legendary in a…
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Ancestry Feats (Level 5)

- **Procedure:** In addition to the ancestry feat you started with, you gain an ancestry feat at 5th level and every 4 levels thereafter.
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Expert Strikes (Level 5)

- **When gained:** Level 5
- **Action economy:** see procedure (often 1 action or free)
- **Procedure (engine):** You’ve practiced martial arts and have now surpassed your former skill. Your proficiency ranks for unarmed attacks and simple weapons increase to expert. When you get a critical hit with an unarmed attack in the brawling group, you get its critical specialization effect.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Perception Expertise (Level 5)

- **When gained:** Level 5
- **Action economy:** class feature / choice
- **Procedure (engine):** You remain alert to threats around you. Your proficiency rank for Perception increases to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Path to Perfection (Level 7)

- **When gained:** Level 7
- **Action economy:** class feature / choice
- **Procedure (engine):** You have progressed along your own path to enlightenment. Choose your Fortitude, Reflex, or Will saving throw. Your proficiency rank for the chosen saving throw increases to master. When you roll a success on the chosen saving throw, you get a critical success instead.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Weapon Specialization (Level 7)

- **When gained:** Level 7
- **Action economy:** class feature / choice
- **Procedure (engine):** You’ve learned how to inflict greater injuries with the weapons you know best. You deal 2 additional damage with weapons and unarmed attacks in which you are an expert. This damage increases to 3 if you’re a master, and 4 if you’re legendary.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Metal Strikes (Level 9)

- **When gained:** Level 9
- **Action economy:** see procedure (often 1 action or free)
- **Procedure (engine):** You can adjust your body to strike with the mystic energy of rare metals. Your unarmed attacks are treated as cold iron and silver. This allows you to deal more damage to a variety of supernatural creatures, such as demons, devils, and fey.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Monk Expertise (Level 9)

- **When gained:** Level 9
- **Action economy:** class feature / choice
- **Procedure (engine):** Your proficiency rank for your monk class DC increases to expert. If you have qi spells, your proficiency rank for spell attacks and spell DCs increases to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Second Path to Perfection (Level 11)

- **When gained:** Level 11
- **Action economy:** class feature / choice
- **Procedure (engine):** You’ve learned to find perfection in every success. Choose a different saving throw than the one you chose for your path to perfection. Your proficiency rank for the chosen saving throw increases to master. If you roll a success with the chosen saving throw, you instead critically succeed.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Graceful Mastery (Level 13)

- **When gained:** Level 13
- **Action economy:** class feature / choice
- **Procedure (engine):** You move with perpetual grace, turning aside blows. Your proficiency rank for unarmored defense increases to master.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Master Strikes (Level 13)

- **When gained:** Level 13
- **Action economy:** see procedure (often 1 action or free)
- **Procedure (engine):** You’ve honed your body into an even deadlier weapon. Your proficiency ranks for unarmed attacks and simple weapons increase to master.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Greater Weapon Specialization (Level 15)

- **When gained:** Level 15
- **Action economy:** class feature / choice
- **Procedure (engine):** Your damage from weapon specialization increases to 4 with weapons and unarmed attacks in which you’re an expert, 6 if you’re a master, and 8 if you’re legendary.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Third Path to Perfection (Level 15)

- **When gained:** Level 15
- **Action economy:** class feature / choice
- **Procedure (engine):** You have made great progress in your personal studies of enlightenment. Choose one of the saving throws you selected for path to perfection or second path to perfection. Your proficiency rank for the chosen type of save increases to legendary. When you roll a critical failure on the chosen type of save, you get a failure instead. When you roll a failure on the chosen type of save against an effect that deals…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Adamantine Strikes (Level 17)

- **When gained:** Level 17
- **Action economy:** see procedure (often 1 action or free)
- **Procedure (engine):** When you focus your will into your limbs, your blows are as unyielding as the hardest of metals. Your unarmed attacks are treated as adamantine.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Graceful Legend (Level 17)

- **When gained:** Level 17
- **Action economy:** class feature / choice
- **Procedure (engine):** Your sublime movement grants you unparalleled protection and offense. Your proficiency rank for unarmored defense increases to legendary, and your proficiency rank for your monk class DC increases to master. If you have qi spells, your proficiency rank for spell attack modifiers and spell DCs increases to master.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Perfected Form (Level 19)

- **When gained:** Level 19
- **Action economy:** class feature / choice
- **Procedure (engine):** You have purged incompetence from your techniques. On your first Strike of your turn, if you roll lower than 10, you can treat the attack roll as a 10. This is a fortune effect. Site Owner: Rose-Winds LLC (Blake Davis) (adsbygoogle = window.adsbygoogle || []).push({}); Character Creation + Ancestries Archetypes Backgrounds Classes Skills Companions Familiars Equipment + All Equipment Adventuring Gear Alchemical…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.


## Focus spells / inseparable kits

- **Qi spells:** optional via feats; not required for L1 flurry slice.

## Store vs derive

- **Store (character / preparation):** class id, level, key attribute choice, ancestry HP contribution, selected skills, feat picks (class/skill/general/ancestry), class choice subtree (racket/muse/order/doctrine/school/thesis/patron/bloodline/mystery/instinct/cause/style/methodology/research field), trained weapon/armor ranks overrides from features.
- **Store (encounter):** HP, conditions, focus points remaining, spell slots remaining / repertoire prepared-for-day, raised shield / stance / rage / panache / hunted prey id, reactions available.
- **Derive at prepare:** proficiency bonuses (level + rank), AC from armor + Dex + bonuses, Strike attack/damage from weapon + attr + proficiency + specialization, Perception/saves/class DC/spell DC, spell attack, MAP rules, off-guard sources.
- **Derive per check:** circumstance/status/item bonus nets by type, degree of success, precision applicability.
- **Stance store:** current stance feat id or none; flurry uses flourish once/round.

## Dependencies (substrate not yet in engine or only partially present)

Baseline on `main` (~`2f56abb`) already runs Stride/Step/Strike, Reactive Strike, dying, shields, prone movement, cover/detection, manipulate, Delay/Ready, Aid (attack-bonus substitute). This class still needs:

- `architecture.md#action-traits`
- `architecture.md#feat-slots`
- `architecture.md#proficiency`

- Skill list + proficiency bonuses on checks (Aid currently substitutes Strike bonus; ledger entry).
- Feat slot model + at least data-driven class feats used in the first slice.
- Armor/weapon itemization beyond fixture-prebaked AC and one Strike (unless the slice keeps baked numbers with a Principle III note).
- Conditions listed under [conditions-and-effects.md](../conditions-and-effects.md) that this class applies or keys off.

## Suggested first slice (room-one style)

Level 1: Flurry of Blows (flourish, two unarmed Strikes), Powerful Fist (lethal d6). Needs flourish trait gating + multiple Strikes in one action.

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
