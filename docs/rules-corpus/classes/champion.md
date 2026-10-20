# Champion

**Source:** Player Core 2 pg. 86 (Player Core 2). Remaster naming. Archives of Nethys class id `58`.

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
- Trained in Reflex
- Expert in Will
- **Skills:** Trained in Religion
- Trained in one skill determined by your choice of deity
- Trained in a number of additional skills equal to 2 plus your Intelligence modifier
- **Attacks:** Trained in simple weapons
- Trained in martial weapons
- Trained in unarmed attacks
- **Defenses:** Trained in all armor
- Trained in unarmored defense
- **Class DC:** Trained in champion class DC
- **Spells:** Trained in spell attack modifier
- Trained in spell DC

### Proficiency progression (engine)

Store rank enums (`Untrained/Trained/Expert/Master/Legendary`) per statistic. Class features that say “your proficiency rank for X increases to Y” are rule elements applied in preparation phases—not ad hoc branches in `encounter.rs`. Attack/defense/Perception/save/class-DC/spell ranks listed above are the L1 baseline; later rows in the advancement table upgrade them.

## Advancement table

| Level | Features gained |
| --- | --- |
| 1 | Ancestry and background, attribute boosts, initial proficiencies, deity, cause, devotion spells, champion feat, shield block |
| 2 | Champion feat, skill feat |
| 3 | Blessing of the devoted, general feat, skill increase |
| 4 | Champion feat, skill feat |
| 5 | Attribute boosts, ancestry feat, skill increase, weapon expertise |
| 6 | Champion feat, skill feat |
| 7 | Armor expertise, general feat, skill increase, weapon specialization |
| 8 | Champion feat, skill feat |
| 9 | Ancestry feat, champion expertise, reflex expertise, relentless reaction, sacred body, skill increase |
| 10 | Attribute boosts, champion feat, skill feat |
| 11 | Divine will, exalted reaction, general feat, perception expertise, skill increase |
| 12 | Champion feat, skill feat |
| 13 | Ancestry feat, armor mastery, skill increase weapon mastery |
| 14 | Champion feat, skill feat |
| 15 | Attribute boosts, general feat, greater weapon specialization, skill increase |
| 16 | Champion feat, skill feat |
| 17 | Ancestry feat, champion mastery, legendary armor, skill increase |
| 18 | Champion feat, skill feat |
| 19 | General feat, hero’s defiance, skill increase |
| 20 | Attribute boosts, champion feat, skill feat |

## Class features (procedure reference)

Summaries below are implementation-shaped paraphrases for engine work. Cite the book section when verifying; do not paste Paizo text into code or fixtures.

### Deity (Level 1)

- **When gained:** Level 1
- **Action economy:** class feature / choice
- **Procedure (engine):** As a champion, you are a warrior in the name of a deity you revere above all others. The most common deities in Pathfinder have edicts, areas of concern, and the benefits you get for being a devotee of that deity. Choose one deity . Skill You become trained in the divine skill listed for your deity. As normal, if you're already trained in that skill, you become trained in a different skill of your choice.…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Cause (Level 1)

- **When gained:** Level 1
- **Action economy:** class feature / choice
- **Procedure (engine):** You devote yourself to a specific cause in your deity’s name. Some causes are limited to certain sanctifications. Your cause adds to your edicts and anathema and grants you a special protective reaction called your champion’s reaction. Causes can be found here .
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Devotion Spells (Level 1)

- **When gained:** Level 1
- **Action economy:** class feature / choice
- **Procedure (engine):** Your deity's power grants you special divine spells called devotion spells, which are a type of focus spell. Choose either shields of the spirit or a spell based on your deity's divine font ( lay on hands if your deity allows heal, touch of the void if your deity allows harm). It costs 1 Focus Point to cast a focus spell. When you gain your first devotion spell, you also gain a focus pool of 1 Focus Point. You…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Champion Feats (cadence)

- **Procedure:** At 1st level and every even-numbered level thereafter, you gain a champion class feat .
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Shield Block (Level 1)

- **When gained:** Level 1
- **Action economy:** class feature / choice
- **Procedure (engine):** You gain the Shield Block general feat, a reaction that lets you reduce damage with your shield.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Skill Feats (Level 2)

- **Procedure:** At 2nd level and every 2 levels thereafter, you gain a skill feat. You must be trained or better in the corresponding skill to select a skill feat.
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Blessing of the Devoted (Level 3)

- **When gained:** Level 3
- **Action economy:** class feature / choice
- **Procedure (engine):** Your deity blesses your service with a boon. This might come in the form of a spirit that visits you and inhabits your items or body, a mysterious divine tattoo upon your body, or the like. Choose one of the following blessings, or any other to which you have access. Blessed Armament: Select one weapon or handwraps of mighty blows . You gain that armament's critical specialization effect, and you grant the…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### General Feats (Level 3)

- **Procedure:** At 3rd level and every 4 levels thereafter, you gain a general feat.
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Skill Increases (Level 3)

- **Procedure:** At 3rd level and every 2 levels thereafter, you gain a skill increase. You can use this increase either to increase your proficiency rank to trained in one skill you’re untrained in, or to increase your proficiency rank in one skill in which you’re already trained to expert. At 7th level, you can use skill increases to increase your proficiency rank to master in a skill in which you’re already an expert, and at…
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Ancestry Feats (Level 5)

- **Procedure:** In addition to the ancestry feat you started with, you gain an ancestry feat at 5th level and every 4 levels thereafter.
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Weapon Expertise (Level 5)

- **When gained:** Level 5
- **Action economy:** class feature / choice
- **Procedure (engine):** You’ve dedicated yourself to learning the intricacies of your weapons. Your proficiency ranks for simple weapons, martial weapons, and unarmed attacks increase to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Armor Expertise (Level 7)

- **When gained:** Level 7
- **Action economy:** class feature / choice
- **Procedure (engine):** You have spent so much time in armor that you know how to make the most of its protection. Your proficiency ranks for light, medium, and heavy armor, as well as for unarmored defense, increase to expert. You gain the armor specialization effects of medium and heavy armor.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Weapon Specialization (Level 7)

- **When gained:** Level 7
- **Action economy:** class feature / choice
- **Procedure (engine):** You’ve learned how to inflict greater injuries with the weapons you know best. You deal 2 additional damage with weapons and unarmed attacks in which you’re an expert. This damage increases to 3 if you’re a master, and to 4 if you’re legendary.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Champion Expertise (Level 9)

- **When gained:** Level 9
- **Action economy:** class feature / choice
- **Procedure (engine):** Prayers strengthen your divine power. Your proficiency ranks for your champion class DC, spell attack modifier, and spell DC increase to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Reflex Expertise (Level 9)

- **When gained:** Level 9
- **Action economy:** class feature / choice
- **Procedure (engine):** You’ve developed a knack for dodging danger. Your proficiency rank for Reflex saves increases to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Relentless Reaction (Level 9)

- **When gained:** Level 9
- **Action economy:** class feature / choice
- **Procedure (engine):** Your champion’s reaction improves, typically to punish the attacker with spiritual backlash. It gains the relentless reaction benefit listed in your cause below your champion’s reaction.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Sacred Body (Level 9)

- **When gained:** Level 9
- **Action economy:** class feature / choice
- **Procedure (engine):** The power of your deity keeps your body safe from harm, as true an edifice of the faith as a great temple. Your proficiency rank for Fortitude saves increases to master. When you roll a success on a Fortitude save, you get a critical success instead.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Divine Will (Level 11)

- **When gained:** Level 11
- **Action economy:** class feature / choice
- **Procedure (engine):** Your faith grants mastery of your will. Your proficiency rank for Will saves increases to master. When you roll a success on a Will save, you get a critical success instead.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Exalted Reaction (Level 11)

- **When gained:** Level 11
- **Action economy:** class feature / choice
- **Procedure (engine):** Your champion’s reaction exalts nearby allies, allowing them to benefit as well. When you use your champion’s reaction, you also get the effect listed for its exalted reaction.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Perception Expertise (Level 11)

- **When gained:** Level 11
- **Action economy:** class feature / choice
- **Procedure (engine):** You remain alert to threats around you. Your proficiency rank for Perception increases to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Armor Mastery (Level 13)

- **When gained:** Level 13
- **Action economy:** class feature / choice
- **Procedure (engine):** Your skill with armor improves, helping you avoid more blows. Your proficiency ranks for light, medium, and heavy armor, as well as for unarmored defense, increase to master.
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

### Champion Mastery (Level 17)

- **When gained:** Level 17
- **Action economy:** class feature / choice
- **Procedure (engine):** You’ve mastered your champion techniques and divine spells. Your proficiency ranks for your champion class DC, spell attack modifier, and spell DC increase to master.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Legendary Armor (Level 17)

- **When gained:** Level 17
- **Action economy:** class feature / choice
- **Procedure (engine):** You shield yourself with steel as easily as with faith. Your proficiency ranks for light, medium, and heavy armor, as well as for unarmored defense, increase to legendary.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Hero's Defiance (Level 19)

- **When gained:** Level 19
- **Action economy:** class feature / choice
- **Procedure (engine):** You can continue fighting as long as you have divine energy. You gain the hero’s defiance devotion spell Site Owner: Rose-Winds LLC (Blake Davis) (adsbygoogle = window.adsbygoogle || []).push({}); Character Creation + Ancestries Archetypes Backgrounds Classes Skills Companions Familiars Equipment + All Equipment Adventuring Gear Alchemical Items Armor Held Items Runes Shields Weapons Worn Items Feats + All Feats…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.


## Focus spells / inseparable kits

- **Devotion spells:** focus spells from cause (e.g. lay-on-hands style effects).

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
- `baseline Raise a Shield / Shield Block (already in rules)`
- `conditions-and-effects.md#armor-and-item-bonuses`
- `spells/framework.md`

- Skill list + proficiency bonuses on checks (Aid currently substitutes Strike bonus; ledger entry).
- Feat slot model + at least data-driven class feats used in the first slice.
- Armor/weapon itemization beyond fixture-prebaked AC and one Strike (unless the slice keeps baked numbers with a Principle III note).
- Conditions listed under [conditions-and-effects.md](../conditions-and-effects.md) that this class applies or keys off.

## Suggested first slice (room-one style)

Level 1: deity+cause stubs, Shield Block, one devotion focus spell (Lay on Hands / Touch of Corruption family). Needs focus pool + simple heal/harm spell.

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
