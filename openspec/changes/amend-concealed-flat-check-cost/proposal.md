# Proposal

## Why

A failed hidden targeting flat check already reports that the action is spent and that the attack counts for the multiple attack penalty. Concealed only says the target is not affected (Player Core pg. 442, https://2e.aonprd.com/Conditions.aspx?ID=62), and a DC 5 failure reports neither cost. Designer ruling R-12 says a failed targeting flat check at any DC spends the action and counts toward MAP, and the target is not affected. This change records that cost on the concealed DC 5 requirement.

## What Changes

- Modify the `seek-and-hidden` requirement "A concealed target takes a DC 5 flat check". A failure or critical failure SHALL report the action spent and the attack counting for MAP. A success or critical success SHALL affect the target and report neither. `counts_for_map` describes the attack case; the caller SHALL apply it to the multiple attack penalty only when the action has the attack trait (Rules 2289). `resolve_targeting` SHALL NOT call `spend_actions` or `strike`, and it gains no new input. Keep every existing scenario title verbatim. The requirement paragraph stays at or under 500 characters. The natural 1 and natural 20 scenarios still state the degree step, and the natural 4 and natural 1 scenarios still state that the target is not affected.
- Cite Rules 2416 (https://2e.aonprd.com/Rules.aspx?ID=2416, Player Core pg. 434), Rules 2289 (https://2e.aonprd.com/Rules.aspx?ID=2289, Player Core pg. 402), and Conditions 62 (https://2e.aonprd.com/Conditions.aspx?ID=62, Player Core pg. 442) on **Source** lines, and link ruling R-12 in `docs/designer/rules-rulings.md`. The flat-check source (Rules 2278) stays.
- The hidden DC 11 requirement already has the failure sentence and already says the check does not call `spend_actions` or `strike`. The one roll at DC 11 when hidden and concealed both apply stays as written.

## Non-goals

- Wiring targeting flat checks into `Encounter::strike`, so the next Strike actually takes the multiple attack penalty, is out of scope and backlog.
- Off-turn cases (a readied attack and Reactive Strike) are out of scope and backlog.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `seek-and-hidden`: "A concealed target takes a DC 5 flat check" gains the failure cost from R-12. A success or critical success affects the target and reports neither. The target of a failure is still not affected.

## Impact

- The implementation edits `resolve_targeting` in `crates/rules/src/detection.rs`. `action_spent` and `counts_for_map` are true today only when the subject is hidden, the target is not affected, and the DC is 11. A concealed natural 4 leaves both false, and that test asserts both are false. The natural 1 test asserts neither flag. The implementation changes the natural 4 asserts to true and adds both asserts on the natural 1 test. A success and a critical success leave both false. `TargetingResult` already has the fields. The function stays attack-agnostic: no new input, and it does not read the attack trait. The caller applies `counts_for_map` to the multiple attack penalty only when the action has the attack trait. The function does not call `spend_actions` or `strike`, and it does not change the relation. No new crate, no new dependency, and no file from `v1-reference`.
- This pull request is spec only. It appends R-12 to `docs/designer/rules-rulings.md`, adds no changelog bullet, and does not archive. The implementation pull request adds one Unreleased changelog bullet, then archives with the subject `openspec archive amend-concealed-flat-check-cost --yes` as its last commit.
