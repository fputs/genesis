# Proposal

## Why

`Encounter::strike` never calls `resolve_targeting`, and neither does a Reactive Strike or a readied Strike. A Strike against a concealed or hidden creature therefore skips the Remaster flat check (concealed DC 5, hidden DC 11). Undetected is already `CannotTarget` in `detection.rs` (DEV-41) and never reaches that roll. DEV-46 currently leaves the flat check to the caller.

## What Changes

- On-turn Strike, a readied Strike, and Reactive Strike call `resolve_targeting` with `area_effect` false after the gates that already draw no die, and before the attack d20. They do not copy the DC 5 or DC 11 table, and they do not add a second flat-check function.
- A failed flat check means the Strike is lost. The result is `Ok(LostStrike)`: no attack d20, no damage die, and no change to Hit Points. On the actor's turn the action is spent and the attack count increments ([R-12](../../../docs/designer/rules-rulings.md)). A Reactive Strike or a readied Strike spends the reaction and does not increment the count (https://2e.aonprd.com/Actions.aspx?ID=2256, Player Core pg. 138; [R-05](../../../docs/designer/rules-rulings.md)).
- An undetected target returns `CannotTarget` on that path's error enum, spends nothing, and draws no die (DEV-41). `CannotTarget` sits after `AttackOverflow` in the reactive error order, and after no reaction in the ready fire order.
- DEV-46 is retired in this change, in the DEV-03 style, and the row points at `wire-targeting-into-strike`. Undetected stays DEV-41. `docs/designer/rules-rulings.md` is not edited.
- `Detections` stays caller-owned. The encounter does not store it and does not call `note_strike`. After `Ok(LostStrike)` the caller still calls `note_strike` (DEV-43), so a lost Strike still reveals the striker. The attack still uses baked Armor Class.
- `TargetingResult` reports the flat-check face `resolve_targeting` already rolled. Strike copies that face onto `LostStrike`. It does not roll a second die to learn the face.

**BREAKING**: `Encounter::strike` returns `StrikeDelivery` (`Rolled` or `Lost`). `ReactiveOutcome` uses that delivery. `ReadySuccess` gains `Lost`. The reactive arm of `resolve_reactions` returns that outcome instead of `()`. `EncounterError`, `ReactiveError`, and `ReadyError` each gain `CannotTarget`. `strike`, `reactive_strike`, and `resolve_reactions` take `&Detections`.

## Non-goals

- Calling `note_strike` from the encounter. The caller calls it after `Ok(LostStrike)` as well as after a rolled Strike (DEV-43). The hidden off-guard −2 stays out of the attack total. Baked Armor Class stays (DEV-01). The caller still combines off-guard.
- Seek, Hide, area effects, and a square guess for an undetected creature (DEV-41).
- Shield Block against a lost Strike. The target's reaction stays ready. `wire-shield-block-into-reactive-strike` stays later.
- Changing the DC, the effect, or the spend flags inside `resolve_targeting`. It still does not call `spend_actions` or `strike`. The result gains the face the roll already has.

## Capabilities

### New Capabilities

None. The flat check already belongs to `seek-and-hidden`. A new capability would duplicate it.

### Modified Capabilities

- `encounter-turns`: On-turn Strike calls `resolve_targeting` after every existing no-die gate. A lost Strike spends the action, counts for MAP, and the caller still calls `note_strike`. Undetected is `CannotTarget`.
- `ready`: Remove "Hidden targets stay with the caller" and keep baked Armor Class. A readied Strike calls `resolve_targeting`. `CannotTarget` is last in "Fire rejects before it rolls".
- `reactive-strike`: Reactive Strike calls `resolve_targeting`. A lost check spends the reaction and does not count for MAP. `CannotTarget` follows `AttackOverflow` in "Errors have one order". The caller still calls `note_strike`.
- `seek-and-hidden`: `TargetingResult` reports the flat-check face when a die was drawn, and no face when none was drawn.

## Impact

`crates/rules/src/encounter.rs` calls `resolve_targeting` from `strike`, `reactive_strike`, and the readied Strike inside `resolve_reactions`. `detection.rs` gains no second DC table. `TargetingResult` gains the face already stored on the flat-check roll. No new crate, no new dependency, and no file from `v1-reference`. This pull request is the spec, plus the DEV-46 retirement: no changelog bullet and no archive. The implementation pull request adds one Unreleased changelog bullet, then archives as its last commit.

This change does not add a ruling. The log is [docs/designer/rules-rulings.md](../../../docs/designer/rules-rulings.md). R-12 already covers a failed flat check on an attack during your turn.

## Designer rulings

Round 1 encoded these rulings. They are not open questions.

1. DEV-46 is retired, not amended, and the row points at `wire-targeting-into-strike`. Undetected stays DEV-41.
2. A lost readied Strike and a lost Reactive Strike spend the reaction and do not change the attack count. A lost readied Strike clears the ready. R-12's MAP sentence stays on your own turn (https://2e.aonprd.com/Actions.aspx?ID=2256, Player Core pg. 138; [R-05](../../../docs/designer/rules-rulings.md)).
3. A lost Strike still reveals the striker. After `Ok(LostStrike)` the caller still calls `note_strike` (DEV-43). The encounter does not. The hidden −2 stays with the caller (DEV-01).
