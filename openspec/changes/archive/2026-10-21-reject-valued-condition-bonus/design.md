# Design

## Context

See proposal.md for why. `with_condition` still returns `Self`. Ruling R-03, in the active `harden-modifier-validation` change, makes that function return `Result` and reject a missing rank or a non-status type with `ModifierError::InvalidValuedCondition`. That change's delta still accepts a status bonus tagged frightened or sickened. The property generator can attach either condition to a bonus. `a_frightened_bonus_keeps_its_declared_value` expects that bonus to total +2.

## Goals / Non-Goals

**Goals:**

- Reject a frightened or sickened bonus at construction, including rank 0.
- Keep the property generator inside the inputs construction accepts.
- Move the bonus-versus-penalty scenario onto condition 10.

**Non-Goals:**

- A new crate, dependency, or error variant.
- Changing same-type stacking, the duplicate-key ladder, or the condition-group tie order.
- Editing the scenario "The lower condition id wins an equal value". That scenario belongs to `amend-condition-id-scenario`.
- Editing `docs/designer/rules-rulings.md` after this change copies R-04.

## Decisions

### 1. Reject the bonus at construction

This change overrides `harden-modifier-validation` tasks 1.1 and 2.1, design section 1, and its spec delta. A frightened or sickened bonus is rejected per R-04, not accepted. Do not edit that change folder. PR #37 is implementing and archiving it. Archive this change after `harden-modifier-validation`.

Extend the R-03 check. A frightened or sickened bonus, at any rank including 0, returns `ModifierError::InvalidValuedCondition`. Do not rewrite it into a status penalty. A penalty of rank 0 stays `Ok` and still contributes nothing. A penalty of rank above 0 still applies as status `-rank`. Any other condition with no rank stays `Ok`.

Alternative considered: drop the bonus while totaling and keep construction successful. Rejected. R-04 says construction, the same point as R-03.

### 2. The generator draws no such bonus

A frightened draw is a status penalty with rank `Some(0)`, `Some(1)`, or `Some(2)`. A bonus draw is not tagged frightened or sickened. Sickened stays out of the generator, as R-03 already requires. The oracle drops rank 0 entries and uses status `-rank` for frightened and sickened penalties. It has no branch that keeps a frightened status bonus. The requirement says penalties use status `-rank`, and the scenario still says they use that value. Both say rank 0 entries are dropped. The description stays under 500 characters.

Alternative considered: let the generator draw the bonus and expect `InvalidValuedCondition`. Rejected. The property test requires every generated modifier to build.

### 3. Condition 10 keeps the old scenario's meaning

The requirement is removed and added as "Valued conditions are validated at construction". The scenario is renamed to "A condition bonus does not hide that condition's penalty". Rename `a_frightened_bonus_does_not_hide_a_frightened_penalty` to `a_condition_bonus_does_not_hide_that_conditions_penalty`. Inputs are condition 10, a condition whose modifiers are not derived from its value: source 7, attack roll, sequence 4, status bonus +2 at rank 3, status penalty -1 at rank 1. Both apply, and the total is +1 in either order. `a_frightened_bonus_keeps_its_declared_value` no longer expects a successful +2. That input is the rejection.

### 4. Archive after harden-modifier-validation

This delta replaces two requirements that `harden-modifier-validation` also replaces. Archive this change only after that one, so the sync keeps the R-04 text. `amend-condition-id-scenario` replaces a different requirement. Do not copy that requirement into this delta.

## Risks / Trade-offs

- [R-03 is not implemented yet] → This change extends that `Result`. Implement it after `harden-modifier-validation`, or in the same construction function if that change has not landed. Do not add a second error variant.
- [Archiving `harden-modifier-validation` later restores the frightened-bonus sentence] → The implementation archives this change last among the two, after that change is already archived.
- [A test still builds a frightened bonus] → Retarget it at `InvalidValuedCondition`. Do not delete `the_lower_condition_id_wins_an_equal_value`.

## Migration Plan

Callers of `with_condition` handle the error, as R-03 already requires. A frightened or sickened status bonus is no longer a success. No stored data changes.
