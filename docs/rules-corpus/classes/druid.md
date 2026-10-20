# Druid

**Source:** Player Core pg. 122 (Player Core). Remaster naming. Archives of Nethys class id `34`.

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
- **Skills:** Trained in Nature
- Trained in one skill determined by your druidic order
- Trained in a number of additional skills equal to 2 plus your Intelligence modifier
- **Attacks:** Trained in simple weapons
- Trained in unarmed attacks
- **Defenses:** Trained in light armor
- Trained in medium armor
- Trained in unarmored defense
- **Class DC:** Trained in druid class DC
- **Spells:** Trained in spell attack modifier
- Trained in spell DC

### Proficiency progression (engine)

Store rank enums (`Untrained/Trained/Expert/Master/Legendary`) per statistic. Class features that say “your proficiency rank for X increases to Y” are rule elements applied in preparation phases—not ad hoc branches in `encounter.rs`. Attack/defense/Perception/save/class-DC/spell ranks listed above are the L1 baseline; later rows in the advancement table upgrade them.

## Advancement table

| Level | Features gained |
| --- | --- |
| 1 | Ancestry and background, attribute boosts, initial proficiencies, druid spellcasting, anathema, druidic order, Shield Block, voice of nature, Wildsong |
| 2 | Druid feat, skill feat |
| 3 | 2nd-rank spells, fortitude expertise, general feat, perception expertise, skill increase |
| 4 | Druid feat, skill feat |
| 5 | 3rd-rank spells, attribute boosts, ancestry feat, reflex expertise, skill increase |
| 6 | Druid feat, skill feat |
| 7 | 4th-rank spells, expert spellcaster, general feat, skill increase |
| 8 | Druid feat, skill feat |
| 9 | 5th-rank spells, ancestry feat, skill increase |
| 10 | Attribute boosts, druid feat, skill feat |
| 11 | 6th-rank spells, general feat, skill increase, weapon expertise, wild willpower |
| 12 | Druid feat, skill feat |
| 13 | 7th-rank spells, ancestry feat, medium armor expertise, skill increase, weapon specialization |
| 14 | Druid feat, skill feat |
| 15 | 8th-rank spells, attribute boosts, general feat, master spellcaster, skill increase |
| 16 | Druid feat, skill feat |
| 17 | 9th-rank spells, ancestry feat, skill increase |
| 18 | Druid feat, skill feat |
| 19 | General feat, legendary spellcaster, primal hierophant, skill increase |
| 20 | Attribute boosts, druid feat, skill feat |

## Class features (procedure reference)

Summaries below are implementation-shaped paraphrases for engine work. Cite the book section when verifying; do not paste Paizo text into code or fixtures.

### Druid Spellcasting (Level 1)

- **When gained:** Level 1
- **Action economy:** class feature / choice
- **Procedure (engine):** The power of the wild world flows through you. You are a spellcaster and can cast spells of the primal tradition using the Cast a Spell activity (see Casting Spells ). As a druid, your spellcasting incantations might be pleas to the environment around you or the invocation of ancient vows; your hands might sway like willows or curl into clawlike shapes as your gestures direct your magic. At 1st level, you can…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Anathema (Level 1)

- **When gained:** Level 1
- **Action economy:** class feature / choice
- **Procedure (engine):** As stewards of the natural order, druids find affronts to nature anathema. If you repeatedly perform these acts, you lose your magical abilities from the druid class, including your druid spellcasting and the benefits of your order. These abilities can be regained only if you demonstrate your repentance with an atone ritual. The following acts are anathema to all druids. Anathema despoil natural places, consume…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Druidic Order (Level 1)

- **When gained:** Level 1
- **Action economy:** class feature / choice
- **Procedure (engine):** Upon becoming a druid, you align yourself with a druidic order, which grants you a class feat, an order spell (see below), and an additional trained skill tied to your order. While you'll always be a member of your initial order, it's not unheard of for a druid to request to study with other orders in search of greater understanding of the natural world, and PC druids are among the most likely to blend the powers…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Shield Block (Level 1)

- **When gained:** Level 1
- **Action economy:** class feature / choice
- **Procedure (engine):** You gain the Shield Block general feat, a reaction that lets you reduce damage with your shield.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Voice of Nature (Level 1)

- **When gained:** Level 1
- **Action economy:** class feature / choice
- **Procedure (engine):** You gain your choice of the Animal Empathy or Plant Empathy druid feat.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Wildsong (Level 1)

- **When gained:** Level 1
- **Action economy:** class feature / choice
- **Procedure (engine):** You know the Wildsong , a secret language known only within druid orders, in addition to any languages you know through your ancestry. The Wildsong is a melodic and tonal language sounding more like animal calls than a spoken tongue, and its alphabet involves fractals and spirals, like the arrangement of a seashell's chambers, a snowflake's crystals, or a fern's fronds. Teaching the Wildsong to non-druids is…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Druid Feats (Level 2)

- **Procedure:** At 2nd level and every even-numbered level, you gain a druid class feat .
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Skill Feats (Level 2)

- **Procedure:** At 2nd level and every 2 levels thereafter, you gain a skill feat . You must be trained or better in the corresponding skill to select a skill feat.
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Perception Expertise (Level 3)

- **When gained:** Level 3
- **Action economy:** class feature / choice
- **Procedure (engine):** You remain alert to threats around you. Your proficiency rank for Perception increases to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### General Feats (Level 3)

- **Procedure:** At 3rd level and every 4 levels thereafter, you gain a general feat .
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Fortitude Expertise (Level 3)

- **When gained:** Level 3
- **Action economy:** class feature / choice
- **Procedure (engine):** Adventures have made your physique more hardy. Your proficiency rank for Fortitude saves increases to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Skill Increases (Level 3)

- **Procedure:** At 3rd level and every 2 levels thereafter, you gain a skill increase. You can use this increase either to increase your proficiency rank to trained in one skill you're untrained in, or to increase your proficiency rank in one skill in which you're already trained to expert. At 7th level, you can use skill increases to increase your proficiency rank to master in a skill in which you're already an expert, and at…
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Ancestry Feats (Level 5)

- **Procedure:** In addition to the ancestry feat you started with, you gain an ancestry feat at 5th level and every 4 levels thereafter. The list of ancestry feats available to you can be found in your ancestry's entry.
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Reflex Expertise (Level 5)

- **When gained:** Level 5
- **Action economy:** class feature / choice
- **Procedure (engine):** You've honed your ability to dodge dangers. Your proficiency rank for Reflex saves increases to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Expert Spellcaster (Level 7)

- **When gained:** Level 7
- **Action economy:** class feature / choice
- **Procedure (engine):** Your command of primal forces has deepened, empowering your spells. Your proficiency ranks for spell attack modifier and spell DC increase to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Weapon Expertise (Level 11)

- **When gained:** Level 11
- **Action economy:** class feature / choice
- **Procedure (engine):** You've improved your combat skill. Your proficiency rank for simple weapons and unarmed attacks increases to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Wild Willpower (Level 11)

- **When gained:** Level 11
- **Action economy:** class feature / choice
- **Procedure (engine):** Your primal will, like that of the greatest beast, can't be tamed. Your proficiency rank for Will saves increases to master. When you roll a success at a Will save, you get a critical success instead.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Medium Armor Expertise (Level 13)

- **When gained:** Level 13
- **Action economy:** class feature / choice
- **Procedure (engine):** You've learned to defend yourself better against attacks. Your proficiency ranks for light armor, medium armor, and unarmored defense increase to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Weapon Specialization (Level 13)

- **When gained:** Level 13
- **Action economy:** class feature / choice
- **Procedure (engine):** You inflict great injuries with the weapons you know best. You deal 2 additional damage with weapons and unarmed attacks in which you are an expert. This damage increases to 3 if you're a master, and to 4 if you're legendary.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Master Spellcaster (Level 15)

- **When gained:** Level 15
- **Action economy:** class feature / choice
- **Procedure (engine):** Primal magic answers your command. Your proficiency ranks for spell attack modifier and spell DC increase to master.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Legendary Spellcaster (Level 19)

- **When gained:** Level 19
- **Action economy:** class feature / choice
- **Procedure (engine):** You have developed an unparalleled rapport with the magic of nature. Your proficiency ranks for spell attack modifier and spell DC increase to legendary.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Primal Hierophant (Level 19)

- **When gained:** Level 19
- **Action economy:** class feature / choice
- **Procedure (engine):** You command the most potent forces of primal magic and can cast a spell of truly incredible power. You gain a single 10th-rank spell slot and can prepare a spell in that slot using druid spellcasting. Unlike with other spell slots, you can't use 10-rank slots with abilities that give you more spell slots or that let you cast spells without expending spell slots. You don't gain more 10th-rank spells as you level…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.


## Focus spells / inseparable kits

- **Order spell:** focus spell from druidic order; anathema constraints are product/policy (see open-questions).

## Store vs derive

- **Store (character / preparation):** class id, level, key attribute choice, ancestry HP contribution, selected skills, feat picks (class/skill/general/ancestry), class choice subtree (racket/muse/order/doctrine/school/thesis/patron/bloodline/mystery/instinct/cause/style/methodology/research field), trained weapon/armor ranks overrides from features.
- **Store (encounter):** HP, conditions, focus points remaining, spell slots remaining / repertoire prepared-for-day, raised shield / stance / rage / panache / hunted prey id, reactions available.
- **Derive at prepare:** proficiency bonuses (level + rank), AC from armor + Dex + bonuses, Strike attack/damage from weapon + attr + proficiency + specialization, Perception/saves/class DC/spell DC, spell attack, MAP rules, off-guard sources.
- **Derive per check:** circumstance/status/item bonus nets by type, degree of success, precision applicability.
- **Spellcasting store:** tradition, casting style (prepared/spontaneous), spell list access, cantrips prepared or in repertoire, ranked slots or repertoire known, signature spells (spontaneous), focus spell ids, focus pool size.
- **Companion/familiar:** separate creature id linked to master when feature grants one; not a party slot.

## Dependencies (substrate not yet in engine or only partially present)

Baseline on `main` (~`2f56abb`) already runs Stride/Step/Strike, Reactive Strike, dying, shields, prone movement, cover/detection, manipulate, Delay/Ready, Aid (attack-bonus substitute). This class still needs:

- `architecture.md#class-choices`
- `architecture.md#feat-slots`
- `architecture.md#proficiency`
- `baseline Raise a Shield / Shield Block (already in rules)`
- `conditions-and-effects.md`
- `conditions-and-effects.md#armor-and-item-bonuses`
- `spells/framework.md`

- Skill list + proficiency bonuses on checks (Aid currently substitutes Strike bonus; ledger entry).
- Feat slot model + at least data-driven class feats used in the first slice.
- Armor/weapon itemization beyond fixture-prebaked AC and one Strike (unless the slice keeps baked numbers with a Principle III note).
- Conditions listed under [conditions-and-effects.md](../conditions-and-effects.md) that this class applies or keys off.

## Suggested first slice (room-one style)

Level 1 prepared primal: order (Animal or Storm recommended), Shield Block, Wildsong. Cast Electric Arc or Stabilize + Heal. Order spell as focus.

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
