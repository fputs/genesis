# Proposal

## Why

The scenario "The lower condition id wins an equal value" puts a circumstance bonus on condition id 3. That id is `ConditionId::SICKENED`. Merged ruling R-03 (`harden-modifier-validation`) rejects sickened unless the type is status, so the scenario cannot be constructed. Condition id 4 is not a named `ConditionId`. The named ids are frightened 2 and sickened 3.

## What Changes

- That scenario's circumstance bonuses use condition ids 8 and 4, and the applied condition id is 4. The lower id still wins.
- `ConditionId::SICKENED` stays 3. No other scenario, requirement, or code changes in this pull request.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `modifiers`: "Duplicate key comparison is a total order" keeps the same rule. Only the equal-value condition-id scenario uses 4 instead of 3.

## Impact

- The implementation updates `the_lower_condition_id_wins_an_equal_value` in `crates/rules/src/modifiers.rs` so the circumstance bonuses use condition ids 8 and 4. No new crate and no new dependency.
- This pull request does not change Rust or `CHANGELOG.md`, and it does not archive. The implementation pull request adds the Unreleased changelog bullet, then archives this change last.
