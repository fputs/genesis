# Swashbuckler

**Source:** Player Core 2 pg. 158 (Player Core 2). Remaster naming. Archives of Nethys class id `63`.

**Role in Genesis:** class content pack entry consumed by phased preparation in `crates/rules` (see [architecture.md](../architecture.md)). Fixtures remain hand-authored until the editor authors the same schemas.

## Identity

| Field | Value |
| --- | --- |
| Key attribute | Dexterity |
| HP per level | 10 plus your Constitution modifier (plus ancestry HP at level 1 total) |
| Book | Player Core 2 |

## Initial proficiencies (level 1)

- At 1st level, you gain the listed proficiency ranks in the following statistics. You are untrained in anything not listed unless you gain a better proficiency rank in some other way.
- **Perception:** Expert in Perception
- **Saving Throws:** Trained in Fortitude
- Expert in Reflex
- Expert in Will
- **Skills:** Trained in Acrobatics
- Trained in one skill determined by your swashbuckler's style
- Trained in a number of additional skills equal to 4 plus your Intelligence modifier
- **Attacks:** Trained in simple weapons
- Trained in martial weapons
- Trained in unarmed attacks
- **Defenses:** Trained in light armor
- Trained in unarmored defense
- **Class DC:** Trained in swashbuckler class DC

### Proficiency progression (engine)

Store rank enums (`Untrained/Trained/Expert/Master/Legendary`) per statistic. Class features that say “your proficiency rank for X increases to Y” are rule elements applied in preparation phases—not ad hoc branches in `encounter.rs`. Attack/defense/Perception/save/class-DC/spell ranks listed above are the L1 baseline; later rows in the advancement table upgrade them.

## Advancement table

| Level | Features gained |
| --- | --- |
| 1 | Ancestry and background, attribute boosts, initial proficiencies, panache, precise strike, stylish combatant, swashbuckler’s style, confident finisher, swashbuckler feat |
| 2 | Skill feat, swashbuckler feat |
| 3 | Fortitude expertise, general feat, opportune riposte, skill increase, stylish tricks, vivacious speed |
| 4 | Skill feat, swashbuckler feat |
| 5 | Attribute boosts, ancestry feat, precise strike 3 (3d6), skill increase, weapon expertise |
| 6 | Skill feat, swashbuckler feat |
| 7 | Confident evasion, general feat, skill increase, stylish tricks, vivacious speed, weapon specialization |
| 8 | Skill feat, swashbuckler feat |
| 9 | Ancestry feat, exemplary finisher, precise strike 4 (4d6), skill increase, swashbuckler expertise |
| 10 | Attribute boosts, skill feat, swashbuckler feat |
| 11 | Continuous flair, general feat, perception mastery, skill increase, vivacious speed |
| 12 | Skill feat, swashbuckler feat |
| 13 | Ancestry feat, assured evasion, light armor expertise, precise strike 5 (5d6), skill increase, weapon mastery |
| 14 | Skill feat, swashbuckler feat |
| 15 | Attribute boosts, general feat, greater weapon specialization, keen flair, skill increase, stylish tricks, vivacious speed |
| 16 | Skill feat, swashbuckler feat |
| 17 | Ancestry feat, precise strike 6 (6d6), reinforced ego, skill increase |
| 18 | Skill feat, swashbuckler feat |
| 19 | Eternal confidence, general feat, light armor mastery, skill increase, vivacious speed |
| 20 | Attribute boosts, skill feat, swashbuckler feat |

## Class features (procedure reference)

Summaries below are implementation-shaped paraphrases for engine work. Cite the book section when verifying; do not paste Paizo text into code or fixtures.

### Panache (Level 1)

- **When gained:** Level 1
- **Action economy:** class feature / choice
- **Procedure (engine):** You care as much about the way you accomplish something as whether you actually accomplish it in the first place. When you perform an action with particular style, you can leverage this moment of verve to perform spectacular, deadly maneuvers. This heightened state of flair is called panache. You gain panache by performing actions that have the bravado trait. Tumble Through and additional actions determined by…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Precise Strike (Level 1)

- **When gained:** Level 1
- **Action economy:** see procedure (often 1 action or free)
- **Procedure (engine):** When you make a Strike with an agile or finesse melee weapon or agile or finesse unarmed attack, you deal 2 additional precision damage. If the Strike is part of a finisher, the additional damage is 2d6 precision damage instead. As your swashbuckler level increases, so does your additional damage for precise strike. At 5th, 9th, 13th, and 17th level, increase the amount of additional damage on a Strike by 1 and…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Stylish Combatant (Level 1)

- **When gained:** Level 1
- **Action economy:** class feature / choice
- **Procedure (engine):** You gain a +1 circumstance bonus to skill checks with the bravado trait while in a combat encounter. While you have panache, you gain a +5-foot status bonus to your Speeds.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Swashbuckler’s Style (Level 1)

- **When gained:** Level 1
- **Action economy:** class feature / choice
- **Procedure (engine):** Your own distinctive style lets you gracefully handle any situation. Choose a swashbuckler’s style. This style determines the additional actions you use to gain panache and makes you trained in the skill related to that action. A list of all swashbuckler's styles can be found here .
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Confident Finisher (Level 1)

- **When gained:** Level 1
- **Action economy:** see procedure (often 1 action or free)
- **Procedure (engine):** You gain an elegant finishing attack you can make when you have panache. You gain the Confident Finisher action. Confident Finisher [one-action] Finisher Swashbuckler Source Player Core 2 pg. 161 You make an incredibly graceful attack, piercing your foe’s defenses. Make a Strike with the following failure effect. Failure You deal half your precise strike damage to the target. This damage type is that of the…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Swashbuckler Feats (cadence)

- **Procedure:** At 1st level and every even-numbered level, you gain a swashbuckler class feat .
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Skill Feats (Level 2)

- **Procedure:** At 2nd level and every 2 levels thereafter, you gain a skill feat. You must be trained or better in the corresponding skill to select a skill feat.
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Fortitude Expertise (Level 3)

- **When gained:** Level 3
- **Action economy:** class feature / choice
- **Procedure (engine):** Your physique has grown hardy in your adventures. Your proficiency rank for Fortitude saves increases to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### General Feats (Level 3)

- **Procedure:** At 3rd level and every 4 levels thereafter, you gain a general feat.
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Opportune Riposte (Level 3)

- **When gained:** Level 3
- **Action economy:** reaction
- **Procedure (engine):** You turn the tables on foes who fumble, capitalizing on their mistake. You gain the Opportune Riposte reaction. Opportune Riposte [reaction] Bravado Swashbuckler Source Player Core 2 pg. 161 Trigger An enemy within your reach critically fails on a Strike against you. You take advantage of an opening from your enemy's fumbled attack. You either make a melee Strike against the triggering enemy or attempt to Disarm…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Skill Increases (Level 3)

- **Procedure:** At 3rd level and every 2 levels thereafter, you gain a skill increase. You can use this either to increase your proficiency rank to trained in one skill you're untrained in, or to increase your proficiency rank in one skill in which you're already trained to expert. At 7th level, you can use skill increases to increase your proficiency rank to master in a skill in which you're already an expert, and at 15th…
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Stylish Tricks (Level 3)

- **When gained:** Level 3
- **Action economy:** class feature / choice
- **Procedure (engine):** At 3rd level, 7th level, and 15th level, you gain an additional skill increase you can apply only to Acrobatics or the skill from your swashbuckler’s style. You also gain an additional skill feat at these levels. This feat must be for Acrobatics or the trained skill from your swashbuckler’s style.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Vivacious Speed (Level 3)

- **When gained:** Level 3
- **Action economy:** class feature / choice
- **Procedure (engine):** When you’ve made an impression, you move even faster than normal, darting about the battlefield with incredible speed. Increase the status bonus to your Speeds from stylish combatant to a +10-foot status bonus; this bonus increases by 5 feet at 7th, 11th, 15th, and 19th levels. When you don’t have panache, you still get half this status bonus to your Speed, rounded down to the nearest 5-foot increment.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Ancestry Feat (Level 5)

- **When gained:** Level 5
- **Action economy:** class feature / choice
- **Procedure (engine):** In addition to the ancestry feat you started with, you gain an ancestry feat at 5th level and every 4 levels thereafter.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Weapon Expertise (Level 5)

- **When gained:** Level 5
- **Action economy:** class feature / choice
- **Procedure (engine):** You’ve dedicated yourself to learning the intricacies of your weapons. Your proficiency ranks for simple weapons, martial weapons, and unarmed attacks increase to expert. You gain access to the critical specialization effects of all weapons for which you have expert proficiency.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Confident Evasion (Level 7)

- **When gained:** Level 7
- **Action economy:** class feature / choice
- **Procedure (engine):** You’ve learned to move quickly to avoid explosions, a dragon’s breath, and worse. Your proficiency rank for Reflex saves increases to master. When you roll a success on a Reflex save, you get a critical success instead.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Weapon Specialization (Level 7)

- **When gained:** Level 7
- **Action economy:** class feature / choice
- **Procedure (engine):** You’ve learned how to inflict greater injuries with the weapons you know best. You deal 2 additional damage with weapons and unarmed attacks in which you are an expert. This damage increases to 3 if you’re a master and 4 if you’re legendary.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Exemplary Finisher (Level 9)

- **When gained:** Level 9
- **Action economy:** see procedure (often 1 action or free)
- **Procedure (engine):** You execute your finishing moves with spectacular flair, adding special effects to your finishers. If a Strike you make as part of a finisher hits a foe, you add one of the following effects to the Strike, depending on your swashbuckler's style Battledancer: You can Step as a free action immediately after the finisher. Braggart: If the foe was temporarily immune to your Demoralize, their temporary immunity ends.…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Swashbuckler Expertise (Level 9)

- **When gained:** Level 9
- **Action economy:** class feature / choice
- **Procedure (engine):** You perform swashbuckling techniques with exceptional flair, making them harder to resist. Your circumstance bonus from Stylish Combatant increases to +2. Your proficiency rank for your swashbuckler class DC increases to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Continuous Flair (Level 11)

- **When gained:** Level 11
- **Action economy:** class feature / choice
- **Procedure (engine):** While not equal to your panache in combat, you have a dramatic flair about you in any situation. The circumstance bonus from Stylish Combatant applies in exploration mode.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Perception Mastery (Level 11)

- **When gained:** Level 11
- **Action economy:** class feature / choice
- **Procedure (engine):** Through your adventures, you’ve developed keen awareness and attention to detail. Your proficiency rank for Perception increases to master.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Assured Evasion (Level 13)

- **When gained:** Level 13
- **Action economy:** class feature / choice
- **Procedure (engine):** Your ability to elude danger is matchless. Your proficiency rank for Reflex saves increases to legendary. When you roll a critical failure on a Reflex save, you get a failure instead. When you roll a failure on a Reflex save against a damaging effect, you take half damage.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Light Armor Expertise (Level 13)

- **When gained:** Level 13
- **Action economy:** class feature / choice
- **Procedure (engine):** You’ve learned to dodge while wearing light or no armor. Your proficiency ranks for light armor and unarmored defense increase to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Weapon Mastery (Level 13)

- **When gained:** Level 13
- **Action economy:** class feature / choice
- **Procedure (engine):** You fully understand your weapons. Your proficiency ranks for simple and martial weapons and unarmed attacks increase to master.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Greater Weapon Specialization (Level 15)

- **When gained:** Level 15
- **Action economy:** class feature / choice
- **Procedure (engine):** Your damage from weapon specialization increases to 4 with weapons and unarmed attacks in which you’re an expert, 6 if you’re a master, and 8 if you’re legendary.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Keen Flair (Level 15)

- **When gained:** Level 15
- **Action economy:** class feature / choice
- **Procedure (engine):** You inflict devastating attacks on even well-defended foes. When you Strike with a weapon or unarmed attack with which you have master proficiency, if you roll a 19 on the die and the roll is a success, you critically succeed instead.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Reinforced Ego (Level 17)

- **When gained:** Level 17
- **Action economy:** class feature / choice
- **Procedure (engine):** You’ve steeled your mind with resolve. Your proficiency rank for Will saves increases to master. When you roll a success on a Will save, you get a critical success instead.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Eternal Confidence (Level 19)

- **When gained:** Level 19
- **Action economy:** class feature / choice
- **Procedure (engine):** As a swashbuckler at the peak of your skill, you swell with confidence in every attack. Your proficiency rank for your swashbuckler class DC increases to master. When you Strike as part of a finisher or Opportune Riposte, you can give the Strike the failure effect from the Confident Finisher action, including the increase from Precise Finisher if you have that feat. You can do so only if the Strike uses a weapon…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Light Armor Mastery (Level 19)

- **When gained:** Level 19
- **Action economy:** class feature / choice
- **Procedure (engine):** Your skill with light armor improves, increasing your ability to dodge blows. Your proficiency ranks for light armor and unarmored defense increase to master. Site Owner: Rose-Winds LLC (Blake Davis) (adsbygoogle = window.adsbygoogle || []).push({}); Character Creation + Ancestries Archetypes Backgrounds Classes Skills Companions Familiars Equipment + All Equipment Adventuring Gear Alchemical Items Armor Held…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.


## Focus spells / inseparable kits

- No focus/spell kit required for the class's martial L1 slice beyond listed features.

## Store vs derive

- **Store (character / preparation):** class id, level, key attribute choice, ancestry HP contribution, selected skills, feat picks (class/skill/general/ancestry), class choice subtree (racket/muse/order/doctrine/school/thesis/patron/bloodline/mystery/instinct/cause/style/methodology/research field), trained weapon/armor ranks overrides from features.
- **Store (encounter):** HP, conditions, focus points remaining, spell slots remaining / repertoire prepared-for-day, raised shield / stance / rage / panache / hunted prey id, reactions available.
- **Derive at prepare:** proficiency bonuses (level + rank), AC from armor + Dex + bonuses, Strike attack/damage from weapon + attr + proficiency + specialization, Perception/saves/class DC/spell DC, spell attack, MAP rules, off-guard sources.
- **Derive per check:** circumstance/status/item bonus nets by type, degree of success, precision applicability.
- **Panache store:** has_panache bool; style id; precise strike dice; finisher availability derived from panache.

## Dependencies (substrate not yet in engine or only partially present)

Baseline on `main` (~`2f56abb`) already runs Stride/Step/Strike, Reactive Strike, dying, shields, prone movement, cover/detection, manipulate, Delay/Ready, Aid (attack-bonus substitute). This class still needs:

- `architecture.md#feat-slots`
- `architecture.md#proficiency`
- `conditions-and-effects.md#armor-and-item-bonuses`
- `conditions-and-effects.md#panache`

- Skill list + proficiency bonuses on checks (Aid currently substitutes Strike bonus; ledger entry).
- Feat slot model + at least data-driven class feats used in the first slice.
- Armor/weapon itemization beyond fixture-prebaked AC and one Strike (unless the slice keeps baked numbers with a Principle III note).
- Conditions listed under [conditions-and-effects.md](../conditions-and-effects.md) that this class applies or keys off.

## Suggested first slice (room-one style)

Level 1: panache gain/spend, Precise Strike, Confident Finisher. Needs panache state + finisher action trait.

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
