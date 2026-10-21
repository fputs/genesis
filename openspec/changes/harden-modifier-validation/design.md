# Design

## Context

See proposal.md for why. `with_condition` returns `Self` and cannot fail. A frightened or sickened penalty is rewritten to status `-rank` whatever type the caller declared. `attack_check_total` passes `SourceId(1)`, `SourceId(2)`, and `SourceId(3)`. The property oracle calls `derived_rank`, `same_duplicate_key`, and `source_ord`. The behavior to implement is `specs/modifiers/spec.md`.

## Goals / Non-Goals

**Goals:**

- Reject invalid frightened and sickened input at construction.
- Make the property oracle a separate RAW reference.
- Apply the multiple attack penalty once, through `MultipleAttackPenalty`. Name the other untyped internal penalty the range penalty. A reserved source does not skip same-type stacking.

**Non-Goals:**

- A new crate, dependency, or field on `Modifier`.
- Changing same-type stacking, the duplicate-key ladder, or the condition-group tie order.
- Editing `docs/designer/rules-rulings.md` after this change copies R-03.
- A type-order selection step.

## Decisions

### 1. Reject at construction

`with_condition` returns `Result`. Frightened or sickened with rank `None`, or with a type other than status, returns `ModifierError::InvalidValuedCondition`. Rank 0 is `Ok` and still contributes nothing when totaled. Off-guard and any other condition with no rank stay `Ok`. A status bonus with a rank stays `Ok` and keeps its value. Do not rewrite a non-status type into status.

A legal penalty of rank above 0 still applies as status `-rank`. That value step stays. The type rewrite does not.

Alternative considered: reject only while totaling. Rejected. R-03 says construction, and a bad modifier must not exist to be stacked.

### 2. Independent oracle

The test builds its own three stages. It does not call `derived_rank`, `same_duplicate_key`, `source_ord`, `condition_prefers`, `key_prefers`, `bonus_prefers`, `penalty_prefers`, `stack_tie`, or `group_kind`.

(a) Group by condition, keyed on (condition, statistic, bonus or penalty, type).
(b) Collapse same-source duplicates (rule R-01), keyed on (source, statistic, type, bonus or penalty), among entries with the same condition id. Stage (b) uses the duplicate-key comparison already required by "Duplicate key comparison is a total order". It does not use the stacking-step order.
(c) For each statistic, keep the best bonus per type, the worst penalty per type, and every untyped penalty (untyped penalties all stack).

Ties at the stacking step go to value first, then lowest sequence, then lower condition id (an entry with no condition id loses), then lower source id. Type is not a tiebreak. This is the stacking step only. It is not the condition-group ladder (highest rank, newest sequence, lowest source, then level).

A condition-10 circumstance penalty of -2 plus an unconditioned circumstance penalty of -3 totals -3, not -5 (Player Core p.400, Rules 2282). Same-type stacking keeps the worst penalty. The two penalties are different condition groups, so they are not summed.

The test encodes source order itself: caller ids first, by number, then the reserved variants in a fixed order. It does not call `source_ord`.

Every generated modifier builds without error. The assertion is what keeps an R-03 rejection from aborting the test. Each draw is:

- a bonus or a penalty, with a legal non-zero value
- a type from circumstance, item, or status, with untyped allowed for penalties only
- a statistic, one of two: attack roll or Armor Class
- frightened, always status, with rank `Some(0..2)`, ranks 0, 1, and 2
- at least one reserved source

Do not draw a zero value or an untyped bonus. Rank 0 builds and contributes nothing. Sickened stays out of this generator and is covered by its rejection scenarios. Keep the unconditioned untyped exact duplicate, the both-levels-absent set, and a conditioned exact duplicate. At least 1,000 random sets, seed `0x0D1A_7E5B`, every permutation of a set of at most 4. The test adds no crate and does not read a clock.

### 3. Reserved attack-check sources

The multiple attack penalty applies once per check (Player Core p.402, Rules 2289). Reuse `ModifierSource::MultipleAttackPenalty` for that internal term. Do not add a separate internal MAP source. A second attack with no caller multiple attack penalty gets -5, once.

The other untyped internal penalty is the range penalty (Player Core p.403, Rules 2290). The range penalty and the circumstance penalty use reserved sources that no caller can construct. They are distinct from `MultipleAttackPenalty`. `attack_check_total` does not pass `SourceId`. A positive untyped term stays a raw addend.

A reserved source does not exempt a term from same-type stacking. An internal circumstance penalty of -2 plus a caller circumstance penalty of -3 from source 1 totals -3, not -5. Untyped penalties still all stack, so a caller untyped penalty is added to the multiple attack penalty and to the range penalty.

Alternative considered: add a new MAP variant beside `MultipleAttackPenalty`. Rejected. Two untyped -5 terms would stack. Alternative considered: reuse `SourceId(1..3)` and tell callers to avoid them. Rejected. Caller source 1 is legal.

### 4. No type-order tiebreak

A grep of `openspec/specs`, `docs`, and unarchived design notes found no selection tiebreak ordered circumstance, item, status, untyped. Archived `amend-modifier-tiebreaks-2` already rejects that alternative. `kind_ord` orders the reported list after selection. It is not a selection step, and this change does not edit it. The group key includes type, so a type step inside a group cannot change the winner.

## Risks / Trade-offs

- [Existing tests chain `with_condition`] → They handle `Result`. `a_frightened_circumstance_penalty_becomes_status_minus_rank` expects the error and is not deleted.
- [The oracle's source table drifts from production] → The table is the order in decision 2. It includes `MultipleAttackPenalty` and the range-penalty and circumstance variants. It does not add a second MAP source.
- [The rulings log still says frightened and sickened are rewritten to status] → This change copies the designer's file exactly, including that earlier amendment and R-03. The implementation does not edit it. The spec replaces the rewrite.

## Migration Plan

Callers of `with_condition` handle the error. Rank 0, other unranked conditions, and a frightened or sickened status bonus with a rank keep today's success path. No stored data changes.
