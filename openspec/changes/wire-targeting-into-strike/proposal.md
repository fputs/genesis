# Proposal

## Why

`Encounter::strike` never calls `resolve_targeting`, and neither does a Reactive Strike or a readied Strike. A Strike against a concealed or hidden creature therefore skips the Remaster flat check (concealed DC 5, hidden DC 11). Undetected is already `CannotTarget` in `detection.rs` (DEV-41) and never reaches that roll. DEV-46 currently leaves the flat check to the caller.

## What Changes

- On-turn Strike, a readied Strike, and Reactive Strike call `resolve_targeting` with `area_effect` false after the gates that already draw no die, and before the attack d20. They do not copy the DC 5 or DC 11 table, and they do not add a second flat-check function.
- A failed flat check means the Strike is lost. The result is `Ok(LostStrike)`: no attack d20, no damage die, and no change to Hit Points. On the actor's turn the action is spent and the attack count increments ([R-12](../../../docs/designer/rules-rulings.md)). A Reactive Strike or a readied Strike spends the reaction and does not increment the count (https://2e.aonprd.com/Actions.aspx?ID=2256, Player Core pg. 138; [R-05](../../../docs/designer/rules-rulings.md)).
- An undetected target returns `CannotTarget` on that path's error enum, spends nothing, and draws no die (DEV-41).
- DEV-46 is retired. The flat check is no longer a caller substitute. Undetected stays DEV-41. This pull request does not edit `docs/designer/`. The implementation pull request marks the row retired.
- `Detections` stays caller-owned. The encounter does not store it, does not call `note_strike`, and still compares the attack with baked Armor Class.
- `TargetingResult` reports the flat-check face `resolve_targeting` already rolled. Strike copies that face onto `LostStrike`. It does not roll a second die to learn the face.

**BREAKING**: `Encounter::strike` returns `StrikeDelivery` (`Rolled` or `Lost`). `ReactiveOutcome` uses that delivery. `ReadySuccess` gains `Lost`. The reactive arm of `resolve_reactions` returns that outcome instead of `()`. `EncounterError`, `ReactiveError`, and `ReadyError` each gain `CannotTarget`. `strike`, `reactive_strike`, and `resolve_reactions` take `&Detections`.

## Non-goals

- Calling `note_strike`, or applying the hidden off-guard −2 inside the attack total. Baked Armor Class stays (DEV-01). The caller still combines off-guard.
- Seek, Hide, area effects, and a square guess for an undetected creature (DEV-41).
- Shield Block against a lost Strike. The target's reaction stays ready. `wire-shield-block-into-reactive-strike` stays later.
- Changing the DC, the effect, or the spend flags inside `resolve_targeting`. It still does not call `spend_actions` or `strike`. The result gains the face the roll already has.

## Capabilities

### New Capabilities

None. The flat check already belongs to `seek-and-hidden`. A new capability would duplicate it.

### Modified Capabilities

- `encounter-turns`: On-turn Strike calls `resolve_targeting`. A lost Strike spends the action and counts for MAP. Undetected is `CannotTarget`.
- `ready`: Remove "Hidden targets stay with the caller". A readied Strike calls `resolve_targeting`.
- `reactive-strike`: Reactive Strike calls `resolve_targeting`. A lost check spends the reaction and does not count for MAP.
- `seek-and-hidden`: `TargetingResult` reports the flat-check face when a die was drawn, and no face when none was drawn.

## Impact

`crates/rules/src/encounter.rs` calls `resolve_targeting` from `strike`, `reactive_strike`, and the readied Strike inside `resolve_reactions`. `detection.rs` gains no second DC table. `TargetingResult` gains the face already stored on the flat-check roll. No new crate, no new dependency, and no file from `v1-reference`. This pull request is the spec only: no changelog bullet and no archive. The implementation pull request adds one Unreleased changelog bullet, then archives as its last commit.

This change does not add a ruling. The log is [docs/designer/rules-rulings.md](../../../docs/designer/rules-rulings.md). R-12 already covers a failed flat check on an attack during your turn. The off-turn cost below is the reading this spec implements, and Designer should confirm it before implementation.

## Questions for Designer

1. Retire DEV-46, rather than amend it. The departure was "the caller rolls." Strike now calls `resolve_targeting`. Undetected stays DEV-41, so an amended DEV-46 would only restate that row.
2. A lost Reactive Strike spends the reaction and does not increment the attack count (https://2e.aonprd.com/Actions.aspx?ID=2256, Player Core pg. 138). A lost readied Strike spends the reaction, clears the ready, and does not increment the count ([R-05](../../../docs/designer/rules-rulings.md)). R-12's MAP sentence stays on your own turn. Confirm that off-turn reading.
3. This change does not call `note_strike` and does not apply the hidden −2. Confirm both stay with the caller, including after a lost Strike (DEV-43, DEV-01).
