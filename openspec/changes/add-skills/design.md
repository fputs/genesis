# Design

## Context

`degree_of_success` and `resolve_check` already apply the four bands and the natural 1 and natural 20 shift. `Proficiency`, `Level`, `LEVEL_RANGE`, and `proficiency_bonus` already exist for saves. `Modifier` already totals item, circumstance, status, and untyped penalties per `Statistic`. `Creature` has no skill training. See proposal.md for why this change exists.

`Encounter::spend_actions` rejects a spend that is not the open Alive turn, and it does not subtract when the count is short. This change names no file on `v1-reference`. The spec commit does not edit `docs/designer/`.

## Goals / Non-Goals

**Goals:**

- One module, `crates/rules/src/skill.rs`, for the sixteen modifiers and the skill DC. It calls `proficiency_bonus`. It does not copy the rank table.

- `Encounter::skill_check` and `Encounter::recall_knowledge` roll with `resolve_check`. Recall Knowledge spends 1 action only after the gates that draw no die.

- Sixteen `Statistic` variants, one per skill, so an item bonus on Athletics does not touch Acrobatics.

**Non-Goals:**

- Lore (https://2e.aonprd.com/Skills.aspx?ID=41, Player Core pg. 240). Each Lore subcategory is its own skill. This change has no Lore variant.

- Demoralize (https://2e.aonprd.com/Actions.aspx?ID=2395, Player Core pg. 240), Grapple (https://2e.aonprd.com/Actions.aspx?ID=2376, Player Core pg. 235), Shove (https://2e.aonprd.com/Actions.aspx?ID=2380, Player Core pg. 235), and Trip (https://2e.aonprd.com/Actions.aspx?ID=2382, Player Core pg. 236). They apply frightened, grabbed, restrained, or prone, and Demoralize adds a 10-minute immunity. Those results belong to a later change that depends on `add-conditions-with-durations`. This change does not define the conditions, the durations, or the actions. That later change can call `skill_check`.

- Trained-only actions, Assurance, skill feats, and class skill increases. The rank is an input. Recall Knowledge does not require trained.

- A GM-chosen key attribute. The table attribute is fixed (DEV-61).

- Simple DCs, level-based DCs, and Recall Knowledge topic lists. The caller passes the skill and the DC.

- An equipment catalog. The caller passes the item modifier.

- The armor check penalty (https://2e.aonprd.com/Rules.aspx?ID=2169, Player Core pg. 271). It applies to Strength- and Dexterity-based skill checks, except checks with the attack trait, and it does not apply when the wearer meets the armor's Strength threshold. This change has no armor and does not produce that penalty. A caller may pass it as an untyped penalty.

- Recall Knowledge's concentrate trait (https://2e.aonprd.com/Actions.aspx?ID=2367, Player Core pg. 231). The trait is not stored. Stupefied will need it, and stupefied is out of scope.

- Replacing baked `stealth_dc` or the Hide check (DEV-10, DEV-40). Initiative stays Perception (DEV-37). A skill check is not an Aid trigger (DEV-51).

- Tumble Through, Climb, Swim, Balance, and the other skill actions. Perception is not a skill. This change's checks do not read or increment the attack count.

- A fact, a false fact, or a follow-up answer. The degree is the whole Recall Knowledge result.

- `add-resistances-weaknesses-persistent` and `add-spells-and-areas`. This change deals no damage and casts no spell.

- Editing `Encounter::strike`. No flat check and no `Detections` argument.

## Decisions

### Types

`Skill` is the sixteen skills in book order: Acrobatics, Arcana, Athletics, Crafting, Deception, Diplomacy, Intimidation, Medicine, Nature, Occultism, Performance, Religion, Society, Stealth, Survival, Thievery. `Skill::statistic` maps each to the `Statistic` of the same name. `Skill::attribute` reads Strength, Dexterity, Intelligence, Wisdom, or Charisma from `Skills`.

`Skills` stores `level: Level` and those five attribute modifiers, plus one `Proficiency` per skill. The attributes are modifiers, not scores. Constitution is not a field. `None` on the creature is the typed "cannot attempt" state.

```rust
pub enum Skill {
    Acrobatics,
    Arcana,
    Athletics,
    Crafting,
    Deception,
    Diplomacy,
    Intimidation,
    Medicine,
    Nature,
    Occultism,
    Performance,
    Religion,
    Society,
    Stealth,
    Survival,
    Thievery,
}

pub struct Skills {
    pub level: Level,
    pub strength: i32,
    pub dexterity: i32,
    pub intelligence: i32,
    pub wisdom: i32,
    pub charisma: i32,
    pub acrobatics: Proficiency,
    pub arcana: Proficiency,
    pub athletics: Proficiency,
    pub crafting: Proficiency,
    pub deception: Proficiency,
    pub diplomacy: Proficiency,
    pub intimidation: Proficiency,
    pub medicine: Proficiency,
    pub nature: Proficiency,
    pub occultism: Proficiency,
    pub performance: Proficiency,
    pub religion: Proficiency,
    pub society: Proficiency,
    pub stealth: Proficiency,
    pub survival: Proficiency,
    pub thievery: Proficiency,
}
```

`Level` and `Proficiency` stay the save types. `Skills` does not accept an `i32` level. `Level::new` still rejects every integer outside `LEVEL_RANGE`.

Alternative considered: one `Attributes` record shared with `SavingThrows`. Rejected. `save_modifier` already shipped. This change does not edit it. DEV-59 records that the two Dexterity or Wisdom values can differ.

Alternative considered: store an ability score and use `(score - 10) / 2`. Rejected. Player Core stores the attribute modifier.

### Modifier formula

`skill_modifier` adds `Skill::attribute`, `proficiency_bonus(level, rank)`, and `total` of the modifiers on that skill. A modifier on another skill is ignored. Proficiency is not a `Modifier`. The existing frightened and sickened valued-condition producer also emits, on that skill statistic, a status penalty equal to each condition's value, including when the caller's entry names another statistic. `total()` then keeps only the worst status penalty. Rank 0 contributes nothing. `skill_dc` is 10 plus that modifier, so the same penalty is on the skill DC. Unconscious adds no skill penalty.

The sum must fit `CheckModifier`. That failure is `SkillError::ModifierOverflow`. The encounter reports `SkillOverflow` before the die. A fitting `CheckModifier` always fits a DC of `modifier + 10`, so `skill_dc` returns that sum and does not fail.

```rust
pub(crate) enum SkillError {
    ModifierOverflow(ArithmeticOverflow),
}

pub(crate) fn skill_modifier(
    training: &Skills,
    skill: Skill,
    modifiers: &[Modifier],
) -> Result<CheckModifier, SkillError>;

pub(crate) fn skill_dc(modifier: CheckModifier) -> i32;
```

### Statistics

Add the sixteen variants after `Will`. Sort keys are 7 through 22 in the `Skill` order above, so Will stays 6. Each variant has its own bucket in the per-statistic group. Extend every exhaustive `Statistic` match. Extend the frightened and sickened valued-condition producer so each of those sixteen statistics receives the status penalty. Do not add an unconscious producer.

### Encounter commands

Both methods take the actor, the skill, the caller DC, the caller modifiers, and the dice. Both return `SkillRoll { natural, total, degree }`. Both are `&mut self`. `skill_check` writes nothing.

```rust
impl Encounter {
    pub fn skill_check(
        &mut self,
        actor: CombatantId,
        skill: Skill,
        dc: i32,
        modifiers: &[Modifier],
        dice: &mut impl Dice,
    ) -> Result<SkillRoll, EncounterError>;

    pub fn recall_knowledge(
        &mut self,
        actor: CombatantId,
        skill: Skill,
        dc: i32,
        modifiers: &[Modifier],
        dice: &mut impl Dice,
    ) -> Result<SkillRoll, EncounterError>;
}
```

`skill_check` is a primitive for a caller-adjudicated check. It is not the creature taking an action. An unconscious creature can't act (https://2e.aonprd.com/Conditions.aspx?ID=95, Player Core pg. 446); that gate stays on Recall Knowledge. `skill_check` checks `UnknownCombatant`, then `NoSkills`, then `SkillOverflow`, then calls `resolve_check`. It does not read vital, the open turn, actions, or the attack count. A dead or unconscious creature still rolls. Unconscious adds no skill penalty.

`recall_knowledge` checks, in order: `require_active_alive` (`TurnNotOpen`, `NotYourTurn`, `NotConscious`), then `InsufficientActions` when fewer than 1 action remains, then `NoSkills`, then `SkillOverflow`. Those draw no die and spend nothing. It then calls `spend_actions(actor, 1)` and `resolve_check`. It does not call `resolve_reactions`, `note_strike`, Strike, damage, Shield Block, or targeting. The attack count stays unchanged.

The Secret trait is an open roll (DEV-60, cross-reference DEV-08). `SkillRoll.natural` is the face from `Dice`. The concentrate trait is not stored.

`BuildParts`, `Contribution`, and `Creature` gain `skills: Option<Skills>`. `derive` copies the option and does not roll. A contribution replaces training when `Some`, as a shield does.

### Facade and errors

Re-export `Skill`, `Skills`, and `SkillRoll` from the crate root. Do not re-export `skill_modifier` or `skill_dc`. Add `skills: None` to the root doctest. The public method set gains `skill_check` and `recall_knowledge` and no other method.

`EncounterError::NoSkills` and `EncounterError::SkillOverflow(ArithmeticOverflow)` are the encounter errors. `ReadyEncounter` gains the same two variants.

`ready_from_encounter` maps `NoSkills` to `ReadyError::Encounter(ReadyEncounter::NoSkills)`. It maps `SkillOverflow(error)` to `ReadyError::Encounter(ReadyEncounter::SkillOverflow(error))`.

`from_spend` maps `NoSkills` to `ReactiveError::NoReaction`. It maps `SkillOverflow(_)` to `ReactiveError::NoReaction`.

`as_damage_overflow` maps `NoSkills` to `ReactiveError::DamageOverflow(ModifierError::Overflow)`. It maps `SkillOverflow(_)` to that same damage overflow. Each new variant has its own arm in all three matches.

### Proposed deviations

| ID | Rule | RAW citation | Our behavior | Reason | Owning change |
|---|---|---|---|---|---|
| DEV-59 | Skill attributes are a second record. This row amends DEV-58 | https://2e.aonprd.com/Rules.aspx?ID=2134 (PC 225) | This row amends DEV-58. Strength, Dexterity, Intelligence, Wisdom, and Charisma live on `Skills` and are not read from `SavingThrows`. The two Dexterity or Wisdom values may differ | Saves already shipped their three attributes. This change does not edit `save_modifier` | add-skills |
| DEV-60 | Recall Knowledge is an open roll. Cross-reference DEV-08 | https://2e.aonprd.com/Actions.aspx?ID=2367 (PC 231). The trait is Secret | The natural result is returned. Cross-reference DEV-08 | Cross-reference DEV-08: no hidden-information channel, the same reason as Seek's open roll | add-skills |
| DEV-61 | The key attribute is the table attribute | https://2e.aonprd.com/Rules.aspx?ID=2134 (PC 225). The GM may call for a different attribute | `Skill::attribute` is fixed | No GM | add-skills |

DEV-56 through DEV-58 stay as add-saving-throws wrote them. They are not reused.

## Risks / Trade-offs

- [NoSkills spends the action] → `NoSkills` and `SkillOverflow` return before `spend_actions`. The gate scenarios lock the action count.

- [A skill statistic shares Fortitude's stack] → The Athletics item bonus scenario totals Fortitude at +0. Sort keys 7 through 22 leave Will at 6.

- [The facade test rejects the new names] → The public-set scenario lists `Skill`, `SkillRoll`, `Skills`, `skill_check`, and `recall_knowledge`. The doctest sets `skills: None`.

- [Skill Dexterity overwrites the save] → DEV-59. The derive scenario keeps save Dexterity +1 and skill Dexterity +4.

- [A dead creature rolls a skill check] → `skill_check` is a primitive for a caller-adjudicated check and ignores vital. An unconscious creature still rolls, with no unconscious penalty. Recall Knowledge still requires an Alive open turn, because an unconscious creature can't act.

## Migration Plan

The implementation adds `skills: None` at every current `BuildParts`, `Contribution`, and `Creature` literal, then sets `Some` only in the new tests. No saved data exists. Revert that commit to roll back. The spec commit has no Rust change.
