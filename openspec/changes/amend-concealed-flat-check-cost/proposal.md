# Proposal

## Why

A failed hidden targeting flat check already reports that the action is spent and that the attack counts for the multiple attack penalty. Concealed only says the target is not affected (Player Core pg. 442, https://2e.aonprd.com/Conditions.aspx?ID=62), and a DC 5 failure reports neither cost. Designer ruling R-12 says a failed targeting flat check at any DC spends the action and counts toward MAP, and the target is not affected. This change records that cost on the concealed DC 5 requirement.

## What Changes

- Modify the `seek-and-hidden` requirement "A concealed target takes a DC 5 flat check". Add the sentence "A failure SHALL report the action spent and the attack counting for MAP". Add that same AND clause to the existing natural 4 and natural 1 DC 5 scenarios. Keep every existing scenario title verbatim. The requirement paragraph stays at or under 500 characters.
- Cite Rules 2416 (https://2e.aonprd.com/Rules.aspx?ID=2416, Player Core pg. 434), Rules 2289 (https://2e.aonprd.com/Rules.aspx?ID=2289, Player Core pg. 402), and Conditions 62 (https://2e.aonprd.com/Conditions.aspx?ID=62, Player Core pg. 442) on **Source** lines, and link ruling R-12 in `docs/designer/rules-rulings.md`. The flat-check source (Rules 2278) stays.
- The hidden DC 11 requirement already has this failure sentence. The one roll at DC 11 when hidden and concealed both apply stays as written. A success at DC 5 still affects the target and does not use this failure report.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `seek-and-hidden`: "A concealed target takes a DC 5 flat check" gains the failure cost from R-12. The target is still not affected.

## Impact

- The implementation edits `resolve_targeting` in `crates/rules/src/detection.rs`. `action_spent` and `counts_for_map` are true today only when the subject is hidden, the target is not affected, and the DC is 11. A concealed natural 4 and a concealed natural 1 leave both false. Those two existing tests assert the report. `TargetingResult` already has the fields. The function still does not call `spend_actions` or `strike`, and it still does not change the relation. No new crate, no new dependency, and no file from `v1-reference`.
- This pull request is spec only. It appends R-12 to `docs/designer/rules-rulings.md`, adds no changelog bullet, and does not archive. The implementation pull request adds one Unreleased changelog bullet, then archives with the subject `openspec archive amend-concealed-flat-check-cost --yes` as its last commit.
