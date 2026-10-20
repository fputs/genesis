# Ranger

**Source:** Player Core pg. 152 (Player Core). Remaster naming. Archives of Nethys class id `36`.

**Role in Genesis:** class content pack entry consumed by phased preparation in `crates/rules` (see [architecture.md](../architecture.md)). Fixtures remain hand-authored until the editor authors the same schemas.

## Identity

| Field | Value |
| --- | --- |
| Key attribute | Strength or Dexterity |
| HP per level | 10 plus your Constitution modifier (plus ancestry HP at level 1 total) |
| Book | Player Core |

## Initial proficiencies (level 1)

- At 1st level, you gain the listed proficiency ranks in the following statistics. You are untrained in anything not listed unless you gain a better proficiency rank in some other way.
- **Perception:** Expert in Perception
- **Saving Throws:** Expert in Fortitude
- Expert in Reflex
- Trained in Will
- **Skills:** Trained in Nature
- Trained in Survival
- Trained in a number of additional skills equal to 4 plus your Intelligence modifier
- **Attacks:** Trained in simple weapons
- Trained in martial weapons
- Trained in unarmed attacks
- **Defenses:** Trained in light armor
- Trained in medium armor
- Trained in unarmored defense
- **Class DC:** Trained in ranger class DC

### Proficiency progression (engine)

Store rank enums (`Untrained/Trained/Expert/Master/Legendary`) per statistic. Class features that say “your proficiency rank for X increases to Y” are rule elements applied in preparation phases—not ad hoc branches in `encounter.rs`. Attack/defense/Perception/save/class-DC/spell ranks listed above are the L1 baseline; later rows in the advancement table upgrade them.

## Advancement table

| Level | Features gained |
| --- | --- |
| 1 | Ancestry and background, attribute boosts, initial proficiencies, hunt prey, hunter’s edge, ranger feat |
| 2 | Ranger feat, skill feat |
| 3 | General feat, will expertise, skill increase |
| 4 | Ranger feat, skill feat |
| 5 | Attribute boosts, ancestry feat, ranger weapon expertise, skill increase, trackless journey |
| 6 | Ranger feat, skill feat |
| 7 | General feat, natural reflexes, skill increase, perception mastery, weapon specialization |
| 8 | Ranger feat, skill feat |
| 9 | Ancestry feat, nature’s edge, ranger expertise, skill increase |
| 10 | Attribute boosts, ranger feat, skill feat |
| 11 | General feat, warden’s endurance, medium armor expertise, skill increase, unimpeded journey |
| 12 | Ranger feat, skill feat |
| 13 | Ancestry feat, skill increase, martial weapon mastery |
| 14 | Ranger feat, skill feat |
| 15 | Attribute boosts, general feat, greater natural reflexes, greater weapon specialization, perception legend, skill increase |
| 16 | Ranger feat, skill feat |
| 17 | Ancestry feat, masterful hunter, skill increase |
| 18 | Ranger feat, skill feat |
| 19 | General feat, medium armor mastery, skill increase, swift prey |
| 20 | Attribute boosts, ranger feat, skill feat |

## Class features (procedure reference)

Summaries below are implementation-shaped paraphrases for engine work. Cite the book section when verifying; do not paste Paizo text into code or fixtures.

### Hunt Prey (Level 1)

- **When gained:** Level 1
- **Action economy:** 1 action
- **Procedure (engine):** Designate one creature as prey (replaces prior). Store prey id. While set, apply Remaster Hunt Prey bonuses (Seek/Track vs prey; ignore second-range-increment penalty on ranged Strikes vs prey).
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Hunter's Edge (Level 1)

- **When gained:** Level 1
- **Action economy:** see procedure (often 1 action or free)
- **Procedure (engine):** You have trained to become a skilled hunter and tracker, gaining an extra benefit when you Hunt Prey depending on the focus of your training. Choose a hunter's edge.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Ranger Feats (cadence)

- **Procedure:** At 1st level and every even-numbered level, you gain a ranger class feat .
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Skill Feats (Level 2)

- **Procedure:** At 2nd level and every 2 levels thereafter, you gain a skill feat . You must be trained or better in the corresponding skill to select a skill feat.
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### General Feats (Level 3)

- **Procedure:** At 3rd level and every 4 levels thereafter, you gain a general feat .
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Skill Increases (Level 3)

- **Procedure:** At 3rd level and every 2 levels thereafter, you gain a skill increase. You can use this increase to either become trained in one skill you're untrained in or become an expert in one skill in which you're already trained. At 7th level, you can use skill increases to become a master in a skill in which you're already an expert, and at 15th level, you can use them to become legendary in a skill in which you're…
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Will Expertise (Level 3)

- **When gained:** Level 3
- **Action economy:** class feature / choice
- **Procedure (engine):** Your mental defenses grow stronger. Your proficiency rank for Will saves increases to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Ancestry Feats (Level 5)

- **Procedure:** In addition to the ancestry feat you started with, you gain an ancestry feat at 5th level and every 4 levels thereafter. The list of ancestry feats available to you can be found in your ancestry's entry.
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Ranger Weapon Expertise (Level 5)

- **When gained:** Level 5
- **Action economy:** class feature / choice
- **Procedure (engine):** You gain every advantage from your weapons. Your proficiency rank for martial weapons, simple weapons, and unarmed attacks increases to expert. You gain access to the critical specialization effects of all such weapons and unarmed attacks when attacking your hunted prey.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Trackless Journey (Level 5)

- **When gained:** Level 5
- **Action economy:** class feature / choice
- **Procedure (engine):** When you move through natural terrains, you are hard to track. You always gain the benefits of the Cover Tracks action in such terrains, without moving at half your Speed.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Natural Reflexes (Level 7)

- **When gained:** Level 7
- **Action economy:** class feature / choice
- **Procedure (engine):** You've honed your reflexes dodging falling trees and pounding hail. Your proficiency rank for Reflex saves increases to master. When you roll a success on a Reflex save, you get a critical success instead.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Perception Mastery (Level 7)

- **When gained:** Level 7
- **Action economy:** class feature / choice
- **Procedure (engine):** You have highly developed acuity and attention to detail. Your proficiency rank for Perception increases to master.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Weapon Specialization (Level 7)

- **When gained:** Level 7
- **Action economy:** class feature / choice
- **Procedure (engine):** You inflict greater injuries with the weapons you know best. You deal 2 additional damage with weapons and unarmed attacks in which you're an expert. This damage increases to 3 if you're a master, and 4 if you're legendary.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Nature's Edge (Level 9)

- **When gained:** Level 9
- **Action economy:** class feature / choice
- **Procedure (engine):** You always find the weak points in your foes' defenses when they're on unfavorable terrain. Enemies are off-guard to you if they're in difficult terrain .
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Ranger Expertise (Level 9)

- **When gained:** Level 9
- **Action economy:** class feature / choice
- **Procedure (engine):** You've gained techniques that improve your cunning and skill. Your proficiency rank for your ranger DC increases to expert. If you have warden spells, your proficiency rank for spell attacks and spell DCs increases to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Warden's Endurance (Level 11)

- **When gained:** Level 11
- **Action economy:** class feature / choice
- **Procedure (engine):** You shake off a grizzly's grab or a poison's ill effects with equal ease. Your proficiency rank for Fortitude saves increases to master. When you roll a success on a Fortitude save, you get a critical success instead.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Medium Armor Expertise (Level 11)

- **When gained:** Level 11
- **Action economy:** class feature / choice
- **Procedure (engine):** You've learned to defend yourself better against attacks. Your proficiency ranks for light armor, medium armor, and unarmored defense increase to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Unimpeded Journey (Level 11)

- **When gained:** Level 11
- **Action economy:** class feature / choice
- **Procedure (engine):** You move quickly through obstacles, whether they're tumbled stone or tangled undergrowth. You can ignore the effects of difficult terrain . As normal for ignoring difficult terrain, this also lets you treat the hindrances of greater difficult terrain as those of difficult terrain.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Martial Weapon Mastery (Level 13)

- **When gained:** Level 13
- **Action economy:** class feature / choice
- **Procedure (engine):** You fully understand the intricacies of weapons. Your proficiency ranks for unarmed attacks, simple weapons, and martial weapons increase to master.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Greater Natural Reflexes (Level 15)

- **When gained:** Level 15
- **Action economy:** class feature / choice
- **Procedure (engine):** You avoid explosions with reflexes a cat would envy. Your proficiency rank for Reflex saves increases to legendary. When you roll a critical failure on a Reflex save, you get a failure instead. When you fail a Reflex save against an effect that deals damage, you halve the damage you take.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Greater Weapon Specialization (Level 15)

- **When gained:** Level 15
- **Action economy:** class feature / choice
- **Procedure (engine):** Your damage from weapon specialization increases to 4 with weapons and unarmed attacks in which you're an expert, 6 if you're a master, and 8 if you're legendary.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Perception Legend (Level 15)

- **When gained:** Level 15
- **Action economy:** class feature / choice
- **Procedure (engine):** You notice the almost imperceptible. Your proficiency rank for Perception increases to legendary.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Masterful Hunter (Level 17)

- **When gained:** Level 17
- **Action economy:** see procedure (often 1 action or free)
- **Procedure (engine):** You have honed your abilities as a hunter to incredible levels. Your proficiency rank for your ranger class DC increases to master. If you have warden spells, your proficiency rank for spell attacks and spell DCs increases to master. When using a ranged weapon that you have master proficiency in, you can ignore the penalty if attacking your hunted prey within the weapon's second and third range increments. If you…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Medium Armor Mastery (Level 19)

- **When gained:** Level 19
- **Action economy:** class feature / choice
- **Procedure (engine):** Your skill with light and medium armor improves, increasing your ability to avoid blows. Your proficiency ranks for light and medium armor, as well as for unarmored defense, increase to master.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Swift Prey (Level 19)

- **When gained:** Level 19
- **Action economy:** class feature / choice
- **Procedure (engine):** You size up your prey with a glance. You can use Hunt Prey as a free action if it's your first action of your turn. Site Owner: Rose-Winds LLC (Blake Davis) (adsbygoogle = window.adsbygoogle || []).push({}); Character Creation + Ancestries Archetypes Backgrounds Classes Skills Companions Familiars Equipment + All Equipment Adventuring Gear Alchemical Items Armor Held Items Runes Shields Weapons Worn Items Feats +…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.


## Focus spells / inseparable kits

- **Focus:** none at L1 by default; later feats/focus spells optional. Animal companion is a separate primitive.

## Store vs derive

- **Store (character / preparation):** class id, level, key attribute choice, ancestry HP contribution, selected skills, feat picks (class/skill/general/ancestry), class choice subtree (racket/muse/order/doctrine/school/thesis/patron/bloodline/mystery/instinct/cause/style/methodology/research field), trained weapon/armor ranks overrides from features.
- **Store (encounter):** HP, conditions, focus points remaining, spell slots remaining / repertoire prepared-for-day, raised shield / stance / rage / panache / hunted prey id, reactions available.
- **Derive at prepare:** proficiency bonuses (level + rank), AC from armor + Dex + bonuses, Strike attack/damage from weapon + attr + proficiency + specialization, Perception/saves/class DC/spell DC, spell attack, MAP rules, off-guard sources.
- **Derive per check:** circumstance/status/item bonus nets by type, degree of success, precision applicability.
- **Hunt Prey store:** current prey `CombatantId` (or none); hunter's edge enum; edge benefits applied as synthetics while prey set.

## Dependencies (substrate not yet in engine or only partially present)

Baseline on `main` (~`2f56abb`) already runs Stride/Step/Strike, Reactive Strike, dying, shields, prone movement, cover/detection, manipulate, Delay/Ready, Aid (attack-bonus substitute). This class still needs:

- `architecture.md#feat-slots`
- `architecture.md#proficiency`
- `conditions-and-effects.md#armor-and-item-bonuses`
- `conditions-and-effects.md#designated-targets`

- Skill list + proficiency bonuses on checks (Aid currently substitutes Strike bonus; ledger entry).
- Feat slot model + at least data-driven class feats used in the first slice.
- Armor/weapon itemization beyond fixture-prebaked AC and one Strike (unless the slice keeps baked numbers with a Principle III note).
- Conditions listed under [conditions-and-effects.md](../conditions-and-effects.md) that this class applies or keys off.

## Suggested first slice (room-one style)

Level 1: Hunt Prey as one action designating a target; Hunter's Edge Precision (+1d8 first hit/round on prey). Needs designated-target state; no companion yet.

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
