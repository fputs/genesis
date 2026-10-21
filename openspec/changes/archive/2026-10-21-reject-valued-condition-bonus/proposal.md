# Proposal

## Why

Designer ruling R-04 extends R-03. Frightened (Player Core p.444) and sickened (Player Core p.446) only ever give a status penalty equal to their value. A bonus tagged with either condition, at any rank including 0, cannot exist. The active `harden-modifier-validation` delta still says a frightened status bonus keeps its declared value, and one stacking scenario builds that bonus.

## What Changes

- **BREAKING**: Construction rejects a frightened or sickened bonus at any rank, including 0, with `InvalidValuedCondition`. It does not panic, and it does not rewrite the bonus into a penalty. A penalty of rank 0 stays legal and still contributes nothing. A penalty of rank above 0 still applies as status `-rank` (Player Core p.442, Rules 2455).
- "Frightened grouping follows the split" is removed and added as "Valued conditions are validated at construction", with the same scenarios. "A frightened bonus does not hide a frightened penalty" is renamed to "A condition bonus does not hide that condition's penalty" and uses condition 10, a condition whose modifiers are not derived from its value. The same source, sequence, status bonus of +2 at rank 3, and status penalty of -1 at rank 1 still both apply, and the attack-roll total is still +1 in either order.
- The property-test requirement and its oracle drop "a frightened status bonus keeps its declared value." The generator never draws a frightened or sickened bonus. Frightened and sickened penalties still use status `-rank`, and rank 0 entries are still dropped.
- This delta carries the R-03 text in the active `harden-modifier-validation` change forward for the two requirements it amends. That change is not archived and is not yet synced into `openspec/specs/modifiers/spec.md`. It does not edit `amend-condition-id-scenario` or the scenario "The lower condition id wins an equal value" (spec PR #38).
- This pull request copies `docs/designer/rules-rulings.md` exactly from the designer log, including R-04. It does not change Rust or `CHANGELOG.md`, and it does not archive.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `modifiers`: Construction rejects a frightened or sickened bonus under "Valued conditions are validated at construction". The bonus-versus-penalty scenario uses condition 10. The property-test generator never draws a frightened or sickened bonus.

## Impact

- The implementation extends the R-03 construction check in `crates/rules/src/modifiers.rs`. No new crate and no new dependency. `crates/rules` stays free of Bevy, serde, files, network, clock, and global randomness.
- Tests that build a frightened or sickened bonus, including `a_frightened_bonus_keeps_its_declared_value`, must follow the new scenarios. `a_frightened_bonus_does_not_hide_a_frightened_penalty` is renamed to `a_condition_bonus_does_not_hide_that_conditions_penalty`. The property generator must not draw those bonuses.
- Archive this change only after `harden-modifier-validation` is archived. Archiving that change later would sync its older requirement text back over these two requirements. `amend-condition-id-scenario` edits a different requirement and stays untouched.
- The implementation pull request adds an Unreleased changelog bullet, then archives with `openspec archive reject-valued-condition-bonus --yes` as its last commit. This pull request does not.
