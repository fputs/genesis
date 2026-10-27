# Proposal

## Why

[Pull request #93](https://github.com/fputs/genesis/pull/93), `add-resistances-weaknesses-persistent`, merges first. It owns `ConditionSlot`, the persistent (damage type, amount) container, and one end-of-turn hook that runs before decrements. Frightened, sickened, and the other Phase 1 combat conditions still need a value, a duration, and a decrement after that hook. The modifier stacker already folds status and circumstance penalties.

## What Changes

- Add `conditions`. A creature holds instances of a minimal set: frightened, sickened, off-guard, prone, clumsy, and enfeebled. Each instance has a duration: until removed, a number of rounds on a named creature's start, or until the end of a named creature's next turn. Valued conditions keep every instance. The active value is the highest. Reducing the value reduces every instance of that name, and 0 ends that instance. Removing a condition removes every instance.

- Frightened decreases by 1 at the end of the frightened creature's turn. Sickened, clumsy, and enfeebled do not. The end tick also resolves "until the end of that creature's next turn". The start tick reduces round durations whose clock is that creature, including when the clock creature is dead and the slot is skipped. Hit Points, temporary Hit Points, and wounded stay unchanged.

- Those conditions produce modifiers the existing stacker already understands. Same-type bonuses and penalties do not stack: the highest bonus and the worst penalty apply, and a bonus and a penalty of the same type both apply. Frightened and sickened are a status penalty equal to the active value on checks and DCs, not on damage and not on a flat check. Off-guard is −2 circumstance to AC. Prone adds that off-guard penalty and −2 circumstance to attack rolls. Clumsy and enfeebled follow the Dexterity-based and Strength-based lists, using `AttackBasis` on the strike.

- The encounter calls the start tick from the start step and the end tick from the end step. Delay already calls the end step: frightened and an ending duration fire then, once. The end of the turn that return opened does not fire them again. Return does not repeat the start tick. Strike, initiative, and a basic save read the tracked modifiers. A caller Armor Class modifier on Strike still does nothing.

- **BREAKING**: `StrikeProfile` gains `basis` beside the `damage_type` from #93. `ConditionSlot` gains one instance variant. `PersistentDamage` stays a damage type and an amount. The crate root gains the condition types, and `Encounter` gains `apply_condition`, `reduce_condition`, and `remove_condition`. `EncounterError` gains `Condition`. `end_turn` keeps the dice argument from #93. `gain_persistent_damage` stays.

## Capabilities

### New Capabilities

- `conditions`: Applying, tracking, and removing the core conditions, their durations, the pure start and end ticks, and the modifiers those conditions contribute.

### Modified Capabilities

- `encounter-turns`: The start step ticks round durations. After #93's persistent hook, the end step decrements end-of-turn conditions. A skipped dead slot still ticks that creature's round clock. Initiative, Strike attack rolls, Strike Armor Class, and Strike damage include the tracked modifiers. The turn return opened does not run the decrement a second time.

- `delay`: Delay runs that end tick. Return still does not run the start tick.

- `saving-throws`: A basic save includes the defender's tracked modifiers on that save.

- `positioning`: A prone condition on combatant state is not the caller-held `Posture`. Drop Prone still does not write that state.

- `api-facade`: The public name set and the encounter method set include the new condition types and the three condition methods.

## Impact

Depends on `add-resistances-weaknesses-persistent` (#93). This branch is based on that pull request. New module `crates/rules/src/conditions.rs`, re-exported from the crate root. The decrement is called when that change's end-of-turn hook returns. This change does not replace `ConditionSlot` or the persistent payload. No new crate and no new dependency. No file on `v1-reference` is named, so none is ported.

This pull request is the spec only. It adds no Rust, no tests, and no changelog bullet, and it does not archive. Designer rules on the spec here. Code waits for a `spec-approved: <sha>` review from Evan.

Out of this change: the resistance, weakness, and persistent-damage rules in #93; skills; spells and areas. Stunned, slowed, grabbed, blinded, drained, and the rest of the appendix stay out. The kind and duration enums are the extension point a later change adds a variant to. This change does not add a second persistent container or a second end-of-turn hook.

The archive commit is last, right before merge, and only after Architect and QA have both passed that head. Proposed DEV-73 is in design.md. Phase 2 reserves DEV-59 through DEV-61 for skills, DEV-62 and DEV-63 for resistances, and DEV-64 through DEV-72 for spells. This change does not use those numbers. The ruling log is [docs/designer/rules-rulings.md](../../../docs/designer/rules-rulings.md). The code phase adds DEV-73 to `docs/designer/rules-deviations.md`. The spec commit does not edit that file.
