# Proposal

## Why

At 0 Hit Points the state is only dead or a valueless "enter dying" mark, so a fight cannot track dying, wake anyone, or end. This change specifies that knockout family. `add-encounter-turns` owns initiative, turn order, and Strike; this change merges after it and does not take those over.

## What Changes

- Add a `dying-and-victory` capability for reaching 0 Hit Points, the dying value, unconscious, recovery checks, wounded, death at dying 4, massive damage, healing out of dying, temporary Hit Points, and victory.
- With `uses_dying_rules`, a blow that reduces a conscious creature to 0 Hit Points sets dying to 1, or 2 when that blow is critical, plus the wounded value. Without the flag, 0 Hit Points is death and wounded does not change. Dying 4 or higher is death at once. The dropping blow does not also apply the "damage while dying" increase.
- While dying, the creature is unconscious. Armor Class and Perception already take unconscious as a status −4. Reflex gains the same penalty on its own statistic. This change does not apply prone, off-guard, or blinded, and it does not drop held items (DEV-12).
- The start-of-turn hook ticks temporary Hit Points, then makes a recovery check when the creature is dying: a flat check at DC 10 + the dying value. Critical success reduces dying by 2, success by 1, failure increases it by 1, and critical failure by 2. Reducing dying to 0 or below removes dying, sets current Hit Points to 1, and increases wounded by 1. The hook does not grant actions or reorder initiative.
- Wounded increases by 1 only when dying is lost (FIX-02). The next time dying is gained, that value is added. Damage while already dying increases dying by 1, or by 2 on a critical blow, and does not change wounded.
- A blow whose damage is at least twice maximum Hit Points kills immediately, with no Hit Point precondition (FIX-15). Temporary Hit Points are not part of maximum Hit Points or of that threshold (R-06). The compared number is the blow from `damage_dealt`, after the existing penalty floor and critical doubling (R-07 / DEV-14).
- Healing that restores Hit Points removes dying, increases wounded by 1, and leaves the creature conscious at the healed Hit Points. Healing never restores temporary Hit Points and does not revive the dead. A recovery that removes dying sets 1 Hit Point; it is not this healing path.
- Temporary Hit Points follow ruling R-06 (DEV-13), recorded in `docs/designer/rules-rulings.md`: keep the higher amount and its duration; on a tie keep the existing amount and its duration; damage removes them first; healing never restores them; they are excluded from maximum Hit Points and from the massive-damage threshold; unused points vanish when the duration expires; 0 means none.
- One side wins when it has a conscious combatant and every other side has none. No conscious combatant on any side is a mutual defeat. Dying and dead are not conscious.
- Newly gaining dying reports an initiative-move notice for `add-encounter-turns`. This change does not reorder combatants, roll initiative, or resolve Strike.
- **BREAKING**: `Vital::EnterDying` and `dying_entered` are replaced by a dying value of 1, 2, or 3. Healing a dead combatant, or a non-positive amount, returns a typed error and leaves the state unchanged.

Out of scope: initiative order, Strike, multiple attack penalty, positioning (prone, off-guard, blinded, grid), dropping held items, doomed, Treat Wounds, hero points, and saving throws other than producing the Reflex penalty. Testing the unconscious −4 against rulings R-01 through R-04 stays open for Designer.

## Capabilities

### New Capabilities

- `dying-and-victory`: Knockout, dying and recovery, the unconscious status penalty, wounded, massive damage, temporary Hit Points, and which side has won.

### Modified Capabilities

- `creature-template`: At 0 Hit Points with `uses_dying_rules`, the state gains the dying value from `dying-and-victory` instead of a valueless enter-dying mark. Damage and healing on that state also apply temporary Hit Points, massive damage, and leaving dying.
- `modifiers`: Reflex is its own statistic. Unconscious contributes a status penalty of −4 to Reflex, separate from Armor Class and Perception.

## Impact

- New module `crates/rules/src/dying.rs`, re-exported from `lib.rs`. `apply_dice_damage` and `apply_healing` stay the damage and healing entries and call dying resolution. `reflex_modifiers` and `Statistic::Reflex` land in `modifiers.rs`. No new crate and no new dependency. `crates/rules` stays free of Bevy, serde, files, network, clock, and global randomness. Callers inject `Dice`. Combatants stay `CombatantId`. Sides reuse `FactionId`.
- This pull request is the spec only. The implementation pull request adds the Unreleased changelog bullet. Archive is the last commit of that implementation pull request, not of this one.
- `add-encounter-turns` calls the start-of-turn hook and consumes the initiative notice. This change does not import that change's types. Damage is only the existing `damage_dealt`, `apply_dice_damage`, and `apply_healing` path.
