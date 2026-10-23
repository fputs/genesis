# Design

## Context

`prepare_to_aid` checks the named ally for unknown, self, and a different side, then a missing modifier, then spends one action. It does not read the ally's vital. `aid` does not read it either. The comment on `aid` says the ally's vital is not an error. `a_dead_ally_can_be_aided` prepares for a Dead ally and expects total 17, Plus1, and a spent reaction. `reject_delayed` already runs after the aider's `NotConscious` check. `set_prepared` in the encounter tests writes `AidPreparation::Prepared` when `prepare_to_aid` cannot. See proposal.md for why Dead, Unconscious, and Dying differ.

## Goals / Non-Goals

**Goals:**

- `prepare_to_aid` accepts Unconscious and Dying and returns `AllyDead` for Dead before the action is spent.
- `aid` returns `AllyDead` for a Dead ally it is prepared for, draws no die, and leaves the reaction ready.
- An `aid` call that is never made spends nothing. An `Ok` result is still the spend, and it still happens after the roll.

**Non-Goals:**

- Rejecting Unconscious or Dying inside `aid`. The caller invokes `aid` when that ally attempts the check, including after they wake.
- A new parameter for "the ally attempted the check". No skill-check trigger. DEV-51 stays as written.
- Moving or removing `reject_delayed`. No edit to `docs/designer/`. No DEV row. No file from `v1-reference`.

## Decisions

### 1. `AllyDead` is its own error

`EncounterError::AllyDead` means the named ally's vital is `Dead`. `NotConscious` stays the aider's error and also covers Unconscious and Dying, so it cannot mean "this ally is dead." `TargetDead` stays the Strike target's error. Prepare and aid both return `AllyDead`.

Alternative considered: reuse `TargetDead`. Rejected. That variant is documented as Strike, and a later change would not be able to tell an aided ally from a Strike target.

### 2. Where the Dead check sits

In `prepare_to_aid`, after `NotAnAlly` and before `NoAidModifier`. Unconscious and Dying are not refusals: the call spends one action and stores `Prepared` for that ally. A Dead enemy is still `NotAnAlly`. A refusal leaves the action count and preparation unchanged.

In `aid`, after the preparation match (`NotThePreparedAlly` / `NotPrepared`) and before `NoAidModifier`. A Dead ally the aider is not prepared for stays `NotPrepared` or `NotThePreparedAlly`. `reject_delayed` stays after the aider's `NotConscious` check. `require_reaction` stays before the roll.

Alternative considered: reject Dead before `NotAnAlly`. Rejected. A different side is `NotAnAlly` today, and this change does not reorder that.

### 3. The `aid` call is the attempt

The book spends the reaction when the ally is about to attempt the check. This crate has no separate attempt signal. `aid` returning `Ok` is that attempt: the roll is produced, then `spend_reaction`, then preparation clears. If the caller never calls `aid`, the reaction stays ready and no die is drawn. A Dead ally cannot attempt, so that call is `AllyDead` before the roll.

Alternative considered: spend the reaction in a second function when the ally's attack resolves. Rejected. `aid` already returns the circumstance for the check the caller is resolving. A second function would be a new procedure.

### 4. Replace the requirement so the old scenario title is gone

OpenSpec 1.14.3 renames requirements, not scenarios. `## RENAMED Requirements` is only a `FROM:` / `TO:` pair of `### Requirement:` headers. A MODIFIED block that omits "A dead ally can be aided" fails validation, and archive throws the same loss. The same requirement name in both ADDED and REMOVED is also an error, so a direct remove-and-add does not validate.

Archive applies RENAMED, then REMOVED, then ADDED. This delta renames "The reaction rolls DC 15 once" to "The reaction rolls DC 15 once retired", removes that temporary name, and adds "The reaction rolls DC 15 once" back without "A dead ally can be aided". The added block keeps the five roll scenarios, "Aid against a dead ally is rejected" for an aider already prepared for a Dead ally, "An ally who dies after preparation is rejected" for an ally who dies after a successful prepare, and "A trigger that never fires spends nothing" through the aider's next `start_turn`. The temporary name is not a behavior and is not in the spec after archive. Archive appends the re-added requirement after the other aid requirements.

## Risks / Trade-offs

- [A direct remove-and-add of the same requirement name fails validation] → The temporary rename exists so REMOVED and ADDED name different headers. Archive deletes the temporary name. The implementation does not keep `a_dead_ally_can_be_aided`.
- [A prepared-for-Dead state cannot be reached by `prepare_to_aid`] → "Aid against a dead ally is rejected" starts from `set_prepared`. "An ally who dies after preparation is rejected" prepares while the ally is Unconscious, then `set_vital_for_test` to Dead, then calls `aid`. Preparation still names that ally (DEV-50).
- [`reject_delayed` is not in the aid spec's error list] → This change does not add it. The implementation leaves that call where it is.
- [Strict validation fails a requirement description past 500 characters] → The added reaction description is 500 characters. The two modified descriptions stay under 500. Citations stay on `**Source**` lines, which are not part of that count.

## Migration Plan

No persisted data and no caller outside this crate. Rollback removes `AllyDead` and restores the success result for a Dead ally, including the scenario "A dead ally can be aided".
