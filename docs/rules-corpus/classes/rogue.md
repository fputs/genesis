# Rogue

**Source:** Player Core pg. 164 (Player Core). Remaster naming. Archives of Nethys class id `37`.

**Role in Genesis:** class content pack entry consumed by phased preparation in `crates/rules` (see [architecture.md](../architecture.md)). Fixtures remain hand-authored until the editor authors the same schemas.

## Identity

| Field | Value |
| --- | --- |
| Key attribute | Dexterity (or other attribute from racket; racket may replace key attribute) |
| HP per level | 8 plus your Constitution modifier (plus ancestry HP at level 1 total) |
| Book | Player Core |

## Initial proficiencies (level 1)

- At 1st level, you gain the listed proficiency ranks in the following statistics. You are untrained in anything not listed unless you gain a better proficiency rank in some other way.
- **Perception:** Expert in Perception
- **Saving Throws:** Trained in Fortitude
- Expert in Reflex
- Expert in Will
- **Skills:** Trained in Stealth
- Trained in one or more skills determined by your rogue's racket
- Trained in a number of additional skills equal to 7 plus your Intelligence modifier
- **Attacks:** Trained in simple weapons
- Trained in martial weapons
- Trained in unarmed attacks
- **Defenses:** Trained in light armor
- Trained in unarmored defense
- **Class DC:** Trained in rogue class DC

### Proficiency progression (engine)

Store rank enums (`Untrained/Trained/Expert/Master/Legendary`) per statistic. Class features that say “your proficiency rank for X increases to Y” are rule elements applied in preparation phases—not ad hoc branches in `encounter.rs`. Attack/defense/Perception/save/class-DC/spell ranks listed above are the L1 baseline; later rows in the advancement table upgrade them.

## Advancement table

| Level | Features gained |
| --- | --- |
| 1 | Ancestry and background, attribute boosts, initial proficiencies, rogue’s racket, sneak attack 1d6, surprise attack, rogue feat, skill feat |
| 2 | Rogue feat, skill feat, skill increase |
| 3 | Deny advantage, general feat, skill feat, skill increase |
| 4 | Rogue feat, skill feat, skill increase |
| 5 | Attribute boosts, ancestry feat, skill feat, skill increase, sneak attack 2d6, weapon tricks |
| 6 | Rogue feat, skill feat, skill increase |
| 7 | Evasive reflexes, general feat, skill feat, skill increase, perception mastery, weapon specialization |
| 8 | Rogue feat, skill feat, skill increase |
| 9 | Ancestry feat, debilitating strike, rogue resilience, skill feat, skill increase |
| 10 | Attribute boosts, rogue feat, skill feat, skill increase |
| 11 | General feat, rogue expertise, skill feat, skill increase, sneak attack 3d6 |
| 12 | Rogue feat, skill feat, skill increase |
| 13 | Ancestry feat, greater rogue reflexes, light armor expertise, master tricks, perception legend, skill feat, skill increase |
| 14 | Rogue feat, skill feat, skill increase |
| 15 | Attribute boosts, double debilitation, general feat, greater weapon specialization, skill feat, skill increase |
| 16 | Rogue feat, skill feat, skill increase |
| 17 | Ancestry feat, skill feat, skill increase, agile mind, sneak attack 4d6 |
| 18 | Rogue feat, skill feat, skill increase |
| 19 | General feat, light armor mastery, master strike, skill feat, skill increase |
| 20 | Attribute boosts, rogue feat, skill feat, skill increase |

## Class features (procedure reference)

Summaries below are implementation-shaped paraphrases for engine work. Cite the book section when verifying; do not paste Paizo text into code or fixtures.

### Rogue's Racket (Level 1)

- **When gained:** Level 1
- **Action economy:** class feature / choice
- **Procedure (engine):** You've begun to develop your techniques and approach to a job, while building your reputation in rogues' circles. Choose a rogue's racket .
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Sneak Attack (engine note)

Precision dice (start **1d6**) on a Strike that deals precision-eligible damage when the target is off-guard to the attacker (or another printed qualifier). Reuse the shared precision helper in [conditions-and-effects.md](../conditions-and-effects.md#precision-damage). Off-guard is already derived for prone / unconscious / flanked.

### Sneak Attack (Level 1)

- **When gained:** Level 1
- **Action economy:** passive on Strike
- **Procedure (engine):** On a Strike with an agile/finesse melee weapon or unarmed, or a ranged weapon/unarmed (thrown melee must also be agile/finesse), if the target is off-guard to you, add precision dice (1d6 at L1; advances with level). Ineffective vs precision immunity. Same helper as dedication sneak attack.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Surprise Attack (Level 1)

- **When gained:** Level 1
- **Action economy:** initiative / first-round derived
- **Procedure (engine):** If initiative used Deception or Stealth, creatures that have not acted yet are off-guard to you during the first round. Needs initiative skill choice beyond Perception.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Rogue Feats (cadence)

- **Procedure:** At 1st level and every even-numbered level, you gain a rogue class feat .
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Skill Feats (cadence)

- **Procedure:** You gain skill feats more often than others. At 1st level and every level thereafter, you gain a skill feat .
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Skill Increases (Level 2)

- **Procedure:** You gain more skill increases than members of other classes. At 2nd level and every level thereafter, you gain a skill increase. You can use this increase to either become trained in one skill you're untrained in or become an expert in one skill in which you're already trained. At 7th level, you can use skill increases to become a master in a skill in which you're already an expert, and at 15th level, you can use…
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Deny Advantage (Level 3)

- **When gained:** Level 3
- **Action economy:** class feature / choice
- **Procedure (engine):** As someone who takes advantage of openings, you are careful not to leave such gaps yourself. You aren't off-guard to hidden , undetected , or flanking creatures of your level or lower, or creatures of your level or lower using surprise attack. However, they can still help their allies flank.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### General Feats (Level 3)

- **Procedure:** At 3rd level and every 4 levels thereafter, you gain a general feat .
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Ancestry Feats (Level 5)

- **Procedure:** In addition to the ancestry feat you started with, you gain an ancestry feat at 5th level and every 4 levels thereafter. The list of ancestry feats available to you can be found in your ancestry's entry.
- **Engine:** feat-slot grant; no special action unless the chosen feat defines one.

### Weapon Tricks (Level 5)

- **When gained:** Level 5
- **Action economy:** class feature / choice
- **Procedure (engine):** You gain expert proficiency in simple weapons, martial weapons, and unarmed attacks. When you critically succeed at an attack roll against an off-guard creature while using an agile or finesse weapon or an agile or finesse unarmed attack, you apply the critical specialization effect for that weapon or unarmed attack.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Evasive Reflexes (Level 7)

- **When gained:** Level 7
- **Action economy:** class feature / choice
- **Procedure (engine):** You've learned to move quickly to avoid explosions, dragons' breath, and worse. Your proficiency rank for Reflex saves increases to master. When you roll a success on a Reflex save, you get a critical success instead.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Perception Mastery (Level 7)

- **When gained:** Level 7
- **Action economy:** class feature / choice
- **Procedure (engine):** You've developed keen awareness and attention to detail. Your proficiency rank for Perception increases to master.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Weapon Specialization (Level 7)

- **When gained:** Level 7
- **Action economy:** class feature / choice
- **Procedure (engine):** You've learned how to inflict greater injuries with the weapons you know best. You deal 2 additional damage with weapons and unarmed attacks in which you're an expert. This damage increases to 3 if you're a master, and 4 if you're legendary
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Debilitating Strike (Level 9)

- **When gained:** Level 9
- **Action economy:** see procedure (often 1 action or free)
- **Procedure (engine):** When taking advantage of an opening, you both hinder and harm your foe. You gain the Debilitating Strike free action. Debilitating Strike [free-action] Rogue Source Player Core pg. 168 Trigger Your Strike hits an off-guard creature and deals damage. You apply one of the following debilitations, which lasts until the end of your next turn. Debilitation The target takes a –10-foot status penalty to its Speeds.…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Rogue Resilience (Level 9)

- **When gained:** Level 9
- **Action economy:** class feature / choice
- **Procedure (engine):** Your physique is incredibly hardy. Your proficiency rank for Fortitude saves increases to expert. When you roll a success on a Fortitude save, you get a critical success instead.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Rogue Expertise (Level 11)

- **When gained:** Level 11
- **Action economy:** class feature / choice
- **Procedure (engine):** Your techniques are now harder to resist. Your proficiency rank for your rogue class DC increases to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Greater Rogue Reflexes (Level 13)

- **When gained:** Level 13
- **Action economy:** class feature / choice
- **Procedure (engine):** You elude danger to a degree that few can match. Your proficiency rank for Reflex saves increases to legendary. When you roll a critical failure on a Reflex save, you get a failure instead. When you roll a failure on a Reflex save against a damaging effect, you take half damage.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Perception Legend (Level 13)

- **When gained:** Level 13
- **Action economy:** class feature / choice
- **Procedure (engine):** You notice things almost impossible for an ordinary person to detect. Your proficiency rank for Perception increases to legendary.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Light Armor Expertise (Level 13)

- **When gained:** Level 13
- **Action economy:** class feature / choice
- **Procedure (engine):** You've learned how to dodge while wearing light or no armor. Your proficiency ranks for light armor and unarmored defense increase to expert.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Master Tricks (Level 13)

- **When gained:** Level 13
- **Action economy:** class feature / choice
- **Procedure (engine):** You've mastered a rogue's fighting moves. Your proficiency ranks increase to master for all simple weapons, martial weapons, and unarmed attacks.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Double Debilitation (Level 15)

- **When gained:** Level 15
- **Action economy:** class feature / choice
- **Procedure (engine):** Your opportunistic attacks are particularly detrimental. When you use Debilitating Strike you can apply two debilitations simultaneously; removing one removes both.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Greater Weapon Specialization (Level 15)

- **When gained:** Level 15
- **Action economy:** class feature / choice
- **Procedure (engine):** Your damage from weapon specialization increases to 4 with weapons and unarmed attacks in which you're an expert, 6 if you're a master, and 8 if you're legendary.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Agile Mind (Level 17)

- **When gained:** Level 17
- **Action economy:** class feature / choice
- **Procedure (engine):** You play mental games and employ cognitive tricks to throw off mind-altering effects. Your proficiency rank for Will saves increases to master. When you roll a success at a Will save, you get a critical success instead.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Light Armor Mastery (Level 19)

- **When gained:** Level 19
- **Action economy:** class feature / choice
- **Procedure (engine):** Your skill with light armor improves, increasing your ability to dodge blows. Your proficiency ranks for light armor and unarmored defense increase to master.
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.

### Master Strike (Level 19)

- **When gained:** Level 19
- **Action economy:** see procedure (often 1 action or free)
- **Procedure (engine):** You can incapacitate an unwary foe with a single strike. Your proficiency rank for your rogue class DC increases to master. You gain the Master Strike free action. Master Strike [free-action] Incapacitation Rogue Source Player Core pg. 169 Trigger Your Strike hits an off-guard creature and deals damage. The target attempts a Fortitude save against your class DC. It then becomes temporarily immune to your Master…
- **Traits / notes:** Remaster name as listed; cite source class entry. Do not copy full feat text into fixtures—store feat id + rule-element contributions.


## Focus spells / inseparable kits

- No focus/spell kit required for the class's martial L1 slice beyond listed features.

## Store vs derive

- **Store (character / preparation):** class id, level, key attribute choice, ancestry HP contribution, selected skills, feat picks (class/skill/general/ancestry), class choice subtree (racket/muse/order/doctrine/school/thesis/patron/bloodline/mystery/instinct/cause/style/methodology/research field), trained weapon/armor ranks overrides from features.
- **Store (encounter):** HP, conditions, focus points remaining, spell slots remaining / repertoire prepared-for-day, raised shield / stance / rage / panache / hunted prey id, reactions available.
- **Derive at prepare:** proficiency bonuses (level + rank), AC from armor + Dex + bonuses, Strike attack/damage from weapon + attr + proficiency + specialization, Perception/saves/class DC/spell DC, spell attack, MAP rules, off-guard sources.
- **Derive per check:** circumstance/status/item bonus nets by type, degree of success, precision applicability.
- **Sneak Attack store:** dice progression by level; applicability is derived (off-guard / other qualifying).

## Dependencies (substrate not yet in engine or only partially present)

Baseline on `main` (~`2f56abb`) already runs Stride/Step/Strike, Reactive Strike, dying, shields, prone movement, cover/detection, manipulate, Delay/Ready, Aid (attack-bonus substitute). This class still needs:

- `architecture.md#class-choices`
- `architecture.md#feat-slots`
- `architecture.md#off-guard`
- `architecture.md#proficiency`
- `conditions-and-effects.md#armor-and-item-bonuses`
- `conditions-and-effects.md#precision-damage`

- Skill list + proficiency bonuses on checks (Aid currently substitutes Strike bonus; ledger entry).
- Feat slot model + at least data-driven class feats used in the first slice.
- Armor/weapon itemization beyond fixture-prebaked AC and one Strike (unless the slice keeps baked numbers with a Principle III note).
- Conditions listed under [conditions-and-effects.md](../conditions-and-effects.md) that this class applies or keys off.

## Suggested first slice (room-one style)

Level 1 room fight: racket choice, Sneak Attack 1d6 when target is off-guard (engine already derives off-guard), Surprise Attack on initiative if undetected/hidden. Needs skill list stub (Stealth trained) and precision damage type.

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
