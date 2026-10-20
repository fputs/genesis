# Barbarian

**Source:** Player Core 2 pg. 70 (Player Core 2). Remaster naming. Archives of Nethys class id `57`.

**Role in Genesis:** class content pack entry consumed by phased preparation in `crates/rules` (see [architecture.md](../architecture.md)). Fixtures remain hand-authored until the editor authors the same schemas.

## Identity

| Field | Value |
| --- | --- |
| Key attribute | Strength |
| HP per level | 12 plus your Constitution modifier (plus ancestry HP at level 1 total) |
| Book | Player Core 2 |

## Initial proficiencies (level 1)

- At 1st level, you gain the listed proficiency ranks in the following statistics. You are untrained in anything not listed unless you gain a better proficiency rank in some other way.
- **Perception:** Expert in Perception
- **Saving Throws:** Expert in Fortitude
- Trained in Reflex
- Expert in Will
- **Skills:** Trained in Athletics
- Trained in a number of additional skills equal to 3 plus your Intelligence modifier
- **Attacks:** Trained in simple weapons
- Trained in martial weapons
- Trained in unarmed attacks
- **Defenses:** Trained in light armor
- Trained in medium armor
- Trained in unarmored defense
- **Class DC:** Trained in barbarian class DC

### Proficiency progression (engine)

Store rank enums (`Untrained/Trained/Expert/Master/Legendary`) per statistic. Class features that say “your proficiency rank for X increases to Y” are rule elements applied in preparation phases—not ad hoc branches in `encounter.rs`. Attack/defense/Perception/save/class-DC/spell ranks listed above are the L1 baseline; later rows in the advancement table upgrade them.

## Advancement table

| Level | Features gained |
| --- | --- |
| 1 | Ancestry and background, attribute boosts, initial proficiencies, Rage, Quick-Tempered, instinct, barbarian feat |
| 2 | Barbarian feat, skill feat |
| 3 | Furious footfalls, general feat, skill increase |
| 4 | Barbarian feat, skill feat |
| 5 | Ancestry feat, attribute boosts, brutality, skill increase |
| 6 | Barbarian feat, skill feat |
| 7 | General feat, juggernaut, skill increase, weapon specialization |
| 8 | Barbarian feat, skill feat |
| 9 | Ancestry feat, raging resistance, reflex expertise, skill increase |
| 10 | Attribute boosts, barbarian feat, skill feat |
| 11 | General feat, mighty rage, skill increase |
| 12 | Barbarian feat, skill feat |
| 13 | Ancestry feat, greater juggernaut, medium armor expertise, skill increase, weapon mastery |
| 14 | Barbarian feat, skill feat |
| 15 | Attribute boosts, general feat, greater weapon specialization, indomitable will, skill increase |
| 16 | Barbarian feat, skill feat |
| 17 | Ancestry feat, perception mastery, revitalizing rage, skill increase |
| 18 | Barbarian feat, skill feat |
| 19 | Armor mastery, devastator, general feat, skill increase |
| 20 | Attribute boosts, barbarian feat, skill feat |

## Class features (procedure reference)

Summaries below are implementation-shaped paraphrases for engine work. Cite the book section when verifying; do not paste Paizo text into code or fixtures.

### Rage (Level 1)

- **When gained:** Level 1
- **Action economy:** 1 action
- **Procedure (engine):** Enter raging state: temporary HP, bonus damage, AC penalty, concentrate limits; ends on encounter end, unconscious, or stop; then fatigued. Instinct may alter resistances.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Quick-Tempered (Level 1)

- **When gained:** Level 1
- **Action economy:** class feature / choice
- **Procedure (engine):** You gain the Quick-Tempered free action, which lets you Rage at the slightest provocation. Quick-Tempered [free-action] Barbarian Source Player Core 2 pg. 72 Trigger You roll initiative. Requirements You are not encumbered or wearing heavy armor. So long as you are able to move freely, your fury is instinctive and instantaneous. You Rage .
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Instinct (Level 1)

- **When gained:** Level 1
- **Action economy:** class feature / choice
- **Procedure (engine):** Your rage wells up from a dominant instinct—one you learned from a tradition or that comes naturally to you. Your instinct gives you an ability, requires you to avoid certain behaviors, grants you increased damage and resistances at higher levels, and allows you to select feats tied to your instinct. Instincts can be found here .
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Barbarian Feats (cadence)

- **Procedure:** At 1st level and every even-numbered level thereafter, you gain a barbarian class feat .
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Skill Feats (Level 2)

- **Procedure:** At 2nd level and every 2 levels thereafter, you gain a skill feat. Skill feats have the skill trait. You must be trained or better in the corresponding skill to select a skill feat.
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Furious Footfalls (Level 3)

- **When gained:** Level 3
- **Action economy:** class feature / choice
- **Procedure (engine):** The urge to fight drives you ever forward. You gain a +5-foot status bonus to your Speed. This bonus increases to +10 feet while you’re raging .
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### General Feats (Level 3)

- **Procedure:** At 3rd level and every 4 levels thereafter, you gain a general feat.
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Skill Increases (Level 3)

- **Procedure:** At 3rd level and every 2 levels thereafter, you gain a skill increase. You can use this increase either to increase your proficiency rank to trained in one skill you're untrained in, or to increase your proficiency rank in one skill in which you're already trained to expert. At 7th level, you can use skill increases to increase your proficiency rank to master in a skill in which you're already an expert, and at…
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Ancestry Feats (Level 5)

- **Procedure:** In addition to the ancestry feat you started with, you gain an ancestry feat at 5th level and every 4 levels thereafter.
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Brutality (Level 5)

- **When gained:** Level 5
- **Action economy:** class feature / choice
- **Procedure (engine):** Your fury makes your weapons lethal. Your proficiency ranks for simple weapons, martial weapons, and unarmed attacks increase to expert. While raging , you have the critical specialization benefits for melee weapons and unarmed attacks.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Juggernaut (Level 7)

- **When gained:** Level 7
- **Action economy:** class feature / choice
- **Procedure (engine):** Your body is accustomed to physical hardship and resistant to ailments. Your proficiency rank for Fortitude saves increases to master. When you roll a success on a Fortitude save, you get a critical success instead.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Weapon Specialization (Level 7)

- **When gained:** Level 7
- **Action economy:** class feature / choice
- **Procedure (engine):** Your rage helps you hit harder. You deal an additional 2 damage with weapons and unarmed attacks in which you have expert proficiency. This damage increases to 3 if you’re a master, and 4 if you’re legendary. You gain your instinct’s specialization ability.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Raging Resistance (Level 9)

- **When gained:** Level 9
- **Action economy:** class feature / choice
- **Procedure (engine):** Repeated exposure and toughened skin allow you to fend off harm. While raging , you gain resistance equal to 3 + your Constitution modifier to damage types based on your instinct.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Reflex Expertise (Level 9)

- **When gained:** Level 9
- **Action economy:** class feature / choice
- **Procedure (engine):** You’ve developed a knack for dodging danger. Your proficiency rank for Reflex saves increases to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Mighty Rage (Level 11)

- **When gained:** Level 11
- **Action economy:** see procedure (often 1 action or free)
- **Procedure (engine):** Your rage intensifies and lets you hit harder at the start of a fight. Your proficiency rank for your barbarian class DC increases to expert. In addition, when you use Quick-Tempered , the first Strike you make during your first turn deals additional damage equal to your Rage damage.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Greater Juggernaut (Level 13)

- **When gained:** Level 13
- **Action economy:** class feature / choice
- **Procedure (engine):** You have a stalwart physiology. Your proficiency rank for Fortitude saves increases to legendary. When you roll a critical failure on a Fortitude save, you get a failure instead. When you roll a failure on a Fortitude save against an effect that deals damage, you halve the damage you take.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Medium Armor Expertise (Level 13)

- **When gained:** Level 13
- **Action economy:** class feature / choice
- **Procedure (engine):** You’ve learned to defend yourself better against attacks. Your proficiency ranks for light armor, medium armor, and unarmored defense increase to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Weapon Mastery (Level 13)

- **When gained:** Level 13
- **Action economy:** class feature / choice
- **Procedure (engine):** Your rage makes you even more effective with the weapons you wield. Your proficiency ranks for simple weapons, martial weapons, and unarmed attacks increase to master.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Greater Weapon Specialization (Level 15)

- **When gained:** Level 15
- **Action economy:** class feature / choice
- **Procedure (engine):** The weapons you’ve mastered become truly fearsome in your hands. Your damage from weapon specialization increases to 4 with weapons and unarmed attacks in which you’re an expert, 6 if you’re a master, and 8 if you’re legendary. You gain a greater benefit from your instinct’s specialization ability.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Indomitable Will (Level 15)

- **When gained:** Level 15
- **Action economy:** class feature / choice
- **Procedure (engine):** Your rage makes it difficult to control you. Your proficiency rank for Will saves increases to master. When you roll a success on a Will save, you get a critical success instead.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Perception Mastery (Level 17)

- **When gained:** Level 17
- **Action economy:** class feature / choice
- **Procedure (engine):** Your instinct heightens each of your senses further. Your proficiency rank for Perception increases to master.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Revitalizing Rage (Level 17)

- **When gained:** Level 17
- **Action economy:** see procedure (often 1 action or free)
- **Procedure (engine):** Your rage sustains you more frequently, leaving you ready to jump into battle anew. Instead of needing to wait 1 minute before you can gain temporary Hit Points from Rage again, you need only to spend at least one full turn not in a Rage.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Armor Mastery (Level 19)

- **When gained:** Level 19
- **Action economy:** class feature / choice
- **Procedure (engine):** Your training and rage deepen your connection to your armor. Your proficiency ranks for light armor, medium armor, and unarmored defense increase to master.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Devastator (Level 19)

- **When gained:** Level 19
- **Action economy:** class feature / choice
- **Procedure (engine):** Your Strikes are so devastating that you hardly care about resistance, and your barbarian abilities are unparalleled. Your proficiency rank for your barbarian class DC increases to master. Your melee Strikes ignore up to 10 resistance to the physical damage they deal. Site Owner: Rose-Winds LLC (Blake Davis) (adsbygoogle = window.adsbygoogle || []).push({}); Character Creation + Ancestries Archetypes Backgrounds…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.


## Focus spells / inseparable kits

- No focus/spell kit required for the class's martial L1 slice beyond listed features.

## Store vs derive

- **Store (character / preparation):** class id, level, key attribute choice, ancestry HP contribution, selected skills, feat picks (class/skill/general/ancestry), class choice subtree (racket/muse/order/doctrine/school/thesis/patron/bloodline/mystery/instinct/cause/style/methodology/research field), trained weapon/armor ranks overrides from features.
- **Store (encounter):** HP, conditions, focus points remaining, spell slots remaining / repertoire prepared-for-day, raised shield / stance / rage / panache / hunted prey id, reactions available.
- **Derive at prepare:** proficiency bonuses (level + rank), AC from armor + Dex + bonuses, Strike attack/damage from weapon + attr + proficiency + specialization, Perception/saves/class DC/spell DC, spell attack, MAP rules, off-guard sources.
- **Derive per check:** circumstance/status/item bonus nets by type, degree of success, precision applicability.
- **Rage store:** raging bool, rage temp HP pool, instinct id; fatigue applied when rage ends.

## Dependencies (substrate not yet in engine or only partially present)

Baseline on `main` (~`2f56abb`) already runs Stride/Step/Strike, Reactive Strike, dying, shields, prone movement, cover/detection, manipulate, Delay/Ready, Aid (attack-bonus substitute). This class still needs:

- `architecture.md#feat-slots`
- `architecture.md#proficiency`
- `conditions-and-effects.md#armor-and-item-bonuses`
- `conditions-and-effects.md#rage`

- Skill list + proficiency bonuses on checks (Aid currently substitutes Strike bonus; ledger entry).
- Feat slot model + at least data-driven class feats used in the first slice.
- Armor/weapon itemization beyond fixture-prebaked AC and one Strike (unless the slice keeps baked numbers with a Principle III note).
- Conditions listed under [conditions-and-effects.md](../conditions-and-effects.md) that this class applies or keys off.

## Suggested first slice (room-one style)

Level 1: Rage as one action (temp HP, bonus damage, AC penalty, fatigue end); one instinct with its anathema stubbed. Needs Rage condition + temporary HP.

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
