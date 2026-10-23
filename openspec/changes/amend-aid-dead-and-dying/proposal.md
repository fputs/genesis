# Proposal

## Why

Preparing Aid and resolving the reaction both accept a Dead ally, and the reaction is spent when `aid` rolls even if that ally never attempts a check. Designer ruled this from the book, with no deviation: an Unconscious or Dying ally may wake before the aider's next turn, a Dead ally is rejected, and the reaction is spent only when the ally attempts the check (Actions 2292, Player Core pg. 416; Conditions 95, Player Core pg. 447; Rules 2324, Player Core pg. 410).

## What Changes

- **BREAKING** Modify the `aid` requirement "The named ally is checked before the action is spent". `prepare_to_aid` accepts an Unconscious or Dying ally. A Dead ally is `AllyDead` after `NotAnAlly` and before `NoAidModifier`, and the action is not spent. Add "Preparing for an unconscious or dying ally is legal" and "Preparing for a dead ally is rejected".
- **BREAKING** Modify the `aid` requirement "The reaction rolls DC 15 once". `aid` on a Dead ally returns `AllyDead`, draws no die, and leaves the reaction ready. The reaction is spent only through `spend_reaction`, and only when that ally attempts the check and the roll is produced. If the trigger never fires, nothing is spent. Add "Aid against a dead ally is rejected" and "A trigger that never fires spends nothing". The scenario "A dead ally can be aided" keeps that title: a MODIFIED requirement that drops a scenario name fails `openspec validate --strict`, and archive refuses the same drop. Its steps become the rejection, the same steps as "Aid against a dead ally is rejected". Unconscious and Dying stay legal on the reaction. The requirement paragraph stays at or under 500 characters.
- Modify the `aid` requirement "Aid errors follow one order" so `AllyDead` sits after `NotPrepared` and before `NoAidModifier`. Every existing scenario title under that requirement stays verbatim. The requirement paragraph stays at or under 500 characters.
- No new capability. No DEV row. This change does not edit `docs/designer/`. The rulings log is linked, not copied.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `aid`: Preparation accepts Unconscious and Dying and rejects Dead. The reaction against a Dead ally is rejected and spends nothing. The reaction is spent only when the aided ally attempts the check.

## Impact

- The implementation edits `prepare_to_aid` and `aid` in `crates/rules/src/encounter.rs` and adds `EncounterError::AllyDead`. `a_dead_ally_can_be_aided` becomes the rejected scenario. `reject_delayed` stays where it is. No new crate, no new dependency, and no file from `v1-reference`.
- This pull request is spec only. It adds no changelog bullet and does not archive. The implementation pull request adds one Unreleased changelog bullet, then archives with the subject `openspec archive amend-aid-dead-and-dying --yes` as its last commit.
- Rulings log: [docs/designer/rules-rulings.md](../../../docs/designer/rules-rulings.md). No row is added there, and no row is added to `docs/designer/rules-deviations.md`.
