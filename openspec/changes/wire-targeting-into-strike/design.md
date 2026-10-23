# Design

## Context

`resolve_targeting` in `crates/rules/src/detection.rs` already rolls concealed DC 5, hidden DC 11, and one roll at DC 11 when both apply. It returns `CannotTarget` for undetected and draws no die. `TargetingResult` drops the `FlatRoll.natural` the roller already produced. `Encounter::strike`, `Encounter::reactive_strike`, and `fire_strike` each draw the attack d20 with no call into that function. `Detections` is caller-owned and is not a field of `Encounter`. See proposal.md for why this is one procedure.

The behavior to implement is the delta specs. `crates/rules` stays dependency-free. No `unwrap`, `expect`, `panic!`, or unchecked indexing. This change names no `v1-reference` file, so none is ported. No file under `docs/designer/` is edited in the spec pull request.

## Goals / Non-Goals

**Goals:**

- One call to `resolve_targeting` on each Strike path, with `area_effect` false, after the gates that already draw no die and before the attack d20.
- `LostStrike` for a flat check that does not affect the target, and `CannotTarget` when that function returns it.
- The flat face on `TargetingResult`, copied onto `LostStrike`, with no second roll.

**Non-Goals:**

- A field for `Detections` on `Encounter`, `Creature`, or `CombatantState`. `note_strike` and the hidden off-guard −2. Area effects. A second flat-check function. Shield Block on a lost Strike.

## Decisions

### Call the existing function

| Path | After these no-die gates | Then |
|---|---|---|
| `Encounter::strike` | `prepare_strike`, and a requested Shield Block's readiness | `resolve_targeting`, then the attack d20 only if `affected` |
| `reactive_strike` | The order in "Errors have one order", then `AttackOverflow` | same |
| readied `fire_strike` | "Fire rejects before it rolls", including `strike_fits` | same |

`area_effect` is false. A missing relation is observed and `NotConcealed`, so the call draws nothing and today's attack-die scripts stay aligned. Hidden and concealed still roll once inside `resolve_targeting`. Do not call `flat_check` from the encounter.

Alternative considered: copy DC 5 and DC 11 into `encounter.rs`. Rejected. The seek-and-hidden spec already owns that table.

Alternative considered: store `Detections` on `Encounter`. Rejected. Seek and Hide return a new map, and the caller already owns it. The three entry points take `&Detections`.

### The face is already on the roll

`FlatRoll.natural` is `Option<u8>`. `TargetingResult` gains `natural: Option<u8>`, copied from that roll. No die, including observed, area, and `CannotTarget`, stores `None`. `LostStrike` is `{ natural: u8, dc: i32, degree: DegreeOfSuccess }`. Build it from an `Ok` result whose `affected` is false and whose `natural`, `dc`, and `degree` are all `Some`. DC 5 and DC 11 always roll, so the scenarios have all three. Match `Some`. Do not `expect` and do not panic. An `affected: false` result that lacks one of those three returns `CannotTarget`, spends nothing, and draws no further die. That arm is not a scenario: today's DCs do not produce it. The numeric degree is the flat check's degree, including a critical failure at natural 1, which still loses.

### Lost is Ok, CannotTarget is the error

A lost check spends, so it is not an error. `StrikeDelivery` is `Rolled(StrikeOutcome)` or `Lost(LostStrike)`. `Encounter::strike` returns `Result<StrikeDelivery, EncounterError>`. `ReactiveOutcome.strike` is that delivery, and `disruption` on `Lost` is `NotDisrupted`. `ReadySuccess` gains `Lost(LostStrike)` beside `Strike(ReadyStrike)`.

`resolve_reactions` today maps the reactive outcome to `()`. The reactive arm becomes `Result<ReactiveOutcome, ReactiveError>` so a lost check is visible in initiative order. The same `&Detections` and the same `Dice` serve every reaction in that call. Relations do not change, so the next reaction reads the same concealment and draws its own flat check.

`CannotTarget` is a variant of `EncounterError`, `ReactiveError`, and `ReadyError`. Map `DetectionError::CannotTarget` to it. That is the only detection error these paths produce. Do not wrap the whole `DetectionError` enum.

On-turn loss calls the same action spend and attack-count increment as a resolved Strike (`finish_strike`). It does not call `shield_block`. Off-turn loss calls `spend_reaction` and, for Ready, `clear_ready`. It does not increment the attack count. `counts_for_map` stays true on the targeting result. The on-turn path applies it. The off-turn paths do not: Reactive Strike does not count (https://2e.aonprd.com/Actions.aspx?ID=2256, Player Core pg. 138), and a readied attack does not add to the count (R-05).

Alternative considered: `Err` for a lost check. Rejected. `AttackOverflow` and `InsufficientActions` leave the action unspent. R-12 spends it.

Alternative considered: optional fields on `StrikeOutcome`. Rejected. A lost check has no attack total and no damage. The enum keeps that unrepresentable.

### Die order

On-turn overflow still happens after the attack d20, as the existing scenario requires. A concealed target that passes DC 5 therefore draws the flat face, then that attack d20, then returns `AttackOverflow` with the action unspent and the count unchanged. A lost check never reaches that d20.

Reactive `AttackOverflow` stays before any die, including the flat check. A failure in the existing gate list draws nothing.

### DEV-46 is retired

The implementation replaces the Our behavior cell of DEV-46 with: **Retired.** On-turn Strike, a readied Strike, and Reactive Strike call `resolve_targeting` before the attack roll. A failed flat check loses the Strike. Undetected stays `CannotTarget` (DEV-41). Keep the RAW citation. Do not add a DEV id. DEV-47 through DEV-49 stay unused. The spec pull request does not edit `docs/designer/rules-deviations.md`.

Alternative considered: amend DEV-46 so it says the caller no longer rolls. Rejected. That sentence would be RAW, and DEV-41 already records the undetected substitute.

## Risks / Trade-offs

- [A test script whose first face is the attack roll shifts if the target is concealed or hidden] → Existing scenarios name no relation, so the added `&Detections` is empty and the first face stays the attack d20. New scenarios name the flat face first.
- [Designer rejects the off-turn reading in proposal.md] → That is a spec edit before implementation. This design follows the specs as written.
- [`clippy::too_many_arguments` already fires on these functions] → Keep the `expect` on the smallest function and extend its reason to include `detections`. Do not add a crate-level allow.
- [Returning `ReactiveOutcome` from `resolve_reactions` changes a public enum] → Callers that matched `Ok(())` match `Ok(outcome)` instead. The outcome was already computed and discarded.

## Migration Plan

Callers pass `&Detections::new()` when they have no relation. A rolled Strike's fields are inside `StrikeDelivery::Rolled`. There is no stored data to migrate. The implementation pull request adds the Unreleased changelog bullet, then `openspec archive wire-targeting-into-strike --yes` as the last commit.
