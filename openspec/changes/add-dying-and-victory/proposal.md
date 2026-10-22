# Proposal

## Why

At 0 Hit Points the state is only dead or a valueless "enter dying" mark, so a fight cannot track dying, stay unconscious at 0, or say which `Side` has won. This change specifies that knockout family. `add-encounter-turns` owns initiative, turn order, Strike, and `Side::{Pc, Enemy}`; this change merges after it and does not take those over.

## What Changes

- Add a `dying-and-victory` capability for reaching 0 Hit Points, the dying value, the unconscious vital, recovery checks, wounded, death at dying 4, massive damage, healing out of dying, temporary Hit Points, and victory.
- With `uses_dying_rules`, a blow that reduces an Alive creature to 0 Hit Points sets dying to 1, or 2 when that blow is critical, plus the wounded value, and sets current Hit Points to 0 in that same step. Without the flag, 0 Hit Points is death and wounded does not change. Dying 4 or higher is death at once. The dropping blow does not also apply the "damage while dying" increase. Doomed is not modeled; the threshold stays 4 (DEV-28).
- `Vital` is `Alive`, `Unconscious`, `Dying` of 1, 2, or 3, or `Dead`. Unconscious means 0 Hit Points and not dying. Dying and Unconscious both take the unconscious penalty. Positive healing is the only wake. The 10-minute natural wake-up is not modeled.
- Armor Class, Perception, and Reflex each take unconscious as a status −4, with reserved condition id `ConditionId::UNCONSCIOUS` (R-09). Armor Class also takes off-guard −2 through the existing `ac_modifiers` input, so the Armor Class total is −6 when nothing else applies. Prone and blinded are not applied (DEV-31). Held items are not dropped (DEV-12). This change does not edit `docs/designer/`.
- The start-of-turn hook ticks temporary Hit Points at the recipient's start of turn (R-06, DEV-32), then makes a recovery check when the creature is dying: a flat check at DC 10 + the dying value. Critical success reduces dying by 2, success by 1, failure increases it by 1, and critical failure by 2. Reducing dying to 0 or below removes dying, leaves current Hit Points at 0, increases wounded by 1, and leaves the creature Unconscious. It does not set 1 Hit Point and does not make them Alive. The hook does not grant actions.
- Wounded is an `i32`. It increases by 1 only when dying is lost (FIX-02). The next time dying is gained, that value is added. Damage while already dying increases dying by 1, or by 2 on a critical blow, and does not change wounded. A blow fully absorbed by temporary Hit Points still increases dying (R-10).
- A blow whose damage is at least twice maximum Hit Points kills immediately, with no Hit Point precondition (FIX-15). Temporary Hit Points are not part of maximum Hit Points or of that threshold (R-06). The compared number is the blow from `damage_dealt`, after the existing penalty floor and critical doubling (R-07 / DEV-14). Every death clears temporary Hit Points (R-10).
- Healing that restores Hit Points on a dying creature removes dying, increases wounded by 1, and leaves them Alive at the healed Hit Points. Healing an Unconscious creature makes them Alive and does not change wounded. Healing never restores temporary Hit Points and does not revive the dead. All damage is lethal (DEV-29).
- Temporary Hit Points follow ruling R-06 (DEV-13): keep the higher amount and its duration; on a tie keep the existing amount and its duration; damage removes them first; healing never restores them; they are excluded from maximum Hit Points and from the massive-damage threshold; unused points vanish when the duration expires; 0 means none. Duration ticks at the recipient's start of turn, which is DEV-32. RAW counts from the source's turn.
- Victory is a pure query. The team identifier is `Side` from `add-encounter-turns`. A Side is defeated when none of its combatants is Alive (DEV-30). The query is not terminal.
- Newly gaining dying sets a report the caller can take once. This change does not reorder combatants or roll initiative. Strike stays in `add-encounter-turns` and calls `apply_resolved_blow`. Follow-up change `wire-dying-into-turns` calls the hook from encounter turns.
- **BREAKING**: `Vital::EnterDying` and `dying_entered` are replaced by `Alive`, `Unconscious`, `Dying` of 1, 2, or 3, and `Dead`. Healing a dead combatant, or a non-positive amount, returns a typed error and leaves the state unchanged.

Out of scope: initiative order, choosing a Strike target, multiple attack penalty, positioning beyond the existing off-guard input, dropping held items, doomed, nonlethal and the other DEV-29 effects, Treat Wounds, hero points, and saving throws other than producing the Reflex penalty. R-09 closes how the unconscious −4 meets R-01 through R-04. DEV-28 through DEV-32 and R-08 through R-10 are cited from `docs/designer/rules-deviations.md` and `docs/designer/rules-rulings.md`. This change does not edit those files.

## Capabilities

### New Capabilities

- `dying-and-victory`: Knockout, dying and recovery, the unconscious vital and its penalty, wounded, massive damage, temporary Hit Points, and which Side has won.

### Modified Capabilities

- `creature-template`: At 0 Hit Points with `uses_dying_rules`, damage resolution records dying as `dying-and-victory` specifies, except massive damage kills. This capability still does not choose the dying value.
- `modifiers`: Reflex is its own statistic. Unconscious is a reserved condition id and a status penalty of −4 on Armor Class, Perception, and Reflex, stacking as R-09 specifies.
- `encounter-turns`: A hitting Strike calls `apply_resolved_blow` with its degree. `note_hit_points` is removed. The requirement "Reaching 0 Hit Points floors at 0" is replaced by resolved blows. The not-Alive turn rule is left as `add-encounter-turns` wrote it. `HpError::NotAHit` comes from that change and is not declared here. This change merges after `add-encounter-turns`.

## Impact

- New module `crates/rules/src/dying.rs`, re-exported from `lib.rs`. `apply_dice_damage` and `apply_healing` stay the damage and healing entries and call dying resolution. `reflex_modifiers`, `Statistic::Reflex`, and `ConditionId::UNCONSCIOUS` land in `modifiers.rs`. No new crate and no new dependency. `crates/rules` stays free of Bevy, serde, files, network, clock, and global randomness. Callers inject `Dice`. Combatants stay `CombatantId`. Victory takes `Side::{Pc, Enemy}` from `add-encounter-turns` and does not take `FactionId`.
- This pull request is the spec only. The implementation pull request adds the Unreleased changelog bullet. Archive is the last commit of that implementation pull request, not of this one.
- `add-encounter-turns` merges first and exports `Side`. This change owns the semantics of R-08, R-10, and temporary Hit Point ticks. Its encounter-turns delta routes Strike damage through `apply_resolved_blow`. `wire-dying-into-turns` only calls `start_of_turn` and `take_initiative_move`, and that follow-up, not this change, removes the interim Armor Class workaround (DEV-36).
