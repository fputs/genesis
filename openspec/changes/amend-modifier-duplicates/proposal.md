# Proposal

## Why

The modifiers fold treats every modifier that shares a source id as one effect, and it does that before it splits a bonus from a penalty. An equal sequence then keeps the higher number, so a penalty tie keeps the milder penalty. That disagrees with Duplicate Effects (Rules 2266, Player Core pg. 398) and Penalties (Rules 2282, Player Core pg. 400). Designer ruling R-01 keys the effect on (source id, statistic, type, bonus or penalty), keeps the worse penalty, and groups frightened and sickened only after that split.

## What Changes

- Amend `modifiers` so duplicate effects collapse only on (source id, statistic, type, bonus or penalty). A bonus and a penalty from one effect both apply. Modifiers from one effect to two statistics both apply. Within a key, the higher level or rank wins, then the newer sequence.
- When that comparison ties, a bonus keeps the higher value and a penalty keeps the worse (more negative) value. The comparison is a total order, so the result does not depend on input order. Equal values of different types from one source at one sequence both survive.
- Frightened and sickened grouping runs after the bonus and penalty split and keys on (condition, statistic), not the condition alone. Frightened 2 on the attack roll and frightened 2 on Armor Class both survive. A bonus tagged `ConditionId::FRIGHTENED` does not hide a frightened penalty. Same-type stacking is per statistic.
- Record the current penalty-tie defect in `docs/designer/rules-deviations.md`, and record rulings R-01 and R-02 in `docs/designer/rules-rulings.md`. R-02 documents the damage floor. It does not change the damage requirement.
- This pull request is the spec and those designer notes. It does not change Rust.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `modifiers`: Duplicate-effect identity, the equal-sequence penalty tie, order-independent resolution, and frightened grouping after the bonus and penalty split.

## Impact

- The implementation edits `crates/rules/src/modifiers.rs` only. `total` keeps its signature. No new crate and no new dependency. `crates/rules` stays free of Bevy, serde, files, network, clock, and global randomness. Callers still pass sequence numbers. Tests use `ScriptedDice`.
- `source_prefers` today keeps `candidate.value > current.value` for a penalty at an equal sequence. That is the defect R-01 corrects. Same-type stacking across different sources is unchanged: the highest bonus, the worst penalty, and the lowest sequence on a cross-source tie.
- The implementation pull request adds an Unreleased changelog bullet, then archives with `openspec archive amend-modifier-duplicates --yes` as its last commit. This pull request does not archive.
